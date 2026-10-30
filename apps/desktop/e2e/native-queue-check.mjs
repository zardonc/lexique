/* global process, window, fetch, console */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, writeFile, utimes, stat } from 'node:fs/promises';
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
    catch { /* The window may still be navigating. */ }
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
  const cache = join(destination, 'Lexique-data', 'temp', 'speech');
  await mkdir(cache, { recursive: true });
  const audio = join(cache, '1.ogg');
  await writeFile(audio, 'OggS synthetic');
  const past = new Date(Date.now() - 2 * 86400_000);
  await utimes(audio, past, past);
  const beforeAccess = (await stat(audio)).atimeMs;
  const initial = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const settings = { retention_percent: 87, new_limit: 7, review_limit: 13 };
    await invoke('review_settings', { settings });
    const queue = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
    return { settings, queue };
  });
  assert.ok(initial.queue.queue.cards.length > 0, 'Fixture must contain an eligible card');
  assert.equal((await stat(audio)).atimeMs, beforeAccess, 'Queue inspection must not renew unused audio');
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 }));
  assert.ok((await stat(audio)).atimeMs > beforeAccess, 'Reading audio for playback must renew its access time');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '查看识义卡' }).waitFor();
  await stop();
  page = await start();
  const restored = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    return { settings: await invoke('review_settings', { settings: null }), queue: await invoke('review_queue', { timezone: 'UTC', skipped: [] }) };
  });
  assert.deepEqual(restored.settings, initial.settings);
  assert.deepEqual(restored.queue.queue.cards, initial.queue.queue.cards);
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await page.getByRole('dialog', { name: '识义卡' }).waitFor({ state: 'hidden' });
  await page.waitForFunction(async () => Boolean(await window.__TAURI_INTERNALS__.invoke('review_undo_target')));
  await stop();
  page = await start();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分' });
  await undo.waitFor();
  await undo.click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await page.getByRole('dialog', { name: '识义卡' }).waitFor({ state: 'hidden' });
  console.log('Native rating, restart, undo and continued rating succeeded');
  console.log(`Native settings and ${restored.queue.queue.cards.length} queued cards survived restart`);
} finally { await stop(); }
