// V01/V02: isolated current Windows binary, real IPC/playback and full Opus decode.
/* global window, document, AudioContext, process, setTimeout, clearTimeout, console, fetch, performance, URL, Blob, atob */
import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const [packagePath, binaryPath, evidencePath, destinationPath] = process.argv.slice(2).map((path) => resolve(path));
if (!destinationPath) throw new Error('Expected package, current binary, V02 evidence, new destination');
const owned = resolve('../../target/issue-24');
assert.ok(destinationPath.startsWith(`${owned}\\`), 'Destination must be a new #24 directory');
await mkdir(destinationPath);
await cp(packagePath, destinationPath, { recursive: true, filter: (path) => path !== join(packagePath, 'Lexique-data') });
await cp(binaryPath, join(destinationPath, 'Lexique.exe'));
const server = createServer();
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const port = server.address().port;
await new Promise((done) => server.close(done));
let child;
let browser;
const timer = setTimeout(() => {
  if (child?.pid) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
  console.error('Native preflight exceeded 60-second execution budget');
  process.exit(1);
}, 60_000);

async function start() {
  child = spawn(join(destinationPath, 'Lexique.exe'), [], { cwd: destinationPath, windowsHide: true,
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --autoplay-policy=no-user-gesture-required --remote-debugging-port=${port}` } });
  child.on('error', (error) => { console.error(error); });
  let endpoint;
  for (let i = 0; i < 80; i++) {
    try { endpoint = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl; if (endpoint) break; }
    catch { /* Endpoint is not ready during startup. */ }
    await delay(100);
  }
  assert.ok(endpoint, 'WebView2 endpoint unavailable');
  browser = await chromium.connectOverCDP(endpoint);
  const page = browser.contexts()[0].pages()[0];
  await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__), undefined, { timeout: 10_000 });
  return page;
}

async function baseline(page) {
  return page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const metrics = [];
    async function measured(command, args) {
      const start = performance.now(); const value = await invoke(command, args);
      metrics.push({ command, ms: performance.now() - start }); return value;
    }
    const request = { operation: 'preflight-create', headword: 'école', notes: 'isolated fixture', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
    const saved = await measured('vocabulary_save', { request });
    const id = saved.record.id;
    const cards = await measured('review_add', { vocabularyId: id });
    const cold = await measured('speech_generate', { vocabularyId: id });
    await measured('review_queue', { timezone: 'UTC', skipped: [] });
    const { bytes: cached } = await measured('speech_cached', { vocabularyId: id });
    const audio = document.createElement('audio');
    audio.src = URL.createObjectURL(new Blob([new Uint8Array(cached)], { type: 'audio/ogg' }));
    const start = performance.now(); await audio.play();
    metrics.push({ command: 'actual-play-start', ms: performance.now() - start });
    await new Promise((done, reject) => { audio.onended = done; audio.onerror = reject; });
    URL.revokeObjectURL(audio.src);
    const generating = measured('speech_generate', { vocabularyId: id, regenerate: true });
    await new Promise((done) => setTimeout(done, 50));
    const preview = measured('speech_preview', { text: 'bonjour' });
    const queries = measured('dictionary_lookup', { key: 'école' });
    const notes = measured('vocabulary_save', { request: { ...request, operation: 'preflight-edit', id, revision: 1, notes: 'concurrent note' } });
    const rating = measured('review_rate', { request: { operation: 'preflight-rate', card_id: cards[0].id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    const [regenerated, interactive, lookup, edited, rated] = await Promise.all([generating, preview, queries, notes, rating]);
    return { metrics, coldBytes: cold.bytes.length, regeneratedBytes: regenerated.bytes.length, interactiveBytes: interactive.bytes.length,
      dictionaryCount: lookup.entries.length, notes: edited.record.notes, rated: rated.status };
  });
}

async function decode(page) {
  const evidence = JSON.parse(await readFile(join(evidencePath, 'results.json'), 'utf8'));
  let decoded = 0;
  const fixed = new Map();
  for (const trial of evidence.trials) {
    for (const result of trial.results) {
      const path = join(evidencePath, trial.directory, result.audio_file);
      const bytes = await readFile(path);
      assert.equal(bytes.length, result.audio_bytes);
      if (trial.seed === 'fixed') {
        const identity = `${trial.spin}/${result.id}`;
        if (trial.background === 0) fixed.set(identity, result.pcm_sha256);
        else assert.equal(result.pcm_sha256, fixed.get(identity), 'Concurrent PCM differs from serial');
      }
      const value = await page.evaluate(async (base64) => {
        const context = new AudioContext({ sampleRate: 48000 });
        try {
          const audio = await context.decodeAudioData(Uint8Array.from(atob(base64), (c) => c.charCodeAt(0)).buffer);
          const samples = audio.getChannelData(0);
          return { channels: audio.numberOfChannels, duration: audio.duration, finite: samples.every(Number.isFinite), audible: samples.some((value) => value !== 0) };
        } finally { await context.close(); }
      }, bytes.toString('base64'));
      assert.equal(value.channels, 1);
      assert.ok(value.finite && value.audible);
      assert.ok(Math.abs(value.duration - result.duration_s) < 0.05);
      decoded++;
    }
  }
  return decoded;
}

try {
  const page = await start();
  const results = await baseline(page);
  assert.equal(results.notes, 'concurrent note');
  assert.equal(results.rated, 'committed');
  assert.ok(results.dictionaryCount > 0 && results.coldBytes > 0 && results.interactiveBytes > 0);
  const decoded = await decode(page);
  const report = { binarySha256: createHash('sha256').update(await readFile(binaryPath)).digest('hex'), hardTimeoutSeconds: 60, results, decoded };
  await writeFile(join(destinationPath, 'results.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
} finally {
  clearTimeout(timer);
  if (browser) await browser.close();
  if (child?.pid && child.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
}
