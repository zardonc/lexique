/* global process, window, fetch, console */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const args = process.argv.slice(2);
if (args.length !== 4) throw new Error('Expected package, executable, database, and new destination');
const [source, executable, database, destination] = args.map((path) => resolve(path));
await mkdir(destination);
await cp(executable, join(destination, 'Lexique.exe'));
await cp(join(source, 'dictionary'), join(destination, 'dictionary'), { recursive: true });
await mkdir(join(destination, 'Lexique-data', 'data'), { recursive: true });
await cp(database, join(destination, 'Lexique-data', 'data', 'lexique.sqlite3'));
const server = createServer();
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const port = server.address().port;
await new Promise((done) => server.close(done));
let child;
let browser;
async function start() {
  child = spawn(join(destination, 'Lexique.exe'), [], { cwd: destination, windowsHide: true,
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --remote-debugging-port=${port}` } });
  let endpoint;
  for (let attempt = 0; attempt < 100; attempt++) {
    try { endpoint = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl; if (endpoint) break; }
    catch { await delay(200); }
  }
  assert.ok(endpoint, 'WebView2 debugging endpoint unavailable');
  browser = await chromium.connectOverCDP(endpoint);
  for (let attempt = 0; attempt < 100; attempt++) {
    const page = browser.contexts().flatMap((context) => context.pages())[0];
    try { if (page && await page.evaluate(() => Boolean(window.__TAURI_INTERNALS__))) return page; }
    catch { /* Window may still be navigating. */ }
    await delay(200);
  }
  throw new Error('Native window unavailable');
}
async function stop() {
  if (browser) { await browser.close(); browser = undefined; }
  if (child?.pid && child.exitCode === null) spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  child = undefined;
  await delay(500);
}
try {
  let page = await start();
  const before = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const rows = await invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
    const id = rows[0].id;
    const cards = await invoke('review_cards', { vocabularyId: id });
    await invoke('review_auto_enroll', { enabled: true });
    await invoke('review_pause', { vocabularyId: id, paused: true });
    return { id, cards };
  });
  assert.equal(before.cards.length, 2);
  await stop();
  page = await start();
  const restored = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    return { enabled: await invoke('review_auto_enroll', { enabled: null }),
      paused: await invoke('review_pause', { vocabularyId: id, paused: null }),
      queue: await invoke('review_queue', { timezone: 'UTC', skipped: [] }),
      cards: await invoke('review_add', { vocabularyId: id }) };
  }, before.id);
  assert.equal(restored.enabled, true);
  assert.equal(restored.paused, true);
  assert.deepEqual(restored.cards, before.cards);
  assert.ok(!restored.queue.queue.cards.some((card) => card.vocabulary_id === before.id));
  const collected = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const request = { operation: 'native-auto-enroll-test', headword: 'bonjour', notes: '', id: null, revision: null };
    const result = await invoke('vocabulary_save', { request });
    return { result, cards: await invoke('review_cards', { vocabularyId: result.record.id }),
      retry: await invoke('vocabulary_save', { request }) };
  });
  assert.equal(collected.result.outcome, 'created');
  assert.equal(collected.cards.length, 2);
  assert.equal(collected.retry.record.id, collected.result.record.id);
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByRole('button', { name: /école/ }).click();
  await page.getByRole('button', { name: '恢复复习', exact: true }).click();
  assert.equal(await page.evaluate((id) => window.__TAURI_INTERNALS__.invoke('review_pause', { vocabularyId: id, paused: null }), before.id), false);
  console.log('Native auto-enrollment preference, pause, queue exclusion and resume survived restart');
} finally { await stop(); }
