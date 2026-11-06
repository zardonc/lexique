/* global process, window, fetch, setTimeout, clearTimeout, console */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile, rename, readdir, unlink, rmdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const [resources, binary, data, destination] = process.argv.slice(2).map((path) => resolve(path));
assert.ok(['issue-40', 'issue-41'].some((issue) => destination.startsWith(resolve('../../target/' + issue) + '\\')));
await mkdir(destination);
for (const name of ['dictionary', 'conjugation']) await cp(join(resources, name), join(destination, name), { recursive: true });
await cp(binary, join(destination, 'Lexique.exe'));
for (const name of ['data', 'temp/speech']) await cp(join(data, name), join(destination, 'Lexique-data', name), { recursive: true });
const server = createServer(); await new Promise((done) => server.listen(0, '127.0.0.1', done));
const port = server.address().port; await new Promise((done) => server.close(done));
let child, browser;
const timer = setTimeout(() => {
  if (child?.pid) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
  console.error('Native conjugation check exceeded 60 seconds'); process.exit(1);
}, 60_000);
async function snapshot(page) {
  return page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const records = await invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
    const cards = await Promise.all(records.map((record) => invoke('review_cards', { vocabularyId: record.id })));
    const progress = await Promise.all(cards.flat().map((card) => invoke('review_progress', { cardId: card.id })));
    const audio = await Promise.all(records.map((record) => invoke('speech_cached', { vocabularyId: record.id })));
    return { records, cards, progress, audio, preferences: await invoke('speech_preferences', { preferences: null }) };
  });
}
async function hashes(directory) {
  const result = {};
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) Object.assign(result, await hashes(path));
    else if (/\.(ogg|opus|wav)$/i.test(path)) result[path] = createHash('sha256').update(await readFile(path)).digest('hex');
  }
  return result;
}
async function check(page) {
  const before = await snapshot(page), audio = await hashes(join(destination, 'Lexique-data/temp/speech'));
  assert.ok(before.records.length && before.cards.flat().length && before.audio.some((bytes) => bytes?.length) && Object.keys(audio).length, 'Use an existing learning/audio fixture');
  const results = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke, results = {};
    for (const key of ['avons aimé', 'aie aimé', 'je suis allée', 'en ayant aimé', 'je avons aimé', 'sommes', 'suis', 'paie', 'falloir', 'enfuir', 'voilà', 'SOMMES', "' OR 1=1 --"]) results[key] = await invoke('conjugation_lookup', { key });
    for (const key of ['', '\u0000', 'é'.repeat(257)]) {
      try { await invoke('conjugation_lookup', { key }); throw new Error('Invalid input accepted'); }
      catch (error) { if (error.code !== 'invalid_query') throw error; }
    }
    return results;
  });
  assert.deepEqual(results.sommes.tables.map((table) => table.lemma), ['sommer', 'être']);
  assert.equal(results.suis.tables.flatMap((table) => table.cells.filter((cell) => cell.form === 'suis')).length, 4);
  const payer = results.paie.tables[0];
  assert.deepEqual(payer.cells.filter((cell) => cell.tense === 'P' && cell.person === '1' && cell.number === 's').map((cell) => cell.form), ['paie', 'paye']);
  assert.equal(results['SOMMES'].tables.length, 0);
  assert.equal(results["' OR 1=1 --"].tables.length, 0);
  await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('suis'); await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.getByRole('button', { name: /^suivre/ })).toContainText('匹配单元格: 3');
  await page.getByRole('button', { name: /^suivre/ }).click();
  await expect(page.locator('.conjugation-block')).toHaveCount(23);
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(3);
  await page.screenshot({ path: join(destination, 'conjugation.png'), fullPage: true });
  for (const [query, count] of [['aie aimé', 2], ['je suis allée', 1]]) {
    await page.getByLabel('法语动词或简单变位词形').fill(query);
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(count);
    if (query === 'aie aimé') {
      for (const [code, form] of [['KP', 'ayant aimé'], ['GP', 'en aimant'], ['GPP', 'en ayant aimé']]) {
        await expect(page.locator(`[data-tense=${code}]`)).toContainText(form);
      }
    } else {
      for (const form of ['je suis allé', 'je suis allée']) {
        await expect(page.locator('[data-tense=PC]').getByText(form, { exact: true })).toBeVisible();
      }
    }
  }
  assert.equal(results['je avons aimé'].tables.length, 0);
  assert.ok(results['en ayant aimé'].tables.some((table) => table.lemma === 'aimer'));
  await page.screenshot({ path: join(destination, 'ordinary.png'), fullPage: true });
  await page.getByLabel('法语动词或简单变位词形').fill('suis');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await failures(page);
  assert.deepEqual(await snapshot(page), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  return { before, audio, results };
}
async function failures(page) {
  const database = join(destination, 'conjugation/conjugation.sqlite3'), saved = database + '.saved';
  await rename(database, saved);
  try {
    const error = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key: 'suis' }).catch((error) => error));
    assert.equal(error.code, 'missing');
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('.conjugation')).toContainText('内置变位资源不存在');
    await expect(page.getByLabel('法语动词或简单变位词形')).toHaveValue('suis');
    await writeFile(database, 'invalid SQLite');
    try {
      const corrupt = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key: 'suis' }).catch((error) => error));
      assert.equal(corrupt.code, 'corrupt');
      await page.getByRole('button', { name: '查询变位', exact: true }).click();
      await expect(page.locator('.conjugation')).toContainText('内置变位资源已损坏');
    } finally { await unlink(database); }
    await mkdir(database);
    try {
      const failed = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key: 'suis' }).catch((error) => error));
      assert.equal(failed.code, 'query_failed');
    } finally { await rmdir(database); }
  } finally { await rename(saved, database); }
}
async function start() {
  child = spawn(join(destination, 'Lexique.exe'), [], { cwd: destination, windowsHide: true,
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --remote-debugging-port=${port}` } });
  child.stderr.on('data', (bytes) => console.error(bytes.toString()));
  let endpoint;
  for (let i = 0; i < 80 && !endpoint; i++) {
    try { endpoint = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl; } catch { await delay(100); }
  }
  assert.ok(endpoint, 'Owned WebView2 endpoint unavailable'); browser = await chromium.connectOverCDP(endpoint);
  let page;
  for (let i = 0; i < 80; i++) {
    page = browser.contexts().flatMap((context) => context.pages()).find((page) => !page.url().startsWith('about:'));
    if (page && await page.evaluate(() => Boolean(window.__TAURI_INTERNALS__) && window.location.origin !== 'null').catch(() => false)) break;
    await delay(100);
  }
  assert.ok(page && await page.evaluate(() => Boolean(window.__TAURI_INTERNALS__) && window.location.origin !== 'null'), `Owned native page unavailable: ${page?.url()}`);
  return page;
}
try {
  const page = await start(), { before, audio, results } = await check(page);
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('finish_close')).catch((error) => {
    assert.match(String(error), /closed|Target|context|session|Frame|destroy/i);
  });
  for (let i = 0; child.exitCode === null && i < 50; i++) await delay(100);
  assert.notEqual(child.exitCode, null, 'Normal close must terminate the application');
  await browser.close(); browser = undefined;
  const restarted = await start();
  assert.deepEqual(await snapshot(restarted), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  await writeFile(join(destination, 'report.json'), JSON.stringify({ status: 'PASS', restart: true, results,
    preserved: { records: before.records.length, cards: before.cards.flat().length, audioFiles: Object.keys(audio).length } }, null, 2));
  console.log('Native conjugation check: PASS');
} finally {
  clearTimeout(timer); if (browser) await browser.close();
  if (child?.pid && child.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
}
