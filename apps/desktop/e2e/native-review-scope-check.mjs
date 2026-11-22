/* global process, window, console, document, HTMLMediaElement, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { cp, readFile, rename, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { fixtureSql, inspectPackage, readState, compareLearning } from './native-backup-state.mjs';

const [resources, binary, destinationPath, scenario] = process.argv.slice(2);
assert.ok(['selection', 'live', 'undo', 'diagnostics', 'failure', 'backup-all', 'backup-single', 'backup-multiple', 'backup-uncategorized', 'backup-needs-selection', 'backup-old', 'backup-invalid'].includes(scenario));
const fixture = await native24Fixture(resources, binary, destinationPath);
const { destination } = fixture;
const revision = spawnSync('git', ['rev-parse', 'HEAD', 'HEAD^{tree}'], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
const status = spawnSync('git', ['status', '--porcelain'], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
assert.equal(revision.status, 0, revision.stderr); assert.equal(status.status, 0, status.stderr);
const [sourceSha, sourceTree] = revision.stdout.trim().split(/\r?\n/);
await writeFile(join(destination, 'run-metadata.json'), JSON.stringify({ ...JSON.parse(await readFile(join(destination, 'run-metadata.json'), 'utf8')),
  sourceSha, sourceTree, dirty: Boolean(status.stdout.trim()), platform: process.platform, architecture: process.arch }, null, 2));
const evidence = { scenario, checks: [], views: [] };
const all = { kind: 'all' }, needs = { kind: 'needs_selection' };
const scope = (category_ids = [], include_uncategorized = false) => ({ kind: 'vocabulary', category_ids, include_uncategorized });
let page, voyageCategory, ecoleCategory, words, grammar;
const invoke = (command, args = {}) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const setScope = (value) => invoke('review_scope', { scope: value });
const applied = () => invoke('review_scope', { scope: null });
const selector = () => page.getByRole('group', { name: 'Review scope', exact: true });
const card = () => page.getByRole('region', { name: 'Review card', exact: true });

async function queue(skipped = [], cursor = null) {
  const value = await invoke('review_queue', { timezone: 'UTC', skipped, cursor });
  evidence.views.push({ summary: value.summary, queue: value.queue, delta: value.delta,
    window: value.window.map((item) => ({ card: item.card, version: item.version, audioBytes: item.audio?.bytes.length ?? 0 })) });
  return value;
}

async function inventory() {
  const cards = [], skipped = [];
  for (let i = 0; i < 8; i++) {
    const view = await queue(skipped);
    if (!view.queue.cards.length) break;
    for (const current of view.queue.cards) { assert.ok(!skipped.includes(current.id)); skipped.push(current.id); cards.push(current); }
  }
  return cards;
}

async function seed() {
  assert.deepEqual(await applied(), all);
  voyageCategory = await invoke('category_create', { name: 'Voyage' }); ecoleCategory = await invoke('category_create', { name: 'École' });
  words = [];
  for (const [headword, category_ids] of [['école', [voyageCategory.id]], ['bonjour', [ecoleCategory.id]], ['œil', []]]) {
    words.push((await invoke('vocabulary_save', { request: { operation: `scope-word-${words.length}`, headword, category_ids,
      notes: 'Synthetic category acceptance', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } })).record);
  }
  grammar = (await invoke('grammar_save', { request: { operation: 'scope-grammar', title: 'Une règle', body: 'Un exemple', category_ids: [voyageCategory.id], id: null, revision: null } })).record;
  grammar = (await invoke('grammar_card_save', { request: { operation: 'scope-grammar-card', id: grammar.id, revision: grammar.revision, question: 'Pourquoi ?', answer: 'Une règle.' } })).record;
  await invoke('interface_preferences', { preferences: { locale: 'en', theme: 'white' } });
  await page.reload(); await expect(page.getByRole('banner')).toBeVisible();
}

async function enroll(records = words) { for (const word of records) await invoke('review_add', { vocabularyId: word.id }); }
async function recognition(word) { return (await invoke('review_cards', { vocabularyId: word.id })).find((value) => value.kind === 'recognition'); }
async function rate(current, operation, presentationRevision) {
  const progress = await invoke('review_progress', { cardId: current.id });
  const request = { operation, card_id: current.id, expected_revision: progress.revision, rating: 'easy', timezone: 'UTC' };
  const result = await invoke('review_rate', { request, presentationRevision: presentationRevision ?? null });
  assert.equal(result.status, 'committed', JSON.stringify(result));
  return { request, result };
}

async function restart() {
  await fixture.stop(true); page = await fixture.start(); page.setDefaultTimeout(6000); await page.context().setOffline(true);
  await expect(page.getByRole('banner')).toBeVisible();
}

async function selection() {
  await enroll([words[0]]);
  const mixed = await inventory(); assert.ok(mixed.some((value) => value.kind === 'grammar')); assert.ok(mixed.some((value) => value.kind === 'recognition'));
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card()).toBeVisible();
  const current = await card().innerText();
  await selector().getByRole('checkbox', { name: voyageCategory.name, exact: true }).check();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage')).toBeVisible();
  assert.equal(await card().innerText(), current); assert.deepEqual(await applied(), all);
  await card().getByRole('button', { name: 'Skip', exact: true }).click();
  await expect(page.getByText('Applied scope: Voyage')).toBeVisible();
  await expect.poll(() => invoke('speech_cached', { vocabularyId: words[0].id }), { timeout: 20000 }).not.toBeNull();
  const first = await recognition(words[0]);
  const listening = await queue([first.id]);
  assert.equal(listening.queue.cards.length, 1); assert.equal(listening.queue.cards[0].kind, 'listening');
  assert.ok(listening.window[0].audio.bytes.length > 100); assert.equal(listening.summary.audio_pending_count, 0);
  await page.evaluate(() => {
    window.scopePlayback = [];
    document.addEventListener('playing', (event) => { if (event.target instanceof HTMLMediaElement) window.scopePlayback.push({ currentTime: event.target.currentTime, duration: event.target.duration }); }, true);
  });
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card().getByRole('heading', { name: words[0].headword })).toBeVisible();
  await card().getByRole('button', { name: 'Skip', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.scopePlayback.length)).toBeGreaterThan(0);
  await expect.poll(() => card().locator('audio').evaluate((audio) => audio.currentTime)).toBeGreaterThan(0);
  assert.ok(await card().locator('audio').evaluate((audio) => Number.isFinite(audio.duration) && audio.duration > 0));
  await card().getByRole('button', { name: 'Show answer', exact: true }).click();
  await expect(card().getByRole('button', { name: 'Good', exact: true })).toBeEnabled();
  await card().getByRole('button', { name: 'Good', exact: true }).click();
  await expect.poll(async () => (await invoke('review_progress', { cardId: listening.queue.cards[0].id })).revision).toBe(1);
  evidence.playback = await page.evaluate(() => window.scopePlayback);
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card()).toBeVisible();
  await selector().getByRole('checkbox', { name: ecoleCategory.name, exact: true }).check();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible();
  assert.deepEqual(await applied(), scope([voyageCategory.id]));
  const persisted = readState(destination);
  await restart(); assert.deepEqual(await applied(), scope([voyageCategory.id]));
  compareLearning(readState(destination), persisted);
  await enroll([words[1], words[2]]);
  const old = await queue();
  const cursor = { revision: old.summary.revision, scope: old.summary.scope, window: old.window.map((entry) => ({
    card: entry.card, record_revision: entry.record.revision, progress_revision: entry.progress.revision, version: entry.version, content_hash: entry.content_hash,
  })) };
  await setScope(scope([ecoleCategory.id]));
  const replaced = await queue([], cursor); assert.equal(replaced.delta.before_revision, null);
  assert.ok(replaced.queue.cards.every((value) => value.vocabulary_id === words[1].id));
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[0].id], category_id: ecoleCategory.id, action: 'add' } });
  await setScope(scope([voyageCategory.id, ecoleCategory.id]));
  const union = await inventory(); assert.ok(union.every((value) => value.kind !== 'grammar' && [words[0].id, words[1].id].includes(value.vocabulary_id)));
  assert.equal(new Set(union.map((value) => value.id)).size, union.length);
  await setScope(scope([], true)); assert.ok((await inventory()).every((value) => value.vocabulary_id === words[2].id));
  await setScope(scope([ecoleCategory.id], true)); assert.ok((await inventory()).every((value) => value.kind !== 'grammar'));
  evidence.checks.push('T01 T02 T04 T05 T07 T08 T09 T10: mixed All, pending skip, real selected audio playback/rating, applied-only restart, stale native cursor, union and Uncategorized');
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await page.screenshot({ path: join(destination, 'native-category-review.png') });
}

async function live() {
  await enroll(); await setScope(scope([voyageCategory.id]));
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card().getByRole('heading', { name: words[0].headword })).toBeVisible();
  await invoke('category_rename', { id: voyageCategory.id, name: 'Voyage renommé' });
  await expect(page.getByText('Applied scope: Voyage renommé')).toBeVisible(); assert.deepEqual(await applied(), scope([voyageCategory.id]));
  const untouched = await queue();
  const failed = await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[1].id], category_id: 99999, action: 'add' } });
  assert.equal(failed[0].status, 'failed');
  assert.deepEqual((await queue()).queue.cards, untouched.queue.cards);
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[0].id], category_id: voyageCategory.id, action: 'remove' } });
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[1].id], category_id: voyageCategory.id, action: 'add' } });
  await expect(card().getByRole('heading', { name: words[0].headword })).toBeVisible();
  const following = await queue(); assert.ok(following.queue.cards.every((value) => value.vocabulary_id === words[1].id));
  await card().getByRole('button', { name: 'Show answer', exact: true }).click();
  await card().getByRole('button', { name: 'Good', exact: true }).click();
  await expect(card().getByRole('heading', { name: words[1].headword })).toBeVisible();
  await setScope(scope([voyageCategory.id, ecoleCategory.id])); await invoke('category_delete', { id: voyageCategory.id }); assert.deepEqual(await applied(), scope([ecoleCategory.id]));
  await invoke('category_delete', { id: ecoleCategory.id }); assert.deepEqual(await applied(), needs);
  await card().getByRole('button', { name: 'Skip', exact: true }).click();
  await expect(page.getByText('Choose a review scope', { exact: true })).toBeVisible();
  await restart(); assert.deepEqual(await applied(), needs);
  evidence.checks.push('T11 T12 T13: native rename, failed membership save, warm-cache committed changes, finish current card, partial/final deletion and restart');
}

async function undo() {
  await enroll();
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[2].id], category_id: voyageCategory.id, action: 'add' } });
  await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 2, review_limit: 2, new_cards_ignore_review_limit: false } });
  await setScope(scope([voyageCategory.id])); const first = await rate(await recognition(words[0]), 'scope-rating-a');
  await setScope(scope([ecoleCategory.id])); const second = await rate(await recognition(words[1]), 'scope-rating-b');
  await setScope(scope([voyageCategory.id])); const exhausted = await queue(); assert.equal(exhausted.queue.reason, 'quota');
  assert.equal(await invoke('review_undo_target'), second.request.operation);
  await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 3, review_limit: 3, new_cards_ignore_review_limit: false } });
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click(); await expect(card()).toBeVisible();
  await selector().getByRole('checkbox', { name: ecoleCategory.name, exact: true }).check();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible();
  await page.getByRole('button', { name: 'Undo last rating', exact: true }).click();
  await expect.poll(async () => (await invoke('review_progress', { cardId: second.request.card_id })).card.phase).toBe('new');
  assert.deepEqual(await applied(), scope([voyageCategory.id]));
  const refunded = await queue(); assert.ok(refunded.queue.cards.some((value) => value.vocabulary_id === words[2].id));
  assert.ok(refunded.queue.cards.every((value) => value.vocabulary_id !== words[1].id));
  const before = readState(destination);
  const request = { operation: before.ratings.find((value) => value.operation === second.request.operation).undo_operation, ratingOperation: second.request.operation };
  assert.ok(request.operation); assert.equal((await invoke('review_undo', request)).status, 'committed'); compareLearning(readState(destination), before);
  const invalid = await invoke('review_undo', { operation: 'scope-invalid-older-undo', ratingOperation: first.request.operation }); assert.equal(invalid.status, 'rejected');
  compareLearning(readState(destination), before);
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible(); assert.deepEqual(await applied(), scope([voyageCategory.id]));
  const progress = await invoke('review_progress', { cardId: first.request.card_id }); assert.notEqual(progress.card.phase, 'new');
  await restart(); assert.deepEqual(await applied(), scope([voyageCategory.id])); assert.ok((await queue()).queue.cards.every((value) => value.vocabulary_id !== words[1].id));
  evidence.checks.push('T03 T15 T16: pre-admission scope, shared quota, globally latest undo outside scope, refund/replay, pending choice preserved and restart');
}

async function diagnostics() {
  await setScope(scope([voyageCategory.id])); assert.equal((await queue()).queue.reason, 'empty_scope');
  await enroll([words[0]]); const current = await recognition(words[0]);
  await invoke('review_pause', { vocabularyId: words[0].id, paused: true }); assert.equal((await queue()).queue.reason, 'paused');
  await invoke('review_pause', { vocabularyId: words[0].id, paused: false });
  await rate(current, 'scope-future'); const future = await queue(); assert.equal(future.queue.reason, 'due'); assert.ok(future.queue.available_at > Date.now() / 1000);
  assert.ok(future.audio_error || future.audio_pending || future.summary.audio_pending_count > 0);
  assert.ok(future.queue.cards.every((value) => value.kind !== 'listening'));
  assert.deepEqual(await applied(), scope([voyageCategory.id]));
  await setScope(needs); assert.equal((await queue()).queue.reason, 'needs_selection');
  await restart(); assert.deepEqual(await applied(), needs);
  evidence.checks.push('T04 T10 T14: no enrollment, paused, future due, unavailable listening audio and explicit choice keep diagnostics/scope');
}

async function failure() {
  await enroll();
  await expect.poll(async () => (await Promise.all(words.map((word) => invoke('speech_cached', { vocabularyId: word.id })))).every(Boolean), { timeout: 20000 }).toBe(true);
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[2].id], category_id: voyageCategory.id, action: 'add' } });
  await setScope(scope([voyageCategory.id]));
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click(); await expect(card()).toBeVisible();
  await card().getByRole('button', { name: 'Show answer', exact: true }).click();
  await selector().getByRole('checkbox', { name: ecoleCategory.name, exact: true }).check();
  await selector().getByRole('checkbox', { name: voyageCategory.name, exact: true }).uncheck();
  let lock, released;
  async function hold() {
    lock = spawn('python', ['-B', '-c', "import sqlite3,sys\nc=sqlite3.connect(sys.argv[1]); c.execute('BEGIN IMMEDIATE'); print('ready',flush=True); sys.stdin.readline(); c.rollback()", join(destination, 'Lexique-data/data/lexique.sqlite3')], { windowsHide: true });
    fixture.ownHelper(lock); let stderr = ''; lock.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
    released = new Promise((done, reject) => { lock.once('error', reject); lock.once('exit', (code) => code === 0 ? done() : reject(new Error(stderr))); });
    released.catch(() => {});
    await new Promise((done, reject) => { lock.stdout.once('data', (bytes) => bytes.toString().trim() === 'ready' ? done() : reject(new Error('SQLite writer did not become ready'))); lock.once('error', reject); lock.once('exit', () => reject(new Error(stderr || 'SQLite writer exited before readiness'))); });
  }
  async function release() { lock.stdin.end('release\n'); await released; }
  await expect(card().getByRole('button', { name: 'Good', exact: true })).toBeEnabled();
  await hold();
  await card().getByRole('button', { name: 'Good', exact: true }).click();
  await expect(card().getByRole('alert')).toContainText('locked', { timeout: 10000 });
  await expect(page.getByText('Applies after rating or skipping the current card: École')).toBeVisible();
  await release(); assert.equal(readState(destination).ratings.length, 0); assert.deepEqual(await applied(), scope([voyageCategory.id]));
  await selector().getByRole('checkbox', { name: voyageCategory.name, exact: true }).check();
  await selector().getByRole('checkbox', { name: ecoleCategory.name, exact: true }).uncheck();
  await card().getByRole('button', { name: 'Retry', exact: true }).click();
  await expect.poll(() => readState(destination).ratings.length).toBe(1);
  await expect(card()).toBeVisible();
  await selector().getByRole('checkbox', { name: ecoleCategory.name, exact: true }).check();
  await selector().getByRole('checkbox', { name: voyageCategory.name, exact: true }).uncheck();
  await hold();
  await card().getByRole('button', { name: 'Skip', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'Could not save review scope' })).toContainText('locked', { timeout: 10000 });
  await expect(card()).toHaveCount(0); await expect(page.getByText('Applied scope: Voyage')).toBeVisible();
  await release(); assert.equal(readState(destination).ratings.length, 1);
  const beforeRetry = readState(destination);
  await page.evaluate(() => window.dispatchEvent(new Event('focus')));
  await invoke('category_rename', { id: ecoleCategory.id, name: 'École actualisée' });
  await expect(page.getByText('Applies after rating or skipping the current card: École actualisée')).toBeVisible();
  await expect(page.getByRole('alert').filter({ hasText: 'Could not save review scope' })).toContainText('locked');
  await expect(page.getByRole('button', { name: 'Retry saving review scope', exact: true })).toBeVisible();
  await expect(card()).toHaveCount(0); assert.deepEqual(await applied(), scope([voyageCategory.id]));
  await page.getByRole('button', { name: 'Retry saving review scope', exact: true }).click();
  await expect(page.getByText('Applied scope: École actualisée')).toBeVisible(); await expect(card()).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);
  const afterRetry = readState(destination);
  assert.deepEqual(afterRetry.ratings, beforeRetry.ratings); assert.deepEqual(afterRetry.progress, beforeRetry.progress);
  assert.deepEqual(afterRetry.retained_review_usage, beforeRetry.retained_review_usage);
  assert.equal(readState(destination).ratings.length, 1); assert.deepEqual(await applied(), scope([ecoleCategory.id]));
  const next = await queue(); assert.ok(next.queue.cards.every((value) => value.vocabulary_id === words[1].id));
  evidence.checks.push('T05 T06: real SQLite write-lock rating/save failures, preserved answer/pending input, cancel/reselect, focus/review-change retains save failure and completed boundary, save-only retry preserves rating/progress/global usage');
}

function helper(action, filename = '', filenameId = 0) {
  const child = spawn('pwsh', ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', action,
    '-Title', 'Lexique', '-SourcePid', String(fixture.hostPid()), '-SourceExecutable', join(destination, 'Lexique.exe'), '-WaitMs', '4000', '-Text', filename, '-FilenameControlId', String(filenameId)], { windowsHide: true });
  fixture.ownHelper(child); let stdout = '', stderr = '';
  child.stdout.on('data', (bytes) => { stdout += bytes.toString(); if (stdout.length > 65536) child.kill(); }); child.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
  const timer = setTimeout(() => child.kill(), 8000);
  return new Promise((done, reject) => {
    child.once('error', (error) => { clearTimeout(timer); reject(error); });
    child.once('exit', (code) => { clearTimeout(timer); if (code !== 0) reject(new Error(`Owned helper failed: ${code}: ${stderr}`)); else { try { done(JSON.parse(stdout)); } catch (error) { reject(error); } } });
  });
}

async function picker(button, filename) {
  const observed = helper('picker-controls'); observed.catch(() => {});
  const clicked = page.getByRole('button', { name: button, exact: true }).click({ noWaitAfter: true }); clicked.catch(() => {});
  const controls = await observed; assert.equal(controls.pid, fixture.hostPid()); assert.equal(controls.class, '#32770');
  const id = button === 'Create backup' ? 1001 : 1148;
  assert.equal(controls.controls.filter((value) => value.class === 'Edit' && value.id === id && value.visible && value.enabled).length, 1);
  const selected = await helper('picker-select', filename, id); await clicked; assert.equal(selected.disappeared, true); assert.equal(selected.filename, filename);
}

function variant(filename, kind) {
  const output = join(destination, `${kind}.lexique-backup`);
  const result = spawnSync('python', ['-B', '-X', 'utf8', '-c', `import zipfile,json,sys
with zipfile.ZipFile(sys.argv[1]) as z: entries=[(i,z.read(i)) for i in z.infolist()]
snapshot=json.loads(next(b for i,b in entries if i.filename=='snapshot.json'))
if sys.argv[3]=='old': del snapshot['preferences']['review_scope']
else: snapshot['preferences']['review_scope']={'kind':'vocabulary','category_ids':[99999],'include_uncategorized':False}
with zipfile.ZipFile(sys.argv[2],'x',compression=zipfile.ZIP_STORED) as z:
 for i,b in entries: z.writestr(i,json.dumps(snapshot,ensure_ascii=False).encode() if i.filename=='snapshot.json' else b)
`, filename, output, kind], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
  assert.equal(result.status, 0, result.stderr); return output;
}

async function backup() {
  await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 20, review_limit: 50, new_cards_ignore_review_limit: false } });
  await invoke('review_auto_enroll', { enabled: false });
  await invoke('speech_preferences', { preferences: { voice: 'female', wpm: 180 } });
  await invoke('conjugation_preferences', { preferences: { mode: 'ending' } });
  await invoke('conjugation_block_order', { order: await invoke('conjugation_block_order') });
  await invoke('vocabulary_sort', { sort: 'updated-desc' });
  await invoke('dictionary_settings', { order: (await invoke('dictionary_settings')).map((value) => value.id) });
  await rate(grammar.card, 'scope-backup-rating');
  const choices = { 'backup-all': all, 'backup-single': scope([voyageCategory.id]), 'backup-multiple': scope([voyageCategory.id, ecoleCategory.id], true),
    'backup-uncategorized': scope([], true), 'backup-needs-selection': needs, 'backup-old': scope([voyageCategory.id]), 'backup-invalid': scope([voyageCategory.id]) };
  const selected = choices[scenario]; await setScope(selected);
  let audio;
  if (scenario === 'backup-single') {
    await enroll([words[0]]);
    await expect.poll(async () => { audio = await invoke('speech_cached', { vocabularyId: words[0].id }); return audio; }, { timeout: 20000 }).not.toBeNull();
    await rate(await recognition(words[0]), 'scope-backup-vocabulary-rating');
  }
  if (scenario === 'backup-all') {
    await enroll([words[0]]); await page.getByRole('button', { name: 'Flashcards', exact: true }).click(); await expect(card()).toBeVisible();
    await selector().getByRole('checkbox', { name: voyageCategory.name, exact: true }).check();
    await expect(page.getByText('Applies after rating or skipping the current card: Voyage')).toBeVisible(); assert.deepEqual(await applied(), all);
  }
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.evaluate(async () => {
    window.scopeBackupEvents = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress']) await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
      handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.scopeBackupEvents.push({ event, payload: value.payload }), false) });
  });
  const latest = (event) => page.evaluate((name) => window.scopeBackupEvents.filter((value) => value.event === name).at(-1)?.payload, event);
  let filename = join(destination, 'scope.lexique-backup'); await picker('Create backup', filename);
  let exported; await expect.poll(async () => { exported = await latest('lexique-backup-progress'); return exported?.state; }, { timeout: 10000 }).toBe('completed');
  const archive = inspectPackage(destination, filename); assert.deepEqual(archive.snapshot.preferences.review_scope, selected);
  if (audio) assert.ok(archive.entries.some((name) => name.startsWith('audio/')));
  assert.equal(archive.snapshot.snapshot_version, 4); assert.equal(archive.snapshot.categories.length, 2);
  await writeFile(join(destination, 'snapshot.json'), JSON.stringify(archive.snapshot, null, 2));
  const original = readState(destination);
  await setScope(scope([ecoleCategory.id]));
  if (scenario === 'backup-invalid') {
    filename = variant(filename, 'invalid'); const before = readState(destination); await picker('Choose backup', filename);
    await expect.poll(async () => (await latest('lexique-backup-preview-progress'))?.state, { timeout: 10000 }).toBe('failed');
    compareLearning(readState(destination), before); assert.deepEqual(await applied(), scope([ecoleCategory.id]));
    assert.equal((await invoke('startup_status')).state, 'ready');
    evidence.checks.push('T18: native dangling incoming scope rejected before replacement, destination learning data/scope preserved'); return;
  }
  if (scenario === 'backup-old') filename = variant(filename, 'old');
  await picker('Choose backup', filename);
  await expect.poll(async () => (await latest('lexique-backup-preview-progress'))?.state, { timeout: 10000 }).toBe('preview_ready');
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore' });
  const confirm = dialog.getByRole('button', { name: 'Confirm and restore', exact: true });
  await expect(confirm).toBeDisabled(); await dialog.getByRole('checkbox').check();
  page = await fixture.followRestart(() => confirm.click()); page.setDefaultTimeout(6000); await page.context().setOffline(true);
  await expect.poll(async () => (await invoke('startup_status')).state, { timeout: 10000 }).toBe('ready');
  assert.deepEqual(await applied(), scenario === 'backup-old' ? all : selected);
  compareLearning(readState(destination), original); assert.deepEqual(await invoke('learning_categories'), [voyageCategory, ecoleCategory]);
  if (audio) assert.deepEqual(await invoke('speech_cached', { vocabularyId: words[0].id }), audio);
  evidence.checks.push(`T17 T18: native full backup/restore and controlled restart for ${scenario}, category identities/memberships and destination replacement`);
}

try {
  for (const name of ['conjugation', 'frequency']) await cp(join(resolve(resources), name), join(destination, name), { recursive: true });
  if (!['selection', 'failure', 'backup-single'].includes(scenario)) await rename(join(destination, 'speech/assets/text_encoder.int8.onnx'), join(destination, 'speech/assets/text_encoder.int8.onnx.unavailable'));
  page = await fixture.start(); page.setDefaultTimeout(6000); await page.context().setOffline(true);
  await expect(page.getByRole('banner')).toBeVisible(); await seed();
  await ({ selection, live, undo, diagnostics, failure }[scenario] ?? backup)();
  evidence.scope = await applied(); evidence.sqlite = readState(destination);
  evidence.storedScope = fixtureSql(destination, "print(json.dumps(dict(c.execute(\"SELECT key,value FROM app_metadata WHERE key='review-scope'\").fetchone())))", {}, true);
  assert.equal(evidence.sqlite.schema, 20); assert.deepEqual(evidence.sqlite.integrity, ['ok']); assert.deepEqual(evidence.sqlite.foreign_keys, []);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify({ scenario, passed: true, checks: evidence.checks.length, queueReads: evidence.views.length }));
  await fixture.stop(true);
} finally { await fixture.finish(); }
