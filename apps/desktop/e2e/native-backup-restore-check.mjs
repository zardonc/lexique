/* global process, window, localStorage, Buffer, console, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, readFile, readdir, rename, stat, unlink, writeFile } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { readState, compareLearning, corruptPackage, fixtureSql, inspectPackage, seedAudio, seedStartupFailure, seedUnknownDictionary } from './native-backup-state.mjs';

const [resources, binary, destinationPath, scenario, input] = process.argv.slice(2);
assert.ok(resources && binary && destinationPath && scenario, 'Expected resources, fresh binary, new destination and scenario');
const fixture = await native24Fixture(resolve(resources), resolve(binary), resolve(destinationPath));
const { destination } = fixture;
for (const name of ['dictionary', 'conjugation', 'frequency']) await cp(resolve('../../target/release', name), join(destination, name), { recursive: true });
const evidence = { scenario, checks: [], pickers: [] };
let page;
const model = join(destination, 'speech/assets/text_encoder.int8.onnx');
await rename(model, model + '.unavailable');

function helper(action, options = {}) {
  const args = ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', action,
    '-SourcePid', String(options.pid ?? fixture.hostPid()), '-SourceExecutable', options.executable ?? join(destination, 'Lexique.exe'),
    '-WaitMs', String(options.waitMs ?? 4000)];
  if (options.filename) args.push('-Text', options.filename, '-FilenameControlId', String(options.filenameId));
  if (options.title) args.push('-Title', options.title);
  const child = spawn('pwsh', args, { windowsHide: true }); fixture.ownHelper(child);
  let stdout = '', stderr = '';
  child.stdout.on('data', (bytes) => { stdout += bytes.toString(); if (stdout.length > 65536) child.kill(); });
  child.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
  const timer = setTimeout(() => child.kill(), 8000);
  return new Promise((done, reject) => {
    child.once('error', reject);
    child.once('exit', (code) => {
      clearTimeout(timer);
      if (code !== 0) reject(new Error(`Owned helper ${action} exit ${code}: ${stderr}`));
      else { try { done(stdout.trim() ? JSON.parse(stdout) : null); } catch (error) { reject(error); } }
    });
  });
}
async function invoke(command, args = {}) {
  const call = { command, args, pending: true };
  (evidence.calls ??= []).push(call);
  try {
    const value = await page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
    call.value = value?.bytes ? { ...value, bytes: value.bytes.length } : value;
    return value;
  } catch (error) {
    call.error = String(error);
    throw error;
  } finally { call.pending = false; }
}
async function observe() {
  await page.evaluate(async () => {
    window.backupAcceptanceStatuses = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress', 'lexique-backup-restore-progress', 'lexique-startup-status']) {
      await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
        handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.backupAcceptanceStatuses.push({ event, payload: value.payload }), false) });
    }
  });
}
async function latest(command) {
  const event = { backup_export_start: 'lexique-backup-progress', backup_preview_start: 'lexique-backup-preview-progress' }[command];
  return page.evaluate((event) => window.backupAcceptanceStatuses?.filter((value) => value.event === event).at(-1)?.payload, event);
}
async function settings() {
  await expect(page.getByRole('banner')).toBeVisible();
  if (await page.getByRole('banner').getByRole('combobox').first().inputValue() !== 'en') {
    await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  }
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
}
async function preferences(a) {
  await invoke('review_settings', { settings: { retention_percent: a ? 87 : 92, new_limit: a ? 2 : 20,
    review_limit: a ? 4 : 50, new_cards_ignore_review_limit: !a } });
  await invoke('speech_preferences', { preferences: { voice: a ? 'female' : 'male', wpm: a ? 100 : 240 } });
  await invoke('interface_preferences', { preferences: { locale: 'en', theme: a ? 'sepia' : 'dark' } });
  await invoke('conjugation_preferences', { preferences: { mode: a ? 'ending' : 'verbal' } });
  const blocks = await invoke('conjugation_block_order');
  await invoke('conjugation_block_order', { order: a ? [...blocks].reverse() : blocks });
  await invoke('vocabulary_sort', { sort: a ? 'updated-asc' : 'headword-desc' });
  const dictionaries = await invoke('dictionary_settings');
  await invoke('dictionary_settings', { order: dictionaries.map((dictionary) => dictionary.id) });
}
async function seed(timeline) {
  const a = timeline === 'A';
  const saved = [];
  for (const [index, headword] of ['école', 'apprendre', 'parler'].entries()) {
    const request = { operation: `${timeline}-save-${index}`, headword, notes: `${timeline} notes ${index}: café, 中文`,
      example_fr: `Je vais à l’école. ${timeline}${index}`, example_zh: `${timeline}：合成学习例句 ${index}`,
      example_sense: 0, more_examples: [], id: null, revision: null };
    const result = await invoke('vocabulary_save', { request });
    assert.equal(result.outcome, 'created');
    saved.push({ request, result });
  }
  const first = saved[0].result.record;
  const request = { ...saved[0].request, operation: `${timeline}-excerpt`, id: first.id, revision: first.revision,
    example_source: first.reference, more_examples: [{ fr: 'Elle apprend le français.', zh: '她学法语。', source: first.reference }, { fr: 'Source inconnue.', zh: '', source: null }] };
  saved.push({ request, result: await invoke('vocabulary_save', { request }) });
  await preferences(a);
  const ids = saved.slice(0, 3).map((value) => value.result.record.id);
  const batch = { operation: `${timeline}-z-batch`, vocabulary_ids: [ids[0], ids[1], ids[1], 9999] };
  const batchResult = await invoke('review_add_batch', { request: batch });
  assert.deepEqual(batchResult.items.map((item) => item.vocabulary_id), [ids[0], ids[1], 9999]);
  const cancelled = { operation: `${timeline}-a-batch`, vocabulary_ids: [ids[2]] };
  await invoke('review_add_batch', { request: cancelled });
  await invoke('review_cancel_batch', { operation: cancelled.operation });
  await invoke('review_pause', { vocabularyId: ids[2], paused: true });
  await invoke('review_auto_enroll', { enabled: a });
  await fixture.stop(true);
  const audio = seedAudio(destination, ids, timeline, resolve('../../docs/evidence/issue-56'));
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await settings(); await observe();
  const cards = await invoke('review_cards', { vocabularyId: ids[0] });
  const secondCards = await invoke('review_cards', { vocabularyId: ids[1] });
  const selected = [cards, secondCards].map((value) => value.find((card) => card.kind === 'recognition'));
  const ratings = [];
  for (const [index, card] of selected.entries()) {
    await invoke('review_queue', { timezone: 'UTC', skipped: [] });
    const progress = await invoke('review_progress', { cardId: card.id });
    const request = { operation: `${timeline}-${index === 0 ? 'z' : 'a'}-rating`, card_id: card.id,
      expected_revision: progress.revision, rating: 'again', timezone: 'UTC' };
    const result = await invoke('review_rate', { request }); assert.equal(result.status, 'committed');
    ratings.push({ request, result });
  }
  const undo = { operation: `${timeline}-undo-a`, ratingOperation: ratings[1].request.operation };
  assert.equal((await invoke('review_undo', undo)).status, 'committed');
  await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  const progress = await invoke('review_progress', { cardId: selected[1].id });
  const finalRating = { operation: `${timeline}-b-rating`, card_id: selected[1].id,
    expected_revision: progress.revision, rating: 'good', timezone: 'UTC' };
  const finalResult = await invoke('review_rate', { request: finalRating }); assert.equal(finalResult.status, 'committed');
  ratings.push({ request: finalRating, result: finalResult });
  const result = { timeline, ids, saved, batch, cancelled, ratings, undo, audio };
  evidence.seed = result; return result;
}
async function picker(button, filename) {
  const observed = helper('picker-controls');
  observed.catch(() => {});
  const clicked = page.getByRole('button', { name: button, exact: true }).click({ noWaitAfter: true });
  clicked.catch(() => {});
  const controls = await observed;
  evidence.pickers.push({ trigger: button, observed: controls });
  assert.equal(controls.pid, fixture.hostPid()); assert.equal(controls.class, '#32770');
  const filenameId = button === 'Create backup' ? 1001 : 1148;
  const edits = controls.controls.filter((control) => control.class === 'Edit' && control.id === filenameId && control.visible && control.enabled);
  assert.equal(edits.length, 1, `Calibrate the filename Edit/${filenameId} before selecting`);
  for (const id of [1, 2]) assert.equal(controls.controls.filter((control) => control.class === 'Button' && control.id === id && control.visible && control.enabled).length, 1);
  const result = await helper(filename ? 'picker-select' : 'picker-cancel', { filename, filenameId: edits[0].id });
  await clicked;
  assert.equal(result.dialog, controls.dialog); assert.equal(result.disappeared, true);
  if (filename) assert.equal(result.filename, filename);
  evidence.pickers.at(-1).action = result;
}
async function exportBackup(filename) {
  await picker('Create backup', filename);
  await expect.poll(async () => (await latest('backup_export_start'))?.operation, { timeout: 8000 }).toBeTruthy();
  const operation = (await latest('backup_export_start')).operation;
  let status;
  await expect.poll(async () => { status = await invoke('backup_operation_status', { operation }); return status.state; }, { timeout: 10000 }).toBe('completed');
  assert.equal(status.result.destination, filename); assert.equal((await stat(filename)).size, status.result.byte_length);
  await expect(page.getByText('Backup saved', { exact: true })).toBeVisible();
  evidence.checks.push({ export: status });
  return status;
}
async function preview(filename, recovery = false) {
  await picker(recovery ? '选择备份' : 'Choose backup', filename);
  await expect.poll(async () => (await latest('backup_preview_start'))?.operation, { timeout: 8000 }).toBeTruthy();
  const operation = (await latest('backup_preview_start')).operation;
  let status;
  await expect.poll(async () => { status = await invoke('backup_preview_status', { operation }); return status.state; }, { timeout: 10000 }).toBe('preview_ready');
  assert.equal(status.preview.byte_length, (await stat(filename)).size);
  evidence.checks.push({ preview: status }); return status;
}
async function rejectPreview(filename) {
  await page.evaluate(() => { window.backupAcceptanceStatuses.length = 0; });
  await picker('Choose backup', filename);
  await expect.poll(async () => (await latest('backup_preview_start'))?.operation, { timeout: 8000 }).toBeTruthy();
  const operation = (await latest('backup_preview_start')).operation;
  let status;
  await expect.poll(async () => { status = await invoke('backup_preview_status', { operation }); return status.state; }, { timeout: 10000 }).toBe('failed');
  assert.equal(status.preview, null); assert.ok(status.error);
  await expect(page.getByRole('alert').last()).toContainText(status.error);
  await expect(page.getByRole('button', { name: 'Restore the whole backup', exact: true })).toHaveCount(0);
  return status;
}
async function artifact(name) {
  const source = resolve(input);
  const owned = relative(resolve('../../target/issue-75'), source);
  assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned), 'Use only an isolated acceptance input');
  assert.deepEqual(JSON.parse(await readFile(join(source, 'cleanup.json'), 'utf8')), { hosts: [], workers: [], helpers: 0 });
  const path = join(destination, name);
  await cp(join(source, name), path); return path;
}
async function rawFiles(root, original = false) {
  const output = {};
  async function tree(directory, prefix) {
    const entries = await readdir(directory, { withFileTypes: true }).catch((error) => { if (error.code === 'ENOENT') return []; throw error; });
    for (const entry of entries) {
      const path = join(directory, entry.name), key = `${prefix}/${entry.name}`;
      if (entry.isDirectory()) await tree(path, key);
      else output[key] = (await readFile(path)).toString('base64');
    }
  }
  if (original) {
    for (const suffix of ['', '-wal', '-shm', '-journal']) {
      const name = 'lexique.sqlite3' + suffix;
      const bytes = await readFile(join(root, 'Lexique-data/data', name)).catch((error) => { if (error.code === 'ENOENT') return null; throw error; });
      if (bytes) output[`database/${name}`] = bytes.toString('base64');
    }
    await tree(join(root, 'Lexique-data/temp/speech'), 'audio');
    await tree(join(root, 'Lexique-data/data/dictionaries'), 'dictionaries');
  } else {
    for (const directory of ['database', 'audio', 'dictionaries']) await tree(join(root, directory), directory);
  }
  const journalPath = original ? join(root, 'Lexique-data/data/.lexique-restore.json') : join(root, '.lexique-restore.json');
  const journal = await readFile(journalPath).catch((error) => { if (error.code === 'ENOENT') return null; throw error; });
  if (journal) output['.lexique-restore.json'] = journal.toString('base64');
  return Object.fromEntries(Object.entries(output).sort(([left], [right]) => left.localeCompare(right)));
}
async function restore(checked, recovery = false) {
  const operation = checked.operation;
  await page.getByRole('button', { name: recovery ? '恢复整个备份' : 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: recovery ? '确认恢复整个备份' : 'Confirm whole-backup restore' });
  const confirm = dialog.getByRole('button', { name: recovery ? '确认并恢复' : 'Confirm and restore', exact: true });
  await expect(confirm).toBeDisabled();
  await dialog.getByRole('checkbox').check(); await expect(confirm).toBeEnabled();
  await page.screenshot({ path: join(destination, 'native-restore-confirmation.png') });
  const predecessor = fixture.hostPid();
  page = await fixture.followRestart(() => confirm.click());
  assert.notEqual(fixture.hostPid(), predecessor);
  page.setDefaultTimeout(8000); await page.context().setOffline(true);
  evidence.successor = { predecessor, pid: fixture.hostPid(), uiBeforeStatus: await page.locator('body').innerText() };
  let status;
  await expect.poll(async () => { status = await invoke('startup_status'); return status.state; }, { timeout: 10000 }).toBe('ready');
  assert.equal(status.operation, operation); assert.equal(status.error, null);
  assert.ok(status.restored); assert.deepEqual(status.restored.counts, checked.preview.counts);
  await expect(page.getByRole('banner')).toBeVisible();
  await expect(page.getByText('Restore completed', { exact: true })).toBeVisible();
  await page.screenshot({ path: join(destination, 'native-restored.png') });
  evidence.checks.push({ restored: status, predecessor, successor: fixture.hostPid() });
  return status;
}
async function verifyAudio(expected, oracle, playback = true) {
  const actual = readState(destination);
  assert.deepEqual(actual.selections, expected.selections, 'Current/shared/previous logical versions and original absolute deadline');
  for (const media of oracle.audio.media) {
    const version = actual.versions.find((value) => value.locator === media.locator);
    const physical = actual.physical.find((value) => value.locator === media.locator);
    assert.ok(version && physical); assert.equal(version.content_hash, media.hash);
    assert.equal(version.byte_length, media.bytes);
    const bytes = await readFile(join(destination, 'Lexique-data/temp/speech', `s-${physical.physical_locator}-${media.hash}.ogg`));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), media.hash);
  }
  if (!playback) {
    const cached = await invoke('speech_cached', { vocabularyId: oracle.ids[0] });
    assert.equal(createHash('sha256').update(Buffer.from(cached.bytes)).digest('hex'), oracle.audio.media[0].hash);
    assert.deepEqual(fixture.workerPids(), []);
    evidence.checks.push({ exactCurrentSharedPreviousCache: oracle.audio, nativeCachedBytes: cached.bytes.length, noSynthesisWorkers: true });
    return;
  }
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
  await page.getByRole('button', { name: 'Open listening card', exact: true }).click();
  const listening = page.getByRole('dialog', { name: 'Listening card', exact: true });
  const audio = listening.locator('audio');
  await expect.poll(() => audio.evaluate((element) => element.currentTime)).toBeGreaterThan(0);
  await expect.poll(() => audio.evaluate((element) => element.ended), { timeout: 15000 }).toBe(true);
  await expect(listening.getByRole('alert')).toHaveCount(0);
  const cached = await invoke('speech_cached', { vocabularyId: oracle.ids[0] });
  const sha256 = createHash('sha256').update(Buffer.from(cached.bytes)).digest('hex');
  assert.equal(sha256, oracle.audio.media[0].hash);
  const decoded = await page.evaluate(async (bytes) => {
    const context = new window.AudioContext();
    try { const buffer = await context.decodeAudioData(new Uint8Array(bytes).buffer);
      return { duration: buffer.duration, audible: buffer.getChannelData(0).some((value) => value !== 0) }; }
    finally { await context.close(); }
  }, cached.bytes);
  const played = await audio.evaluate((element) => ({ currentTime: element.currentTime, ended: element.ended, playbackRate: element.playbackRate }));
  assert.ok(decoded.duration > 0 && decoded.audible);
  assert.deepEqual(fixture.workerPids(), [], 'Cached product playback must not start synthesis');
  evidence.checks.push({ cachedProductPlayback: { ...decoded, ...played, bytes: cached.bytes.length, sha256 }, synthesisAssetsUnavailable: true });
  await page.screenshot({ path: join(destination, 'native-restored-listening.png') });
  await listening.getByRole('button', { name: 'Skip', exact: true }).click();
}
async function verifyReplays(oracle) {
  const original = oracle.saved[0];
  const replay = await invoke('vocabulary_save', { request: original.request });
  assert.equal(replay.outcome, 'replayed'); assert.equal(replay.record.id, original.result.record.id);
  await assert.rejects(invoke('vocabulary_save', { request: { ...original.request, notes: 'Conflicting operation identity' } }));
  for (const rating of oracle.ratings) {
    const replayed = await invoke('review_rate', { request: rating.request });
    if (rating.request.operation === oracle.undo.ratingOperation) {
      assert.equal(replayed.status, 'rejected'); assert.equal(replayed.code, 'conflict');
    } else assert.deepEqual(replayed, rating.result);
  }
  assert.equal(await invoke('review_undo_target'), oracle.ratings.at(-1).request.operation);
  assert.equal((await invoke('review_undo', oracle.undo)).status, 'committed');
  assert.equal(await invoke('review_undo_target'), oracle.ratings.at(-1).request.operation);
  const conflicting = await invoke('review_rate', { request: { ...oracle.ratings[0].request, rating: 'easy' } });
  assert.equal(conflicting.status, 'rejected');
  const batch = await invoke('review_add_batch', { request: oracle.batch });
  assert.deepEqual(batch.items.map((value) => value.vocabulary_id), [oracle.ids[0], oracle.ids[1], 9999]);
  await assert.rejects(invoke('review_add_batch', { request: { ...oracle.batch, vocabulary_ids: [oracle.ids[2]] } }));
  const target = await invoke('review_undo_target');
  const undone = await invoke('review_undo', { operation: `${oracle.timeline}-native-nearest-undo`, ratingOperation: target });
  assert.equal(undone.status, 'committed');
  evidence.checks.push({ replayedSaves: 1, replayedRatings: oracle.ratings.length - 1, undoneRatingRejected: 1, replayedUndo: 1, replayedBatch: 1,
    conflictsRejected: 3, nearestUndo: { target, result: undone }, quotaAfterUndo: await invoke('review_queue', { timezone: 'UTC', skipped: [] }) });
}
async function importMdx(name) {
  const previous = await invoke('dictionary_settings');
  await page.locator('#dictionary-import').setInputFiles(resolve('e2e/fixtures/mdx', name));
  let catalog;
  await expect.poll(async () => { catalog = await invoke('dictionary_settings'); return catalog.length; }).toBe(previous.length + 1);
  await expect(page.locator('#dictionary-import')).toBeEnabled();
  await expect(page.locator('.dictionary-settings').getByRole('alert')).toHaveCount(0);
  return catalog.at(-1);
}
async function relinkMdx(id, name, confirmation, cancel = false) {
  const file = resolve('e2e/fixtures/mdx', name);
  const hash = createHash('sha256').update(await readFile(file)).digest('hex');
  await page.locator(`#dictionary-relink-${id}`).setInputFiles(file);
  if (confirmation) {
    const dialog = page.getByRole('dialog', { name: 'Confirm dictionary content', exact: true });
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText(confirmation === 'unknown' ? 'The original dictionary content is unknown' : 'The selected MDX has different content');
    await page.screenshot({ path: join(destination, `native-mdx-${confirmation}-${cancel ? 'cancel' : 'confirm'}.png`) });
    await dialog.getByRole('button', { name: cancel ? 'Cancel' : 'Use this MDX', exact: true }).click();
    await expect(dialog).not.toBeVisible();
  }
  if (!cancel) {
    await expect.poll(async () => (await invoke('dictionary_settings')).find((item) => item.id === id)?.content_hash).toBe(hash);
    await expect.poll(async () => (await invoke('dictionary_settings')).find((item) => item.id === id)?.availability).toBe('available');
  }
  await expect(page.locator(`#dictionary-relink-${id}`)).toBeEnabled();
  evidence.checks.push({ mdxRelink: { id, file: name, confirmation: confirmation || 'exact', cancel, hash } });
}

try {
  if (scenario === 'playback-B') {
    await artifact('restored-A.json');
    const original = readState(resolve(input));
    await cp(join(resolve(input), 'Lexique-data'), join(destination, 'Lexique-data'), { recursive: true });
    const cloned = readState(destination);
    compareLearning(cloned, original);
    assert.deepEqual(cloned.selections, original.selections);
    assert.deepEqual(cloned.versions, original.versions);
    await writeFile(join(destination, 'populated-A-before-B.json'), JSON.stringify(cloned, null, 2));
    evidence.checks.push({ populatedATargetBeforeRetainedBRestore: true, originalAudioLocators: cloned.versions.map((version) => version.locator) });
  }
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await settings(); await observe();
  if (scenario === 'windows') {
    evidence.checks.push(await helper('picker-windows'));
  } else if (scenario === 'picker') {
    await assert.rejects(helper('picker-controls', { pid: 0 }), /owned PID/);
    await assert.rejects(helper('picker-controls', { executable: process.execPath }), /executable mismatch/);
    await assert.rejects(helper('picker-controls', { waitMs: 100 }), /not found before its deadline/);
    evidence.checks.push({ ownershipRefusals: 3 });
    await picker('Create backup');
    await expect(page.getByText('Backup cancelled.', { exact: true })).toBeVisible();
    const filename = join(destination, 'empty.lexique-backup');
    const exported = await exportBackup(filename); assert.equal(exported.result.counts.vocabulary, 0);
    await picker('Choose backup');
    await expect(page.getByText('Backup check cancelled.', { exact: true })).toBeVisible();
    const checked = await preview(filename); assert.equal(checked.preview.counts.vocabulary, 0);
    await expect(page.getByText('Backup verified. Current learning data is unchanged.', { exact: true })).toBeVisible();
    await page.screenshot({ path: join(destination, 'native-picker.png') });
    await fixture.stop(true); evidence.checks.push({ normalClose: true });
  } else if (scenario === 'close-guard') {
    const request = { operation: 'native-close-save', headword: 'école', notes: 'Saved before native close',
      example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
    await invoke('vocabulary_save', { request });
    await page.getByRole('button', { name: 'Notebook', exact: true }).click();
    await page.locator('.vocabulary-list button').filter({ hasText: 'école' }).click();
    const notes = page.getByLabel('Personal notes', { exact: true });
    await notes.fill('Pending native close edit');
    const before = readState(destination);
    await helper('close', { title: 'Lexique' });
    const dialog = page.getByRole('dialog', { name: 'You have unsaved changes', exact: true });
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: 'Keep editing', exact: true }).click();
    await expect(notes).toHaveValue('Pending native close edit');
    compareLearning(readState(destination), before);
    await helper('close', { title: 'Lexique' }); await expect(dialog).toBeVisible();
    await fixture.stop(true, () => dialog.getByRole('button', { name: 'Save and leave', exact: true }).click());
    assert.equal(readState(destination).records[0].notes, 'Pending native close edit');
    evidence.checks.push({ nativeClosePreservesUnsavedGuard: true, keepEditingRetainedDraft: true, saveAndClosePersisted: true });
  } else if (scenario.startsWith('reject-packages-')) {
    await seed('B');
    const before = readState(destination);
    const filename = await artifact('A.lexique-backup');
    const groups = [
      ['unsupported-format', 'unsupported-snapshot', 'count', 'unknown-field'],
      ['invalid-number', 'invalid-relation', 'duplicate-json', 'duplicate-entry'],
      ['path-entry', 'truncated', 'missing-audio', 'crc'],
      ['hash', 'compression'],
    ];
    const variants = groups[Number(scenario.slice(-1)) - 1]; assert.ok(variants);
    const pending = [{ key: 'lexique-pending-rating-native75', value: '{raw rating request' },
      { key: 'lexique-pending-undo', value: ' raw undo string ' }, { key: 'lexique-pending-review-batch', value: '[raw batch' }];
    await page.evaluate((entries) => { for (const { key, value } of entries) localStorage.setItem(key, value); }, pending);
    for (const variant of variants) {
      const invalid = corruptPackage(destination, filename, variant);
      const failed = await rejectPreview(invalid.path);
      compareLearning(readState(destination), before);
      assert.deepEqual(await page.evaluate((entries) => entries.map(({ key }) => localStorage.getItem(key)), pending), pending.map((entry) => entry.value));
      evidence.checks.push({ invalidPackage: invalid, failure: failed, populatedBUnchanged: true, rawPendingKeysUnchanged: true });
    }
    const request = { operation: `native-${scenario}-write`, headword: 'œil', notes: 'Writing resumed after rejected backup',
      example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
    assert.equal((await invoke('vocabulary_save', { request })).outcome, 'created');
    assert.ok(readState(destination).records.some((record) => record.headword === 'œil' && record.notes === request.notes));
    await page.screenshot({ path: join(destination, 'native-package-rejected.png') });
    await fixture.stop(true); evidence.checks.push({ writingResumedAfterRejection: true, normalClose: true });
  } else if (scenario === 'export-failure-cancel') {
    await seed('B'); const before = readState(destination);
    const previous = fixtureSql(destination, `
value=c.execute("SELECT value FROM app_metadata WHERE key='speech-preferences'").fetchone()[0]
c.execute("UPDATE app_metadata SET value='invalid source preference JSON' WHERE key='speech-preferences'"); c.commit()
print(json.dumps(value))`);
    const filename = join(destination, 'failed.lexique-backup');
    await page.evaluate(() => { window.backupAcceptanceStatuses.length = 0; });
    await picker('Create backup', filename);
    await expect.poll(async () => (await latest('backup_export_start'))?.operation).toBeTruthy();
    const operation = (await latest('backup_export_start')).operation;
    let failed;
    await expect.poll(async () => { failed = await invoke('backup_operation_status', { operation }); return failed.state; }).toBe('failed');
    assert.equal(failed.result, null); assert.ok(failed.error);
    await expect(page.getByRole('alert').last()).toContainText(failed.error);
    await assert.rejects(stat(filename), /ENOENT/);
    await page.screenshot({ path: join(destination, 'native-export-failure.png') });
    fixtureSql(destination, `c.execute("UPDATE app_metadata SET value=? WHERE key='speech-preferences'",[p['value']]); c.commit(); print('{}')`, { value: previous });
    compareLearning(readState(destination), before);
    await page.evaluate(() => { window.backupAcceptanceStatuses.length = 0; });
    await picker('Create backup');
    await expect.poll(async () => (await latest('backup_export_start'))?.operation).toBeTruthy();
    const cancelledId = (await latest('backup_export_start')).operation;
    const cancelled = await invoke('backup_operation_status', { operation: cancelledId });
    assert.equal(cancelled.state, 'cancelled'); assert.equal(cancelled.result, null);
    compareLearning(readState(destination), before);
    const request = { operation: 'native-after-export-failure', headword: 'œil', notes: 'Native writing resumed after export failure',
      example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
    assert.equal((await invoke('vocabulary_save', { request })).outcome, 'created');
    const retry = await exportBackup(join(destination, 'after-failure.lexique-backup')); assert.equal(retry.result.counts.vocabulary, 4);
    await fixture.stop(true);
    evidence.checks.push({ visibleNativeExportFailure: failed, noSelectablePartialPackage: true, cancelled,
      populatedBRetained: true, writingAndExportResumed: true });
  } else if (scenario === 'source-missing' || scenario === 'source-invalid') {
    const prepared = await seed('A');
    const before = readState(destination);
    await fixture.stop(true);
    const media = prepared.audio.media[0];
    const physical = before.physical.find((row) => row.locator === media.locator);
    const filename = join(destination, 'Lexique-data/temp/speech', `s-${physical.physical_locator}-${media.hash}.ogg`);
    if (scenario === 'source-missing') await unlink(filename);
    else await writeFile(filename, 'synthetic invalid selected audio');
    page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
    await settings(); await observe();
    const output = join(destination, `${scenario}.lexique-backup`);
    const exported = await exportBackup(output);
    const counts = exported.result.counts;
    assert.equal(counts.source_missing_audio, scenario === 'source-missing' ? 1 : 0);
    assert.equal(counts.source_invalid_audio, scenario === 'source-invalid' ? 1 : 0);
    const independent = inspectPackage(destination, output);
    const current = independent.snapshot.audio_versions.find((version) => version.locator === media.locator);
    assert.equal(current.payload_status, scenario.replace('-', '_'));
    assert.ok(!independent.entries.includes(`audio/${media.hash}.ogg`));
    compareLearning(readState(destination), before);
    await preview(output); compareLearning(readState(destination), before);
    await page.screenshot({ path: join(destination, 'native-source-missing-or-invalid.png') });
    await fixture.stop(true);
    evidence.checks.push({ originalUnavailableAudioExported: current, corruptBytesExcluded: true, learningUnchanged: true });
  } else if (scenario.startsWith('recovery-')) {
    const kind = scenario.slice('recovery-'.length);
    assert.ok(['corrupt', 'unsupported', 'migration', 'close', 'journal'].includes(kind));
    await seed('B'); await importMdx('other.mdx');
    const pending = [{ key: 'lexique-pending-rating-raw75', value: '{ unparsed raw rating' },
      { key: 'lexique-pending-undo', value: ' raw undo with whitespace ' }, { key: 'lexique-pending-review-batch', value: '[unparsed batch' }];
    await page.evaluate((entries) => { for (const { key, value } of entries) localStorage.setItem(key, value); localStorage.setItem('native75-raw-control', 'preserved'); }, pending);
    await fixture.stop(true);
    const seeded = seedStartupFailure(destination, ['close', 'journal'].includes(kind) ? 'corrupt' : kind);
    if (kind === 'journal') await writeFile(join(destination, 'Lexique-data/data/.lexique-restore.json'), '{"stage_root":"../untrusted"');
    const original = await rawFiles(destination, true);
    await writeFile(join(destination, 'raw-before.json'), JSON.stringify(original, null, 2));
    page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
    await page.getByRole('combobox').first().selectOption('en'); await observe();
    const failed = await invoke('startup_status');
    assert.equal(failed.state, 'recovery_required'); assert.ok(failed.error);
    const error = { corrupt: /not a database|malformed/i, unsupported: /schema.*999|999.*schema/i, migration: /duplicate column|migration/i, close: /not a database|malformed/i, journal: /restore record/i }[kind];
    assert.match(failed.error, error);
    assert.deepEqual(fixture.workerPids(), []);
    await assert.rejects(invoke('runtime_status'));
    assert.deepEqual(await rawFiles(destination, true), original);
    evidence.checks.push({ startupFailure: failed, sourceFixture: seeded, learningWorkersAbsent: true });
    if (kind === 'close') {
      await fixture.stop(true, () => page.getByRole('button', { name: 'Close app', exact: true }).click());
      assert.deepEqual(await rawFiles(destination, true), original);
      evidence.checks.push({ visibleRecoveryClose: true, exactOriginalFilesRetained: true });
    } else {
      const expected = JSON.parse(await readFile(await artifact('state-A.json'), 'utf8'));
      const filename = await artifact('A.lexique-backup');
      const checked = await preview(filename); const restored = await restore(checked);
      compareLearning(readState(destination), expected);
      const retained = restored.retained_copies.find((copy) => copy.kind === 'raw_files' && copy.id === checked.operation + '-raw');
      assert.ok(retained);
      const rawRoot = join(destination, 'Lexique-data/data/retained-restores', checked.operation, 'raw-files');
      assert.equal(retained.path.startsWith('\\\\?\\') ? retained.path.slice(4) : retained.path, rawRoot);
      assert.deepEqual(await rawFiles(rawRoot), original);
      const requests = JSON.parse(await readFile(join(rawRoot, 'pending-requests.json'), 'utf8'));
      assert.deepEqual(requests.sort((a, b) => a.key.localeCompare(b.key)), [...pending].sort((a, b) => a.key.localeCompare(b.key)));
      assert.deepEqual(await page.evaluate((entries) => entries.map(({ key }) => localStorage.getItem(key)), pending), [null, null, null]);
      assert.equal(await page.evaluate(() => localStorage.getItem('native75-raw-control')), 'preserved');
      await page.screenshot({ path: join(destination, 'native-raw-recovery-restored.png') });
      await fixture.stop(true);
      evidence.checks.push({ exactRawDatabaseSidecarsAudioMdx: Object.keys(original), rawPendingStringsRetained: pending,
        oldPendingKeysClearedBeforeReady: true, independentRestoredAReadback: true, retained });
    }
  } else if (scenario === 'post-rating-restart') {
    const oracle = JSON.parse(await readFile(await artifact('oracle-A.json'), 'utf8'));
    const filename = await artifact('A.lexique-backup');
    const checked = await preview(filename);
    await restore(checked);
    await verifyReplays(oracle);
    const card = (await invoke('review_cards', { vocabularyId: oracle.ids[1] })).find((entry) => entry.kind === 'recognition');
    const progress = await invoke('review_progress', { cardId: card.id });
    const request = { operation: 'A-native-after-undo-rating', card_id: card.id, expected_revision: progress.revision, rating: 'good', timezone: 'UTC' };
    const rated = await invoke('review_rate', { request }); assert.equal(rated.status, 'committed');
    const expected = readState(destination);
    const quota = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
    const afterQueue = readState(destination);
    await fixture.stop(true);
    page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
    await expect(page.getByRole('banner')).toBeVisible();
    compareLearning(readState(destination), afterQueue);
    assert.deepEqual(await invoke('review_rate', { request }), rated);
    const nearest = await invoke('review_undo_target'); assert.equal(nearest, request.operation);
    await writeFile(join(destination, 'new-rating-restarted.json'), JSON.stringify(readState(destination), null, 2));
    await fixture.stop(true);
    evidence.checks.push({ originalRatingOrder: expected.ratings.map((rating) => ({ operation: rating.operation, reviewed_at: rating.reviewed_at })),
      newRatingThenOrdinaryRestart: { request, rated }, nearestUndoAfterRestart: nearest, originalQuotaAfterRating: quota,
      independentStateAfterRestartUnchanged: true });
  } else if (scenario === 'startup-probe') {
    const filename = await artifact('A.lexique-backup');
    const checked = await preview(filename);
    await restore(checked);
    await fixture.stop(true);
    evidence.checks.push({ successorStartupReady: true, normalClose: true });
  } else if (scenario === 'export-A') {
    const prepared = await seed('A');
    const filename = join(destination, 'A.lexique-backup');
    const exported = await exportBackup(filename);
    assert.equal(exported.result.counts.vocabulary, 3);
    await page.screenshot({ path: join(destination, 'native-export-A.png') });
    await fixture.stop(true);
    const state = readState(destination);
    assert.equal(state.records.length, 3); assert.equal(state.cards.length, 6);
    assert.equal(state.progress.length, 2); assert.equal(state.absent_progress.length, 4);
    assert.deepEqual(state.records.map((record) => record.notes), ['A notes 0: café, 中文', 'A notes 1: café, 中文', 'A notes 2: café, 中文']);
    assert.deepEqual(state.ratings.map((rating) => rating.operation), ['A-z-rating', 'A-a-rating', 'A-b-rating']);
    await writeFile(join(destination, 'state-A.json'), JSON.stringify(state, null, 2));
    await writeFile(join(destination, 'oracle-A.json'), JSON.stringify(prepared, null, 2));
    evidence.checks.push({ independentlyReadCollections: Object.keys(state), cardsWithoutProgress: state.absent_progress,
      packageSha256: createHash('sha256').update(await readFile(filename)).digest('hex'), noSynthesisWorkers: fixture.workerPids() });
  } else if (scenario === 'restore-A') {
    const prepared = await seed('B');
    const expected = JSON.parse(await readFile(await artifact('state-A.json'), 'utf8'));
    const oracle = JSON.parse(await readFile(await artifact('oracle-A.json'), 'utf8'));
    const filename = await artifact('A.lexique-backup');
    const before = readState(destination);
    await writeFile(join(destination, 'state-B.json'), JSON.stringify(before, null, 2));
    await writeFile(join(destination, 'oracle-B.json'), JSON.stringify(prepared, null, 2));
    const pending = [
      { key: `lexique-pending-rating-${prepared.ratings.at(-1).request.card_id}`, value: JSON.stringify(prepared.ratings.at(-1).request) },
      { key: 'lexique-pending-undo', value: JSON.stringify(prepared.undo) },
      { key: 'lexique-pending-review-batch', value: JSON.stringify(prepared.batch) },
    ];
    await page.evaluate((pending) => {
      for (const { key, value } of pending) localStorage.setItem(key, value);
      localStorage.setItem('issue-75-control', 'retained-unrelated-setting');
    }, pending);
    const checked = await preview(filename);
    const restored = await restore(checked);
    assert.deepEqual(await page.evaluate((pending) => pending.map(({ key }) => localStorage.getItem(key)), pending), [null, null, null]);
    assert.equal(await page.evaluate(() => localStorage.getItem('issue-75-control')), 'retained-unrelated-setting');
    const retained = restored.retained_copies.find((copy) => copy.kind === 'logical_backup' && copy.id === checked.operation + '-backup');
    assert.ok(retained, 'Own actual B preservation package must be returned by the successor');
    compareLearning(readState(destination), expected);
    await writeFile(join(destination, 'restored-A.json'), JSON.stringify(readState(destination), null, 2));
    await verifyAudio(expected, oracle, false); await verifyReplays(oracle);
    await fixture.stop(true);
    const retainedBytes = await readFile(retained.path);
    await writeFile(join(destination, 'B-retained.lexique-backup'), retainedBytes);
    evidence.checks.push({ pendingKnownKeysCleared: pending.map((value) => value.key), unrelatedKeyRetained: true,
      retainedB: { ...retained, sha256: createHash('sha256').update(retainedBytes).digest('hex') }, independentAReadback: true });
  } else if (scenario === 'restore-B') {
    const expected = JSON.parse(await readFile(await artifact('state-B.json'), 'utf8'));
    const oracle = JSON.parse(await readFile(await artifact('oracle-B.json'), 'utf8'));
    const filename = await artifact('B-retained.lexique-backup');
    const checked = await preview(filename); await restore(checked);
    compareLearning(readState(destination), expected);
    await writeFile(join(destination, 'restored-B.json'), JSON.stringify(readState(destination), null, 2));
    await verifyAudio(expected, oracle, false); await verifyReplays(oracle);
    await fixture.stop(true); evidence.checks.push({ independentBReadback: true, restoredActualRetainedPackage: true });
  } else if (scenario === 'playback-A' || scenario === 'playback-B') {
    const a = scenario === 'playback-A';
    const expected = JSON.parse(await readFile(await artifact(a ? 'state-A.json' : 'state-B.json'), 'utf8'));
    const oracle = JSON.parse(await readFile(await artifact(a ? 'oracle-A.json' : 'oracle-B.json'), 'utf8'));
    const filename = await artifact(a ? 'A.lexique-backup' : 'B-retained.lexique-backup');
    const checked = await preview(filename); await restore(checked);
    compareLearning(readState(destination), expected);
    if (!a) {
      const restored = readState(destination);
      assert.deepEqual(restored.versions.map((version) => version.locator).sort(), expected.versions.map((version) => version.locator).sort());
      assert.ok(restored.versions.every((version) => !['a'.repeat(64), 'b'.repeat(64)].includes(version.locator)));
      evidence.checks.push({ populatedAReplacedByActualRetainedB: true, independentBPreferencesQueueAndAudio: true });
    }
    await fixture.stop(true);
    page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
    await expect(page.getByRole('banner')).toBeVisible();
    assert.equal((await invoke('startup_status')).state, 'ready');
    await verifyAudio(expected, oracle);
    await fixture.stop(true); evidence.checks.push({ restoredCachePlaybackAfterOrdinaryReopen: true });
  } else if (scenario === 'export-mdx') {
    await preferences(true); await invoke('review_auto_enroll', { enabled: false });
    await invoke('selection_shortcut', { shortcut: 'Ctrl+Alt+Shift+U' });
    await invoke('ocr_shortcut', { shortcut: 'Ctrl+Alt+Shift+L' });
    const imported = [];
    for (const name of ['verbs.mdx', 'other.mdx', 'verbs-updated.mdx']) imported.push(await importMdx(name));
    const saved = [];
    for (const [index, headword] of ['être', 'double', 'suivre'].entries()) {
      const request = { operation: `MDX-save-${index}`, headword, notes: `Frozen MDX note ${index}`, example_fr: 'Source originale.',
        example_zh: '原始出处。', example_sense: 0, more_examples: [], id: null, revision: null };
      const result = await invoke('vocabulary_save', { request, dictionary_id: imported[index].id });
      assert.equal(result.record.reference.dictionary_id, imported[index].id);
      saved.push({ request, result });
    }
    const catalog = await invoke('dictionary_settings');
    const order = [catalog[0].id, catalog[1].id, imported[2].id, imported[0].id, imported[1].id];
    await invoke('dictionary_settings', { order });
    await fixture.stop(true); seedUnknownDictionary(destination, imported[2].id);
    page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true); await settings(); await observe();
    const filename = join(destination, 'MDX.lexique-backup');
    const result = await exportBackup(filename); assert.equal(result.result.counts.custom_dictionaries, 3);
    await fixture.stop(true);
    await writeFile(join(destination, 'state-mdx.json'), JSON.stringify(readState(destination), null, 2));
    await writeFile(join(destination, 'oracle-mdx.json'), JSON.stringify({ imported, saved, order }, null, 2));
    evidence.checks.push({ portableCatalogWithMissingUnknownContent: true, originalCustomSources: saved.map((value) => value.result.record.reference) });
  } else if (scenario === 'migrate-mdx') {
    await preferences(false); await invoke('review_auto_enroll', { enabled: false });
    await invoke('selection_shortcut', { shortcut: 'Ctrl+Alt+Shift+J' });
    await invoke('ocr_shortcut', { shortcut: 'Ctrl+Alt+Shift+P' });
    const existing = await importMdx('verbs.mdx'); assert.equal(existing.id, 'mdx-1');
    const originalFile = join(destination, 'Lexique-data/data/dictionaries/mdx-1.mdx');
    const originalBytes = await readFile(originalFile);
    const devices = readState(destination).devices.filter((row) => row.key !== 'dictionary-files');
    const expected = JSON.parse(await readFile(await artifact('state-mdx.json'), 'utf8'));
    const oracle = JSON.parse(await readFile(await artifact('oracle-mdx.json'), 'utf8'));
    const filename = await artifact('MDX.lexique-backup');
    const checked = await preview(filename);
    assert.deepEqual(checked.preview.pending_dictionaries.map((entry) => entry.id), oracle.order.filter((id) => id.startsWith('mdx-') && id !== existing.id));
    await restore(checked);
    compareLearning(readState(destination), expected);
    assert.deepEqual(readState(destination).devices.filter((row) => row.key !== 'dictionary-files'), devices);
    assert.equal((await invoke('selection_shortcut')).shortcut, 'Ctrl+Alt+Shift+J');
    assert.equal((await invoke('ocr_shortcut')).shortcut, 'Ctrl+Alt+Shift+P');
    await settings();
    const catalog = await invoke('dictionary_settings');
    assert.deepEqual(catalog.map((entry) => entry.id), oracle.order);
    assert.equal(catalog.find((entry) => entry.id === 'mdx-1').availability, 'available');
    for (const id of ['mdx-2', 'mdx-3']) assert.equal(catalog.find((entry) => entry.id === id).availability, 'missing');
    await relinkMdx('mdx-2', 'other.mdx');
    await relinkMdx('mdx-1', 'verbs-updated.mdx', 'different', true);
    assert.equal((await invoke('dictionary_settings')).find((entry) => entry.id === 'mdx-1').content_hash, existing.content_hash);
    await relinkMdx('mdx-1', 'verbs-updated.mdx', 'different');
    await relinkMdx('mdx-3', 'verbs-updated.mdx', 'unknown', true);
    assert.equal((await invoke('dictionary_settings')).find((entry) => entry.id === 'mdx-3').availability, 'missing');
    await relinkMdx('mdx-3', 'verbs-updated.mdx', 'unknown');
    const added = await importMdx('other.mdx');
    assert.ok(Number(added.id.slice(4)) > 3, 'New import reserves restored and retained identities');
    assert.deepEqual((await invoke('dictionary_settings')).slice(0, -1).map((entry) => entry.id), oracle.order);
    const after = readState(destination);
    assert.deepEqual(after.records, expected.records); assert.deepEqual(after.saves, expected.saves);
    assert.deepEqual(await readFile(originalFile), originalBytes, 'Original target MDX bytes remain retained');
    await page.screenshot({ path: join(destination, 'native-mdx-migrated.png') });
    await fixture.stop(true);
    await writeFile(join(destination, 'migrated-mdx.json'), JSON.stringify(after, null, 2));
    evidence.checks.push({ independentDirectoryMigration: true, targetShortcutsRetained: devices, exactTargetContentAutomaticallyAttached: true,
      missingContentReattached: true, frozenSourcesAndOperationsUnchanged: true, previousTargetFileUnchanged: true, collisionSafeNewId: added.id });
  } else throw new Error(`Unknown native acceptance scenario: ${scenario}, ${input || ''}`);
  evidence.passed = true;
} catch (error) {
  evidence.error = String(error);
  if (page && !page.isClosed()) {
    evidence.ui = await page.locator('body').innerText().catch(() => '');
    await page.screenshot({ path: join(destination, 'failure.png') }).catch(() => {});
  }
  throw error;
}
finally {
  if (page && !page.isClosed()) evidence.events = await page.evaluate(() => window.backupAcceptanceStatuses ?? []).catch(() => []);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
  await fixture.finish();
}
console.log(JSON.stringify({ scenario, passed: evidence.passed, checks: evidence.checks.length, pickers: evidence.pickers.length, destination }));
