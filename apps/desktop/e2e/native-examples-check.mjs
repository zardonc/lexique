/* global process, window, fetch, console */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

if (process.argv.length !== 6) throw new Error('Expected package, executable, database, and new destination');
const [source, executable, database, destination] = process.argv.slice(2).map((path) => resolve(path));
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
  const initial = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const records = await invoke('vocabulary_list', { query: '', sort: 'updated-desc' });
    const record = records[0];
    if (!record) throw new Error('Synthetic database has no vocabulary');
    const cards = await invoke('review_cards', { vocabularyId: record.id });
    return { record, cards, progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))) };
  });
  assert.ok(initial.cards.length > 0, 'Fixture must contain review cards');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').first().click();
  await page.getByLabel('法语例句').fill('Je lis un livre.');
  await page.getByLabel('例句中文译文').fill('我读一本书。');
  await page.getByRole('button', { name: '添加例句' }).click();
  await page.getByLabel('法语例句').nth(1).fill('Une autre phrase.');
  await page.getByLabel('例句中文译文').nth(1).fill('另一句话。');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('status').filter({ hasText: '已保存' }).waitFor();
  await stop();
  page = await start();
  const restored = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const cards = await invoke('review_cards', { vocabularyId: id });
    return { records: await invoke('vocabulary_list', { query: '', sort: 'updated-desc' }), cards,
      progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))) };
  }, initial.record.id);
  const record = restored.records.find((item) => item.id === initial.record.id);
  assert.equal(record.example_fr, 'Je lis un livre.');
  assert.equal(record.example_zh, '我读一本书。');
  assert.deepEqual(record.more_examples, [{ fr: 'Une autre phrase.', zh: '另一句话。' }]);
  assert.deepEqual(restored.cards, initial.cards);
  assert.deepEqual(restored.progress, initial.progress);
  console.log('Native examples and review card progress survived restart');
} finally { await stop(); }
