/* global process, window, console, document, HTMLMediaElement, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { fixtureSql, inspectPackage, readState, compareLearning } from './native-backup-state.mjs';

const [resources, binary, destinationPath, scenario] = process.argv.slice(2);
assert.ok(['selection', 'live', 'undo', 'diagnostics', 'backup-all', 'backup-single', 'backup-multiple', 'backup-uncategorized', 'backup-needs-selection', 'backup-old', 'backup-invalid'].includes(scenario));
const fixture = await native24Fixture(resources, binary, destinationPath);
const { destination } = fixture;
const evidence = { scenario, checks: [], views: [] };
const all = { kind: 'all' }, needs = { kind: 'needs_selection' };
const scope = (category_ids = [], include_uncategorized = false) => ({ kind: 'vocabulary', category_ids, include_uncategorized });
let page, a, b, words, grammar;
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
  a = await invoke('category_create', { name: 'Voyage' }); b = await invoke('category_create', { name: 'École' });
  words = [];
  for (const [headword, category_ids] of [['école', [a.id]], ['bonjour', [b.id]], ['œil', []]]) {
    words.push((await invoke('vocabulary_save', { request: { operation: `scope-${headword}`, headword, category_ids,
      notes: 'Synthetic category acceptance', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } })).record);
  }
  grammar = (await invoke('grammar_save', { request: { operation: 'scope-grammar', title: 'Une règle', body: 'Un exemple', category_ids: [a.id], id: null, revision: null } })).record;
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
  await selector().getByRole('checkbox', { name: a.name, exact: true }).check();
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
  await card().getByRole('button', { name: 'Show answer', exact: true }).click();
  await expect(card().getByRole('button', { name: 'Good', exact: true })).toBeEnabled();
  await card().getByRole('button', { name: 'Good', exact: true }).click();
  await expect.poll(async () => (await invoke('review_progress', { cardId: listening.queue.cards[0].id })).revision).toBe(1);
  evidence.playback = await page.evaluate(() => window.scopePlayback);
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card()).toBeVisible();
  await selector().getByRole('checkbox', { name: b.name, exact: true }).check();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible();
  assert.deepEqual(await applied(), scope([a.id]));
  const persisted = readState(destination);
  await restart(); assert.deepEqual(await applied(), scope([a.id]));
  compareLearning(readState(destination), persisted);
  await enroll([words[1], words[2]]);
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[0].id], category_id: b.id, action: 'add' } });
  await setScope(scope([a.id, b.id]));
  const union = await inventory(); assert.ok(union.every((value) => value.kind !== 'grammar' && [words[0].id, words[1].id].includes(value.vocabulary_id)));
  assert.equal(new Set(union.map((value) => value.id)).size, union.length);
  await setScope(scope([], true)); assert.ok((await inventory()).every((value) => value.vocabulary_id === words[2].id));
  await setScope(scope([b.id], true)); assert.ok((await inventory()).every((value) => value.kind !== 'grammar'));
  evidence.checks.push('T01 T02 T04 T05 T07 T09 T10: mixed All, pending skip, real selected audio playback/rating, restart, union and Uncategorized');
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await page.screenshot({ path: join(destination, 'native-category-review.png') });
}

async function live() {
  await enroll(); await setScope(scope([a.id]));
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(card().getByRole('heading', { name: words[0].headword })).toBeVisible();
  await invoke('category_rename', { id: a.id, name: 'Voyage renommé' });
  await expect(page.getByText('Applied scope: Voyage renommé')).toBeVisible(); assert.deepEqual(await applied(), scope([a.id]));
  const untouched = await queue();
  const failed = await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[1].id], category_id: 99999, action: 'add' } });
  assert.equal(failed[0].status, 'failed');
  assert.deepEqual((await queue()).queue.cards, untouched.queue.cards);
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[0].id], category_id: a.id, action: 'remove' } });
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[1].id], category_id: a.id, action: 'add' } });
  await expect(card().getByRole('heading', { name: words[0].headword })).toBeVisible();
  const following = await queue(); assert.ok(following.queue.cards.every((value) => value.vocabulary_id === words[1].id));
  await card().getByRole('button', { name: 'Show answer', exact: true }).click();
  await card().getByRole('button', { name: 'Good', exact: true }).click();
  await expect(card().getByRole('heading', { name: words[1].headword })).toBeVisible();
  await setScope(scope([a.id, b.id])); await invoke('category_delete', { id: a.id }); assert.deepEqual(await applied(), scope([b.id]));
  await invoke('category_delete', { id: b.id }); assert.deepEqual(await applied(), needs);
  await card().getByRole('button', { name: 'Skip', exact: true }).click();
  await expect(page.getByText('Choose a review scope', { exact: true })).toBeVisible();
  await restart(); assert.deepEqual(await applied(), needs);
  evidence.checks.push('T11 T12 T13: native rename, failed membership save, warm-cache committed changes, finish current card, partial/final deletion and restart');
}

async function undo() {
  await enroll();
  await invoke('category_bulk', { request: { record_kind: 'vocabulary', ids: [words[2].id], category_id: a.id, action: 'add' } });
  await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 2, review_limit: 2, new_cards_ignore_review_limit: false } });
  await setScope(scope([a.id])); const first = await rate(await recognition(words[0]), 'scope-rating-a');
  await setScope(scope([b.id])); const second = await rate(await recognition(words[1]), 'scope-rating-b');
  await setScope(scope([a.id])); const exhausted = await queue(); assert.equal(exhausted.queue.reason, 'quota');
  assert.equal(await invoke('review_undo_target'), second.request.operation);
  const request = { operation: 'scope-undo-b', ratingOperation: second.request.operation };
  assert.equal((await invoke('review_undo', request)).status, 'committed'); assert.deepEqual(await applied(), scope([a.id]));
  const refunded = await queue(); assert.ok(refunded.queue.cards.some((value) => value.vocabulary_id === words[2].id));
  assert.ok(refunded.queue.cards.every((value) => value.vocabulary_id !== words[1].id));
  const before = readState(destination); assert.equal((await invoke('review_undo', request)).status, 'committed'); compareLearning(readState(destination), before);
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click(); await expect(card()).toBeVisible();
  await selector().getByRole('checkbox', { name: b.name, exact: true }).check();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible();
  await page.getByRole('button', { name: 'Undo last rating', exact: true }).click();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage, École')).toBeVisible(); assert.deepEqual(await applied(), scope([a.id]));
  const progress = await invoke('review_progress', { cardId: first.request.card_id }); assert.equal(progress.card.phase, 'new');
  await restart(); assert.deepEqual(await applied(), scope([a.id])); assert.ok((await queue()).queue.cards.every((value) => value.vocabulary_id !== words[1].id));
  evidence.checks.push('T03 T15 T16: pre-admission scope, shared quota, globally latest undo outside scope, refund/replay, pending choice preserved and restart');
}

async function diagnostics() {
  await setScope(scope([a.id])); assert.equal((await queue()).queue.reason, 'empty_scope');
  await enroll([words[0]]); const current = await recognition(words[0]);
  await invoke('review_pause', { vocabularyId: words[0].id, paused: true }); assert.equal((await queue()).queue.reason, 'paused');
  await invoke('review_pause', { vocabularyId: words[0].id, paused: false });
  await rate(current, 'scope-future'); const future = await queue(); assert.equal(future.queue.reason, 'due'); assert.ok(future.queue.available_at > Date.now() / 1000);
  const listening = (await invoke('review_cards', { vocabularyId: words[0].id })).find((value) => value.kind === 'listening');
  const unavailable = await invoke('review_availability', { cardId: listening.id, timezone: 'UTC', presentationRevision: null });
  assert.equal(unavailable.reason, 'audio'); assert.deepEqual(await applied(), scope([a.id]));
  await setScope(needs); assert.equal((await queue()).queue.reason, 'needs_selection');
  await restart(); assert.deepEqual(await applied(), needs);
  evidence.checks.push('T04 T10 T14: no enrollment, paused, future due, unavailable listening audio and explicit choice keep diagnostics/scope');
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
  const choices = { 'backup-all': all, 'backup-single': scope([a.id]), 'backup-multiple': scope([a.id, b.id], true),
    'backup-uncategorized': scope([], true), 'backup-needs-selection': needs, 'backup-old': scope([a.id]), 'backup-invalid': scope([a.id]) };
  const selected = choices[scenario]; await setScope(selected);
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
  assert.equal(archive.snapshot.snapshot_version, 4); assert.equal(archive.snapshot.categories.length, 2);
  await writeFile(join(destination, 'snapshot.json'), JSON.stringify(archive.snapshot, null, 2));
  const original = readState(destination);
  await setScope(scope([b.id]));
  if (scenario === 'backup-invalid') {
    filename = variant(filename, 'invalid'); const before = readState(destination); await picker('Choose backup', filename);
    await expect.poll(async () => (await latest('lexique-backup-preview-progress'))?.state, { timeout: 10000 }).toBe('failed');
    compareLearning(readState(destination), before); assert.deepEqual(await applied(), scope([b.id]));
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
  compareLearning(readState(destination), original); assert.deepEqual(await invoke('learning_categories'), [a, b]);
  evidence.checks.push(`T17 T18: native full backup/restore and controlled restart for ${scenario}, category identities/memberships and destination replacement`);
}

try {
  for (const name of ['conjugation', 'frequency']) await cp(join(resolve(resources), name), join(destination, name), { recursive: true });
  if (scenario !== 'selection') await rename(join(destination, 'speech/assets/text_encoder.int8.onnx'), join(destination, 'speech/assets/text_encoder.int8.onnx.unavailable'));
  page = await fixture.start(); page.setDefaultTimeout(6000); await page.context().setOffline(true);
  await expect(page.getByRole('banner')).toBeVisible(); await seed();
  await ({ selection, live, undo, diagnostics }[scenario] ?? backup)();
  evidence.scope = await applied(); evidence.sqlite = readState(destination);
  evidence.storedScope = fixtureSql(destination, "print(json.dumps(dict(c.execute(\"SELECT key,value FROM app_metadata WHERE key='review-scope'\").fetchone())))", {}, true);
  assert.equal(evidence.sqlite.schema, 20); assert.deepEqual(evidence.sqlite.integrity, ['ok']); assert.deepEqual(evidence.sqlite.foreign_keys, []);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify({ scenario, passed: true, checks: evidence.checks.length, queueReads: evidence.views.length }));
  await fixture.stop(true);
} finally { await fixture.finish(); }
