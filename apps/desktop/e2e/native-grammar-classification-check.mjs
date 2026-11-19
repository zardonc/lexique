/* global process, window, console, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { cp, writeFile } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { inspectPackage } from './native-backup-state.mjs';

const [resources, binary, destinationPath, scenario, source] = process.argv.slice(2);
assert.ok(['notes', 'backup'].includes(scenario));
const fixture = await native24Fixture(resources, binary, destinationPath);
const { destination } = fixture;
const evidence = { scenario, checks: [] };
let page;
const invoke = (command, args = {}) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const noteRequest = (operation, title, body, category_ids = [], record) => ({ operation, title, body, category_ids, id: record?.id ?? null, revision: record?.revision ?? null });
const grammar = () => invoke('grammar_list', { query: '', sort: 'created-asc' });

function helper(action, filename, filenameId = 0) {
  const child = spawn('pwsh', ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', action,
    '-Title', 'Lexique',
    '-SourcePid', String(fixture.hostPid()), '-SourceExecutable', join(destination, 'Lexique.exe'), '-WaitMs', '4000',
    '-Text', filename ?? '', '-FilenameControlId', String(filenameId)], { windowsHide: true });
  fixture.ownHelper(child);
  let stdout = '', stderr = '';
  child.stdout.on('data', (bytes) => { stdout += bytes.toString(); if (stdout.length > 65536) child.kill(); });
  child.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
  const timer = setTimeout(() => child.kill(), 8000);
  return new Promise((done, reject) => {
    child.once('error', (error) => { clearTimeout(timer); reject(error); });
    child.once('exit', (code) => {
      clearTimeout(timer);
      if (code !== 0) reject(new Error(`Owned helper failed: ${code}: ${stderr}`));
      else { try { done(stdout.trim() ? JSON.parse(stdout) : null); } catch (error) { reject(error); } }
    });
  });
}

async function picker(button, filename) {
  const observed = helper('picker-controls'); observed.catch(() => {});
  const clicked = page.getByRole('button', { name: button, exact: true }).click({ noWaitAfter: true }); clicked.catch(() => {});
  const controls = await observed;
  assert.equal(controls.pid, fixture.hostPid()); assert.equal(controls.class, '#32770');
  const id = button === 'Create backup' ? 1001 : 1148;
  assert.equal(controls.controls.filter((control) => control.class === 'Edit' && control.id === id && control.visible && control.enabled).length, 1);
  for (const buttonId of [1, 2]) assert.equal(controls.controls.filter((control) => control.class === 'Button' && control.id === buttonId && control.visible && control.enabled).length, 1);
  const selected = await helper('picker-select', filename, id);
  await clicked; assert.equal(selected.disappeared, true); assert.equal(selected.filename, filename);
  evidence.checks.push({ picker: button, filenameId: id, ownedPid: controls.pid });
}

async function notes() {
  assert.deepEqual(await invoke('learning_categories'), []);
  const temps = await invoke('category_create', { name: 'Temps' }), errors = await invoke('category_create', { name: 'À revoir' });
  const word = (await invoke('vocabulary_save', { request: { operation: 'native-notes-word', headword: 'école', notes: 'À apprendre.',
    example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null, category_ids: [temps.id] } })).record;
  const first = (await invoke('grammar_save', { request: noteRequest('native-note-first', 'Temps du passé', 'Une règle.\nUn exemple.', [temps.id, errors.id]) })).record;
  let second = (await invoke('grammar_save', { request: noteRequest('native-note-second', first.title, '', [temps.id]) })).record;
  assert.notEqual(first.id, second.id);
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await expect(page.locator('.vocabulary-list button')).toHaveCount(2);
  await page.locator('.vocabulary-list button').filter({ hasText: first.body }).click();
  await page.getByRole('button', { name: '制作问答卡' }).click();
  await expect(page.getByLabel('题面')).toHaveValue(first.title);
  await expect(page.getByLabel('答案', { exact: true })).toHaveValue(first.body);
  await page.getByLabel('题面').fill('Quand employer le passé composé ?');
  await page.getByLabel('答案', { exact: true }).fill('Une action achevée.\nJe suis partie.');
  await page.getByRole('button', { name: '保存并加入复习' }).click();
  await expect(page.getByRole('button', { name: '暂停复习' })).toBeEnabled();
  let saved = await invoke('grammar_get', { id: first.id });
  second = (await invoke('grammar_card_save', { request: { operation: 'native-second-card', id: second.id, revision: second.revision, question: 'Une question', answer: 'Une réponse' } })).record;
  await invoke('review_add', { vocabularyId: word.id });
  const mixed = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  assert.ok(mixed.queue.cards.some((card) => card.kind === 'grammar'));
  assert.ok(mixed.queue.cards.some((card) => card.kind === 'recognition'));
  await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 2, review_limit: 50, new_cards_ignore_review_limit: false } });
  await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  for (const record of [saved, second]) await invoke('review_rate', { request: { operation: `native-rating-${record.id}`, card_id: record.card.id, expected_revision: 0, rating: 'easy', timezone: 'UTC' } });
  const progress = await invoke('review_progress', { cardId: saved.card.id });
  saved = (await invoke('grammar_card_save', { request: { operation: 'native-edit-card', id: saved.id, revision: saved.revision, question: 'Pourquoi ce temps ?', answer: 'Une action terminée.' } })).record;
  assert.deepEqual(await invoke('review_progress', { cardId: saved.card.id }), progress);
  saved = (await invoke('grammar_save', { request: noteRequest('native-edit-note', saved.title, 'Une note modifiée.\nDeuxième ligne.', [errors.id], saved) })).record;
  assert.equal(saved.card.answer, 'Une action terminée.');
  await invoke('review_pause', { vocabularyId: saved.id, paused: true });
  await invoke('review_pause', { vocabularyId: saved.id, paused: false });
  await invoke('review_pause', { vocabularyId: saved.id, paused: true });
  await invoke('vocabulary_sort', { sort: 'headword-desc' });
  await page.getByLabel('排序方式').selectOption('created-asc');
  await page.getByRole('button', { name: '新建语法笔记' }).click();
  await page.getByLabel('语法标题', { exact: true }).fill('Un brouillon non enregistré');
  await helper('close');
  const leave = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await expect(leave).toBeVisible();
  await leave.getByRole('button', { name: '继续编辑' }).click();
  await fixture.stop(true, async () => { await helper('close'); await leave.getByRole('button', { name: '放弃修改' }).click(); });
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await expect(page.getByRole('banner')).toBeVisible();
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await expect(page.getByRole('tab', { name: '生词', exact: true })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByLabel('筛选分类')).toHaveValue('all');
  await expect(page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' })).toHaveValue('');
  await expect(page.getByLabel('排序方式')).toHaveValue('headword-desc');
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await expect(page.getByLabel('排序方式')).toHaveValue('created-asc');
  assert.deepEqual(await grammar(), [saved, second]);
  assert.deepEqual(await invoke('review_progress', { cardId: saved.card.id }), progress);
  assert.equal(await invoke('review_pause', { vocabularyId: saved.id, paused: null }), true);
  await invoke('category_delete', { id: temps.id });
  await page.reload(); await expect(page.getByRole('banner')).toBeVisible();
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByLabel('筛选分类').selectOption('uncategorized');
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '删除语法笔记', exact: true }).click();
  const deletion = page.getByRole('dialog', { name: '永久删除语法笔记？' });
  await expect(deletion).toContainText('评分历史'); await expect(deletion).toContainText('当日额度保留');
  await deletion.getByRole('button', { name: '确认删除', exact: true }).click();
  await expect(deletion).toBeHidden();
  assert.equal(await invoke('review_undo_target'), null);
  const limited = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  assert.equal(limited.queue.reason, 'quota'); assert.deepEqual(limited.queue.cards, []);
  const replacement = (await invoke('grammar_save', { request: noteRequest('native-replacement-note', first.title, '') })).record;
  const replacementCategory = await invoke('category_create', { name: temps.name });
  assert.ok(replacement.id > second.id); assert.ok(replacementCategory.id > temps.id);
  await invoke('grammar_delete', { id: second.id, revision: second.revision });
  assert.equal((await invoke('grammar_get', { id: replacement.id })).id, replacement.id);
  evidence.records = await grammar(); evidence.categories = await invoke('learning_categories'); evidence.progress = progress;
  evidence.checks.push('independent notes, shared categories, mixed review, card progress, native close guard, restart, quota and deletion identities');
  await page.screenshot({ path: join(destination, 'native-notes.png') });
  await fixture.stop(true);
}

async function backup() {
  const original = await grammar(), originalCategories = await invoke('learning_categories');
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.evaluate(async () => {
    window.grammarBackupEvents = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress']) await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
      handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.grammarBackupEvents.push({ event, payload: value.payload }), false) });
  });
  const latest = (event) => page.evaluate((event) => window.grammarBackupEvents.filter((value) => value.event === event).at(-1)?.payload, event);
  const filename = join(destination, 'grammar.lexique-backup');
  await picker('Create backup', filename);
  let exported;
  await expect.poll(async () => { exported = await latest('lexique-backup-progress'); return exported?.state; }, { timeout: 10000 }).toBe('completed');
  assert.equal(exported.result.counts.grammar, original.length); assert.equal(exported.result.counts.categories, originalCategories.length);
  const archive = inspectPackage(destination, filename); assert.equal(archive.snapshot.snapshot_version, 3);
  assert.ok(archive.snapshot.deleted_grammar_operations.includes('native-note-second'));
  assert.ok(archive.snapshot.retained_review_usage.length); assert.equal(archive.snapshot.preferences.undo_blocked, true);
  const transient = (await invoke('grammar_save', { request: noteRequest('native-after-backup', 'Après la sauvegarde', 'Une modification') })).record;
  const transientCard = (await invoke('grammar_card_save', { request: { operation: 'native-after-backup-card', id: transient.id, revision: transient.revision, question: 'Après ?', answer: 'Oui.' } })).record.card;
  const transientCategory = await invoke('category_create', { name: 'Après sauvegarde' });
  await picker('Choose backup', filename);
  let preview;
  await expect.poll(async () => { preview = await latest('lexique-backup-preview-progress'); return preview?.state; }, { timeout: 10000 }).toBe('preview_ready');
  assert.deepEqual(preview.preview.counts, exported.result.counts);
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore' });
  const confirm = dialog.getByRole('button', { name: 'Confirm and restore', exact: true });
  await expect(confirm).toBeDisabled(); await dialog.getByRole('checkbox').check();
  await page.screenshot({ path: join(destination, 'native-grammar-restore-confirmation.png') });
  page = await fixture.followRestart(() => confirm.click()); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  let startup;
  await expect.poll(async () => { startup = await invoke('startup_status'); return startup.state; }, { timeout: 10000 }).toBe('ready');
  assert.equal(startup.error, null); assert.deepEqual(startup.restored.counts, exported.result.counts);
  assert.deepEqual(await grammar(), original); assert.deepEqual(await invoke('learning_categories'), originalCategories);
  assert.equal(await invoke('review_undo_target'), null);
  const replacement = (await invoke('grammar_save', { request: noteRequest('native-post-restore', 'Après restauration', '') })).record;
  const card = (await invoke('grammar_card_save', { request: { operation: 'native-post-restore-card', id: replacement.id, revision: replacement.revision, question: 'Nouvelle carte ?', answer: 'Oui.' } })).record.card;
  const category = await invoke('category_create', { name: 'Après restauration' });
  assert.ok(replacement.id > transient.id); assert.ok(card.id > transientCard.id); assert.ok(category.id > transientCategory.id);
  evidence.checks.push('snapshot v3, grammar and category counts, whole-backup restore, retained usage, blocked undo and identity watermarks');
  evidence.restored = startup.restored; evidence.identities = { record: replacement.id, card: card.id, category: category.id };
  await page.screenshot({ path: join(destination, 'native-grammar-restored.png') });
  await fixture.stop(true);
}

try {
  for (const name of ['conjugation', 'frequency']) await cp(join(resolve(resources), name), join(destination, name), { recursive: true });
  if (scenario === 'backup') {
    const owned = relative(resolve('../../target/grammar-classification'), resolve(source));
    assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
    await cp(join(resolve(source), 'Lexique-data/data'), join(destination, 'Lexique-data/data'), { recursive: true });
  }
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await expect(page.getByRole('banner')).toBeVisible();
  await (scenario === 'notes' ? notes() : backup());
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify({ scenario, passed: true, checks: evidence.checks.length }));
} finally { await fixture.finish(); }
