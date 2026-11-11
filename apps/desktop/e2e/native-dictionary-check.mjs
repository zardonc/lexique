/* global process, window, document, URL, getComputedStyle */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { spawnSync } from 'node:child_process';
import { cp, readFile, rename, writeFile } from 'node:fs/promises';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.ok(process.argv.length >= 8, 'Expected mode, package, binary, conjugation, frequency and new destination');
const [mode, ...paths] = process.argv.slice(2);
assert.ok(['tabs', 'lemmas', 'bundle', 'links'].includes(mode));
const [resources, binary, conjugation, frequency, destination] = paths.map((path) => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
await cp(conjugation, join(destination, 'conjugation'), { recursive: true });
await cp(frequency, join(destination, 'frequency'), { recursive: true });
const originals = fileURLToPath(new URL('./fixtures/mdx/', import.meta.url));
const database = join(destination, 'Lexique-data/data/lexique.sqlite3');
const copies = join(destination, 'Lexique-data/data/dictionaries');
const results = [], externalRequests = [];
let page;

async function start() {
  page = await fixture.start();
  await page.context().route('**/*', (route) => {
    const host = new URL(route.request().url()).hostname;
    if (['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(host)) return route.continue();
    externalRequests.push(host); return route.abort();
  });
}

async function invoke(command, args = {}) {
  return page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
}

async function lookupFailure(key, id) {
  return page.evaluate(async ({ key, id }) => {
    try { await window.__TAURI_INTERNALS__.invoke('dictionary_lookup', { key, dictionary_id: id }); return null; }
    catch (error) { return { code: error.code, message: error.message }; }
  }, { key, id });
}

async function importMdx(path) {
  const before = await invoke('dictionary_settings');
  const catalog = await page.evaluate(({ bytes, name }) => window.__TAURI_INTERNALS__.invoke('dictionary_import', new Uint8Array(bytes), {
    headers: { 'x-lexique-dictionary-name': name },
  }), { bytes: [...await readFile(path)], name: Buffer.from(basename(path), 'utf8').toString('base64') });
  const added = catalog.filter((item) => !before.some((previous) => previous.id === item.id));
  assert.equal(added.length, 1);
  return added[0];
}

async function query(key) {
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByLabel('法语词或词组', { exact: true }).fill(key);
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.getByRole('tab', { selected: true })).toBeVisible();
}

function snapshot() {
  const result = spawnSync(join(destination, 'speech/runtime/python.exe'), ['-B', '-X', 'utf8', '-c', `import sqlite3,json,sys
c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True);c.row_factory=sqlite3.Row
tables=['vocabulary','review_cards','review_progress','review_ratings','word_audio_selections']
r={name:[dict(row) for row in c.execute('SELECT * FROM '+name+' ORDER BY 1')] for name in tables}
r['protected']=[dict(row) for row in c.execute('SELECT locator,owned,expires_at,gc_state FROM audio_version_protection WHERE owned=1 ORDER BY locator')]
r['schema']=c.execute('PRAGMA user_version').fetchone()[0]
print(json.dumps(r))`, database], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

async function tabs() {
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), ['builtin', 'builtin-fra-eng']);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  for (const file of ['verbs.mdx', 'other.mdx', 'formatting.mdx']) {
    await page.getByLabel('添加 MDX 词典', { exact: true }).setInputFiles(join(originals, file));
    await expect(page.getByRole('button', { name: `向前移动: ${file}`, exact: true })).toBeVisible();
    await expect(page.getByLabel('添加 MDX 词典', { exact: true })).toBeEnabled();
  }
  await page.getByRole('button', { name: '向前移动: verbs.mdx', exact: true }).click();
  await expect(page.getByLabel('添加 MDX 词典', { exact: true })).toBeEnabled();
  const imported = await invoke('dictionary_settings');
  const first = imported.find((item) => item.title === 'verbs.mdx');
  const second = imported.find((item) => item.title === 'other.mdx');
  const formatting = imported.find((item) => item.title === 'formatting.mdx');
  const order = [first.id, second.id, 'builtin', formatting.id, 'builtin-fra-eng'];
  await invoke('dictionary_settings', { order });
  const firstPath = join(copies, `${first.id}.mdx`), secondPath = join(copies, `${second.id}.mdx`);
  await query('double');
  await expect(page.locator('iframe.dictionary-html')).toHaveCount(2);
  await rename(secondPath, `${secondPath}.lazy-retained`);
  await page.getByRole('tab', { name: 'other.mdx', exact: true }).click();
  await expect(page.getByRole('tabpanel')).toContainText('词典资源不存在');
  await rename(`${secondPath}.lazy-retained`, secondPath);
  await page.getByRole('tabpanel').getByRole('button', { name: '重新读取', exact: true }).click();
  await expect(page.frameLocator('iframe.dictionary-html').getByText('another sense', { exact: true })).toBeVisible();
  await rename(firstPath, `${firstPath}.cache-retained`);
  await page.getByRole('tab', { name: 'verbs.mdx', exact: true }).click();
  await expect(page.locator('iframe.dictionary-html')).toHaveCount(2);
  await rename(`${firstPath}.cache-retained`, firstPath);
  const duplicate = await invoke('dictionary_lookup', { key: 'double' });
  assert.equal(duplicate.dictionary_id, first.id);
  assert.deepEqual(duplicate.entries.map((entry) => entry.html), ['<p>first sense</p>', '<p>second sense</p>']);
  assert.notEqual(duplicate.entries[0].ordinal, duplicate.entries[1].ordinal);
  assert.equal(duplicate.source.license, null);
  results.push({ check: 'import, persisted order, duplicate physical records and lazy bodies', status: 'passed' });

  const preferences = await invoke('speech_preferences');
  await query('école');
  await expect(page.getByRole('tab', { name: 'verbs.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText('未找到完全匹配的词条。', { exact: true })).toBeVisible();
  await page.getByRole('tab', { name: 'formatting.mdx', exact: true }).click();
  await expect(page.frameLocator('iframe.dictionary-html').getByText('French sample: école', { exact: true })).toBeVisible();
  const frame = page.frames().find((candidate) => candidate.parentFrame() === page.mainFrame());
  const isolation = await frame.evaluate(() => {
    let blocked = false;
    try { Reflect.get(window.parent, '__TAURI_INTERNALS__'); } catch { blocked = true; }
    const paragraph = document.querySelector('p');
    return { blocked, color: getComputedStyle(paragraph).color,
      weight: getComputedStyle(paragraph).fontWeight, active: document.querySelectorAll('script,img,iframe,a').length };
  });
  assert.deepEqual(isolation, { blocked: true, color: 'rgb(18, 52, 86)', weight: '700', active: 0 });
  assert.deepEqual(await invoke('speech_preferences'), preferences);
  assert.equal(await page.evaluate(() => Boolean(window.__dictionary_escaped)), false);
  assert.deepEqual(externalRequests, []);
  await page.locator('iframe.dictionary-html').scrollIntoViewIfNeeded();
  await page.evaluate(() => new Promise((done) => window.requestAnimationFrame(() => window.requestAnimationFrame(done))));
  await page.screenshot({ path: join(destination, 'dictionary-isolation.png'), fullPage: true });
  results.push({ check: 'empty first dictionary and native HTML isolation with inline formatting', status: 'passed' });

  await rename(firstPath, `${firstPath}.retained`);
  const fallback = await invoke('dictionary_lookup', { key: 'double' });
  assert.equal(fallback.dictionary_id, second.id);
  assert.equal(fallback.warnings[0].code, 'missing');
  await query('double');
  await expect(page.getByRole('tab', { name: 'other.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  await page.getByRole('tab', { name: 'verbs.mdx', exact: true }).click();
  await expect(page.getByRole('tabpanel')).toContainText('词典资源不存在');
  await rename(`${firstPath}.retained`, firstPath);
  await rename(secondPath, `${secondPath}.retained`); await writeFile(secondPath, 'broken MDX');
  assert.equal((await lookupFailure('double', second.id)).code, 'corrupt');
  await rename(secondPath, `${secondPath}.corrupt`); await rename(`${secondPath}.retained`, secondPath);
  const catalogBefore = await invoke('dictionary_settings');
  const unsupported = await page.evaluate(async (bytes) => {
    try { await window.__TAURI_INTERNALS__.invoke('dictionary_import', new Uint8Array(bytes), {
      headers: { 'x-lexique-dictionary-name': window.btoa('v3.mdx') },
    }); return null; } catch (error) { return { code: error.code, message: error.message }; }
  }, [...await readFile(resolve('../../tools/mdict-probe/tests/fixtures/unsupported-v3-header.mdx'))]);
  assert.equal(unsupported.code, 'unsupported');
  assert.deepEqual(await invoke('dictionary_settings'), catalogBefore);
  for (const [file, key] of [['response-count.mdx', 'many'], ['response-bytes.mdx', 'huge']]) {
    const source = await importMdx(join(originals, file));
    assert.equal((await lookupFailure(key, source.id)).code, 'limit');
    await invoke('dictionary_remove', { id: source.id });
  }
  results.push({ check: 'missing, corruption, unsupported format and bounded responses', status: 'passed' });

  await fixture.stop(true); await start();
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), order);
  await invoke('dictionary_remove', { id: second.id });
  assert.ok((await readFile(secondPath)).length > 0);
  assert.equal((await invoke('dictionary_lookup', { key: 'double' })).dictionary_id, first.id);
  results.push({ check: 'restart restores order and removal preserves imported files', status: 'passed' });
}

async function lemmas() {
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const source = await importMdx(join(originals, 'verbs.mdx'));
  await invoke('dictionary_settings', { order: [source.id, 'builtin', 'builtin-fra-eng'] });
  const request = { operation: 'native-d9-save', headword: 'être', notes: 'Note conservée : être.', example_fr: '', example_zh: '',
    example_sense: null, more_examples: [], id: null, revision: null };
  const saved = await invoke('vocabulary_save', { request, dictionary_id: source.id });
  const cards = await invoke('review_add', { vocabularyId: saved.record.id });
  await invoke('speech_generate', { vocabularyId: saved.record.id });
  const recognition = cards.find((card) => card.kind === 'recognition');
  const rated = await invoke('review_rate', { request: { operation: 'native-d9-rating', card_id: recognition.id,
    expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  assert.equal(rated.status, 'committed');
  const before = snapshot(); assert.equal(before.schema, 17);
  assert.equal(before.word_audio_selections.length, 1);
  await query('être');
  await page.getByRole('button', { name: '查看现有记录', exact: true }).click();
  await page.getByLabel('独立笔记', { exact: true }).fill('unsaved native note');
  await page.getByRole('tab', { name: 'FreeDict fra-zho', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('unsaved native note');
  await expect(page.getByRole('tab', { name: 'verbs.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  await page.getByRole('tab', { name: 'FreeDict fra-zho', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: '放弃修改', exact: true }).click();
  assert.deepEqual(snapshot(), before);

  await query('suis');
  for (const lemma of ['être', 'suivre']) await expect(page.getByRole('button', { name: `查询原形：${lemma}`, exact: true })).toBeVisible();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('suis');
  await page.screenshot({ path: join(destination, 'lemma-choices.png'), fullPage: true });
  await page.getByRole('button', { name: '查询原形：être', exact: true }).click();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('être');
  await expect(page.getByRole('tab', { name: 'verbs.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  const duplicate = await invoke('vocabulary_save', { request: { ...request, operation: 'native-d9-duplicate', notes: 'must not replace the note' }, dictionary_id: source.id });
  assert.equal(duplicate.outcome, 'existing'); assert.deepEqual(duplicate.record, saved.record);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'explicit ambiguous choices, note protection and duplicate collection preserve FSRS and selected audio', status: 'passed' });

  await query('été');
  await expect(page.frameLocator('iframe.dictionary-html').getByText('summer (noun)', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '查询原形：être', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('été');
  for (const [form, lemma] of [['je me suis lavé', 'se laver'], ["je m'abstiens", "s'abstenir"]]) {
    await query(form);
    await page.getByRole('button', { name: `查询原形：${lemma}`, exact: true }).click();
    await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue(lemma);
    await expect(page.getByRole('tab', { name: 'verbs.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  }
  await query('mangeons');
  await expect(page.locator('.dictionary-lemmas')).toContainText('当前词典中没有已核对的动词原形词条。');
  await expect(page.getByRole('button', { name: '查询原形：manger', exact: true })).toHaveCount(0);
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('mangeons');
  results.push({ check: 'ordinary homograph, pronominal prefixes, exact apostrophe and missing target', status: 'passed' });

  await fixture.stop(true);
  await writeFile(join(copies, `${source.id}.mdx`), await readFile(join(originals, 'verbs-updated.mdx')));
  await start();
  assert.deepEqual(snapshot(), before);
  const updated = await invoke('dictionary_lookup', { key: 'être', dictionary_id: source.id });
  assert.notEqual(updated.sha256, saved.record.reference.sha256);
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue(request.notes);
  await expect(page.locator('.dictionary-original')).toContainText('版本');
  await expect(page.frameLocator('iframe.dictionary-html').getByText('to be, revised', { exact: true })).toBeVisible();
  await invoke('dictionary_remove', { id: source.id });
  const replay = await invoke('vocabulary_save', { request, dictionary_id: source.id });
  assert.deepEqual(replay.record, saved.record);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'source upgrade, restart and replay with an unregistered source preserve learning snapshots', status: 'passed' });
}

async function bundle() {
  const id = 'builtin-fra-eng', title = 'FreeDict fra-eng 0.4.1';
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), ['builtin', id]);
  const lookup = await invoke('dictionary_lookup', { key: 'école', dictionary_id: id });
  assert.equal(lookup.source.license, 'GPL-2.0-or-later');
  assert.equal(lookup.source.version, '0.4.1');
  assert.ok(lookup.entries.some((entry) => entry.html.includes('school')));
  await query('école');
  await page.getByRole('tab', { name: title, exact: true }).click();
  await expect(page.locator('iframe.dictionary-html').first().contentFrame().getByRole('heading', { name: 'école', exact: true })).toBeVisible();
  await expect(page.locator('.dictionary-source')).toContainText('GPL-2.0-or-later');
  results.push({ check: 'fresh bundled FreeDict lookup, real HTML and verified provenance', status: 'passed' });

  const request = { operation: 'native-mdx-bundle-save', headword: 'école', notes: 'Note conservée : école.', example_fr: '', example_zh: '',
    example_sense: null, more_examples: [], id: null, revision: null };
  const saved = await invoke('vocabulary_save', { request, dictionary_id: id });
  assert.equal(saved.record.reference.dictionary_id, id);
  assert.equal(saved.record.reference.license, 'GPL-2.0-or-later');
  const cards = await invoke('review_add', { vocabularyId: saved.record.id });
  await invoke('speech_generate', { vocabularyId: saved.record.id });
  const recognition = cards.find((card) => card.kind === 'recognition');
  await invoke('review_rate', { request: { operation: 'native-mdx-bundle-rating', card_id: recognition.id,
    expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  const custom = await importMdx(join(originals, 'verbs.mdx'));
  const before = snapshot();
  assert.equal(before.word_audio_selections.length, 1);
  const legacy = [{ id: custom.id, title: custom.title }, { id: 'builtin', title: 'FreeDict fra-zho' }];
  await fixture.stop(true);
  const changed = spawnSync(join(destination, 'speech/runtime/python.exe'), ['-B', '-X', 'utf8', '-c',
    "import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute('UPDATE app_metadata SET value=? WHERE key=?',(sys.argv[2],'dictionary-settings')); c.commit()",
    database, JSON.stringify({ next_id: 2, dictionaries: legacy })], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(changed.status, 0, changed.stderr);
  await start();
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), [custom.id, 'builtin', id]);
  assert.deepEqual(await readFile(join(copies, `${custom.id}.mdx`)), await readFile(join(originals, 'verbs.mdx')));
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'legacy order upgrades on restart while user MDX, notes, FSRS and selected audio remain intact', status: 'passed' });

  const mdx = join(destination, 'dictionary/freedict-fra-eng-mdx/freedict-fra-eng-0.4.1.mdx');
  await rename(mdx, `${mdx}.retained`);
  assert.equal((await lookupFailure('école', id)).code, 'missing');
  await query('école');
  await page.getByRole('tab', { name: title, exact: true }).click();
  await expect(page.getByRole('tabpanel')).toContainText('词典资源不存在');
  await rename(`${mdx}.retained`, mdx);
  await page.getByRole('tabpanel').getByRole('button', { name: '重新读取', exact: true }).click();
  await expect(page.locator('iframe.dictionary-html').first().contentFrame().getByRole('heading', { name: 'école', exact: true })).toBeVisible();
  assert.deepEqual(snapshot(), before);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.locator('.dictionary-settings');
  await expect(settings.locator('li').filter({ hasText: title }).getByRole('button', { name: '移出词典列表', exact: true })).toHaveCount(0);
  await expect(page.locator('.dictionary-resources details')).toHaveCount(8);
  const invalid = await page.evaluate(async () => {
    try { await window.__TAURI_INTERNALS__.invoke('dictionary_resource_open', { id: 'https://example.org' }); return null; }
    catch (error) { return error.code; }
  });
  assert.equal(invalid, 'invalid_query');
  assert.deepEqual(externalRequests, []);
  await page.locator('.dictionary-resources').screenshot({ path: join(destination, 'dictionary-resources.png') });
  results.push({ check: 'resource failure/recovery, protected bundle, eight optional categories and ID-only native link command', status: 'passed' });
}

async function links() {
  const before = snapshot();
  await invoke('dictionary_resource_open', { id: 'larousse-synonyms' });
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'native allowlisted resource launch preserves learning data', status: 'passed',
    resource_id: 'larousse-synonyms', url: 'https://www.larousse.fr/dictionnaires/synonymes' });
}

try {
  await start();
  await ({ tabs, lemmas, bundle, links }[mode])();
  await fixture.stop(true);
  await writeFile(join(destination, 'result.json'), JSON.stringify({ mode, results, externalRequests }, null, 2));
  process.stdout.write(JSON.stringify({ mode, results, externalRequests }) + '\n');
} finally { await fixture.finish(); }
