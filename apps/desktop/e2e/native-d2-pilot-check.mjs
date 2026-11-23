/* global process, window, URL, setTimeout, clearTimeout, Buffer */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { chmod, cp, mkdir, readFile, rename, stat, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { compareLearning, fixtureSql, inspectPackage, readState, seedAudio } from './native-backup-state.mjs';

const [scenario, resources, binary, conjugation, frequency, curationPath, destinationPath, input] = process.argv.slice(2);
assert.ok([scenario, resources, binary, conjugation, frequency, curationPath, destinationPath].every(Boolean),
  'Expected scenario, package, binary, conjugation, frequency, reviewed curation and new destination; backup/profile scenarios also take an input');
assert.ok(['pilot', 'batch', 'review', 'no-speech', 'catalogue', 'missing', 'corrupt', 'future', 'unsupported', 'newer', 'upgrade', 'limit', 'export', 'restore', 'legacy', 'reject'].includes(scenario));
const curation = JSON.parse(await readFile(resolve(curationPath), 'utf8'));
const frozenScope = (await readFile(resolve(curationPath, '..', 'scope-manifest.csv'), 'utf8')).trim().split(/\r?\n/).slice(1);
assert.equal(frozenScope.length, 4087);
const expectedRows = Array.from({ length: 3950 }, (_, index) => index + 1)
  .filter((row) => ![87, 305, 816, 849, 859, 860, 934, 1005, 1051, 1208, 1218, 1219, 1220, 1300, 1386, 2560, 2626, 2792, 3098, 3783, 3885, 3922, 3944].includes(row));
const membership = expectedRows.map((row) => {
  const match = frozenScope[row - 1].match(/^"((?:[^"]|"")*)","([^"]+)",/);
  assert.ok(match, `Invalid frozen scope row ${row}`);
  return [row, match[1].replaceAll('""', '"'), match[2]];
});
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

function installNetworkBoundary() {
  return page.context().route('**/*', (route) => {
    const host = new URL(route.request().url()).hostname;
    if (['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(host)) return route.continue();
    evidence.externalRequests.push(host); return route.abort();
  });
}

async function start() {
  page = await fixture.start();
  page.setDefaultTimeout(8000);
  await installNetworkBoundary();
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
      assert.ok(provenance.title.includes(source.title)); assert.ok(provenance.version.includes(String(source.revid)));
      assert.ok(provenance.attribution.includes(source.attribution)); assert.ok(provenance.attribution.includes(source.history_url));
      assert.equal(provenance.source_url, source.fixed_url);
      assert.equal(provenance.license.replaceAll(/[- ]/g, ''), expected.content_license.replaceAll(/[- ]/g, ''));
      assert.equal(provenance.license_url, source.license_url);
      assert.ok(provenance.span.includes(`line ${expected.source_span.line_1_based}`));
      assert.ok(provenance.span.includes('selected-source-excerpts.json'));
      assert.equal(provenance.modification, expected.modification);
    }
  }
  return actual;
}

function request(headword, suffix = headword.replaceAll(/[^a-z]/g, 'x')) {
  return { operation: `native-79-${suffix}`, headword, notes: `Note conservée : ${headword}.`, example_fr: 'Cet être est vivant.',
    example_zh: '这个生物是活的。', example_source: null, example_sense: headword === 'être' ? 1 : 0,
    more_examples: [{ fr: 'Cet être a besoin d’eau.', zh: '这个生物需要水。', source: null }], id: null, revision: null };
}

function explicitExampleSources(value) {
  return { ...value, example_source: value.example_source ?? null,
    more_examples: value.more_examples.map((example) => ({ ...example, source: example.source ?? null })) };
}

// FSRS memory and parameters are f32; compare exact 32-bit values across JSON representations.
function fsrsCard(card) {
  return { ...card, memory: card.memory && { stability: Math.fround(card.memory.stability), difficulty: Math.fround(card.memory.difficulty) } };
}

function fsrsRating(rating) {
  assert.ok([false, true, 0, 1].includes(rating.undo_queue_rebuilt));
  return { ...rating, undo_queue_rebuilt: Boolean(rating.undo_queue_rebuilt), before_state: fsrsCard(rating.before_state),
    result: { ...rating.result, card: fsrsCard(rating.result.card),
      model: { ...rating.result.model, parameters: rating.result.model.parameters.map(Math.fround) } } };
}

