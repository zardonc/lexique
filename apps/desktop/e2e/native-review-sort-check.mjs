/* global process, window, fetch, console */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const [source, executable, destination] = process.argv.slice(2).map((path) => resolve(path));
if (!source || !executable || !destination || process.argv.length !== 5) throw new Error('Expected resource directory, executable, and new destination');
await mkdir(destination);
await cp(executable, join(destination, 'Lexique.exe'));
await cp(join(source, 'dictionary'), join(destination, 'dictionary'), { recursive: true });
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
  await stop();
  page = await start();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.waitForFunction(() => [...document.querySelectorAll('select')].some((select) => select.value === 'review-due-asc'));
  await page.locator('.vocabulary-list button').first().getByText('école').waitFor();
  console.log('Native rating, review-time order and preference survive restart');
} finally { await stop(); }
