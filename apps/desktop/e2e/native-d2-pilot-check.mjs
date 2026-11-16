/* global process, window, URL, setTimeout, clearTimeout, Buffer */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { chmod, cp, readFile, rename, stat, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { compareLearning, fixtureSql, inspectPackage, readState } from './native-backup-state.mjs';

const [scenario, resources, binary, conjugation, frequency, curationPath, destinationPath, input] = process.argv.slice(2);
assert.ok([scenario, resources, binary, conjugation, frequency, curationPath, destinationPath].every(Boolean),
  'Expected scenario, package, binary, conjugation, frequency, reviewed curation and new destination; backup/profile scenarios also take an input');
assert.ok(['pilot', 'review', 'no-speech', 'catalogue', 'missing', 'corrupt', 'future', 'newer', 'limit', 'export', 'restore', 'legacy', 'reject'].includes(scenario));
const curation = JSON.parse(await readFile(resolve(curationPath), 'utf8'));
const membership = [[1, 'être', 'AUX'], [2, 'être', 'NOM'], [3, 'être', 'VER'], [4, 'avoir', 'AUX'], [5, 'avoir', 'VER'],
  [70, 'bien', 'ADV'], [71, 'bien', 'NOM'], [360, "aujourd'hui", 'ADV'], [422, 'fort', 'ADJ'], [423, 'fort', 'ADV'],
  [483, 'coeur', 'NOM'], [1911, 'côte', 'NOM']];
assert.deepEqual(curation.groups.map((group) => [group.frozen_row, group.headword, group.lexique_cgram]), membership);
assert.equal(curation.scope_denominator_groups, 4087);
const fixture = await native24Fixture(resolve(resources), resolve(binary), resolve(destinationPath));
const { destination } = fixture;
await cp(resolve(conjugation), join(destination, 'conjugation'), { recursive: true });
await cp(resolve(frequency), join(destination, 'frequency'), { recursive: true });
const coreFile = join(destination, 'dictionary/core-pilot-79/core.sqlite3');
const evidence = { scenario, checks: [], calls: [], externalRequests: [], pickers: [], networkBoundary: 'External WebView routes denied after native startup; whole-machine network denial unrun' };
evidence.coreResourceBytes = (await stat(coreFile)).size;
const model = join(destination, 'speech/assets/text_encoder.int8.onnx');
let page;

async function start() {
  page = await fixture.start();
  page.setDefaultTimeout(8000);
  await page.context().route('**/*', (route) => {
    const host = new URL(route.request().url()).hostname;
    if (['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(host)) return route.continue();
    evidence.externalRequests.push(host); return route.abort();
  });
  await expect(page.getByRole('banner')).toBeVisible();
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
}

async function invoke(command, args = {}) {
  const call = { command, args };
  evidence.calls.push(call);
  const started = performance.now();
  try {
    const value = await page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
    if (command === 'dictionary_lookup') call.response_bytes = Buffer.byteLength(JSON.stringify(value), 'utf8');
    return value;
  }
  catch (error) { call.error = String(error); throw error; }
  finally { call.round_trip_ms = performance.now() - started; }
}

async function rejected(command, args, code) {
  const error = await page.evaluate(async ({ command, args }) => {
    try { await window.__TAURI_INTERNALS__.invoke(command, args); return null; }
    catch (error) { return typeof error === 'string' ? { message: error } : error; }
  }, { command, args });
  assert.ok(error, `${command} must fail visibly`);
  if (code) assert.equal(error.code, code);
  evidence.checks.push({ rejected: command, error });
  return error;
}

function senses(lookup) { return lookup.entries.flatMap((entry) => entry.core_senses ?? []); }

function admitted(lookup, key) {
  assert.equal(lookup.dictionary_id, 'builtin-core');
  const groups = curation.groups.filter((group) => group.headword === key);
  const actual = senses(lookup);
  assert.equal(actual.length, groups.reduce((count, group) => count + group.senses.length, 0));
  for (const group of groups) for (const reviewed of group.senses) {
    const sense = actual.find((item) => item.sense_id === reviewed.sense_id);
    assert.ok(sense, reviewed.sense_id);
    assert.equal(sense.frozen_row, group.frozen_row); assert.equal(sense.headword, group.headword);
    assert.equal(sense.source_headword, reviewed.source_headword); assert.equal(sense.pos, group.lexique_cgram);
    assert.equal(sense.usage, reviewed.use);
    for (const [field, inputField] of Object.entries({ definition_fr: 'french_definition', ipa: 'ipa', meaning_en: 'english', meaning_zh: 'chinese' })) {
      const expected = reviewed.fields[inputField], source = curation.sources.find((item) => item.id === expected.source_id);
      assert.equal(sense[field], expected.text, `${sense.sense_id}/${field}`);
      const provenance = sense.field_sources[field];
      assert.equal(provenance.title, source.title); assert.ok(provenance.version.includes(String(source.revid)));
      assert.equal(provenance.attribution, source.attribution); assert.equal(provenance.source_url, source.fixed_url);
      assert.equal(provenance.license, expected.content_license); assert.equal(provenance.license_url, source.license_url);
      assert.ok(provenance.span.includes(expected.source_span.raw_source_excerpt));
      assert.equal(provenance.modification, expected.modification);
    }
  }
  return actual;
}

function request(headword, suffix = headword.replaceAll(/[^a-z]/g, 'x')) {
  return { operation: `native-79-${suffix}`, headword, notes: `Note conservée : ${headword}.`, example_fr: 'Cet être est vivant.',
    example_zh: '这个生物是活的。', example_source: null, example_sense: headword === 'être' ? 1 : 0,
    more_examples: [{ fr: 'Elle est calme.', zh: '她很平静。', source: null }], id: null, revision: null };
}

async function save(headword, suffix) {
  const lookup = await invoke('dictionary_lookup', { key: headword, dictionary_id: 'builtin-core' });
  const expected = admitted(lookup, headword);
  const submitted = request(headword, suffix);
  const result = await invoke('vocabulary_save', { request: submitted, dictionary_id: 'builtin-core' });
  assert.equal(result.outcome, 'created'); assert.equal(result.record.reference.dictionary_id, 'builtin-core');
  assert.deepEqual(result.record.digest.map((item) => item.core), expected);
  for (const item of result.record.digest) {
    assert.equal(item.translation, null); assert.deepEqual(item.pronunciations, [item.core.ipa]); assert.deepEqual(item.grammar, [item.core.pos]);
  }
  return { record: result.record, submitted };
}

async function fields(scope, values) {
  for (const sense of values) {
    const block = scope.locator(`.core-sense[data-sense-id="${sense.sense_id}"]`);
    await expect(block).toHaveAttribute('data-pos', sense.pos);
    for (const [field, language] of Object.entries({ definition_fr: 'fr', ipa: 'und', meaning_en: 'en', meaning_zh: 'zh', usage: 'en' })) {
      const text = block.getByText(sense[field], { exact: true }).first();
      await expect(text).toBeVisible(); await expect(text).toHaveAttribute('lang', language);
    }
  }
}

async function query(key) {
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByLabel('French word or phrase', { exact: true }).fill(key);
  await page.getByRole('button', { name: 'Look up', exact: true }).click();
  await expect(page.getByRole('tab', { selected: true })).toContainText('Lexique');
}

async function openRecord(record) {
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').filter({ has: page.getByText(record.headword, { exact: true }) }).click();
  await fields(page.locator('.vocabulary-summary'), record.digest.map((item) => item.core));
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(record.notes);
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue(String(record.example_sense));
}

async function rate(record) {
  const cards = await invoke('review_add', { vocabularyId: record.id });
  const recognition = cards.find((card) => card.kind === 'recognition');
  const result = await invoke('review_rate', { request: { operation: 'native-79-rating', card_id: recognition.id,
    expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  assert.equal(result.status, 'committed');
}

async function pilot() {
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), ['builtin-core', 'builtin', 'builtin-fra-eng']);
  await invoke('dictionary_settings', { order: ['builtin', 'builtin-fra-eng', 'builtin-core'] });
  const before = readState(destination);
  for (const field of ['digest', 'reference']) await rejected('vocabulary_save', {
    request: { ...request('bien', `forged-${field}`), [field]: [] }, dictionary_id: 'builtin-core',
  });
  compareLearning(readState(destination), before);
  const saved = [];
  for (const [index, key] of [...new Set(curation.groups.map((group) => group.headword))].entries()) saved.push(await save(key, `word-${index}`));
  const first = saved[0];
  for (const [queryKey, frozen] of [['e\u0302tre', 'être'], ['cœur', 'coeur'], ['coeur', 'coeur'], ['aujourd’hui', "aujourd'hui"]]) {
    admitted(await invoke('dictionary_lookup', { key: queryKey, dictionary_id: 'builtin-core' }), frozen);
  }
  assert.deepEqual((await invoke('dictionary_lookup', { key: 'mot-absent-du-pilote', dictionary_id: 'builtin-core' })).entries, []);
  const replay = await invoke('vocabulary_save', { request: { ...first.submitted, headword: first.submitted.headword.normalize('NFD') }, dictionary_id: 'builtin-core' });
  assert.equal(replay.outcome, 'replayed'); assert.deepEqual(replay.record, first.record);
  const duplicate = await invoke('vocabulary_save', { request: { ...first.submitted, operation: 'native-79-duplicate', notes: 'Must preserve the established note' }, dictionary_id: 'builtin-core' });
  assert.equal(duplicate.outcome, 'existing'); assert.deepEqual(duplicate.record, first.record);
  await rate(first.record);
  const edited = await invoke('vocabulary_save', { request: { ...first.submitted, operation: 'native-79-note-edit', id: first.record.id,
    revision: first.record.revision, notes: 'Notes après modification.' }, dictionary_id: 'builtin-core' });
  assert.equal(edited.outcome, 'updated'); assert.deepEqual(edited.record.digest, first.record.digest);
  await invoke('dictionary_settings', { order: ['builtin-core', 'builtin', 'builtin-fra-eng'] });
  for (const value of saved) {
    const row = readState(destination).records.find((item) => item.id === value.record.id);
    assert.deepEqual(row.snapshot.digest, value.record.digest); assert.deepEqual(row.snapshot.reference, value.record.reference);
    assert.equal(row.example_sense, value.record.example_sense); assert.deepEqual(row.more_examples, value.record.more_examples);
  }
  await query('être'); await fields(page.locator('.dictionary-entry'), edited.record.digest.map((item) => item.core));
  for (const [locale, navigation, sourceLabel, auxiliary] of [['en', 'Dictionary', 'Field sources', 'Auxiliary (AUX)'],
    ['fr', 'Dictionnaire', 'Sources des champs', 'Auxiliaire (AUX)'], ['zh', '词典', '字段来源', '助动词 (AUX)']]) {
    await page.getByRole('banner').getByRole('combobox').first().selectOption(locale);
    await expect(page.getByRole('button', { name: navigation, exact: true })).toBeVisible();
    await fields(page.locator('.dictionary-entry'), edited.record.digest.map((item) => item.core));
    await expect(page.locator('.dictionary-entry .core-sense[data-pos="AUX"]').locator('h4').first()).toHaveText(auxiliary);
    const source = page.locator('.dictionary-entry details.core-sources').first();
    await expect(source.locator('summary')).toHaveText(sourceLabel);
    if (!await source.evaluate((node) => node.open)) await source.locator('summary').click();
    await expect(source.locator('.core-field-source')).toHaveCount(4);
    await expect(source.getByRole('link').first()).toBeVisible();
    await page.screenshot({ path: join(destination, `native-core-lookup-${locale}.png`), fullPage: true });
  }
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  const persisted = readState(destination);
  await fixture.stop(true); await start();
  compareLearning(readState(destination), persisted);
  assert.deepEqual(await invoke('vocabulary_find', { headword: 'e\u0302tre' }), edited.record);
  await openRecord(edited.record); await page.screenshot({ path: join(destination, 'native-core-details.png'), fullPage: true });
  if (scenario === 'no-speech') {
    await rejected('speech_generate', { vocabularyId: edited.record.id, operation: 'native-79-no-speech' });
    assert.deepEqual((await invoke('vocabulary_find', { headword: 'être' })).digest, edited.record.digest);
  }
  evidence.checks.push({ check: '12 groups / 48 reviewed fields, explicit trusted provider, spelling bindings, retry, duplicate, notes, examples, FSRS and normal restart', status: 'passed' });
}

async function review() {
  const { record } = await save('être', 'review');
  await invoke('review_add', { vocabularyId: record.id });
  const audio = await invoke('speech_generate', { vocabularyId: record.id, operation: 'native-79-speech' });
  assert.ok(audio.bytes.length > 0);
  await openRecord(record);
  for (const kind of ['recognition', 'listening']) {
    await page.getByRole('button', { name: `Open ${kind} card`, exact: true }).click();
    const dialog = page.getByRole('dialog', { name: `${kind === 'recognition' ? 'Recognition' : 'Listening'} card`, exact: true });
    await dialog.getByRole('button', { name: 'Show answer', exact: true }).click();
    await fields(dialog, record.digest.map((item) => item.core));
    await expect(dialog.locator('p').filter({ hasText: 'Meaning for examples' })).toContainText(record.digest[record.example_sense].core.meaning_zh);
    await page.screenshot({ path: join(destination, `native-core-${kind}-back.png`), fullPage: true });
    await dialog.getByRole('button', { name: 'Close card', exact: true }).click();
  }
  const persisted = readState(destination);
  await fixture.stop(true); await start();
  compareLearning(readState(destination), persisted);
  assert.deepEqual(readState(destination).selections, persisted.selections);
  const cached = await invoke('speech_cached', { vocabularyId: record.id });
  assert.deepEqual(cached.bytes, audio.bytes);
  await openRecord(record);
  evidence.checks.push({ check: 'Both native review backs, exact four-field senses, example association and selected audio after normal restart', status: 'passed' });
}

async function helper(action, options = {}) {
  const args = ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', action,
    '-SourcePid', String(fixture.hostPid()), '-SourceExecutable', join(destination, 'Lexique.exe'), '-WaitMs', '4000'];
  if (options.filename) args.push('-Text', options.filename, '-FilenameControlId', String(options.filenameId));
  const child = spawn('pwsh', args, { windowsHide: true }); fixture.ownHelper(child);
  let stdout = '', stderr = '';
  child.stdout.on('data', (bytes) => { stdout += bytes.toString(); if (stdout.length > 65536) child.kill(); });
  child.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
  const timer = setTimeout(() => child.kill(), 8000);
  return new Promise((done, reject) => {
    child.once('error', reject);
    child.once('exit', (code) => {
      clearTimeout(timer);
      if (code !== 0) reject(new Error(`Owned picker helper ${action} exit ${code}: ${stderr}`));
      else { try { done(JSON.parse(stdout)); } catch (error) { reject(error); } }
    });
  });
}

async function watchBackup() {
  await page.evaluate(async () => {
    window.pilotBackupEvents = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress']) {
      await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
        handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.pilotBackupEvents.push({ event, payload: value.payload }), false) });
    }
  });
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
}

