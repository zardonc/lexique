/* global process, window, document, fetch, console */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const [source, executable, destination, fixture] = process.argv.slice(2).map((path) => resolve(path));
if (!source || !executable || !destination || ![5, 6].includes(process.argv.length)) throw new Error('Expected resource directory, executable, new destination, and optional fixture database');
await mkdir(destination);
await cp(executable, join(destination, 'Lexique.exe'));
await cp(join(source, 'dictionary'), join(destination, 'dictionary'), { recursive: true });
if (fixture) {
  await mkdir(join(destination, 'Lexique-data', 'data'), { recursive: true });
  await cp(fixture, join(destination, 'Lexique-data', 'data', 'lexique.sqlite3'));
}
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
  if (fixture) {
    const progress = await page.evaluate(async () => {
      const invoke = window.__TAURI_INTERNALS__.invoke;
      const rows = await invoke('vocabulary_list', { query: '', sort: 'review-due-asc' });
      const result = {};
      for (const word of ['avion', 'zèbre', 'bonjour']) {
        const cards = await invoke('review_cards', { vocabularyId: rows.find((row) => row.headword === word).id });
        result[word] = await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id })));
      }
      return result;
    });
    const [early, late] = progress['zèbre'].map((item) => item.card.due_at);
    assert.equal(progress.avion[0].card.due_at, early);
    assert.ok(early < progress.bonjour[0].card.due_at && progress.bonjour[0].card.due_at < late);
    await page.getByRole('button', { name: '生词本', exact: true }).click();
    await page.waitForFunction(() => document.querySelectorAll('.vocabulary-list button').length === 5);
    assert.deepEqual(await page.locator('.vocabulary-list button strong').allTextContents(), ['avion', 'zèbre', 'bonjour', 'bateau', 'chat']);
    await page.getByRole('searchbox').fill('filtered');
    await page.waitForFunction(() => document.querySelectorAll('.vocabulary-list button').length === 2);
    assert.deepEqual(await page.locator('.vocabulary-list button strong').allTextContents(), ['avion', 'zèbre']);
    await stop();
    page = await start();
    await page.getByRole('button', { name: '生词本', exact: true }).click();
    await page.waitForFunction(() => document.querySelectorAll('.vocabulary-list button').length === 5);
    assert.equal(await page.getByLabel('排序方式').inputValue(), 'review-due-asc');
    assert.deepEqual(await page.locator('.vocabulary-list button strong').allTextContents(), ['avion', 'zèbre', 'bonjour', 'bateau', 'chat']);
    console.log('Native UI preserves same-time order and the earliest of two scheduled cards after restart');
  } else {
  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    for (const word of ['bonjour', 'école']) {
      const request = { operation: `sort-${word === 'bonjour' ? 'one' : 'two'}`, headword: word, notes: '', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
      const saved = await invoke('vocabulary_save', { request });
      await invoke('review_add', { vocabularyId: saved.record.id });
    }
  });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByLabel('排序方式').selectOption('review-due-asc');
  await page.locator('.vocabulary-list button').first().getByText('bonjour').waitFor();
  await page.getByRole('button', { name: /école/ }).click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '记得' }).click();
  await page.locator('.vocabulary-list button').first().getByText('école').waitFor();
  await page.getByRole('button', { name: /bonjour/ }).click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '忘记' }).click();
  await page.locator('.vocabulary-list button').first().getByText('bonjour').waitFor();
  await stop();
  page = await start();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.waitForFunction(() => [...document.querySelectorAll('select')].some((select) => select.value === 'review-due-asc'));
  await page.locator('.vocabulary-list button').first().getByText('bonjour').waitFor();
  console.log('Native rating, review-time order and preference survive restart');
  }
} finally { await stop(); }
