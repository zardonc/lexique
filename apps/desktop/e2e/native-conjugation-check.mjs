/* global process, window, fetch, setTimeout, clearTimeout, console, performance */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile, rename, readdir, unlink, rmdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const [resources, binary, data, destination] = process.argv.slice(2).map((path) => resolve(path));
assert.ok(['issue-40', 'issue-41', 'issue-42', 'issue-43', 'issue-44', 'conjugation-refactor'].some((issue) => destination.startsWith(resolve('../../target/' + issue) + '\\')));
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
  const timings = await page.evaluate(async () => {
    const query = 'elles se sont abstenues', milliseconds = [];
    for (let index = 0; index < 6; index++) {
      const started = performance.now();
      const result = await window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key: query });
      milliseconds.push(performance.now() - started);
      if (result.tables.length !== 1 || result.tables[0].lemma !== 's’abstenir') throw new Error('Timing query did not resolve');
    }
    return { query, first_ms: milliseconds[0], warm_ms: milliseconds.slice(1), boundary: 'native IPC; first invocation and subsequent invocations, no OS cache flush' };
  });
  const results = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke, results = {};
    for (const key of ['avons aimé', 'aie aimé', 'je suis allée', 'en ayant aimé', 'j’ai abélisé', 'je suis monté', 'j’ai monté', 'j’habite', 'je hais', 'gésir', 'j’occirai', 'ayons chu', 'il est avenu', 'j’ai pu', 'aie occis', 'il stupéfait', 'aie stupéfait', 'aie forclos', 'j’abstiens', 'j’ai parvenu', 'je avons aimé', 'sommes', 'suis', 'paie', 'falloir', 'enfuir', 'voilà', 'SOMMES', "' OR 1=1 --"]) results[key] = await invoke('conjugation_lookup', { key });
    for (const key of ['', '\u0000', 'é'.repeat(257)]) {
      try { await invoke('conjugation_lookup', { key }); throw new Error('Invalid input accepted'); }
      catch (error) { if (error.code !== 'invalid_query') throw error; }
    }
    return results;
  });
  assert.deepEqual(results.sommes.tables.map((table) => table.lemma), ['sommer', 'être']);
  assert.deepEqual(results.sommes.tables[1].meanings.senses[0].glosses, ['be']);
  assert.deepEqual(results['j’ai abélisé'].tables[0].meanings, { status: 'unavailable', senses: [] });
  assert.equal(results.suis.tables.flatMap((table) => table.cells.filter((cell) => cell.form === 'suis')).length, 4);
  const payer = results.paie.tables[0];
  assert.deepEqual(payer.cells.filter((cell) => cell.tense === 'P' && cell.person === '1' && cell.number === 's').map((cell) => cell.form), ['paie', 'paye']);
  assert.deepEqual(results['SOMMES'].tables, results.sommes.tables);
  assert.equal(results["' OR 1=1 --"].tables.length, 0);
  await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('suis'); await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.getByRole('button', { name: /^suivre/ })).toContainText('匹配单元格: 3');
  await expect(page.getByRole('button', { name: /^suivre/ })).toContainText('act on, act upon, follow, observe');
  await page.getByRole('button', { name: /^suivre/ }).click();
  await expect(page.locator('.conjugation-table .conjugation-meaning')).toHaveText('act on, act upon, follow, observe');
  await expect(page.locator('.conjugation-block')).toHaveCount(23);
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(3);
  await page.screenshot({ path: join(destination, 'conjugation.png'), fullPage: true });
  await page.screenshot({ path: join(destination, 'meanings.png') });
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
  for (const query of ['j’abstiens', 'j’ai parvenu']) assert.equal(results[query].tables.length, 0);
  assert.deepEqual(results['j’ai pu'].tables.map((table) => table.lemma), ['paître', 'pouvoir']);
  for (const [query, lemma] of [['j’ai abélisé', 'abéliser'], ['je suis monté', 'monter'], ['j’ai monté', 'monter'], ['j’habite', 'habiter'], ['je hais', 'haïr'], ['j’occirai', 'occire'], ['ayons chu', 'choir'], ['il est avenu', 'avenir'], ['aie occis', 'occire'], ['il stupéfait', 'stupéfaire'], ['aie stupéfait', 'stupéfaire'], ['aie forclos', 'forclore']]) {
    assert.ok(results[query].tables.some((table) => table.lemma === lemma));
    await page.getByLabel('法语动词或简单变位词形').fill(query);
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('.conjugation-expression[data-matched=true]').filter({ hasText: new RegExp(`^${query}$`, 'u') }).first()).toHaveText(query);
  }
  await page.getByLabel('法语动词或简单变位词形').fill('gésir');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  for (const code of ['PC', 'J', 'YP']) await expect(page.locator(`[data-tense=${code}]`)).toContainText('不适用（已验证）');
  await page.screenshot({ path: join(destination, 'ordinary.png'), fullPage: true });
  await page.getByLabel('法语动词或简单变位词形').fill('abstenir');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.locator('.conjugation-meaning')).toHaveText("s'abstenir · abstain");
  for (const [query, lemma] of [['se laver', 'se laver'], ['elles se sont abstenues', 's’abstenir'],
    ['elles se sont parlé', 'se parler'], ['va-t’en', 's’en aller']]) {
    const result = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }), query);
    results[query] = result;
    assert.deepEqual(result.tables.map((table) => table.lemma), [lemma]);
    await page.getByLabel('法语动词或简单变位词形').fill(query);
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('.conjugation-table h3')).toHaveText(lemma);
    await expect(page.locator('.conjugation-block')).toHaveCount(23);
    await expect(page.locator('.conjugation-expression[data-matched=true]').filter({ hasText: new RegExp(`^${query}$`, 'u') }).first()).toHaveText(query);
    if (query === 'se laver') {
      await expect(page.locator('.conjugation-table')).toContainText('代词式配合取决于语境');
      await expect(page.locator('[data-tense=Y]')).toContainText('lave-toi');
      await page.screenshot({ path: join(destination, 'pronominal.png') });
    }
    if (query === 'elles se sont abstenues') {
      await expect(page.locator('[data-tense=PC]')).toContainText('vous vous êtes abstenue');
      await expect(page.locator('[data-tense=PC]')).toContainText('vous vous êtes abstenues');
      await expect(page.locator('.conjugation-meaning')).toContainText('abstain');
    }
    if (query === 'elles se sont parlé') await expect(page.locator('[data-tense=PC]')).not.toContainText('parlées');
  }
  for (const [query, lemmas, expression, count] of [
    [' SOMMES ', ['sommer', 'être'], 'sommes', 2],
    [' ABSTENUES ', ['abstenir', 's’abstenir'], 'abstenues', 1],
    ['\u00a0E\u0302TRE\u202f', ['être'], 'être', 1],
    ["  J'AI  ABE\u0301LISE\u0301  ", ['abéliser'], 'j’ai abélisé', 1],
    [" QUE  J'AIME ", ['aimer'], 'que j’aime', 1],
    [' ELLES\u00a0SE  SONT ABSTENUES ', ['s’abstenir'], 'elles se sont abstenues', 1],
    [" VA-T'EN ", ['s’en aller'], 'va-t’en', 1],
    [" S'ABSTENIR ", ['s’abstenir'], 's’abstenir', 1],
    ['V’LÀ', ['voilà'], "v'là", 1],
  ]) {
    const result = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }), query);
    results[query] = result;
    assert.equal(result.query, query);
    assert.deepEqual(result.tables.map((table) => table.lemma), lemmas);
    await page.getByLabel('法语动词或简单变位词形').fill(query);
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.getByLabel('法语动词或简单变位词形')).toHaveValue(query);
    if (lemmas.length > 1) await page.getByRole('button', { name: new RegExp(`^${lemmas[0]}`, 'u') }).click();
    await expect(page.locator('.conjugation-table h3')).toHaveText(lemmas[0]);
    await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(count);
    if (/[ ’']/u.test(expression)) await expect(page.locator('.conjugation-expression[data-matched=true]')).toHaveText([expression]);
  }
  await page.screenshot({ path: join(destination, 'typographic.png') });
  for (const query of ['ETE', " J'AI AIME ", ' JE AVONS AIMÉ ', ' JE SE LAVE ', 'ELLES SE SONT ABSTENUS', ' AIMÉ AVONS ']) {
    const result = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }), query);
    results[query] = result;
    assert.equal(result.query, query); assert.equal(result.tables.length, 0);
    await page.getByLabel('法语动词或简单变位词形').fill(query);
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.getByLabel('法语动词或简单变位词形')).toHaveValue(query);
    await expect(page.locator('.conjugation').getByRole('status')).toBeVisible();
    await expect(page.locator('.conjugation-table')).toHaveCount(0);
  }
  await page.getByLabel('法语动词或简单变位词形').fill('suis');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await failures(page);
  assert.deepEqual(await snapshot(page), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  return { before, audio, results, timings };
}
async function failures(page) {
  const query = '  SUIS  ';
  await page.getByLabel('法语动词或简单变位词形').fill(query);
  const database = join(destination, 'conjugation/conjugation.sqlite3'), saved = database + '.saved';
  await rename(database, saved);
  try {
    const error = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }).catch((error) => error), query);
    assert.equal(error.code, 'missing');
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('.conjugation')).toContainText('内置变位资源不存在');
    await expect(page.getByLabel('法语动词或简单变位词形')).toHaveValue(query);
    await writeFile(database, 'invalid SQLite');
    try {
      const corrupt = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }).catch((error) => error), query);
      assert.equal(corrupt.code, 'corrupt');
      await page.getByRole('button', { name: '查询变位', exact: true }).click();
      await expect(page.locator('.conjugation')).toContainText('内置变位资源已损坏');
    } finally { await unlink(database); }
    await mkdir(database);
    try {
      const failed = await page.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }).catch((error) => error), query);
      assert.equal(failed.code, 'query_failed');
      await page.getByRole('button', { name: '查询变位', exact: true }).click();
      await expect(page.locator('.conjugation').getByRole('alert')).toBeVisible();
      await expect(page.getByLabel('法语动词或简单变位词形')).toHaveValue(query);
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
  const page = await start(), { before, audio, results, timings } = await check(page);
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('finish_close')).catch((error) => {
    assert.match(String(error), /closed|Target|context|session|Frame|destroy/i);
  });
  for (let i = 0; child.exitCode === null && i < 50; i++) await delay(100);
  assert.notEqual(child.exitCode, null, 'Normal close must terminate the application');
  await browser.close(); browser = undefined;
  const restarted = await start();
  const restartedQuery = " VA-T'EN ";
  const restartedResult = await restarted.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }), restartedQuery);
  assert.equal(restartedResult.query, restartedQuery);
  assert.deepEqual(restartedResult.tables.map((table) => table.lemma), ['s’en aller']);
  assert.deepEqual(await snapshot(restarted), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  await writeFile(join(destination, 'report.json'), JSON.stringify({ status: 'PASS', restart: true, normalized_restart: true, results, timings,
    preserved: { records: before.records.length, cards: before.cards.flat().length, audioFiles: Object.keys(audio).length } }, null, 2));
  console.log('Native conjugation check: PASS');
} finally {
  clearTimeout(timer); if (browser) await browser.close();
  if (child?.pid && child.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
}
