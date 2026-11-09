/* global process, window, performance, URL */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 7, 'Expected package, binary, conjugation, frequency and new destination');
const [resources, binary, conjugation, frequency, destination] = process.argv.slice(2).map((path) => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
await cp(conjugation, join(destination, 'conjugation'), { recursive: true });
await cp(frequency, join(destination, 'frequency'), { recursive: true });
const database = join(destination, 'Lexique-data/data/lexique.sqlite3');
const resource = join(destination, 'frequency/frequency.sqlite3');
const backup = join(destination, 'frequency/frequency.original');
const python = join(destination, 'speech/runtime/python.exe');
function snapshot() {
  const result = spawnSync(python, ['-B', '-X', 'utf8', '-c', `import sqlite3,json,sys
c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True)
c.row_factory=sqlite3.Row
tables=['vocabulary','review_cards','review_progress','review_ratings','word_audio_selections']
r={name:[dict(row) for row in c.execute('SELECT * FROM '+name+' ORDER BY 1')] for name in tables}
r['protected']=[dict(row) for row in c.execute('SELECT locator,owned,expires_at,gc_state FROM audio_version_protection WHERE owned=1 ORDER BY locator')]
r['schema']=c.execute('PRAGMA user_version').fetchone()[0]
print(json.dumps(r))`, database], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
async function notebook(page) {
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.locator('.vocabulary-list li')).toHaveCount(2);
}
async function blockInternet(page) {
  await page.context().route('**/*', (route) => {
    const host = new URL(route.request().url()).hostname;
    return ['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(host) ? route.continue() : route.abort();
  });
}
const results = [];
try {
  let page = await fixture.start();
  await blockInternet(page);
  const seeded = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const saved = [];
    for (const [index, headword] of ['école', 'œil'].entries()) saved.push((await invoke('vocabulary_save', { request: {
      operation: `native-frequency-${index}`, headword, notes: `Note conservée : ${headword}.`, example_fr: '', example_zh: '', example_sense: null,
      more_examples: [], id: null, revision: null,
    } })).record);
    const cards = await invoke('review_add', { vocabularyId: saved[0].id });
    await invoke('speech_generate', { vocabularyId: saved[0].id });
    const recognition = cards.find((card) => card.kind === 'recognition');
    const rated = await invoke('review_rate', { request: { operation: 'native-frequency-rating', card_id: recognition.id,
      expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    if (rated.status !== 'committed') throw new Error('Native fixture rating failed');
    return saved.map((record) => record.id);
  });
  const before = snapshot();
  const lookup = await page.evaluate(async () => {
    const start = performance.now();
    const result = await window.__TAURI_INTERNALS__.invoke('frequency_lookup', {
      keys: ['a priori', 'abaisser', 'cote', 'côte', 'côté', 'suis', 'étaient', 'a contrario', 'œil', 'o'],
    });
    return { result, milliseconds: performance.now() - start };
  });
  assert.equal(lookup.result.source.version, '3.83');
  assert.deepEqual(lookup.result.entries.slice(0, 5).map((entry) => entry.per_million), [1.45, 4.93, 8.48, 35.16, 285.99]);
  assert.equal(lookup.result.entries[5].reason, 'ambiguous');
  assert.equal(lookup.result.entries[6].per_million, 40411.43);
  assert.equal(lookup.result.entries[7].per_million, 0);
  assert.equal(lookup.result.entries[8].reason, 'not_found');
  assert.equal(lookup.result.entries[9].reason, 'missing_pos');
  await notebook(page);
  await expect(page.locator('.vocabulary-list li').filter({ hasText: 'école' }).locator('.frequency-badge')).toHaveText('高频');
  await expect(page.locator('.vocabulary-list li').filter({ hasText: 'œil' }).locator('.frequency-badge')).toHaveText('暂无词频');
  await page.locator('.vocabulary-list li').filter({ hasText: 'école' }).getByRole('button').click();
  await page.getByText('词频数值与来源', { exact: true }).click();
  await expect(page.locator('.frequency-details')).toContainText('206.88');
  await expect(page.locator('.frequency-details')).toContainText('CC BY-SA 4.0');
  await page.screenshot({ path: join(destination, 'notebook-frequency.png'), fullPage: true });
  const batch = await page.evaluate(async () => {
    const start = performance.now();
    const result = await window.__TAURI_INTERNALS__.invoke('frequency_lookup', { keys: Array(100).fill('école') });
    return { count: result.entries.length, milliseconds: performance.now() - start };
  });
  assert.equal(batch.count, 100);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'native values, notebook, provenance and 100-word batch', passed: true, query_ms: lookup.milliseconds, batch_ms: batch.milliseconds });
  await fixture.stop(true);
  await rename(resource, backup);
  page = await fixture.start(); await blockInternet(page); await notebook(page);
  await expect(page.getByRole('alert')).toContainText('Could not read resource');
  await expect(page.locator('.frequency-badge')).toHaveText(['暂无词频', '暂无词频']);
  await page.locator('.vocabulary-list li').filter({ hasText: 'école' }).getByRole('button').click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('Note conservée : école.');
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'missing resource after restart preserves notes, progress and selected audio', passed: true });
  await fixture.stop(true);
  await writeFile(resource, 'corrupt resource');
  page = await fixture.start(); await blockInternet(page); await notebook(page);
  await expect(page.getByRole('alert')).toContainText('Resource checksum mismatch');
  assert.deepEqual(snapshot(), before);
  await cp(backup, resource);
  await page.getByRole('button', { name: '重试词频' }).click();
  await expect(page.locator('.vocabulary-list li').filter({ hasText: 'école' }).locator('.frequency-badge')).toHaveText('高频');
  results.push({ check: 'corrupt resource and restored read-only resource', passed: true });
  await fixture.stop(true);
  page = await fixture.start(); await blockInternet(page); await notebook(page);
  await expect(page.locator('.vocabulary-list li').filter({ hasText: 'école' }).locator('.frequency-badge')).toHaveText('高频');
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'normal restart preserves all learning state', passed: true });
  await writeFile(join(destination, 'frequency-results.json'), JSON.stringify({ ids: seeded, schema: before.schema,
    network: 'WebView denies external requests; native frequency reads bundled files', results }, null, 2));
} finally { await fixture.finish(); }
