// Run against an isolated copy of a development package and test database.
/* global process, fetch, window, console, Storage */
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
    catch { /* The native window may still be navigating. */ }
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
  await page.evaluate(() => {
    const setItem = Storage.prototype.setItem;
    Storage.prototype.setItem = function (key, value) {
      if (key.startsWith('lexique-pending-rating-')) window.testRatingRequest = JSON.parse(value);
      return setItem.call(this, key, value);
    };
  });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').first().click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '记得' }).click();
  await card.waitFor({ state: 'hidden' });
  const initial = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const cards = await invoke('review_cards', { vocabularyId: 1 });
    const request = window.testRatingRequest;
    const result = (await invoke('review_rate', { request })).result;
    const retry = (await invoke('review_rate', { request })).result;
    return { cards, request, result, retry };
  });
  assert.deepEqual(initial.retry, initial.result);
  assert.equal(initial.result.card.phase, 'learning_last');
  assert.equal(initial.result.step_seconds, 600);
  await stop();
  page = await start();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').first().click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await page.getByRole('region', { name: '识义卡' }).getByRole('button', { name: '显示答案' }).click();
  await page.getByText('尚未到期', { exact: false }).waitFor();
  const restored = await page.evaluate(async ({ request, cards }) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const progress = await invoke('review_progress', { cardId: cards[0].id });
    const retry = (await invoke('review_rate', { request })).result;
    const sibling = await invoke('review_rate', { request: { ...request, operation: 'native-sibling', card_id: cards[1].id } });
    return { progress, retry, sibling };
  }, initial);
  assert.equal(restored.progress.revision, 1);
  assert.equal(restored.progress.card.phase, initial.result.card.phase);
  assert.equal(restored.progress.card.due_at, initial.result.card.due_at);
  assert.equal(restored.progress.card.last_review_at, initial.result.card.last_review_at);
  assert.ok(Math.abs(restored.progress.card.memory.stability - initial.result.card.memory.stability) < 1e-6);
  assert.ok(Math.abs(restored.progress.card.memory.difficulty - initial.result.card.memory.difficulty) < 1e-6);
  assert.deepEqual(restored.retry, initial.result);
  assert.equal(restored.sibling.code, 'deferred');
  console.log(JSON.stringify({ initial, restored }, null, 2));
} finally { await stop(); }
