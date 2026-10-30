/* global process, window, fetch, console */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, rename, stat } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const args = process.argv.slice(2);
if (args.length !== 4) throw new Error('Expected package, test database, new destination, and new moved destination');
const [source, database, destination, moved] = args.map((path) => resolve(path));
try { await stat(moved); throw new Error('Moved destination already exists'); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
await mkdir(destination);
await cp(source, destination, { recursive: true, filter: (path) => path !== join(source, 'Lexique-data') });
await mkdir(join(destination, 'Lexique-data', 'data'), { recursive: true });
await cp(database, join(destination, 'Lexique-data', 'data', 'lexique.sqlite3'));
const server = createServer();
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const port = server.address().port;
await new Promise((done) => server.close(done));
let child;
let browser;
let directory = destination;
async function start() {
  child = spawn(join(directory, 'Lexique.exe'), [], { cwd: directory, windowsHide: true,
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
  const before = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const words = await invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
    const word = words.find((item) => item.headword === 'école');
    if (!word) throw new Error('Fixture must contain école');
    const cards = await invoke('review_add', { vocabularyId: word.id });
    const audio = await invoke('speech_generate', { vocabularyId: word.id });
    return { word, cards, audio, progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))) };
  });
  assert.equal(before.cards.length, 2);
  assert.deepEqual(before.audio.slice(0, 4), [79, 103, 103, 83]);
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await page.getByRole('dialog', { name: '识义卡' }).waitFor({ state: 'hidden' });
  await page.waitForFunction(async () => Boolean(await window.__TAURI_INTERNALS__.invoke('review_undo_target')));
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await page.getByLabel('例句对应释义').selectOption('0');
  await page.getByLabel('法语例句').fill('Je vais à l’école.');
  await page.getByLabel('例句中文译文').fill('我去学校。');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('status').filter({ hasText: '已保存' }).waitFor();
  const changed = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const settings = { retention_percent: 87, new_limit: 7, review_limit: 13 };
    await invoke('review_settings', { settings });
    await invoke('review_pause', { vocabularyId: id, paused: true });
    return { settings, target: await invoke('review_undo_target') };
  }, before.word.id);
  assert.ok(changed.target);
  await stop();
  await rename(destination, moved);
  directory = moved;
  page = await start();
  const restored = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    return { words: await invoke('vocabulary_list', { query: '', sort: 'headword-asc' }),
      settings: await invoke('review_settings', { settings: null }),
      paused: await invoke('review_pause', { vocabularyId: id, paused: null }),
      target: await invoke('review_undo_target'), audio: await invoke('speech_cached', { vocabularyId: id }) };
  }, before.word.id);
  assert.deepEqual(restored.settings, changed.settings);
  assert.equal(restored.paused, true);
  assert.equal(restored.target, changed.target);
  assert.deepEqual(restored.audio, before.audio);
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '撤销最近评分' }).click();
  await expect.poll(() => page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_undo_target'))).toBeNull();
  const after = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const words = await invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
    const cards = await invoke('review_cards', { vocabularyId: id });
    return { word: words.find((item) => item.id === id), settings: await invoke('review_settings', { settings: null }),
      paused: await invoke('review_pause', { vocabularyId: id, paused: null }),
      progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))) };
  }, before.word.id);
  assert.equal(after.word.example_fr, 'Je vais à l’école.');
  assert.equal(after.word.example_zh, '我去学校。');
  assert.deepEqual(after.settings, changed.settings);
  assert.equal(after.paused, true);
  assert.deepEqual(after.progress.map((item) => item.card), before.progress.map((item) => item.card));
  assert.ok(after.progress.every((item, index) => item.revision >= before.progress[index].revision));
  console.log('Native rating, independent edits, portable move and undo preserved their state');
} finally { await stop(); }
