/* global process, window, fetch, console, HTMLMediaElement */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, rename, stat, readFile, writeFile } from 'node:fs/promises';
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
function sql(statement) {
  const result = spawnSync(process.env.LEXIQUE_TEST_PYTHON ?? 'python', ['-c',
    'import sqlite3,sys,json\nwith sqlite3.connect(sys.argv[1]) as c:\n print(json.dumps(c.execute(sys.stdin.read()).fetchall()))',
    join(directory, 'Lexique-data', 'data', 'lexique.sqlite3')], { input: statement, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
async function snapshot(page, id) {
  return page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const cards = await invoke('review_cards', { vocabularyId: id });
    return { cards, progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))),
      queue: await invoke('review_queue', { timezone: Intl.DateTimeFormat().resolvedOptions().timeZone, skipped: [] }),
      target: await invoke('review_undo_target') };
  }, id);
}
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
  try {
    if (browser) await browser.close();
  } finally {
    browser = undefined;
    if (child?.pid && child.exitCode === null) spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
    child = undefined;
    await delay(500);
  }
}
try {
  let page = await start();
  await page.evaluate(() => {
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      this.addEventListener('ended', () => { window.testWordAudioEnded = true; }, { once: true });
      return play.call(this);
    };
  });
  const wordsBeforePreview = sql('SELECT * FROM vocabulary ORDER BY id');
  for (const text of ['', 'a'.repeat(201), 'bad\nword']) {
    await assert.rejects(page.evaluate((text) => window.__TAURI_INTERNALS__.invoke('speech_preview', { text }), text));
  }
  await page.getByLabel('法语词或词组').fill('bonjour');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: /播放单词: bonjour/ }).click();
  await expect(page.getByRole('button', { name: /播放单词: bonjour/ })).toBeEnabled({ timeout: 30000 });
  assert.equal(await page.getByRole('alert').count(), 0, await page.getByRole('alert').allTextContents());
  await expect.poll(() => page.evaluate(() => window.testWordAudioEnded), { timeout: 30000 }).toBe(true);
  assert.deepEqual(sql('SELECT * FROM vocabulary ORDER BY id'), wordsBeforePreview);
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
  const queueCard = page.getByRole('region', { name: '复习卡片' });
  await expect(queueCard.getByRole('heading', { name: 'école' })).toBeVisible();
  await page.screenshot({ path: join(directory, 'unified-queue.png') });
  await queueCard.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(queueCard.locator('audio')).toBeAttached();
  assert.deepEqual((await snapshot(page, before.word.id)).progress, before.progress);
  // Corrupt only the isolated cache, keeping the real native IPC and decoder.
  const cache = join(directory, 'Lexique-data', 'temp', 'speech', `${before.word.id}.ogg`);
  const audio = await readFile(cache);
  await writeFile(cache, 'OggS invalid playback fixture');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await queueCard.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(queueCard.getByRole('alert')).toContainText('播放失败');
  await queueCard.getByRole('button', { name: '显示答案' }).click();
  await expect(queueCard.getByRole('button', { name: '记得', exact: true })).toBeDisabled();
  await queueCard.getByRole('button', { name: '跳过', exact: true }).click();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await page.getByLabel('法语例句').fill('Une phrase.');
  const saveActions = page.getByRole('button', { name: '保存', exact: true }).locator('..');
  await expect(saveActions.getByRole('alert')).toContainText('请为新增或修改的例句选择释义');
  await page.getByLabel('例句对应释义').selectOption('0');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('status').filter({ hasText: '已保存' }).waitFor();
  await page.evaluate(() => { window.testWordAudioEnded = false; });
  await page.getByRole('button', { name: /播放单词: école/ }).click();
  await expect.poll(() => page.evaluate(() => window.testWordAudioEnded), { timeout: 30000 }).toBe(true);
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const listening = page.getByRole('dialog', { name: '听音卡' });
  await expect(listening.getByRole('alert')).toContainText('播放失败');
  await listening.getByRole('button', { name: '显示答案' }).click();
  await expect(listening.getByRole('button', { name: '记得', exact: true })).toBeDisabled();
  assert.deepEqual((await snapshot(page, before.word.id)).progress, before.progress);
  await page.screenshot({ path: join(directory, 'playback-failure.png') });
  await listening.getByRole('button', { name: '跳过', exact: true }).click();
  await writeFile(cache, audio);
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  await expect.poll(() => listening.locator('audio').evaluate((audio) => audio.currentTime)).toBeGreaterThan(0);
  await expect.poll(() => listening.locator('audio').evaluate((audio) => audio.ended), { timeout: 20_000 }).toBe(true);
  await expect(listening.getByRole('alert')).toHaveCount(0);
  await page.screenshot({ path: join(directory, 'listening-played.png') });
  await listening.getByRole('button', { name: '显示答案' }).click();
  await listening.getByRole('button', { name: '记得', exact: true }).click();
  await listening.waitFor({ state: 'hidden' });
  const listeningRated = await snapshot(page, before.word.id);
  assert.deepEqual(listeningRated.progress[0], before.progress[0]);
  assert.equal(listeningRated.progress[1].revision, before.progress[1].revision + 1);
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '撤销最近评分' }).click();
  await expect.poll(() => page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_undo_target'))).toBeNull();
  const afterListeningUndo = await snapshot(page, before.word.id);
  assert.deepEqual(afterListeningUndo.progress.map((item) => item.card), before.progress.map((item) => item.card));
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await queueCard.getByRole('button', { name: '显示答案' }).click();
  const ratingBefore = await snapshot(page, before.word.id);
  const ratingsBefore = sql('SELECT * FROM review_ratings ORDER BY rowid');
  sql("CREATE TRIGGER issue22_fail_rating BEFORE INSERT ON review_ratings BEGIN SELECT RAISE(ABORT, 'issue22 rating write failure'); END");
  await queueCard.getByRole('button', { name: '记得', exact: true }).click();
  const recognition = queueCard;
  await expect(recognition.getByRole('alert')).toContainText('issue22 rating write failure');
  assert.deepEqual(await snapshot(page, before.word.id), ratingBefore);
  assert.deepEqual(sql('SELECT * FROM review_ratings ORDER BY rowid'), ratingsBefore);
  await page.screenshot({ path: join(directory, 'rating-failure.png') });
  sql('DROP TRIGGER issue22_fail_rating');
  await recognition.getByRole('button', { name: '重试', exact: true }).click();
  await page.waitForFunction(async () => Boolean(await window.__TAURI_INTERNALS__.invoke('review_undo_target')));
  await expect(queueCard.locator('audio')).toBeAttached();
  await expect.poll(() => queueCard.locator('audio').evaluate((audio) => audio.ended), { timeout: 20_000 }).toBe(true);
  await queueCard.getByRole('button', { name: '跳过', exact: true }).click();
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await page.getByLabel('例句对应释义').selectOption('0');
  await page.getByLabel('法语例句').fill('Je vais à l’école.');
  await page.getByLabel('例句中文译文').fill('我去学校。');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('status').filter({ hasText: '已保存' }).waitFor();
  const changed = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const settings = { retention_percent: 87, new_limit: 7, review_limit: 13, new_cards_ignore_review_limit: true };
    await invoke('review_settings', { settings });
    await invoke('review_pause', { vocabularyId: id, paused: true });
    await invoke('review_auto_enroll', { enabled: true });
    const target = await invoke('review_undo_target');
    const rejected = await invoke('review_undo', { operation: 'm2-invalid-undo', ratingOperation: 'm2-invalid-rating' });
    return { settings, target, rejected, targetAfterRejection: await invoke('review_undo_target') };
  }, before.word.id);
  assert.ok(changed.target);
  assert.equal(changed.rejected.status, 'rejected');
  assert.equal(changed.targetAfterRejection, changed.target);
  // A real new collection after rating must survive undo, including its queue entries.
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByLabel('法语词或词组').fill('bonjour');
  await page.getByRole('button', { name: '查词', exact: true }).last().click();
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('status').filter({ hasText: '已保存' }).waitFor();
  const collected = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const word = await invoke('vocabulary_find', { headword: 'bonjour' });
    return { word, cards: await invoke('review_cards', { vocabularyId: word.id }) };
  });
  assert.equal(collected.cards.length, 2);
  // Cached école succeeds while a missing runtime fails bonjour in the same UI batch.
  await expect(page.getByText('听音卡：语音已生成', { exact: true })).toBeVisible({ timeout: 30_000 });
  const bonjourCache = join(directory, 'Lexique-data', 'temp', 'speech', `${collected.word.id}.ogg`);
  await rename(bonjourCache, `${bonjourCache}.saved`);
  const python = join(directory, 'speech', 'runtime', 'python.exe');
  await rename(python, `${python}.disabled`);
  try {
    await page.getByRole('button', { name: '生词本', exact: true }).click();
    await page.getByLabel('选择 école', { exact: true }).check();
    await page.getByLabel('选择 bonjour', { exact: true }).check();
    const batchBefore = await snapshot(page, before.word.id);
    await page.getByRole('button', { name: '批量加入复习', exact: true }).click();
    await expect(page.getByText('école：成功', { exact: true })).toBeVisible();
    await expect(page.getByText('bonjour：卡片已加入，语音失败', { exact: true })).toBeVisible();
    assert.deepEqual(await snapshot(page, before.word.id), batchBefore);
    await page.screenshot({ path: join(directory, 'batch-partial-failure.png') });
  } finally { await rename(`${python}.disabled`, python); }
  await page.getByRole('button', { name: '重试 bonjour', exact: true }).click();
  await expect(page.getByText('bonjour：成功', { exact: true })).toBeVisible({ timeout: 30_000 });
  assert.deepEqual((await snapshot(page, collected.word.id)).cards, collected.cards);
  const beforeMove = await snapshot(page, before.word.id);
  const quotaBeforeMove = sql('SELECT quota, count(*) FROM review_ratings WHERE undo_operation IS NULL GROUP BY quota');
  await stop();
  await rename(destination, moved);
  directory = moved;
  page = await start();
  const restored = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    return { words: await invoke('vocabulary_list', { query: '', sort: 'headword-asc' }),
      settings: await invoke('review_settings', { settings: null }),
      paused: await invoke('review_pause', { vocabularyId: id, paused: null }),
      autoEnroll: await invoke('review_auto_enroll', { enabled: null }),
      target: await invoke('review_undo_target'), audio: await invoke('speech_cached', { vocabularyId: id }) };
  }, before.word.id);
  assert.deepEqual(restored.settings, changed.settings);
  assert.equal(restored.paused, true);
  assert.equal(restored.autoEnroll, true);
  assert.equal(restored.target, changed.target);
  assert.deepEqual(restored.audio, before.audio);
  assert.deepEqual(await snapshot(page, before.word.id), beforeMove);
  assert.deepEqual(sql('SELECT quota, count(*) FROM review_ratings WHERE undo_operation IS NULL GROUP BY quota'), quotaBeforeMove);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const ignoreReviewLimit = page.getByLabel('新卡不受复习上限限制');
  await expect(ignoreReviewLimit).toBeChecked();
  await ignoreReviewLimit.uncheck();
  await page.getByRole('button', { name: '保存复习设置' }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  assert.equal(await page.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('review_settings', { settings: null })).new_cards_ignore_review_limit), false);
  await ignoreReviewLimit.check();
  await page.getByRole('button', { name: '保存复习设置' }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undoBefore = await snapshot(page, before.word.id);
  const undoRatingsBefore = sql('SELECT * FROM review_ratings ORDER BY rowid');
  sql("CREATE TRIGGER issue22_fail_undo BEFORE UPDATE OF undo_operation ON review_ratings BEGIN SELECT RAISE(ABORT, 'issue22 undo write failure'); END");
  await page.getByRole('button', { name: '撤销最近评分' }).click();
  await expect(page.getByRole('alert')).toContainText(/撤销失败:.*issue22 undo write failure/);
  assert.deepEqual(await snapshot(page, before.word.id), undoBefore);
  assert.deepEqual(sql('SELECT * FROM review_ratings ORDER BY rowid'), undoRatingsBefore);
  await page.screenshot({ path: join(directory, 'undo-failure.png') });
  sql('DROP TRIGGER issue22_fail_undo');
  await page.getByRole('button', { name: '撤销最近评分' }).click();
  await expect.poll(() => page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_undo_target'))).toBeNull();
  const after = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const words = await invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
    const cards = await invoke('review_cards', { vocabularyId: id });
    return { word: words.find((item) => item.id === id), settings: await invoke('review_settings', { settings: null }),
      paused: await invoke('review_pause', { vocabularyId: id, paused: null }),
      autoEnroll: await invoke('review_auto_enroll', { enabled: null }),
      progress: await Promise.all(cards.map((card) => invoke('review_progress', { cardId: card.id }))) };
  }, before.word.id);
  assert.equal(after.word.example_fr, 'Je vais à l’école.');
  assert.equal(after.word.example_zh, '我去学校。');
  assert.deepEqual(after.settings, changed.settings);
  assert.equal(after.paused, true);
  assert.equal(after.autoEnroll, true);
  assert.deepEqual(after.progress.map((item) => item.card), before.progress.map((item) => item.card));
  assert.ok(after.progress.every((item, index) => item.revision >= before.progress[index].revision));
  assert.deepEqual((await snapshot(page, collected.word.id)).cards, collected.cards);
  const finalQueue = (await snapshot(page, collected.word.id)).queue.queue.cards;
  assert.ok(finalQueue.some((card) => card.vocabulary_id === collected.word.id));
  assert.deepEqual(sql('SELECT quota, count(*) FROM review_ratings WHERE undo_operation IS NULL GROUP BY quota'), []);
  const finalCard = page.getByRole('region', { name: '复习卡片' });
  await expect(finalCard.getByRole('heading', { name: 'bonjour' })).toBeVisible();
  await finalCard.getByRole('button', { name: '显示答案' }).click();
  await finalCard.getByRole('button', { name: '记得', exact: true }).click();
  await expect(finalCard.locator('audio')).toBeAttached();
  await expect.poll(() => finalCard.locator('audio').evaluate((element) => element.ended), { timeout: 20_000 }).toBe(true);
  await finalCard.getByRole('button', { name: '显示答案' }).click();
  await finalCard.getByRole('button', { name: '记得', exact: true }).click();
  await expect.poll(() => sql("SELECT count(*) FROM review_ratings WHERE undo_operation IS NULL AND quota = 'new'")[0][0]).toBe(2);
  await writeFile(join(directory, 'native-m2-results.json'), JSON.stringify({
    listeningRated, ratingBefore, collected, beforeMove, quotaBeforeMove, restoredSettings: restored.settings,
    after, finalQueue, checks: ['playback failure blocks rating', 'real playback and independent listening rating',
      'rating write failure rolls back and UI retry commits', 'partial batch failure and successful retry',
      'portable move preserves queue and consumed quota', 'undo write failure rolls back and UI retry commits',
      'undo preserves actual later collection and auto-enrolled queue'],
  }, null, 2));
  console.log('Native rating, independent edits, portable move and undo preserved their state');
} finally { await stop(); }
