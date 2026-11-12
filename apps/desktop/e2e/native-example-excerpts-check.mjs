/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected package, binary and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((value) => resolve(value));
const fixture = await native24Fixture(resources, binary, destination);
await cp(join(dirname(binary), 'dictionary'), join(destination, 'dictionary'), { recursive: true });
function sql(code) {
  const result = spawnSync(join(destination, 'speech/runtime/python.exe'), ['-B', '-X', 'utf8', '-c',
    'import sqlite3,json,sys\nc=sqlite3.connect(sys.argv[1]);c.row_factory=sqlite3.Row\n' + code,
    join(destination, 'Lexique-data/data/lexique.sqlite3')], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim() ? JSON.parse(result.stdout) : null;
}
function learning() {
  return sql("print(json.dumps({t:[dict(r) for r in c.execute('SELECT * FROM '+t+' ORDER BY 1')] for t in ['review_cards','review_progress','review_ratings','word_audio_selections']}))");
}
async function records(page) {
  return page.evaluate(() => window.__TAURI_INTERNALS__.invoke('vocabulary_list', { query: '', sort: 'updated-desc' }));
}
async function excerpt(page, fr, zh) {
  await page.getByRole('button', { name: '摘录例句', exact: true }).click();
  await page.getByLabel('摘录法语').fill(fr); await page.getByLabel('摘录中文').fill(zh);
  await page.getByRole('button', { name: '加入草稿', exact: true }).click();
}
let hidden = false;
try {
  let page = await fixture.start(); await page.context().setOffline(true);
  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const saved = await invoke('vocabulary_save', { request: { operation: 'excerpt-legacy', headword: 'école', notes: 'Legacy notes',
      example_fr: '', example_zh: '', example_sense: 0, more_examples: [{ fr: 'Legacy example', zh: '' }], id: null, revision: null } });
    const cards = await invoke('review_add', { vocabularyId: saved.record.id }); await invoke('speech_generate', { vocabularyId: saved.record.id });
    const recognition = cards.find((card) => card.kind === 'recognition');
    const rating = await invoke('review_rate', { request: { operation: 'excerpt-rating', card_id: recognition.id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    if (rating.status !== 'committed') throw new Error('Fixture rating failed');
  });
  const before = learning(); await fixture.stop(true);
  sql('c.execute("ALTER TABLE vocabulary DROP COLUMN example_source");c.execute("PRAGMA user_version=17");c.commit()');
  page = await fixture.start(); await page.context().setOffline(true);
  const legacy = (await records(page))[0]; assert.equal(legacy.example_source, undefined); assert.equal(legacy.more_examples[0].source, undefined);
  await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
  await excerpt(page, 'Une école.', '学校'); await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('status').filter({ hasText: '已保存' })).toBeVisible();
  const first = (await records(page))[0]; assert.equal(first.example_source.dictionary_id, 'builtin');
  await page.getByRole('tab').filter({ hasText: /English|anglais|fra-eng|法英/i }).click();
  await excerpt(page, 'Une école.', '学校'); await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('status').filter({ hasText: '已保存' })).toBeVisible();
  await excerpt(page, 'Une école.', '学校'); await expect(page.getByText('例句已存在，已定位原例句。')).toBeVisible();
  await expect(page.locator('#example-fr-1')).toBeFocused(); await page.getByRole('button', { name: '取消摘录', exact: true }).click();
  const saved = (await records(page))[0]; assert.equal(saved.more_examples.length, 2); assert.equal(saved.more_examples[1].source.dictionary_id, 'builtin-fra-eng');
  const protections = await page.evaluate(async (record) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const request = { operation: 'excerpt-forged', headword: record.headword, notes: record.notes,
      example_fr: record.example_fr, example_zh: record.example_zh, example_sense: record.example_sense,
      example_source: { ...record.example_source, version: 'forged-version' }, more_examples: record.more_examples, id: record.id, revision: record.revision };
    let rejected = false;
    try { await invoke('vocabulary_save', { request }); } catch { rejected = true; }
    const conflict = await invoke('vocabulary_save', { request: { ...request, operation: 'excerpt-stale', example_source: record.example_source, revision: record.revision - 1 } });
    const replay = await invoke('vocabulary_save', { request: { operation: 'excerpt-legacy', headword: 'école', notes: 'Legacy notes',
      example_fr: '', example_zh: '', example_sense: 0, more_examples: [{ fr: 'Legacy example', zh: '' }], id: null, revision: null } });
    return { rejected, conflict: conflict.outcome, replay: replay.outcome };
  }, saved);
  assert.deepEqual(protections, { rejected: true, conflict: 'conflict', replay: 'replayed' });
  assert.deepEqual((await records(page))[0], saved);
  assert.deepEqual(learning(), before); await fixture.stop(true);
  await rename(join(destination, 'dictionary'), join(destination, 'dictionary.unavailable')); hidden = true;
  page = await fixture.start(); await page.context().setOffline(true);
  assert.deepEqual((await records(page))[0], saved);
  await page.getByRole('button', { name: '生词本', exact: true }).click(); await page.locator('.vocabulary-list button').click();
  await page.locator('#example-fr').fill('Une école modifiée.'); await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('status').filter({ hasText: '已保存' })).toBeVisible();
  const edited = (await records(page))[0]; assert.deepEqual(edited.example_source, saved.example_source); assert.deepEqual(edited.more_examples, saved.more_examples);
  assert.equal(edited.notes, legacy.notes); assert.deepEqual(edited.reference, legacy.reference); assert.deepEqual(edited.digest, legacy.digest); assert.deepEqual(learning(), before);
  await fixture.stop(true); page = await fixture.start(); assert.deepEqual((await records(page))[0], edited);
  assert.deepEqual(learning(), before);
  await writeFile(join(destination, 'example-excerpt-results.json'), JSON.stringify({ status: 'passed', legacyUpgrade: 'passed',
    sources: 'builtin and builtin-fra-eng', explicitSaveAndDuplicate: 'passed', resourceRemovalAndEdit: 'passed', restart: 'passed', learningAndSelectedAudio: 'preserved' }, null, 2));
  console.log('Native excerpt provenance, legacy upgrade, resource removal and restart passed');
} finally {
  await fixture.finish(); if (hidden) await rename(join(destination, 'dictionary.unavailable'), join(destination, 'dictionary'));
}