async function picker(button, filename) {
  const observed = helper('picker-controls'); observed.catch(() => {});
  const clicked = page.getByRole('button', { name: button, exact: true }).click({ noWaitAfter: true }); clicked.catch(() => {});
  const controls = await observed, filenameId = button === 'Create backup' ? 1001 : 1148;
  assert.equal(controls.pid, fixture.hostPid()); assert.equal(controls.class, '#32770');
  assert.equal(controls.controls.filter((control) => control.class === 'Edit' && control.id === filenameId && control.visible && control.enabled).length, 1);
  for (const id of [1, 2]) assert.equal(controls.controls.filter((control) => control.class === 'Button' && control.id === id && control.visible && control.enabled).length, 1);
  const result = await helper('picker-select', { filename, filenameId }); await clicked;
  assert.equal(result.dialog, controls.dialog); assert.equal(result.disappeared, true); assert.equal(result.filename, filename);
  evidence.pickers.push({ button, controls, result });
}

async function backupStatus(kind, state) {
  const event = kind === 'export' ? 'lexique-backup-progress' : 'lexique-backup-preview-progress';
  let operation, status;
  await expect.poll(async () => { operation = await page.evaluate((event) => window.pilotBackupEvents.filter((item) => item.event === event).at(-1)?.payload.operation, event); return operation; }).toBeTruthy();
  await expect.poll(async () => { status = await invoke(kind === 'export' ? 'backup_operation_status' : 'backup_preview_status', { operation }); return status.state; }, { timeout: 12000 }).toBe(state);
  return status;
}

