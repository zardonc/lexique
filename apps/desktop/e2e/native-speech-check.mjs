// Run against an isolated development package and an existing test database.
/* global process, fetch, window, document, HTMLMediaElement, Buffer, console */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, rename, symlink, unlink, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const inputs = process.argv.slice(2);
if (inputs.length !== 3) throw new Error('Expected package, test database, and new destination');
const [source, database, destination] = inputs.map((path) => resolve(path));
const server = createServer();
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const port = server.address().port;
await new Promise((done) => server.close(done));
await mkdir(destination);
await cp(source, destination, { recursive: true, filter: (path) => path !== join(source, 'Lexique-data') });
if (process.env.LEXIQUE_TEST_EXECUTABLE) {
  await cp(process.env.LEXIQUE_TEST_EXECUTABLE, join(destination, 'Lexique.exe'));
}
const data = join(destination, 'Lexique-data', 'data');
const cache = join(destination, 'Lexique-data', 'temp', 'speech');
await mkdir(data, { recursive: true });
await mkdir(cache, { recursive: true });
await cp(database, join(data, 'lexique.sqlite3'));
await writeFile(join(cache, '1.ogg'), 'invalid audio');
let child;
let browser;
async function start() {
  child = spawn(join(destination, 'Lexique.exe'), [], {
    cwd: destination,
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --remote-debugging-port=${port}` },
    windowsHide: true,
  });
  let diagnostics = '';
  child.stderr.on('data', (bytes) => { diagnostics = (diagnostics + bytes.toString()).slice(-4096); });
  child.on('error', (error) => { diagnostics = String(error); });
  let endpoint;
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/version`);
      endpoint = (await response.json()).webSocketDebuggerUrl;
      if (endpoint) break;
    } catch { await delay(200); }
  }
  assert.ok(endpoint, `WebView2 debugging endpoint unavailable: ${diagnostics}`);
  browser = await chromium.connectOverCDP(endpoint);
  let page;
  for (let attempt = 0; attempt < 100; attempt++) {
    page = browser.contexts().flatMap((context) => context.pages())[0];
    try {
      if (page && await page.evaluate(() => Boolean(window.__TAURI_INTERNALS__))) break;
    } catch { /* WebView2 can still be navigating during startup. */ }
    await delay(200);
  }
  assert.ok(page, 'Native window unavailable');
  await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__), undefined, { timeout: 20_000 });
  return page;
}
async function stop() {
  if (browser) { await browser.close(); browser = undefined; }
  if (child?.pid && child.exitCode === null) {
    spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  }
  child = undefined;
  await delay(500);
}
try {
  const page = await start();
  const result = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const words = await invoke('vocabulary_list', { query: '', sort: 'updated-desc' });
    const cards = await invoke('review_cards', { vocabularyId: 1 });
    const added = await Promise.all([
      invoke('review_add', { vocabularyId: 1 }),
      invoke('review_add', { vocabularyId: 1 }),
    ]);
    const generated = await Promise.all([
      invoke('speech_generate', { vocabularyId: 1 }),
      invoke('speech_generate', { vocabularyId: 1 }),
    ]);
    return { words, cards, added, generated };
  });
  assert.equal(result.cards.length, 2);
  assert.deepEqual(result.added, [result.cards, result.cards]);
  assert.deepEqual(result.generated[0], result.generated[1]);
  assert.equal(Buffer.from(result.generated[0]).subarray(0, 4).toString(), 'OggS');
  const before = await readFile(join(cache, '1.ogg'));
  const python = join(destination, 'speech', 'runtime', 'python.exe');
  await rename(python, `${python}.disabled`);
  const error = await page.evaluate(async () => {
    try { await window.__TAURI_INTERNALS__.invoke('speech_generate', { vocabularyId: 1, regenerate: true }); return ''; }
    catch (cause) { return String(cause); }
  });
  assert.match(error, /Could not start offline speech/);
  assert.deepEqual(await readFile(join(cache, '1.ogg')), before);
  await rename(`${python}.disabled`, python);
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_generate', { vocabularyId: 1, regenerate: true }));
  assert.equal((await readFile(join(cache, '1.ogg'))).subarray(0, 4).toString(), 'OggS');
  const audio = join(cache, '1.ogg');
  const outside = join(destination, 'outside.ogg');
  await writeFile(outside, 'outside data');
  await rename(audio, `${audio}.saved`);
  try {
    await symlink(outside, audio, 'file');
    const rejected = await page.evaluate(async () => {
      try { await window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 }); return false; }
      catch { return true; }
    });
    assert.equal(rejected, true, 'cache file link must be rejected');
    assert.equal(await readFile(outside, 'utf8'), 'outside data');
    await unlink(audio);
    console.log('Native IPC: cache file link rejected');
  } catch (error) {
    if (error.code !== 'EPERM') throw error;
    console.log('Native IPC: cache file link check skipped, symlink privilege unavailable');
  } finally {
    await rename(`${audio}.saved`, audio);
  }
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await expect(page.getByText('听音卡：语音已生成', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const recognition = page.getByRole('dialog', { name: '识义卡' });
  await expect(recognition).toContainText('école');
  await expect(recognition).not.toContainText('学校');
  await recognition.getByRole('button', { name: '显示答案' }).click();
  await expect(recognition).toContainText('学校');
  await expect(recognition).toContainText('待补例句');
  await recognition.getByRole('button', { name: '关闭卡片' }).click();
  await page.evaluate(() => {
    Reflect.set(window, 'nativePlayCount', 0);
    document.addEventListener('play', (event) => {
      if (event.target instanceof HTMLMediaElement) {
        Reflect.set(window, 'nativePlayCount', Number(Reflect.get(window, 'nativePlayCount')) + 1);
      }
    }, true);
  });
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const listening = page.getByRole('dialog', { name: '听音卡' });
  await expect(listening).not.toContainText('école');
  await expect(listening).toContainText('机器合成语音');
  await expect.poll(() => listening.locator('audio').evaluate((audio) => audio.currentTime)).toBeGreaterThan(0);
  await expect.poll(() => listening.locator('audio').evaluate((audio) => audio.ended), { timeout: 20_000 }).toBe(true);
  assert.equal(await page.evaluate(() => Reflect.get(window, 'nativePlayCount')), 1);
  await expect(listening.getByRole('alert')).toHaveCount(0);
  await page.screenshot({ path: join(destination, 'listening-front.png') });
  await listening.getByRole('button', { name: '显示答案' }).click();
  await expect(listening).toContainText('école');
  await expect(listening).toContainText('学校');
  await expect(listening).toContainText('待补例句');
  assert.equal(await page.evaluate(() => Reflect.get(window, 'nativePlayCount')), 1);
  await listening.locator('details summary').click();
  await expect(listening.locator('details')).toContainText(result.words.find((word) => word.id === 1).reference.dictionary);
  await page.screenshot({ path: join(destination, 'listening-back.png') });
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_preferences', {
    preferences: { voice: 'female', speed: 'slow' },
  }));
  await listening.getByRole('button', { name: '重播' }).click();
  await expect.poll(() => page.evaluate(() => Reflect.get(window, 'nativePlayCount'))).toBe(2);
  assert.equal(await listening.locator('audio').evaluate((audio) => audio.playbackRate), 0.6);
  await listening.getByRole('button', { name: '跳过' }).click();
  await expect(listening).not.toBeVisible();
  assert.deepEqual(await readFile(join(cache, '1.ogg')), await page.evaluate(async () =>
    window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 })).then((bytes) => Buffer.from(bytes)));
  const savedAudio = await readFile(join(cache, '1.ogg'));
  await stop();
  const reopened = await start();
  const restored = await reopened.evaluate(async () => ({
    words: await window.__TAURI_INTERNALS__.invoke('vocabulary_list', { query: '', sort: 'updated-desc' }),
    cards: await window.__TAURI_INTERNALS__.invoke('review_add', { vocabularyId: 1 }),
    audio: await window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 }),
  }));
  assert.deepEqual(restored.words, result.words);
  assert.deepEqual(restored.cards, result.cards);
  assert.deepEqual(Buffer.from(restored.audio), savedAudio);
  await reopened.getByRole('button', { name: '生词本', exact: true }).click();
  await reopened.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await expect(reopened.getByText('听音卡：语音已生成', { exact: true })).toBeVisible();
  console.log('Native UI: real playback, slow replay, hidden front, reveal, skip and restart recovery passed');
  console.log('Native IPC: corrupt cache, concurrent generation, failed and successful regeneration passed');
} finally {
  await stop();
}
