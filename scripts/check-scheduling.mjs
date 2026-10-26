import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { isDeepStrictEqual } from 'node:util';
import { gzipSync } from 'node:zlib';

const root = new URL('../', import.meta.url);
const wasmPath = new URL('target/wasm32-unknown-unknown/release/lexique_core.wasm', root);
const nativePath = new URL(`target/release/examples/schedule${process.platform === 'win32' ? '.exe' : ''}`, root);
const fixture = await readFile(new URL('crates/core/tests/fixtures/scheduling.json', root));
const vectors = JSON.parse(fixture);
const wasmBytes = await readFile(wasmPath);
const module = new WebAssembly.Module(wasmBytes);
assert.deepEqual(WebAssembly.Module.imports(module), [], 'Scheduling WASM must run without host imports');
const { exports: wasm } = new WebAssembly.Instance(module);
const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });
function execute(input) {
  const bytes = typeof input === 'string' ? encoder.encode(input) : input;
  const pointer = wasm.schedule_request(bytes.length);
  assert.notEqual(pointer, 0);
  new Uint8Array(wasm.memory.buffer, pointer, bytes.length).set(bytes);
  const length = wasm.schedule_execute();
  return JSON.parse(decoder.decode(new Uint8Array(wasm.memory.buffer, wasm.schedule_result(), length)));
}
assert.equal(wasm.schedule_request(16_385), 0, 'ABI rejects oversized allocation');
assert.equal(execute(new Uint8Array([255])).Err, 'request is not UTF-8');
assert.ok(execute('{').Err);
assert.ok(execute('').Err);

// Fixed seed broadens floating-point calibration without depending on host time or randomness.
let seed = 13;
function fraction() {
  seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
  return seed / 0x1_0000_0000;
}
const requests = vectors.map(vector => vector.request);
for (let index = 0; index < 4096; index++) {
  const now = 1_792_929_600;
  requests.push({
    card: {
      phase: ['learning_first', 'learning_last', 'review', 'relearning'][index % 4],
      memory: { stability: 0.001 + fraction() * 36_499.999, difficulty: 1 + fraction() * 9 },
      last_review_at: now - Math.floor(fraction() * 36_500) * 86400,
      due_at: now,
    },
    rating: ['again', 'hard', 'good', 'easy'][Math.floor(fraction() * 4)],
    now,
    timezone: ['Europe/Paris', 'America/New_York', 'Asia/Tokyo'][index % 3],
    retention_percent: 70 + index % 30,
  });
}
const native = spawnSync(fileURLToPath(nativePath), [], {
  input: requests.map(request => JSON.stringify(request)).join('\n') + '\n',
  encoding: 'utf8', timeout: 60_000, maxBuffer: 32 * 1024 * 1024,
});
assert.ifError(native.error);
assert.equal(native.status, 0, native.stderr);
const outputs = native.stdout.trim().split(/\r?\n/).map(line => JSON.parse(line));
assert.equal(outputs.length, requests.length);

function subset(actual, expected, path) {
  if (expected !== null && typeof expected === 'object') {
    for (const [key, value] of Object.entries(expected)) subset(actual[key], value, `${path}.${key}`);
  } else if (path.includes('.memory.')) {
    assert.ok(Math.abs(actual - expected) < 0.000001, `${path}: ${actual} != ${expected}`);
  } else {
    assert.equal(actual, expected, path);
  }
}
let maxAbsoluteError = 0;
let maxRelativeError = 0;
const scheduleMismatches = [];
for (const [index, request] of requests.entries()) {
  const compiled = execute(JSON.stringify(request));
  const reference = outputs[index];
  const vector = vectors[index];
  if (vector?.error) {
    assert.ok(reference.Err?.includes(vector.error), `${vector.name}: ${JSON.stringify(reference)}`);
    assert.deepEqual(compiled, reference, vector.name);
    continue;
  }
  assert.ok(reference.Ok && compiled.Ok, `Unexpected failure at ${index}: ${JSON.stringify({ reference, compiled })}`);
  if (vector) subset(reference.Ok, vector.expected, vector.name);
  for (const key of ['stability', 'difficulty']) {
    const left = reference.Ok.card.memory[key];
    const right = compiled.Ok.card.memory[key];
    maxAbsoluteError = Math.max(maxAbsoluteError, Math.abs(left - right));
    maxRelativeError = Math.max(maxRelativeError, Math.abs(left - right) / Math.max(Math.abs(left), Math.abs(right)));
    // Calibrated against the committed evidence; never apply a float tolerance to dates or stages.
    assert.ok(Math.abs(left - right) <= 0.000001 + Math.abs(left) * 0.000002, `Memory drift at ${index}.${key}: ${left} / ${right}`);
  }
  const left = structuredClone(reference.Ok);
  const right = structuredClone(compiled.Ok);
  delete left.card.memory;
  delete right.card.memory;
  if (!isDeepStrictEqual(left, right)) {
    scheduleMismatches.push({ name: vector?.name ?? `calibration-${index - vectors.length}`, request, native: reference.Ok, wasm: compiled.Ok });
  }
  assert.deepEqual(execute(JSON.stringify(request)), compiled, 'Replaying a request must be deterministic');
  const historical = { ...request, model: reference.Ok.model, retention_percent: reference.Ok.retention_percent, timezone_database: reference.Ok.timezone_database };
  assert.deepEqual(execute(JSON.stringify(historical)), compiled, 'Replay uses recorded model and retention');
}
const nativeBytes = await readFile(nativePath);
const artifact = bytes => ({ bytes: bytes.length, gzipBytes: gzipSync(bytes).length, sha256: createHash('sha256').update(bytes).digest('hex') });
const report = {
  admission: scheduleMismatches.length === 0 ? 'passed' : 'blocked',
  model: outputs.find(output => output.Ok).Ok.model.version,
  platform: process.platform, arch: process.arch, node: process.version,
  fixedVectors: vectors.length, calibrationVectors: requests.length - vectors.length,
  maxAbsoluteError, maxRelativeError,
  tolerance: { absolute: 0.000001, relative: 0.000002 },
  exact: ['phase', 'step_seconds', 'interval_days', 'due_at', 'last_review_at', 'elapsed_days', 'model', 'retention_percent', 'timezone_database'],
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  native: artifact(nativeBytes), wasm: artifact(wasmBytes),
  scheduleMismatches,
};
await mkdir(new URL('target/issue-13/', root), { recursive: true });
await writeFile(new URL('target/issue-13/conformance.json', root), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
assert.equal(scheduleMismatches.length, 0, 'FSRS admission blocked: native/WASM integer schedules differ; see target/issue-13/conformance.json');