async function exportBackup() {
  const saved = [];
  for (const [index, key] of [...new Set(curation.groups.map((group) => group.headword))].entries()) saved.push(await save(key, `export-${index}`));
  await rate(saved[0].record);
  await watchBackup();
  const filename = join(destination, 'pilot.lexique-backup');
  await picker('Create backup', filename);
  const status = await backupStatus('export', 'completed');
  assert.equal(status.result.destination, filename); assert.equal((await stat(filename)).size, status.result.byte_length);
  const archive = inspectPackage(destination, filename);
  assert.equal(archive.manifest.format_version, 1); assert.equal(archive.manifest.snapshot_version, 2); assert.equal(archive.snapshot.snapshot_version, 2);
  assert.equal(archive.snapshot.records.length, 7);
  for (const value of saved) assert.deepEqual(archive.snapshot.records.find((record) => record.id === value.record.id), value.record);
  evidence.checks.push({ check: 'Native export contains all 12 four-field senses/provenance with examples, cards and ratings; ZIP v1/logical v2', status: 'passed', export: status });
}

async function restoreBackup(legacy = false) {
  assert.ok(input, 'Expected an existing backup to copy into the isolated destination');
  const filename = join(destination, legacy ? 'legacy.lexique-backup' : 'pilot.lexique-backup');
  await cp(resolve(input), filename);
  const original = await readFile(filename), archive = inspectPackage(destination, filename);
  assert.equal(archive.manifest.format_version, 1); assert.equal(archive.snapshot.snapshot_version, legacy ? 1 : 2);
  const seed = await save('bien', 'before-restore'); await rate(seed.record);
  await watchBackup(); await picker('Choose backup', filename);
  const checked = await backupStatus('preview', 'preview_ready');
  assert.equal(checked.preview.byte_length, original.length);
  await page.screenshot({ path: join(destination, 'native-backup-preview.png'), fullPage: true });
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore', exact: true });
  const confirm = dialog.getByRole('button', { name: 'Confirm and restore', exact: true });
  await expect(confirm).toBeDisabled(); await dialog.getByRole('checkbox').check();
  page = await fixture.followRestart(() => confirm.click());
  await page.context().route('**/*', (route) => ['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(new URL(route.request().url()).hostname) ? route.continue() : route.abort());
  let status;
  await expect.poll(async () => { status = await invoke('startup_status'); return status.state; }, { timeout: 12000 }).toBe('ready');
  assert.equal(status.operation, checked.operation); assert.deepEqual(status.restored.counts, checked.preview.counts);
  assert.deepEqual(await readFile(filename), original);
  const restored = readState(destination);
  assert.equal(restored.records.length, archive.snapshot.records.length);
  for (const record of archive.snapshot.records) {
    const actual = await invoke('vocabulary_find', { headword: record.headword });
    assert.equal(actual.id, record.id); assert.equal(actual.notes, record.notes); assert.equal(actual.revision, record.revision);
    assert.deepEqual(actual.reference, record.reference); assert.deepEqual(actual.digest, record.digest);
    assert.equal(actual.example_sense, record.example_sense); assert.deepEqual(actual.more_examples, record.more_examples);
    const row = restored.records.find((item) => item.id === record.id);
    assert.deepEqual(row.snapshot.digest, record.digest);
  }
  assert.deepEqual(restored.cards, archive.snapshot.cards);
  assert.equal(restored.ratings.length, archive.snapshot.ratings.length);
  assert.deepEqual(restored.selections, archive.snapshot.audio_selections);
  const operation = archive.snapshot.save_operations[0];
  const replay = await invoke('vocabulary_save', { request: operation.request, dictionary_id: 'builtin-core' });
  assert.equal(replay.outcome, 'replayed'); assert.equal(replay.record.id, operation.record_id);
  if (legacy) {
    for (const record of archive.snapshot.records) assert.ok(record.digest.every((item) => !item.core));
    for (const selection of archive.snapshot.audio_selections) {
      if (selection.current_version) assert.ok((await invoke('speech_cached', { vocabularyId: selection.vocabulary_id })).bytes.length > 0);
    }
    const previousIds = archive.snapshot.dictionary_catalog.entries.map((item) => item.id);
    const catalog = await invoke('dictionary_settings');
    assert.deepEqual(catalog.map((item) => item.id), [...previousIds, 'builtin-core']);
    const current = replay.record;
    const edited = await invoke('vocabulary_save', { request: { ...request(current.headword, 'legacy-edit'), id: current.id, revision: current.revision,
      notes: current.notes + '\nNote après restauration.', example_fr: current.example_fr, example_zh: current.example_zh,
      example_source: current.example_source, example_sense: current.example_sense, more_examples: current.more_examples }, dictionary_id: 'builtin-core' });
    assert.equal(edited.outcome, 'updated'); assert.deepEqual(edited.record.digest, current.digest);
    assert.deepEqual(readState(destination).cards, restored.cards); assert.deepEqual(readState(destination).selections, restored.selections);
    const key = curation.groups.find((group) => !archive.snapshot.records.some((record) => record.headword === group.headword)).headword;
    await save(key, 'after-legacy');
  } else await openRecord(replay.record);
  evidence.checks.push({ check: legacy ? 'Genuine native v1 backup, old translations/examples/operations/cards/ratings/audio, edit and new core save after restore' : 'Native v2 preview/restore retains complete four-field senses and provenance', status: 'passed' });
}

async function rejectBackup() {
  assert.ok(input);
  const original = join(destination, 'pilot.lexique-backup'); await cp(resolve(input), original);
  const variant = process.env.LEXIQUE_BACKUP_VARIANT ?? 'future';
  assert.ok(['future', 'mismatch', 'malformed'].includes(variant));
  const filename = fixtureSql(destination, `
import zipfile
source=pathlib.Path(p['source']); target=source.with_name('invalid-'+p['variant']+'.lexique-backup')
with zipfile.ZipFile(source) as z: entries=[(i,z.read(i)) for i in z.infolist()]
manifest=json.loads(dict((i.filename,b) for i,b in entries)['manifest.json'])
snapshot=json.loads(dict((i.filename,b) for i,b in entries)['snapshot.json'])
if p['variant']=='future': manifest['snapshot_version']=snapshot['snapshot_version']=3
elif p['variant']=='mismatch': manifest['snapshot_version']=1
else: snapshot['records'][0]['digest'][0]['core']['meaning_zh']=''
with zipfile.ZipFile(target,'w',compression=zipfile.ZIP_STORED) as z:
 for info,data in entries:
  if info.filename=='manifest.json': data=json.dumps(manifest,ensure_ascii=False).encode()
  elif info.filename=='snapshot.json': data=json.dumps(snapshot,ensure_ascii=False).encode()
  z.writestr(info,data)
print(json.dumps(str(target)))`, { source: original, variant }, true);
  await save('être', 'preserve-on-failure');
  await watchBackup(); const before = readState(destination), bytes = await readFile(original);
  await picker('Choose backup', filename); const failure = await backupStatus('preview', 'failed');
  assert.ok(failure.error); assert.equal(failure.preview, null);
  await expect(page.getByRole('alert').last()).toContainText(failure.error);
  compareLearning(readState(destination), before); assert.deepEqual(await readFile(original), bytes);
  evidence.checks.push({ check: `Native ${variant} backup rejection preserves current learning and existing archive`, status: 'passed', failure });
}

async function lifecycle() {
  const { record, submitted } = await save('être', 'lifecycle'); await rate(record);
  const before = readState(destination);
  await fixture.stop(true);
  if (scenario === 'missing') await rename(coreFile, coreFile + '.unavailable');
  else if (scenario === 'corrupt') { await chmod(coreFile, 0o666); await writeFile(coreFile, 'Synthetic invalid isolated core resource'); }
  else { await chmod(coreFile, 0o666); fixtureSql(destination, `
c.close(); resource=sqlite3.connect(p['resource'])
resource.execute("UPDATE metadata SET value='2' WHERE key='format_version'")
resource.commit(); resource.close()
print(json.dumps(True))`, { resource: coreFile }, true); }
  await start();
  await rejected('dictionary_lookup', { key: 'être', dictionary_id: 'builtin-core' }, scenario === 'missing' ? 'missing' : 'corrupt');
  assert.ok((await invoke('dictionary_lookup', { key: 'école', dictionary_id: 'builtin' })).entries.length > 0);
  await rejected('vocabulary_save', { request: request('bien', 'unavailable-new'), dictionary_id: 'builtin-core' });
  const replay = await invoke('vocabulary_save', { request: submitted, dictionary_id: 'builtin-core' });
  assert.equal(replay.outcome, 'replayed'); assert.deepEqual(replay.record, record);
  const edited = await invoke('vocabulary_save', { request: { ...submitted, operation: 'native-79-unavailable-edit', id: record.id, revision: record.revision,
    notes: 'Note modifiée sans la ressource.' }, dictionary_id: 'builtin-core' });
  assert.equal(edited.outcome, 'updated'); assert.deepEqual(edited.record.digest, record.digest);
  assert.deepEqual(readState(destination).cards, before.cards); assert.deepEqual(readState(destination).progress, before.progress);
  await openRecord(edited.record);
  evidence.checks.push({ check: `${scenario} resource fails visibly; other providers, committed replay, note edit and frozen content remain usable`, status: 'passed' });
}

async function catalogue() {
  const bytes = [...await readFile(resolve('e2e/fixtures/mdx/other.mdx'))];
  for (let index = 0; index < 15; index++) {
    const catalog = await page.evaluate(({ bytes, title }) => window.__TAURI_INTERNALS__.invoke('dictionary_import', new Uint8Array(bytes),
      { headers: { 'x-lexique-dictionary-name': title } }), { bytes, title: Buffer.from(`pilot-${index}.mdx`).toString('base64') });
    assert.equal(catalog.length, 4 + index);
  }
  const full = await invoke('dictionary_settings'); assert.equal(full.length, 18);
  const order = [...full].reverse().map((item) => item.id); await invoke('dictionary_settings', { order });
  const error = await page.evaluate(async (bytes) => {
    try { await window.__TAURI_INTERNALS__.invoke('dictionary_import', new Uint8Array(bytes), { headers: { 'x-lexique-dictionary-name': 'c2l4dGVlbnRoLm1keA==' } }); return null; }
    catch (error) { return String(error); }
  }, bytes);
  assert.ok(error);
  const before = readState(destination);
  await fixture.stop(true);
  const legacyOrder = fixtureSql(destination, `
raw=json.loads(c.execute("SELECT value FROM app_metadata WHERE key='dictionary-settings'").fetchone()[0])
raw['entries']=[item for item in raw['entries'] if item['id']!='builtin-core']
c.execute("UPDATE app_metadata SET value=? WHERE key='dictionary-settings'",(json.dumps(raw,ensure_ascii=False),)); c.commit()
print(json.dumps([item['id'] for item in raw['entries']]))`);
  await start();
  const upgraded = await invoke('dictionary_settings'); assert.deepEqual(upgraded.map((item) => item.id), [...legacyOrder, 'builtin-core']);
  assert.deepEqual(readState(destination).devices, before.devices);
  assert.deepEqual(upgraded.filter((item) => item.id.startsWith('mdx-')).map((item) => [item.id, item.content_hash]),
    [...full].reverse().filter((item) => item.id.startsWith('mdx-')).map((item) => [item.id, item.content_hash]));
  for (const item of upgraded.filter((item) => item.id.startsWith('mdx-'))) {
    assert.equal(item.availability, 'available'); assert.ok((await invoke('dictionary_lookup', { key: 'école', dictionary_id: item.id })).entries.length > 0);
  }
  evidence.checks.push({ check: '15 custom dictionary slots, 18 total, capacity rejection and upgrade append preserve order, identities, file bindings and lookup', status: 'passed' });
}

async function copiedProfile() {
  assert.ok(input, 'Expected a closed isolated native profile');
  assert.deepEqual(JSON.parse(await readFile(join(resolve(input), 'cleanup.json'), 'utf8')), { hosts: [], workers: [], helpers: 0 });
  await cp(join(resolve(input), 'Lexique-data'), join(destination, 'Lexique-data'), { recursive: true });
}

async function replacement() {
  const previous = readState(destination), record = await invoke('vocabulary_find', { headword: 'être' });
  if (scenario === 'limit') {
    await rejected('dictionary_lookup', { key: 'être', dictionary_id: 'builtin-core' }, 'limit');
    await rejected('vocabulary_save', { request: request('être', 'limit-new'), dictionary_id: 'builtin-core' });
    assert.ok((await invoke('dictionary_lookup', { key: 'école', dictionary_id: 'builtin' })).entries.length > 0);
  } else {
    const current = await invoke('dictionary_lookup', { key: 'être', dictionary_id: 'builtin-core' });
    assert.notEqual(current.source.version, record.reference.version);
    await query('être'); await page.getByRole('button', { name: 'View existing record', exact: true }).click();
    await expect(page.getByText('The dictionary version has changed. The saved digest keeps its original content.', { exact: true })).toBeVisible();
    await save('bien', 'newer-content');
  }
  assert.deepEqual((await invoke('vocabulary_find', { headword: 'être' })).digest, record.digest);
  assert.deepEqual(readState(destination).progress, previous.progress); assert.deepEqual(readState(destination).selections, previous.selections);
  const committed = previous.saves.find((operation) => operation.record_id === record.id);
  assert.equal((await invoke('vocabulary_save', { request: committed.request, dictionary_id: 'builtin-core' })).outcome, 'replayed');
  await openRecord(record);
  evidence.checks.push({ check: scenario === 'limit' ? 'Genuine native core result limit preserves saved content and committed retry' : 'New resource version is visible while previous digest, progress and audio remain frozen', status: 'passed' });
}

try {
  if (['no-speech', 'legacy'].includes(scenario)) {
    const present = await stat(model).catch((error) => { if (error.code === 'ENOENT') return null; throw error; });
    if (present) await rename(model, model + '.unavailable');
    evidence.speechModel = 'absent in isolated package';
  }
  if (['newer', 'limit'].includes(scenario)) await copiedProfile();
  await start();
  if (['pilot', 'no-speech'].includes(scenario)) await pilot();
  else if (scenario === 'review') await review();
  else if (scenario === 'catalogue') await catalogue();
  else if (['missing', 'corrupt', 'future'].includes(scenario)) await lifecycle();
  else if (scenario === 'export') await exportBackup();
  else if (['restore', 'legacy'].includes(scenario)) await restoreBackup(scenario === 'legacy');
  else if (scenario === 'reject') await rejectBackup();
  else await replacement();
  await fixture.stop(true); evidence.finalState = readState(destination);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
} catch (error) {
  evidence.failure = String(error);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2));
  throw error;
} finally { await fixture.finish(); }
