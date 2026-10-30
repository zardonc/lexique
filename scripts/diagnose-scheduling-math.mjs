import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const root = new URL('../', import.meta.url);
const directory = new URL('target/issue-13-math/', root);
await mkdir(directory, { recursive: true });
const source = fileURLToPath(new URL('docs/evidence/issue-13/math-probe.rs', root));
const native = fileURLToPath(new URL(`math-probe${process.platform === 'win32' ? '.exe' : ''}`, directory));
const compiled = fileURLToPath(new URL('math-probe.wasm', directory));
function run(command, args) {
  const result = spawnSync(command, args, { cwd: fileURLToPath(root), encoding: 'utf8', timeout: 60_000 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
const toolchain = run('rustc', ['-Vv']);
const flags = ['--edition=2024', '--crate-name=math_probe', '-C', 'opt-level=3'];
run('rustc', [source, ...flags, '-o', native]);
run('rustc', [source, ...flags, '--crate-type=cdylib', '--target=wasm32-unknown-unknown', '-o', compiled]);
const nativeBits = run(native, []).split(/\r?\n/).map(Number);
assert.equal(nativeBits.length, 2);
const module = new WebAssembly.Module(await readFile(compiled));
assert.deepEqual(WebAssembly.Module.imports(module), []);
const { exports: wasm } = new WebAssembly.Instance(module);
const inputs = [
  { operation: 0, name: 'exp((1-r)*w10)', x: Math.fround(0.014672648161649704), y: 0 },
  { operation: 1, name: 'powf(stability,-w9)', x: Math.fround(837.2720947265625), y: Math.fround(-0.16660000383853912) },
];
const operations = inputs.map((input, index) => ({
  ...input, nativeBits: nativeBits[index], wasmBits: wasm.math(input.operation, input.x, input.y) >>> 0,
}));
const mismatches = operations.filter(result => result.nativeBits !== result.wasmBits).length;
const report = { toolchain, platform: process.platform, arch: process.arch, node: process.version, operations, mismatches };
await writeFile(new URL('result.json', directory), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
assert.equal(mismatches, 0, 'Platform math differs without FSRS or application dependencies; see target/issue-13-math/result.json');