function snapshotLearning(state, snapshot) {
  assert.deepEqual(state.cards, snapshot.cards);
  assert.deepEqual(state.progress, snapshot.progress.map(({ card, ...row }) => ({ ...row, state: card })));
  assert.deepEqual(state.ratings.map(fsrsRating), snapshot.ratings.map(({ sequence, ...row }, index) => {
    assert.equal(sequence, index + 1); return fsrsRating(row);
  }));
  const operations = (values) => values.map((item) => ({ ...item, request: explicitExampleSources(item.request) }));
  assert.deepEqual(operations(state.saves), operations([...snapshot.save_operations].sort((a, b) => a.operation.localeCompare(b.operation))));
  assert.deepEqual(state.selections, snapshot.audio_selections);
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

async function sourceExamples(saved, operation) {
  const submitted = { ...saved.submitted, operation, id: saved.record.id, revision: saved.record.revision,
    example_source: saved.record.reference, more_examples: saved.record.more_examples.map((example) => ({ ...example, source: saved.record.reference })) };
  const { record } = await invoke('vocabulary_save', { request: submitted, dictionary_id: 'builtin-core' });
  return { record, submitted };
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
  const core = page.locator('#dictionary-tab-builtin-core'); await core.click();
  await expect(core).toHaveAttribute('aria-selected', 'true');
}

async function openRecord(record) {
  await page.getByRole('button', { name: 'Notes', exact: true }).click();
  await page.getByLabel('Records per page').selectOption('100');
  await page.locator('.vocabulary-list button').filter({ has: page.getByText(record.headword, { exact: true }) }).click();
  await fields(page.locator('.vocabulary-summary'), record.digest.map((item) => item.core));
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(record.notes);
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue(String(record.example_sense));
}

async function rate(record) {
  const cards = await invoke('review_add', { vocabularyId: record.id });
  const recognition = cards.find((card) => card.kind === 'recognition');
  const result = await invoke('review_rate', { request: { operation: `native-79-rating-${record.id}`, card_id: recognition.id,
    expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  assert.equal(result.status, 'committed', JSON.stringify(result));
}

async function pilot() {
  assert.deepEqual((await invoke('dictionary_settings')).map((item) => item.id), ['builtin-core', 'builtin', 'builtin-fra-eng']);
  assert.equal((await invoke('dictionary_lookup', { key: 'être', dictionary_id: 'builtin-core' })).source.version, curation.content_version);
  await invoke('dictionary_settings', { order: ['builtin', 'builtin-fra-eng', 'builtin-core'] });
  const before = readState(destination);
  for (const field of ['digest', 'reference']) await rejected('vocabulary_save', {
    request: { ...request('bien', `forged-${field}`), [field]: [] }, dictionary_id: 'builtin-core',
  });
  compareLearning(readState(destination), before);
  const saved = [];
  for (const [index, key] of [...new Set(curation.groups.map((group) => group.headword))].entries()) saved.push(await save(key, `word-${index}`));
  const first = saved[0];
  for (const [queryKey, frozen] of [['e\u0302tre', 'être'], ['cœur', 'coeur'], ['coeur', 'coeur'], ['aujourd’hui', "aujourd'hui"], ['l’', "l'"], ['d’', "d'"], ['t’', "t'"], ['s’', "s'"]]) {
    admitted(await invoke('dictionary_lookup', { key: queryKey, dictionary_id: 'builtin-core' }), frozen);
  }
  assert.deepEqual((await invoke('dictionary_lookup', { key: 'mot-absent-du-pilote', dictionary_id: 'builtin-core' })).entries, []);
  const replay = await invoke('vocabulary_save', { request: first.submitted, dictionary_id: 'builtin-core' });
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
    const provenance = edited.record.digest[0].core.field_sources.definition_fr;
    await expect(source.getByText(provenance.source_url, { exact: true }).first()).toBeVisible();
    await expect(source.getByText(provenance.license_url, { exact: true }).first()).toBeVisible();
    await page.screenshot({ path: join(destination, `native-core-lookup-${locale}.png`), fullPage: true });
  }
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  const persisted = readState(destination);
  await fixture.stop(true); await start();
  compareLearning(readState(destination), persisted);
  assert.deepEqual(await invoke('vocabulary_find', { headword: 'e\u0302tre' }), edited.record);
  await openRecord(edited.record); await page.screenshot({ path: join(destination, 'native-core-details.png'), fullPage: true });
  const batchRecord = saved.find((value) => value.record.headword === 'bon').record;
  assert.deepEqual(await invoke('vocabulary_find', { headword: 'bon' }), batchRecord);
  await openRecord(batchRecord);
  await page.screenshot({ path: join(destination, 'native-core-batch-bon-details.png'), fullPage: true });
  for (const key of ['bon', 'petit', 'par']) {
    const value = saved.find((item) => item.record.headword === key).record;
    await query(key); await fields(page.locator('.dictionary-entry'), value.digest.map((item) => item.core));
    await page.screenshot({ path: join(destination, `native-core-batch-${key}-lookup.png`), fullPage: true });
  }
  await query('bon');
  for (const [locale, label] of [['en', 'Interjection / onomatopoeia'], ['fr', 'Interjection / onomatopée'], ['zh', '感叹词／拟声词']]) {
    await page.getByRole('banner').getByRole('combobox').first().selectOption(locale);
    await expect(page.locator('.dictionary-entry .core-sense[data-pos="ONO"] h4')).toHaveText(`${label} (ONO)`);
    await fields(page.locator('.dictionary-entry'), batchRecord.digest.map((item) => item.core));
    await page.screenshot({ path: join(destination, `native-core-batch-bon-${locale}.png`), fullPage: true });
  }
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  const possessive = saved.find((value) => value.record.headword === 'mon').record;
  await query('mon');
  for (const [locale, label] of [['en', 'Possessive adjective'], ['fr', 'Adjectif possessif'], ['zh', '物主形容词']]) {
    await page.getByRole('banner').getByRole('combobox').first().selectOption(locale);
    await expect(page.locator('.dictionary-entry .core-sense[data-pos="ADJ:pos"] h4')).toHaveText(`${label} (ADJ:pos)`);
    await fields(page.locator('.dictionary-entry'), possessive.digest.map((item) => item.core));
  }
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en');
  await openRecord(possessive);
  if (scenario === 'no-speech') {
    await rejected('speech_generate', { vocabularyId: edited.record.id, operation: 'native-79-no-speech' });
    assert.deepEqual((await invoke('vocabulary_find', { headword: 'être' })).digest, edited.record.digest);
  }
  evidence.checks.push({ check: `50 batch-83 groups / 200 fields, including 49 everyday groups and the approved golf exception; ${curation.groups.length} total groups / ${curation.groups.length * 4} fields and ${curation.annotation_exception_groups} approved annotation exceptions; trusted provider, spelling bindings, retry, duplicate, notes, examples, FSRS and normal restart`, status: 'passed' });
  evidence.checks.push({ check: 'Batch bon adjective/adverb/noun/interjection saved fields and provenance match audited content after normal restart; bon, petit and par restrictions remain readable; ONO has English, French and Chinese labels', status: 'passed' });
  evidence.checks.push({ check: 'mon/ADJ:pos lookup and saved fields remain readable with English, French and Chinese subclass labels', status: 'passed' });
}

async function contentBatch() {
  const issue = Number(input);
  assert.ok(Number.isInteger(issue) && issue >= 84 && issue <= 159, 'Expected batch issue 84–159');
  const firstRow = 151 + (issue - 84) * 50;
  const groups = curation.groups.filter((group) => group.frozen_row >= firstRow && group.frozen_row < firstRow + 50);
  const annotationRows = ({ 87: [305], 97: [816, 849], 98: [859, 860], 99: [934],
    101: [1005], 102: [1051], 105: [1208, 1218, 1219, 1220], 106: [1300], 108: [1386],
    132: [2560], 133: [2626], 136: [2792], 142: [3098], 156: [3783], 158: [3885], 159: [3922, 3944] })[issue] ?? [];
  assert.equal(groups.length, 50 - annotationRows.length);
  for (const row of annotationRows) {
    const match = frozenScope[row - 1].match(/^"((?:[^"]|"")*)","([^"]+)",/);
    assert.ok(match, `Invalid annotation row ${row}`);
    const [, headword, pos] = match;
    const lookup = await invoke('dictionary_lookup', { dictionary_id: 'builtin-core', key: headword });
    assert.ok(senses(lookup).every((sense) => sense.pos !== pos), `Annotation row ${row} has no admitted sense`);
  }
  const aliases = new Map(curation.spelling_relations
    .filter((relation) => groups.some((group) => group.headword === relation.frozen_headword
      && group.lexique_cgram === relation.lexique_cgram))
    .map((relation) => [relation.source_headword, relation.frozen_headword]));
  for (const [key, frozen] of aliases) {
    admitted(await invoke('dictionary_lookup', { dictionary_id: 'builtin-core', key }), frozen);
  }
  const saved = [];
  for (const [index, key] of [...new Set(groups.map((group) => group.headword))].entries()) {
    const result = await save(key, `batch-${issue}-${index}`);
    assert.equal(result.record.reference.version, curation.content_version);
    saved.push(result);
  }
  const representative = issue === 115 ? saved.find(({ record }) => record.headword === 'tien') : saved[0];
  assert.ok(representative);
  await query(representative.record.headword);
  await fields(page.locator('.dictionary-entry'), representative.record.digest.map((item) => item.core));
  if (issue === 115) {
    const published = representative.record.digest.map((item) => item.core)
      .find((sense) => sense.field_sources.definition_fr.version.includes('A8T0594-1935-tome2-page659'));
    assert.ok(published);
    const block = page.locator(`.dictionary-entry .core-sense[data-sense-id="${published.sense_id}"]`);
    await block.locator('.core-sources summary').click();
    for (const [index, field] of ['definition_fr', 'ipa', 'meaning_en', 'meaning_zh'].entries()) {
      const source = published.field_sources[field], section = block.locator('.core-field-source').nth(index);
      for (const value of [source.title, source.version, source.span]) await expect(section.getByText(value, { exact: true })).toBeVisible();
    }
    evidence.checks.push({ check: 'The DAF8 tien noun displays its fixed printed edition, selected capture lines and four field sources', status: 'passed' });
  }
  await page.screenshot({ path: join(destination, `batch-${issue}-lookup.png`), fullPage: true });
  const before = readState(destination);
  await fixture.stop(true);
  await start();
  for (const { record } of saved) assert.deepEqual(await invoke('vocabulary_find', { headword: record.headword }), record);
  compareLearning(readState(destination), before);
  await openRecord(representative.record);
  await page.screenshot({ path: join(destination, `batch-${issue}-reopened.png`), fullPage: true });
  evidence.checks.push({ check: `Batch #${issue}: ${groups.length} admitted groups / ${groups.length * 4} fields match audited content; ${annotationRows.length} approved annotation exceptions; ${aliases.size} reviewed spelling queries; ${saved.length} headwords saved and reopened with unchanged provenance and learning state`, status: 'passed' });
}

async function review() {
  const { record } = await save('être', 'review');
  await invoke('review_add', { vocabularyId: record.id });
  const audio = await invoke('speech_generate', { vocabularyId: record.id, operation: 'native-79-speech' });
  assert.ok(audio.bytes.length > 0);
  evidence.contentLengths = record.digest.map(({ core }) => ({ sense_id: core.sense_id,
    fields: Object.fromEntries(['definition_fr', 'ipa', 'meaning_en', 'meaning_zh', 'usage'].map((field) => [field, [...core[field]].length])),
    spans: Object.fromEntries(Object.entries(core.field_sources).map(([field, source]) => [field, [...source.span].length])) }));
  await query('être'); await fields(page.locator('.dictionary-entry'), record.digest.map((item) => item.core));
  await fixture.stop(true); await rename(coreFile, coreFile + '.unavailable'); await start();
  await openRecord(record);
  await expect(page.locator('.dictionary-original').getByRole('alert')).toBeVisible();
  for (const kind of ['recognition', 'listening']) {
    await page.getByRole('button', { name: `Open ${kind} card`, exact: true }).click();
    const dialog = page.getByRole('dialog', { name: `${kind === 'recognition' ? 'Recognition' : 'Listening'} card`, exact: true });
    await dialog.getByRole('button', { name: 'Show answer', exact: true }).click();
    await fields(dialog, record.digest.map((item) => item.core));
    await expect(dialog.locator('p').filter({ hasText: 'Meaning for examples' })).toContainText(record.digest[record.example_sense].core.meaning_zh);
    await page.screenshot({ path: join(destination, `native-core-${kind}-back.png`), fullPage: true });
    await dialog.getByRole('button', { name: kind === 'recognition' ? 'Close card' : 'Skip', exact: true }).click();
  }
  const persisted = readState(destination);
  await fixture.stop(true); await start();
  compareLearning(readState(destination), persisted);
  assert.deepEqual(readState(destination).selections, persisted.selections);
  const cached = await invoke('speech_cached', { vocabularyId: record.id });
  assert.deepEqual(cached.bytes, audio.bytes);
  await openRecord(record);
  evidence.checks.push({ check: 'Complete native lookup, resource-absent fallback and both review backs retain four-field senses, example association and selected audio after normal restart', status: 'passed' });
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
  saved[0] = await sourceExamples(saved[0], 'native-80-export-example-sources');
  await rate(saved[0].record);
  await invoke('review_add', { vocabularyId: saved[1].record.id });
  await seedSelectedAudio(saved.slice(0, 2).map(({ record }) => record.id)); await start();
  const before = readState(destination); evidence.beforeState = before;
  await watchBackup();
  const filename = join(destination, 'pilot.lexique-backup');
  await picker('Create backup', filename);
  const status = await backupStatus('export', 'completed');
  assert.equal(status.result.destination, filename); assert.equal((await stat(filename)).size, status.result.byte_length);
  const archive = inspectPackage(destination, filename);
  assert.equal(archive.manifest.format_version, 1); assert.equal(archive.manifest.snapshot_version, 4); assert.equal(archive.snapshot.snapshot_version, 4);
  assert.equal(archive.snapshot.records.length, 7);
  assert.equal(archive.snapshot.cards.length, 4); assert.equal(archive.snapshot.ratings.length, 1);
  snapshotLearning(before, archive.snapshot); compareLearning(readState(destination), before);
  assert.equal(archive.snapshot.audio_selections.length, 2);
  assert.ok(archive.snapshot.audio_selections.every((selection) => selection.current_version && selection.previous_version));
  for (const value of saved) assert.deepEqual(archive.snapshot.records.find((record) => record.id === value.record.id), explicitExampleSources(value.record));
  evidence.checks.push({ check: 'Native export contains all 12 four-field senses/provenance with sourced examples, cards, ratings and selected/previous audio; ZIP v1/logical v2', status: 'passed', export: status });
}

async function restoreBackup(legacy = false) {
  assert.ok(input, 'Expected an existing backup to copy into the isolated destination');
  const filename = join(destination, legacy ? 'legacy.lexique-backup' : 'pilot.lexique-backup');
  await cp(resolve(input), filename);
  const original = await readFile(filename), archive = inspectPackage(destination, filename);
  assert.equal(archive.manifest.format_version, 1); assert.equal(archive.snapshot.snapshot_version, legacy ? 1 : 4);
  const seed = await save('bien', 'before-restore'); await rate(seed.record);
  await seedSelectedAudio([seed.record.id]); await start();
  await watchBackup(); await picker('Choose backup', filename);
  const checked = await backupStatus('preview', 'preview_ready');
  assert.equal(checked.preview.byte_length, original.length);
  await page.screenshot({ path: join(destination, 'native-backup-preview.png'), fullPage: true });
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore', exact: true });
  const confirm = dialog.getByRole('button', { name: 'Confirm and restore', exact: true });
  await expect(confirm).toBeDisabled();
  const beforeCancel = readState(destination);
  evidence.beforeState = beforeCancel;
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click(); await expect(dialog).toHaveCount(0);
  const cancelled = readState(destination); compareLearning(cancelled, beforeCancel); preservedAudio(cancelled, beforeCancel);
  assert.deepEqual(await readFile(filename), original);
  evidence.checks.push({ check: 'Native confirmation cancellation retains current records, learning, selected/protected audio and the original backup', status: 'passed' });
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click(); await dialog.getByRole('checkbox').check();
  page = await fixture.followRestart(() => confirm.click());
  await installNetworkBoundary();
  let status;
  await expect.poll(async () => { status = await invoke('startup_status'); return status.state; }, { timeout: 12000 }).toBe('ready');
  assert.equal(status.operation, checked.operation); assert.deepEqual(status.restored.counts, checked.preview.counts);
  assert.deepEqual(await readFile(filename), original);
  const restored = readState(destination);
  evidence.restoredState = restored;
  assert.equal(restored.records.length, archive.snapshot.records.length);
  for (const record of archive.snapshot.records) {
    const actual = await invoke('vocabulary_find', { headword: record.headword });
    assert.deepEqual(explicitExampleSources(actual), record);
    const row = restored.records.find((item) => item.id === record.id);
    assert.deepEqual(row.snapshot.digest, record.digest);
  }
  snapshotLearning(restored, archive.snapshot);
  for (const selection of archive.snapshot.audio_selections) {
    if (selection.current_version) assert.ok((await invoke('speech_cached', { vocabularyId: selection.vocabulary_id })).bytes.length > 0);
  }
  const previousIds = archive.snapshot.dictionary_catalog.entries.map((item) => item.id);
  const catalog = await invoke('dictionary_settings');
  assert.deepEqual(catalog.map((item) => item.id), legacy ? [...previousIds, 'builtin-core'] : previousIds);
  assert.deepEqual(catalog.filter((item) => item.id.startsWith('mdx-')).map((item) => [item.id, item.content_hash]),
    archive.snapshot.dictionary_catalog.entries.filter((item) => item.id.startsWith('mdx-')).map((item) => [item.id, item.content_hash]));
  const operation = archive.snapshot.save_operations[0];
  const replay = await invoke('vocabulary_save', { request: operation.request, dictionary_id: 'builtin-core' });
  assert.equal(replay.outcome, 'replayed'); assert.equal(replay.record.id, operation.record_id);
  if (legacy) {
    for (const record of archive.snapshot.records) assert.ok(record.digest.every((item) => !item.core));
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
if p['variant']=='future': manifest['snapshot_version']=snapshot['snapshot_version']=5
elif p['variant']=='mismatch': manifest['snapshot_version']=1
else: snapshot['records'][0]['digest'][0]['core']['meaning_zh']=''
with zipfile.ZipFile(target,'w',compression=zipfile.ZIP_STORED) as z:
 for info,data in entries:
  if info.filename=='manifest.json': data=json.dumps(manifest,ensure_ascii=False).encode()
  elif info.filename=='snapshot.json': data=json.dumps(snapshot,ensure_ascii=False).encode()
  z.writestr(info,data)
print(json.dumps(str(target)))`, { source: original, variant }, true);
  const saved = await save('être', 'preserve-on-failure'); await rate(saved.record);
  await seedSelectedAudio([saved.record.id]); await start();
  await watchBackup(); const before = readState(destination), bytes = await readFile(original);
  evidence.beforeState = before;
  await picker('Choose backup', filename); const failure = await backupStatus('preview', 'failed');
  assert.ok(failure.error); assert.equal(failure.preview, null);
  await expect(page.getByRole('alert').last()).toContainText(failure.error);
  const after = readState(destination);
  compareLearning(after, before); preservedAudio(after, before); assert.deepEqual(await readFile(original), bytes);
  evidence.checks.push({ check: `Native ${variant} backup rejection preserves current learning and existing archive`, status: 'passed', failure });
}

function preservedAudio(actual, expected) {
  for (const key of ['owners', 'versions', 'selections', 'imports', 'legacy', 'audio_operations', 'publications', 'repairs', 'protection', 'physical', 'devices']) {
    assert.deepEqual(actual[key], expected[key], `Resource changes retain ${key}`);
  }
}

async function seedSelectedAudio(ids) {
  await fixture.stop(true);
  const media = join(destination, 'fixture-media'); await mkdir(media);
  for (const [filename, source] of [['female-1.ogg', 'female-normal-01-48k.opus'], ['female-2.ogg', 'female-normal-02-48k.opus']]) {
    await cp(new URL(`../../../docs/evidence/issue-14/supertonic-opus/${source}`, import.meta.url), join(media, filename));
  }
  seedAudio(destination, ids, 'A', media);
}

async function failedProvider(key, code) {
  const failure = await rejected('dictionary_lookup', { key, dictionary_id: 'builtin-core' }, code);
  await query(key); await expect(page.locator('.dictionary').getByRole('alert')).toContainText(failure.message);
  const alternate = await invoke('dictionary_lookup', { key: 'école', dictionary_id: 'builtin' });
  assert.ok(alternate.entries.length > 0);
  await query('école'); await page.locator('#dictionary-tab-builtin').click();
  await expect(page.locator('.dictionary-entry')).toHaveCount(alternate.entries.length);
}

async function lifecycle() {
  const saved = await save('être', 'lifecycle');
  assert.deepEqual((await invoke('dictionary_lookup', { key: 'mot-absent-native-80', dictionary_id: 'builtin-core' })).entries, []);
  const { record, submitted } = await sourceExamples(saved, 'native-80-source-associations');
  const legacyLookup = await invoke('dictionary_lookup', { key: 'école', dictionary_id: 'builtin' });
  const legacy = await invoke('vocabulary_save', { request: { ...request('école', 'legacy-lifecycle'),
    example_sense: legacyLookup.entries.length - 1 }, dictionary_id: 'builtin' });
  await rate(record); await rate(legacy.record);
  const beforeDuplicate = readState(destination);
  const duplicate = await invoke('vocabulary_save', { request: { ...saved.submitted, operation: 'native-80-duplicate', notes: 'Must preserve the original note' }, dictionary_id: 'builtin-core' });
  assert.equal(duplicate.outcome, 'existing'); assert.deepEqual(duplicate.record, record);
  const afterDuplicate = readState(destination);
  assert.equal(afterDuplicate.saves.length, beforeDuplicate.saves.length + 1);
  compareLearning({ ...afterDuplicate, saves: afterDuplicate.saves.filter((item) => item.operation !== 'native-80-duplicate') }, beforeDuplicate);
  await seedSelectedAudio([record.id, legacy.record.id]);
  const before = readState(destination);
  evidence.beforeState = before;
  assert.equal(before.records.length, 2); assert.equal(before.cards.length, 4); assert.equal(before.ratings.length, 2);
  assert.equal(before.selections.length, 2);
  assert.ok(before.selections.every((selection) => selection.current_version && selection.previous_version));
  assert.ok(before.protection.some((version) => version.owned));
  assert.ok(before.protection.some((version) => version.protected_until));
  const files = await Promise.all(before.physical.map(async (version) => {
    const path = join(destination, `Lexique-data/temp/speech/s-${version.physical_locator}-${version.content_hash}.ogg`);
    return { path, bytes: await readFile(path) };
  }));
  if (scenario === 'missing') await rename(coreFile, coreFile + '.unavailable');
  else if (scenario === 'corrupt') { await chmod(coreFile, 0o666); await writeFile(coreFile, 'Synthetic invalid isolated core resource'); }
  else { await chmod(coreFile, 0o666); fixtureSql(destination, `
c.close(); resource=sqlite3.connect(p['resource'])
resource.execute("UPDATE metadata SET value='2' WHERE key='format_version'")
resource.commit(); resource.close()
print(json.dumps(True))`, { resource: coreFile }, true); }
  await start();
  await failedProvider('être', scenario === 'missing' ? 'missing' : 'corrupt');
  await rejected('vocabulary_save', { request: request('bien', 'unavailable-new'), dictionary_id: 'builtin-core' });
  const replay = await invoke('vocabulary_save', { request: submitted, dictionary_id: 'builtin-core' });
  assert.equal(replay.outcome, 'replayed'); assert.deepEqual(replay.record, record);
  const originalReplay = await invoke('vocabulary_save', { request: saved.submitted, dictionary_id: 'builtin-core' });
  assert.equal(originalReplay.outcome, 'replayed'); assert.deepEqual(originalReplay.record, record);
  const retained = readState(destination);
  compareLearning(retained, before); preservedAudio(retained, before);
  const edited = await invoke('vocabulary_save', { request: { ...submitted, operation: 'native-79-unavailable-edit', id: record.id, revision: record.revision,
    notes: 'Note modifiée sans la ressource.' }, dictionary_id: 'builtin-core' });
  assert.equal(edited.outcome, 'updated'); assert.deepEqual(edited.record.digest, record.digest);
  const after = readState(destination);
  assert.deepEqual(after.records, before.records.map((row) => row.id === record.id ? { ...row,
    notes: 'Note modifiée sans la ressource.', revision: row.revision + 1, updated_at: edited.record.updated_at } : row));
  for (const operation of before.saves) assert.deepEqual(after.saves.find((item) => item.operation === operation.operation), operation);
  assert.equal(after.saves.length, before.saves.length + 1);
  compareLearning({ ...after, records: before.records, saves: before.saves }, before); preservedAudio(after, before);
  for (const file of files) assert.deepEqual(await readFile(file.path), file.bytes);
  assert.ok((await invoke('speech_cached', { vocabularyId: record.id })).bytes.length > 0);
  await openRecord(edited.record);
  evidence.checks.push({ check: `${scenario} resource retains core and legacy records, digest indices, source associations, history, operations and current/protected audio`, status: 'passed' });
  evidence.before = before; evidence.after = after;
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
    assert.equal(item.availability, 'available'); assert.ok((await invoke('dictionary_lookup', { key: 'être', dictionary_id: item.id })).entries.length > 0);
  }
  evidence.checks.push({ check: '15 custom dictionary slots, 18 total, capacity rejection and upgrade append preserve order, identities, file bindings and lookup', status: 'passed' });
}

async function copiedProfile() {
  assert.ok(input, 'Expected a closed isolated native profile');
  assert.deepEqual(JSON.parse(await readFile(join(resolve(input), 'cleanup.json'), 'utf8')), { hosts: [], workers: [], helpers: 0 });
  await cp(join(resolve(input), 'Lexique-data'), join(destination, 'Lexique-data'), { recursive: true });
}

async function replacement() {
  const previous = readState(scenario === 'upgrade' ? resolve(input) : destination), record = await invoke('vocabulary_find', { headword: 'être' });
  evidence.beforeState = previous;
  if (scenario === 'limit' || scenario === 'unsupported') {
    await failedProvider(scenario === 'limit' ? 'bien' : 'être', scenario === 'limit' ? 'limit' : 'unsupported');
    await rejected('vocabulary_save', { request: request('bien', 'limit-new'), dictionary_id: 'builtin-core' });
    assert.ok((await invoke('dictionary_lookup', { key: 'école', dictionary_id: 'builtin' })).entries.length > 0);
  } else {
    const current = await invoke('dictionary_lookup', { key: 'être', dictionary_id: 'builtin-core' });
    assert.notEqual(current.source.version, record.reference.version);
    if (scenario === 'upgrade') {
      assert.equal(current.source.version, curation.content_version);
      admitted(current, 'être');
    } else assert.notEqual(senses(current).find((sense) => sense.frozen_row === 2).meaning_zh, record.digest.find((entry) => entry.core?.frozen_row === 2).core.meaning_zh);
    await query('être'); await page.getByRole('button', { name: 'View existing record', exact: true }).click();
    await openRecord(record);
    await expect(page.getByText('The dictionary version has changed. The saved digest keeps its original content.', { exact: true })).toBeVisible();
  }
  assert.deepEqual((await invoke('vocabulary_find', { headword: 'être' })).digest, record.digest);
  const preserved = readState(destination);
  const expected = scenario === 'upgrade' ? {
    ...previous,
    preferences: [...previous.preferences.filter((item) => item.key !== 'interface-preferences'), {
      key: 'interface-preferences',
      value: { locale: 'en', theme: previous.preferences.find((item) => item.key === 'interface-preferences')?.value.theme ?? 'white' },
    }].sort((left, right) => left.key.localeCompare(right.key)),
  } : previous;
  if (scenario === 'upgrade') evidence.fixtureInterfacePreferences = expected.preferences.find((item) => item.key === 'interface-preferences');
  compareLearning(preserved, expected);
  preservedAudio(preserved, previous);
  if (scenario === 'upgrade') assert.ok((await invoke('speech_cached', { vocabularyId: record.id })).bytes.length > 0);
  const committed = previous.saves.find((operation) => operation.record_id === record.id && operation.outcome === 'created');
  assert.equal((await invoke('vocabulary_save', { request: committed.request, dictionary_id: 'builtin-core' })).outcome, 'replayed');
  if (scenario === 'newer') await save('bien', 'newer-content');
  if (scenario === 'upgrade') {
    const existing = await invoke('vocabulary_find', { headword: 'lâche' });
    if (existing) {
      assert.deepEqual(existing.digest.filter((item) => item.core).map((item) => item.core.frozen_row), [1450, 1451]);
      const lookup = await invoke('dictionary_lookup', { key: 'lâche', dictionary_id: 'builtin-core' });
      admitted(lookup, 'lâche');
      assert.deepEqual(senses(lookup).map((sense) => sense.frozen_row), [1450, 1451]);
      assert.deepEqual(await invoke('vocabulary_find', { headword: 'lâche' }), existing);
      evidence.checks.push({ check: 'The existing lâche adjective and noun digests remain frozen after the resource update', status: 'passed' });
    }
    for (const row of [3451, 3950]) {
      const key = curation.groups.find((group) => group.frozen_row === row).headword;
      await save(key, `batch-150-159-upgrade-${key}`);
    }
  }
  await openRecord(record);
  evidence.checks.push({ check: ['newer', 'upgrade'].includes(scenario) ? 'New resource version is visible while previous digest, progress and audio remain frozen' : `Native ${scenario} rejection preserves saved content and committed retry`, status: 'passed' });
}

try {
  if (['no-speech', 'batch', 'legacy'].includes(scenario)) {
    const present = await stat(model).catch((error) => { if (error.code === 'ENOENT') return null; throw error; });
    if (present) await rename(model, model + '.unavailable');
    evidence.speechModel = 'absent in isolated package';
  }
  if (['newer', 'upgrade', 'limit', 'unsupported'].includes(scenario)) await copiedProfile();
  await start();
  if (['pilot', 'no-speech'].includes(scenario)) await pilot();
  else if (scenario === 'batch') await contentBatch();
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
