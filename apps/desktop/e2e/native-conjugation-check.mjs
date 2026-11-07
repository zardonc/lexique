/* global process, window, fetch, setTimeout, clearTimeout, console, performance */
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile, rename, readdir, unlink, rmdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

const [resources, binary, data, destination] = process.argv.slice(2, 6).map((path) => resolve(path));
const navigationOnly = process.argv[6] === '--navigation';
assert.ok(process.argv.length === 6 || (process.argv.length === 7 && navigationOnly), 'Four paths and optional --navigation are required');
assert.ok(['issue-40', 'issue-41', 'issue-42', 'issue-43', 'issue-44', 'issue-45', 'issue-46', 'issue-47', 'conjugation-refactor'].some((issue) => destination.startsWith(resolve('../../target/' + issue) + '\\')));
const defaultOrder = ['P', 'PC', 'I', 'F', 'C', 'S', 'PQP', 'J', 'PA', 'FA', 'CP', 'SP', 'T', 'SQP', 'Y', 'YP', 'W', 'WP', 'G', 'K', 'KP', 'GP', 'GPP'];
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
    const audio = await Promise.all(records.map((record, index) => cards[index].some((card) => card.kind === 'listening')
      ? invoke('speech_cached', { vocabularyId: record.id }) : null));
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
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode: 'ending' });
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), defaultOrder);
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
  const coloring = await checkColoring(page);
  await preferenceFailures(page);
  await failures(page);
  const blockOrder = await checkOrder(page);
  await orderFailures(page, blockOrder.custom);
  assert.deepEqual(await snapshot(page), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  return { before, audio, results, timings, coloring, blockOrder };
}
async function renderedOrder(page) {
  return page.locator('.conjugation-block').evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')));
}
async function checkOrder(page) {
  const custom = ['P', 'PC', 'I', 'F', 'C', 'S', 'J', 'PQP', 'PA', 'FA', 'CP', 'SP', 'T', 'SQP', 'Y', 'YP', 'W', 'WP', 'G', 'K', 'KP', 'GP', 'GPP'];
  const controls = page.locator('.conjugation-order');
  const input = page.getByLabel('法语动词或简单变位词形');
  await input.fill('nous avions lavé'); await input.press('Enter');
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  assert.deepEqual(await renderedOrder(page), defaultOrder);
  await controls.getByLabel('要移动的区块', { exact: true }).selectOption('PQP');
  const later = controls.getByRole('button', { name: '向后移动', exact: true });
  await later.focus(); await later.press('Space');
  await expect(page.locator('.conjugation-block').nth(7)).toHaveAttribute('data-tense', 'PQP');
  await expect(later).toBeFocused();
  assert.deepEqual(await renderedOrder(page), custom);
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), custom);
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode: 'main' });
  const priority = await page.locator('.conjugation-block[data-priority=true]').evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')));
  assert.deepEqual(priority, ['P', 'PC', 'I', 'F', 'C', 'S', 'PQP']);
  const match = page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true]');
  await expect(match).toHaveText(['nous avions lavé']);
  await expect(match.locator('.conjugation-colored')).toHaveText(['lavé']);
  await expect(match).toHaveCSS('text-decoration-line', 'underline');
  for (const query of ['falloir', 'se laver']) {
    await input.fill(query); await input.press('Enter');
    await expect(page.locator('.conjugation-table h3')).toHaveText(query);
    await expect(page.locator('.conjugation-block')).toHaveCount(23);
    assert.deepEqual(await renderedOrder(page), custom);
    if (query === 'falloir') await expect(page.locator('[data-tense=Y]')).toContainText('不适用（已验证）');
    await expect(page.getByRole('button', { name: '2 · 主要动词', exact: true })).toHaveAttribute('aria-pressed', 'true');
  }
  await page.screenshot({ path: join(destination, 'block-order.png') });
  return { custom, priority, verbs: ['laver', 'falloir', 'se laver'], keyboard: true };
}
async function orderFailures(page, custom) {
  const controls = page.locator('.conjugation-order');
  const reset = controls.getByRole('button', { name: '重置区块顺序', exact: true });
  preferenceSql(`c.execute("CREATE TRIGGER issue46_order_write_failure BEFORE INSERT ON app_metadata WHEN NEW.key='conjugation-block-order' BEGIN SELECT RAISE(ABORT, 'Block order storage is read-only'); END")`);
  try {
    await reset.click();
    await expect(controls.getByRole('alert')).toContainText('read-only');
    assert.deepEqual(await renderedOrder(page), custom);
    assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), custom);
  } finally { preferenceSql(`c.execute('DROP TRIGGER issue46_order_write_failure')`); }
  for (const order of [defaultOrder.slice(1), ['P', ...defaultOrder.slice(1, -1), 'P'],
    [...defaultOrder.slice(0, -1), 'unknown'], [...defaultOrder.slice(0, -1), 23], { order: defaultOrder }]) {
    const error = await page.evaluate((order) => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order }).catch(String), order);
    assert.match(error, /23 tense blocks|invalid|expected/i);
  }
  for (const saved of ['not-json', '[]']) {
    preferenceSql(`c.execute("UPDATE app_metadata SET value=? WHERE key='conjugation-block-order'", (${JSON.stringify(saved)},))`);
    try {
      const error = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null }).catch(String));
      assert.match(error, /Invalid saved conjugation block order/);
      await page.reload(); await page.getByRole('button', { name: '动词变位', exact: true }).click();
      await expect(controls.getByRole('alert')).toContainText('Invalid saved conjugation block order');
      await expect(controls.getByRole('combobox')).toBeDisabled();
      await expect(reset).toBeDisabled();
      await expect(page.getByRole('button', { name: '2 · 主要动词', exact: true })).toHaveAttribute('aria-pressed', 'true');
    } finally {
      preferenceSql(`c.execute("UPDATE app_metadata SET value=? WHERE key='conjugation-block-order'", (json.dumps(${JSON.stringify(custom)}),))`);
    }
    await controls.getByRole('button', { name: '重新读取区块顺序', exact: true }).click();
    await expect(controls.getByRole('combobox')).toBeEnabled();
    await expect(controls.getByRole('alert')).toHaveCount(0);
  }
  preferenceSql(`c.execute('ALTER TABLE app_metadata RENAME TO issue46_unavailable_preferences')`);
  try {
    await reset.click();
    await expect(controls.getByRole('alert')).toContainText('no such table: app_metadata');
    await page.reload(); await page.getByRole('button', { name: '动词变位', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText('no such table: app_metadata');
    await expect(page.locator('.conjugation-coloring').getByRole('alert')).toContainText('no such table: app_metadata');
    await expect(reset).toBeDisabled();
  } finally { preferenceSql(`c.execute('ALTER TABLE issue46_unavailable_preferences RENAME TO app_metadata')`); }
  await controls.getByRole('button', { name: '重新读取区块顺序', exact: true }).click();
  await page.locator('.conjugation-coloring').getByRole('button', { name: '重新读取着色偏好', exact: true }).click();
  await expect(controls.getByRole('alert')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '2 · 主要动词', exact: true })).toHaveAttribute('aria-pressed', 'true');
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), custom);
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode: 'main' });
}
async function checkColoring(page) {
  const input = page.getByLabel('法语动词或简单变位词形');
  const modes = ['1 · 完整动词部分', '2 · 主要动词', '3 · 主要动词词尾'];
  const checks = [];
  for (const [query, expected] of [
    ['nous avions lavé', [['avions lavé'], ['lavé'], ['é']]],
    ['lavais', [['lavais'], ['lavais'], ['ais']]],
    ['sois-toi lavé', [['sois', 'lavé'], ['lavé'], ['é']]],
    ['en se lavant', [['lavant'], ['lavant'], ['ant']]],
    ['va-t’en', [['va'], ['va'], []]],
  ]) {
    await input.fill(query); await input.press('Enter');
    const match = page.locator('.conjugation-expression[data-matched=true]').first();
    await expect(match).toHaveText(query === 'lavais' ? 'je lavais' : query);
    for (let index = 0; index < modes.length; index++) {
      const choice = page.getByRole('button', { name: modes[index], exact: true });
      await choice.focus(); await choice.press('Space');
      await expect(choice).toHaveAttribute('aria-pressed', 'true');
      await expect(match.locator('.conjugation-colored')).toHaveText(expected[index]);
      await expect(match).toHaveCSS('text-decoration-line', 'underline');
      await expect(page.locator('.conjugation-block[data-priority=true]')).toHaveCount(7);
    }
    if (query === 'va-t’en') await expect(page.locator('.conjugation-table .conjugation-colored')).toHaveCount(0);
    checks.push({ query, verbal: expected[0], main: expected[1], ending: expected[2] });
  }
  await input.fill('se laver'); await input.press('Enter');
  const variants = page.locator('[data-tense=PC] [data-person="1s"] .conjugation-expression');
  await expect(variants).toHaveText(['je me suis lavée', 'je me suis lavées', 'je me suis lavé', 'je me suis lavés']);
  await expect(variants.locator('.conjugation-colored')).toHaveText(['ée', 'ées', 'é', 'és']);
  await page.getByRole('button', { name: modes[1], exact: true }).click();
  await input.fill('finir'); await input.press('Enter');
  await expect(page.locator('.conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText(['finir']);
  await page.screenshot({ path: join(destination, 'coloring.png') });
  return checks;
}
function preferenceSql(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,sys,json\nc=sqlite3.connect(sys.argv[1])\n${source}\nc.commit()`,
    join(destination, 'Lexique-data/data/lexique.sqlite3')], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
  assert.equal(result.status, 0, result.stderr);
}
async function preferenceFailures(page) {
  const controls = page.locator('.conjugation-coloring');
  const main = page.getByRole('button', { name: '2 · 主要动词', exact: true });
  preferenceSql(`c.execute("CREATE TRIGGER issue45_preference_write_failure BEFORE INSERT ON app_metadata WHEN NEW.key='conjugation-preferences' BEGIN SELECT RAISE(ABORT, 'Coloring preference storage is read-only'); END")`);
  try {
    await page.getByRole('button', { name: '1 · 完整动词部分', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText('read-only');
    await expect(main).toHaveAttribute('aria-pressed', 'true');
    assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode: 'main' });
  } finally { preferenceSql(`c.execute('DROP TRIGGER issue45_preference_write_failure')`); }
  for (const preferences of [{ mode: 'invalid' }, { mode: 'main', unknown: true }]) {
    const error = await page.evaluate((preferences) => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences }).catch(String), preferences);
    assert.match(error, /invalid|unknown|expected/i);
  }
  preferenceSql(`c.execute("UPDATE app_metadata SET value=? WHERE key='conjugation-preferences'", ('{"mode":"broken"}',))`);
  try {
    const error = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null }).catch(String));
    assert.match(error, /Invalid saved conjugation preferences/);
    await page.reload(); await page.getByRole('button', { name: '动词变位', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText('Invalid saved conjugation preferences');
    await expect(main).toBeDisabled();
  } finally { preferenceSql(`c.execute("UPDATE app_metadata SET value=? WHERE key='conjugation-preferences'", ('{"mode":"main"}',))`); }
  await controls.getByRole('button', { name: '重新读取着色偏好', exact: true }).click();
  await expect(main).toHaveAttribute('aria-pressed', 'true');
  await expect(controls.getByRole('alert')).toHaveCount(0);
  preferenceSql(`c.execute('ALTER TABLE app_metadata RENAME TO issue45_unavailable_preferences')`);
  try {
    await page.getByRole('button', { name: '1 · 完整动词部分', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText('no such table: app_metadata');
    await expect(main).toHaveAttribute('aria-pressed', 'true');
    await page.reload(); await page.getByRole('button', { name: '动词变位', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText('no such table: app_metadata');
    await expect(page.locator('.conjugation-order').getByRole('alert')).toContainText('no such table: app_metadata');
    await expect(main).toBeDisabled();
  } finally { preferenceSql(`c.execute('ALTER TABLE issue45_unavailable_preferences RENAME TO app_metadata')`); }
  await controls.getByRole('button', { name: '重新读取着色偏好', exact: true }).click();
  await page.locator('.conjugation-order').getByRole('button', { name: '重新读取区块顺序', exact: true }).click();
  await expect(main).toHaveAttribute('aria-pressed', 'true');
  await expect(controls.getByRole('alert')).toHaveCount(0);
  await expect(page.locator('.conjugation-order').getByRole('alert')).toHaveCount(0);
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
async function close(page) {
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('finish_close')).catch((error) => {
    assert.match(String(error), /closed|Target|context|session|Frame|destroy/i);
  });
  for (let i = 0; child.exitCode === null && i < 50; i++) await delay(100);
  assert.notEqual(child.exitCode, null, 'Normal close must terminate the application');
  await browser.close(); browser = undefined;
}
async function dictionarySearch(page, key) {
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  const input = page.getByLabel('French word or phrase'); await input.fill(key); await input.press('Enter');
}
async function conjugationSearch(page, key) {
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  const input = page.getByLabel('French verb or simple form'); await input.fill(key); await input.press('Enter');
}
async function navigationBrowse(page) {
  await dictionarySearch(page, 'laver');
  const forward = page.getByRole('button', { name: 'Open conjugation: laver', exact: true });
  await forward.focus(); await forward.press('Enter');
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  await expect(page.locator('.conjugation-table h3')).toBeFocused();
  await expect(page.locator('.conjugation-block')).toHaveCount(23);
  await page.getByRole('button', { name: '2 · Main verb', exact: true }).click();
  await page.getByLabel('Block to move', { exact: true }).selectOption('PQP');
  await page.getByRole('button', { name: 'Move later', exact: true }).click();
  const custom = ['P', 'PC', 'I', 'F', 'C', 'S', 'J', 'PQP', ...defaultOrder.slice(8)];
  assert.deepEqual(await renderedOrder(page), custom);
  const back = page.getByRole('button', { name: 'Open dictionary: laver', exact: true });
  await back.focus(); await back.press('Space');
  await expect(page.locator('.dictionary-entry h3')).toHaveText('laver');
  await expect(page.locator('.dictionary-entry h3')).toBeFocused();
  await page.screenshot({ path: join(destination, 'navigation-dictionary.png') });
  await conjugationSearch(page, 'suis');
  const choices = page.getByRole('group', { name: 'Matching verbs' });
  await expect(choices.getByRole('button')).toHaveCount(2);
  await expect(page.locator('.conjugation-table')).toHaveCount(0);
  await choices.getByRole('button', { name: /^être/ }).click();
  await page.getByRole('button', { name: 'Open dictionary: être', exact: true }).click();
  await page.getByRole('button', { name: 'Open conjugation: être', exact: true }).click();
  await expect(choices.getByRole('button')).toHaveCount(2);
  await expect(page.locator('.conjugation-table h3')).toHaveText('être');
  await expect(page.getByLabel('French verb or simple form')).toHaveValue('suis');
  await conjugationSearch(page, " S'ADAPTER ");
  await expect(page.locator('.conjugation-table h3')).toHaveText('s’adapter');
  await page.getByRole('button', { name: 'Open dictionary: s’adapter', exact: true }).click();
  await expect(page.getByLabel('French word or phrase')).toHaveValue('s’adapter');
  await page.getByRole('button', { name: 'Open conjugation: s’adapter', exact: true }).click();
  await expect(page.getByLabel('French verb or simple form')).toHaveValue(" S'ADAPTER ");
  await conjugationSearch(page, 'se laver');
  await expect(page.locator('.dictionary-link')).toHaveText('No corresponding dictionary verb entry.');
  await expect(page.getByRole('button', { name: 'Open dictionary: laver', exact: true })).toHaveCount(0);
  await dictionarySearch(page, 'été'); await expect(page.locator('.dictionary-entry h3')).toHaveText('été');
  await expect(page.locator('.conjugation-link')).toHaveCount(0);
  await dictionarySearch(page, 'faire l’amour');
  await expect(page.locator('.conjugation-link')).toHaveText('No verified conjugation link for this dictionary verb.');
  await dictionarySearch(page, 'laver');
  for (const [locale, theme, open, close] of [['zh', 'white', '打开动词变位: laver', '打开词典: laver'],
    ['fr', 'sepia', 'Ouvrir la conjugaison: laver', 'Ouvrir le dictionnaire: laver'], ['en', 'dark', 'Open conjugation: laver', 'Open dictionary: laver']]) {
    await page.getByRole('banner').getByRole('combobox').nth(0).selectOption(locale);
    await page.getByRole('banner').getByRole('combobox').nth(1).selectOption(theme);
    await page.getByRole('button', { name: open, exact: true }).click();
    assert.deepEqual(await renderedOrder(page), custom);
    await page.getByRole('button', { name: close, exact: true }).click();
    await expect(page.locator('.dictionary-entry h3')).toBeFocused();
  }
  return custom;
}
async function navigationResources(page) {
  for (const resource of ['conjugation', 'dictionary']) {
    const database = join(destination, resource, `${resource}.sqlite3`), saved = database + '.saved';
    await rename(database, saved);
    try {
      for (const state of ['missing', 'corrupt']) {
        if (state === 'corrupt') await writeFile(database, 'invalid SQLite');
        await page.getByRole('button', { name: 'Notebook', exact: true }).click();
        if (resource === 'conjugation') await dictionarySearch(page, 'laver');
        else await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
        await expect(page.locator(resource === 'conjugation' ? '.dictionary-original' : '.conjugation-table').getByRole('alert')).toContainText(state);
        await expect(page.getByRole('button', { name: resource === 'conjugation' ? 'Open conjugation: laver' : 'Open dictionary: laver', exact: true })).toHaveCount(0);
      }
    } finally { await unlink(database).catch((error) => { if (error.code !== 'ENOENT') throw error; }); await rename(saved, database); }
    await dictionarySearch(page, 'laver');
    await page.getByRole('button', { name: 'Open conjugation: laver', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Open dictionary: laver', exact: true })).toBeEnabled();
  }
}
async function navigationEdits(page) {
  const before = await snapshot(page);
  await dictionarySearch(page, 'laver'); await page.getByRole('button', { name: 'View existing record', exact: true }).click();
  const notes = page.getByLabel('Personal notes', { exact: true }); await notes.fill('native pending edit');
  const link = page.getByRole('button', { name: 'Open conjugation: laver', exact: true }); await link.click();
  const dialog = page.getByRole('dialog', { name: 'You have unsaved changes' });
  await dialog.getByRole('button', { name: 'Keep editing' }).click();
  await expect(notes).toHaveValue('native pending edit'); await link.click();
  preferenceSql(`c.execute("CREATE TRIGGER issue47_save_failure BEFORE UPDATE ON vocabulary BEGIN SELECT RAISE(ABORT, 'navigation save rejected'); END")`);
  try {
    await dialog.getByRole('button', { name: 'Save and leave' }).click();
    await expect(dialog).toContainText('navigation save rejected'); await expect(notes).toHaveValue('native pending edit');
    assert.deepEqual(await snapshot(page), before);
  } finally { preferenceSql(`c.execute('DROP TRIGGER issue47_save_failure')`); }
  await dialog.getByRole('button', { name: 'Discard changes' }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  assert.deepEqual(await snapshot(page), before);
  await dictionarySearch(page, 'laver'); await page.getByRole('button', { name: 'View existing record', exact: true }).click();
  await notes.fill('native saved navigation edit'); await link.click();
  await dialog.getByRole('button', { name: 'Save and leave' }).click();
  await expect(dialog).toBeHidden(); await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  return page.evaluate(() => window.__TAURI_INTERNALS__.invoke('vocabulary_find', { headword: 'laver' }));
}
async function navigationAcceptance(page) {
  const original = await snapshot(page);
  assert.ok(original.records.length && original.cards.flat().length && original.audio.some((bytes) => bytes?.length), 'Use the existing learning/audio fixture');
  const seed = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('vocabulary_save', { request: {
    headword: 'laver', notes: 'native original navigation notes', example_fr: '', example_zh: '', example_sense: null, more_examples: [],
    id: null, revision: null, operation: window.crypto.randomUUID(),
  } }));
  assert.equal(seed.outcome, 'created');
  const before = await snapshot(page), audio = await hashes(join(destination, 'Lexique-data/temp/speech'));
  assert.deepEqual(before.records.filter((record) => record.id !== seed.record.id), original.records);
  await page.getByLabel('界面语言').selectOption('en');
  const custom = await navigationBrowse(page);
  await navigationResources(page);
  assert.deepEqual(await snapshot(page), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  const edited = await navigationEdits(page), after = await snapshot(page);
  assert.equal(edited.notes, 'native saved navigation edit'); assert.equal(edited.revision, seed.record.revision + 1);
  assert.deepEqual({ ...edited, notes: seed.record.notes, revision: seed.record.revision, updated_at: seed.record.updated_at }, seed.record);
  assert.deepEqual(after, { ...before, records: before.records.map((record) => record.id === edited.id ? edited : record) });
  const outcomes = await page.evaluate(async (record) => {
    const invoke = window.__TAURI_INTERNALS__.invoke, request = { headword: record.headword, notes: 'must not overwrite', example_fr: '', example_zh: '', example_sense: null, more_examples: [] };
    const duplicate = await invoke('vocabulary_save', { request: { ...request, id: null, revision: null, operation: window.crypto.randomUUID() } });
    const conflict = await invoke('vocabulary_save', { request: { ...request, id: record.id, revision: record.revision - 1, operation: window.crypto.randomUUID() } });
    return [duplicate.outcome, conflict.outcome];
  }, edited);
  assert.deepEqual(outcomes, ['existing', 'conflict']); assert.deepEqual(await snapshot(page), after);
  await page.screenshot({ path: join(destination, 'navigation-conjugation.png') });
  await close(page); page = await start(); await page.getByLabel('界面语言').selectOption('en');
  await dictionarySearch(page, 'laver'); await page.getByRole('button', { name: 'Open conjugation: laver', exact: true }).click();
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  assert.deepEqual(await renderedOrder(page), custom); assert.deepEqual(await snapshot(page), after);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  await page.screenshot({ path: join(destination, 'navigation-restart.png') });
  await close(page);
  await writeFile(join(destination, 'report.json'), JSON.stringify({ status: 'PASS', navigation: true, restart: true,
    selected: ['laver', 'être', 's’adapter'], originalQuery: " S'ADAPTER ", ambiguousCandidates: ['suivre', 'être'],
    unavailable: ['se laver dictionary entry', 'été noun link', 'faire l’amour conjugation link'],
    resourceFailures: ['missing conjugation', 'corrupt conjugation', 'missing dictionary', 'corrupt dictionary'],
    edits: ['stay', 'failed save retained', 'discard', 'save and leave', ...outcomes], mode: 'main', blockOrder: custom,
    languages: ['zh', 'fr', 'en'], themes: ['white', 'sepia', 'dark'],
    preserved: { preExistingRecords: original.records.length, records: after.records.length, cards: after.cards.flat().length,
      progress: after.progress.length, audioFiles: Object.keys(audio).length, references: true, selectedAudio: true, speechPreferences: true },
  }, null, 2));
  console.log('Native dictionary–conjugation navigation: PASS');
}
try {
  let page = await start();
  if (navigationOnly) { await navigationAcceptance(page); } else {
  const { before, audio, results, timings, coloring, blockOrder } = await check(page), modesRestored = [];
  for (const [mode, label, expected] of [['main', '2 · 主要动词', 'lavé'], ['verbal', '1 · 完整动词部分', 'avions lavé'], ['ending', '3 · 主要动词词尾', 'é']]) {
    await page.getByRole('button', { name: label, exact: true }).click();
    await expect(page.getByRole('button', { name: label, exact: true })).toHaveAttribute('aria-pressed', 'true');
    await close(page); page = await start();
    assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode });
    assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), blockOrder.custom);
    await page.getByRole('button', { name: '动词变位', exact: true }).click();
    await expect(page.getByRole('button', { name: label, exact: true })).toHaveAttribute('aria-pressed', 'true');
    await page.getByLabel('法语动词或简单变位词形').fill('nous avions lavé');
    await page.getByRole('button', { name: '查询变位', exact: true }).click();
    await expect(page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText([expected]);
    assert.deepEqual(await renderedOrder(page), blockOrder.custom);
    modesRestored.push(mode);
  }
  const reset = page.locator('.conjugation-order').getByRole('button', { name: '重置区块顺序', exact: true });
  await reset.focus(); await reset.press('Space');
  await expect(page.locator('.conjugation-block').nth(6)).toHaveAttribute('data-tense', 'PQP');
  assert.deepEqual(await renderedOrder(page), defaultOrder);
  await close(page); page = await start();
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_block_order', { order: null })), defaultOrder);
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('conjugation_preferences', { preferences: null })), { mode: 'ending' });
  await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('falloir');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('falloir');
  assert.deepEqual(await renderedOrder(page), defaultOrder);
  await page.screenshot({ path: join(destination, 'block-order-reset-restart.png') });
  const restarted = page;
  const restartedQuery = " VA-T'EN ";
  const restartedResult = await restarted.evaluate((key) => window.__TAURI_INTERNALS__.invoke('conjugation_lookup', { key }), restartedQuery);
  assert.equal(restartedResult.query, restartedQuery);
  assert.deepEqual(restartedResult.tables.map((table) => table.lemma), ['s’en aller']);
  assert.deepEqual(await snapshot(restarted), before);
  assert.deepEqual(await hashes(join(destination, 'Lexique-data/temp/speech')), audio);
  await writeFile(join(destination, 'report.json'), JSON.stringify({ status: 'PASS', restart: true, normalized_restart: true, results, timings,
    coloring, modesRestored, preferenceFailures: ['write', 'invalid IPC', 'corrupt settings', 'unavailable storage'],
    blockOrder: { ...blockOrder, resetRestored: true, failures: ['write', 'invalid IPC', 'corrupt JSON', 'invalid saved order', 'unavailable storage'] },
    preserved: { records: before.records.length, cards: before.cards.flat().length, audioFiles: Object.keys(audio).length } }, null, 2));
  console.log('Native conjugation check: PASS');
  }
} finally {
  clearTimeout(timer); if (browser) await browser.close();
  if (child?.pid && child.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
}
