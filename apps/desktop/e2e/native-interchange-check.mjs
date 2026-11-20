/* global process, window, console, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { cp, readFile, writeFile, rename } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { inspectPackage, readState, seedAudio, compareLearning } from './native-backup-state.mjs';

const [resources, binary, destinationPath, scenario, source] = process.argv.slice(2);
assert.ok(['seed', 'csv', 'markdown', 'zip', 'consent', 'enroll', 'backup'].includes(scenario));
const fixture = await native24Fixture(resources, binary, destinationPath);
const { destination } = fixture;
const evidence = { scenario, checks: [], pickers: [] };
let page;
const invoke = (command, args = {}) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const words = () => invoke('vocabulary_list', { query: '', sort: 'headword-asc' });
const texts = (records) => records.map(({ headword, notes, example_fr, example_zh, more_examples }) => ({ headword, notes,
  examples: [{ fr: example_fr, zh: example_zh }, ...more_examples].filter(({ fr, zh }) => fr || zh).map(({ fr, zh }) => ({ fr, zh })) }));
const request = (operation, headword, notes = '', record) => ({ operation, headword, notes, example_fr: '', example_zh: '', example_sense: null, more_examples: [],
  id: record?.id ?? null, revision: record?.revision ?? null, category_ids: record?.category_ids ?? [] });
function python(code, args = []) {
  const result = spawnSync('python', ['-B', '-X', 'utf8', '-c', code, ...args], { windowsHide: true, encoding: 'utf8', timeout: 10000, maxBuffer: 4 * 1024 * 1024 });
  assert.equal(result.status, 0, result.stderr || String(result.error));
  return result.stdout.trim() ? JSON.parse(result.stdout) : null;
}
function helper(action, filename = '', filenameId = 0) {
  const child = spawn('pwsh', ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', action, '-Title', 'Lexique',
    '-SourcePid', String(fixture.hostPid()), '-SourceExecutable', join(destination, 'Lexique.exe'), '-WaitMs', '4000', '-Text', filename, '-FilenameControlId', String(filenameId)], { windowsHide: true });
  fixture.ownHelper(child);
  let stdout = '', stderr = '';
  child.stdout.on('data', (bytes) => { stdout += bytes.toString(); if (stdout.length > 65536) child.kill(); });
  child.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
  const timer = setTimeout(() => child.kill(), 8000);
  return new Promise((done, reject) => {
    child.once('error', (error) => { clearTimeout(timer); reject(error); });
    child.once('exit', (code) => { clearTimeout(timer); if (code !== 0) reject(new Error(`Owned picker helper: ${code}: ${stderr}`));
      else { try { done(stdout.trim() ? JSON.parse(stdout) : null); } catch (error) { reject(error); } } });
  });
}
async function picker(button, filename, save = false) {
  const observed = helper('picker-controls'); observed.catch(() => {});
  const clicked = page.getByRole('button', { name: button, exact: true }).click({ noWaitAfter: true }); clicked.catch(() => {});
  const controls = await observed, id = save ? 1001 : 1148;
  assert.equal(controls.pid, fixture.hostPid()); assert.equal(controls.class, '#32770');
  assert.equal(controls.controls.filter((control) => control.class === 'Edit' && control.id === id && control.visible && control.enabled).length, 1);
  for (const buttonId of [1, 2]) assert.equal(controls.controls.filter((control) => control.class === 'Button' && control.id === buttonId && control.visible && control.enabled).length, 1);
  const selected = await helper(filename ? 'picker-select' : 'picker-cancel', filename, id);
  await clicked; assert.equal(selected.disappeared, true);
  if (filename) assert.equal(selected.filename, filename);
  evidence.pickers.push({ button, filename, filenameId: id, pid: controls.pid });
}
async function notebook() { await page.getByRole('button', { name: '笔记', exact: true }).click(); }
async function confirm() { await page.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('导入已保存'); }
async function exportCsv(filename) {
  await page.getByRole('button', { name: '导出 CSV', exact: true }).first().click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await dialog.getByLabel('导出范围').selectOption('all');
  await picker('保存 CSV', filename, true); await expect(dialog.getByRole('status').last()).toHaveText('已导出');
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
}
async function seed() {
  const category = await invoke('category_create', { name: 'Interchange retained' });
  const rich = (await invoke('vocabulary_save', { dictionary_id: 'builtin-core', request: { ...request('native-rich', 'école', 'old notes'), category_ids: [category.id] } })).record;
  assert.ok(rich.reference); assert.ok(rich.digest.some((entry) => entry.core));
  const ordinary = (await invoke('vocabulary_save', { request: request('native-ordinary', 'or') })).record;
  assert.ok(ordinary.reference); assert.equal(ordinary.notes, '');
  const externalFile = join(destination, 'existing-external.md');
  await writeFile(externalFile, '# Lexique note v1\n## headword\n```text\nnative-unlisted\n```\n');
  await notebook(); await picker('导入 Markdown', externalFile); await confirm();
  const external = (await words()).find((record) => record.headword === 'native-unlisted');
  assert.equal(external.reference, null); assert.deepEqual(external.digest, []);
  assert.deepEqual(await invoke('review_cards', { vocabularyId: external.id }), []);
  const grammar = (await invoke('grammar_save', { request: { operation: 'native-grammar', id: null, revision: null, title: 'Une règle', body: '中文\nUne règle.', category_ids: [category.id] } })).record;
  const grammarCard = (await invoke('grammar_card_save', { request: { operation: 'native-grammar-card', id: grammar.id, revision: grammar.revision, question: 'Pourquoi ?', answer: 'Une réponse.' } })).record.card;
  const cards = await invoke('review_add', { vocabularyId: rich.id });
  const recognition = cards.find((card) => card.kind === 'recognition');
  await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  await invoke('review_rate', { request: { operation: 'native-rich-rating', card_id: recognition.id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  await invoke('review_rate', { request: { operation: 'native-grammar-rating', card_id: grammarCard.id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
  await invoke('review_pause', { vocabularyId: rich.id, paused: true });
  await invoke('review_auto_enroll', { enabled: true });
  evidence.records = await words(); await fixture.stop(true);
  evidence.audio = seedAudio(destination, [rich.id, ordinary.id], 'A', resolve('../../docs/evidence/issue-56'));
  evidence.checks.push('real linked rich and empty ordinary notes, grammar, category, independent progress, pause, auto-enroll and selected audio fixture');
}
async function csv() {
  const before = readState(destination), original = await words();
  const rows = [['mot', 'memo', 'francais', 'chinois', 'ignored'], ['école', '  中文; "quoted"\r\n`````\n', ' À l’école.\n', ' 中文 ', 'x'],
    ['native-unlisted', '\nexternal\n', '', '单侧', 'x'], ['aller', '', '', '', 'x'], ['or', '', '', '', 'x'], ['invalid', 'x'.repeat(20001), '', '', 'x']];
  const filename = join(destination, 'manual-semicolon.csv');
  python("import csv,json,sys\nwith open(sys.argv[1],'w',encoding='utf-8-sig',newline='') as f: csv.writer(f,delimiter=';').writerows(json.loads(sys.argv[2]))", [filename, JSON.stringify(rows)]);
  await notebook(); await page.getByLabel('分隔符', { exact: true }).selectOption('semicolon'); await picker('导入 CSV', filename);
  await expect(page.getByRole('alert')).toContainText('Map the required headword');
  for (const [field, value] of [['headword', '0'], ['notes', '1'], ['example_fr', '2'], ['example_zh', '3']]) await page.getByLabel(field, { exact: true }).selectOption(value);
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await page.getByLabel('导入决定：école', { exact: true }).selectOption('update');
  await page.getByLabel('导入决定：native-unlisted', { exact: true }).selectOption('update');
  await page.getByLabel('排除无效组：invalid', { exact: true }).check();
  await expect(page.getByRole('region', { name: 'CSV 导入预览', exact: true })).toContainText('内容相同，将跳过');
  await confirm();
  const after = await words(), rich = after.find((record) => record.headword === 'école'), old = original.find((record) => record.id === rich.id);
  for (const field of ['id', 'reference', 'digest', 'category_ids', 'created_at']) assert.deepEqual(rich[field], old[field]);
  assert.equal(rich.notes, rows[1][1]); assert.equal(rich.example_fr, rows[1][2]); assert.equal(rich.example_zh, rows[1][3]);
  const external = after.find((record) => record.headword === 'native-unlisted');
  assert.equal(external.reference, null); assert.deepEqual(external.digest, []);
  assert.deepEqual(external.import_source, original.find((record) => record.id === external.id).import_source);
  assert.equal(after.length, 4); assert.ok(after.find((record) => record.headword === 'aller').reference);
  assert.deepEqual(after.find((record) => record.headword === 'aller').import_source, { format: 'csv', filename: 'manual-semicolon.csv' });
  assert.equal(after.find((record) => record.headword === 'or').revision, original.find((record) => record.headword === 'or').revision);
  const saved = readState(destination);
  for (const field of ['cards', 'progress', 'ratings', 'paused', 'selections', 'categories', 'record_categories', 'grammar_cards']) assert.deepEqual(saved[field], before[field], field);
  assert.deepEqual(saved.tasks.map((task) => task.id), before.tasks.map((task) => task.id));
  assert.deepEqual(saved.owners, before.owners);
  const exportPath = join(destination, 'roundtrip-start.csv'); await exportCsv(exportPath);
  evidence.text = texts(after); evidence.initialCsv = python("import csv,json,sys\nwith open(sys.argv[1],encoding='utf-8-sig',newline='') as f: print(json.dumps(list(csv.DictReader(f)),ensure_ascii=False))", [exportPath]);
  await fixture.stop(true); page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  assert.deepEqual(await words(), after); evidence.checks.push('real semicolon picker, manual mapping, mixed atomic save, invalid exclusion, exact text, no new cards, restart and retained learning state');
}
async function markdown() {
  const before = await words(); await notebook();
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('native-unlisted');
  await expect(page.locator('.vocabulary-list button')).toHaveCount(1); await page.locator('.vocabulary-list button').click();
  await page.locator('.vocabulary-editor').getByRole('button', { name: '导出 Markdown', exact: true }).click();
  const filename = join(destination, 'single.md'); await picker('保存 Markdown', filename, true);
  await page.getByRole('dialog', { name: '导出笔记' }).getByRole('button', { name: '关闭', exact: true }).click();
  await picker('导入 Markdown', filename);
  await expect(page.getByRole('region', { name: 'Markdown 导入预览', exact: true })).toContainText('内容相同，将跳过'); await confirm();
  assert.deepEqual(await words(), before); assert.ok((await readFile(filename, 'utf8')).includes('\nexternal\n'));
  evidence.checks.push('native single Markdown export/import preserves exact text and skips identical content');
}
async function zip() {
  const before = await words(); await notebook();
  const all = join(destination, 'all.zip'), search = join(destination, 'search.zip');
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('native-unlisted');
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog).toContainText('1 条生词'); await picker('保存 Markdown ZIP', search, true);
  await dialog.getByLabel('导出范围').selectOption('all'); await expect(dialog).toContainText('4 条生词'); await picker('保存 Markdown ZIP', all, true);
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  const entries = python("import zipfile,json,sys\nprint(json.dumps([{'name':i.filename,'text':z.read(i).decode('utf-8')} for z in [zipfile.ZipFile(sys.argv[1])] for i in z.infolist()],ensure_ascii=False))", [all]);
  assert.equal(entries.length, 4); assert.equal(new Set(entries.map((entry) => entry.name.toLowerCase())).size, 4);
  assert.equal(python("import zipfile,json,sys\nprint(json.dumps(zipfile.ZipFile(sys.argv[1]).namelist()))", [search]).length, 1);
  const mixed = join(destination, 'edited.zip');
  python("import zipfile,json,sys\nentries=json.loads(sys.argv[2])\nwith zipfile.ZipFile(sys.argv[1],'w',compression=zipfile.ZIP_DEFLATED) as z:\n for i,e in enumerate(entries): z.writestr('../../body-'+str(i)+'.md',e['text'])\n z.writestr('nested/duplicate.md',entries[0]['text'])\n z.writestr('readme.txt','not a template')", [mixed, JSON.stringify(entries)]);
  await picker('导入 Markdown ZIP', mixed);
  await expect(page.getByRole('region', { name: 'Markdown ZIP 导入预览', exact: true })).toContainText('../../body-0.md');
  await page.getByLabel('排除无效组：readme.txt', { exact: true }).check(); await confirm();
  assert.deepEqual(await words(), before);
  const csvPath = join(destination, 'roundtrip-end.csv'); await exportCsv(csvPath);
  const actual = python("import csv,json,sys\nwith open(sys.argv[1],encoding='utf-8-sig',newline='') as f: print(json.dumps(list(csv.DictReader(f)),ensure_ascii=False))", [csvPath]);
  const prior = JSON.parse(await readFile(join(resolve(source), 'results.json'), 'utf8')); assert.deepEqual(actual, prior.initialCsv);
  const corrupt = join(destination, 'broken.zip'); await writeFile(corrupt, (await readFile(all)).subarray(0, 30));
  await picker('导入 Markdown ZIP', corrupt); await expect(page.getByRole('alert')).toContainText('invalid ZIP container');
  assert.deepEqual(await words(), before); await picker('导入 Markdown ZIP', '');
  evidence.checks.push('all/search ZIP counts, unique names, deflated nested paths read directly, dedupe, invalid entry exclusion, corrupt container, picker cancellation and CSV/Markdown/ZIP/CSV exact text');
}
async function consent() {
  const learning = readState(destination);
  await rename(join(destination, 'dictionary/dictionary.sqlite3'), join(destination, 'dictionary/withheld.sqlite3'));
  const filename = join(destination, 'resource-error.zip');
  python("import zipfile,sys\nwith zipfile.ZipFile(sys.argv[1],'w') as z: z.writestr('new.md','# Lexique note v1\\n## headword\\n```text\\nnative-resource-error\\n```\\n')", [filename]);
  const before = await words(); await notebook(); await picker('导入 Markdown ZIP', filename);
  const button = page.getByRole('button', { name: '确认导入', exact: true }); await expect(button).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check(); await expect(button).toBeEnabled();
  await page.getByRole('button', { name: '取消导入', exact: true }).click(); assert.deepEqual(await words(), before);
  await picker('导入 Markdown ZIP', filename); await expect(button).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check(); await confirm();
  const added = (await words()).find((record) => record.headword === 'native-resource-error'); assert.equal(added.reference, null);
  assert.deepEqual(added.import_source, { format: 'zip', filename: 'resource-error.zip' });
  const saved = readState(destination);
  for (const field of ['cards', 'progress', 'ratings', 'paused', 'selections']) assert.deepEqual(saved[field], learning[field], field);
  assert.deepEqual(saved.tasks.map((task) => task.id), learning.tasks.map((task) => task.id));
  evidence.checks.push('real missing resource, explicit unlinked consent, cancellation without writes and consent reset');
}
async function enroll() {
  await notebook(); const records = await words(), external = records.find((record) => record.headword === 'native-unlisted');
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill(external.headword); await expect(page.locator('.vocabulary-list button')).toHaveCount(1);
  await page.locator('.vocabulary-list button').click(); await page.getByLabel('独立笔记').fill('edited after restart');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect.poll(() => invoke('review_cards', { vocabularyId: external.id })).toHaveLength(2);
  const cards = await invoke('review_cards', { vocabularyId: external.id }); const before = readState(destination);
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('');
  await expect(page.locator('.vocabulary-list button')).toHaveCount(records.length);
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByRole('button', { name: '批量加入复习', exact: true }).click();
  await expect.poll(async () => (await invoke('review_batch_status', { operation: null }))?.items.every((item) => item.created_cards !== null || item.error !== null)).toBe(true);
  for (const record of records) assert.deepEqual((await invoke('review_cards', { vocabularyId: record.id })).map((card) => card.kind).sort(), ['listening', 'recognition']);
  assert.deepEqual(await invoke('review_cards', { vocabularyId: external.id }), cards);
  const rich = records.find((record) => record.headword === 'école'); assert.equal(await invoke('review_pause', { vocabularyId: rich.id, paused: null }), true);
  const after = readState(destination); assert.deepEqual(after.progress.filter((p) => before.progress.some((old) => old.card_id === p.card_id)), before.progress);
  const audio = await invoke('speech_cached', { vocabularyId: rich.id }); assert.ok(audio.bytes.length > 100); evidence.audioBytes = audio.bytes.length;
  evidence.checks.push('external edit after restart, explicit single and batch dual-card enrollment, duplicate identities, retained progress/pause and real selected audio bytes');
}
async function backup() {
  const original = await words();
  await page.getByRole('banner').getByRole('combobox').first().selectOption('en'); await page.getByRole('button', { name: 'Settings', exact: true }).click();
  // Restore materializes default preference rows; persist the native baseline before comparing SQLite.
  for (const [command, argument] of [['review_settings', 'settings'], ['speech_preferences', 'preferences'], ['conjugation_preferences', 'preferences'], ['conjugation_block_order', 'order'], ['vocabulary_sort', 'sort'], ['grammar_sort', 'sort']]) {
    const value = await invoke(command, { [argument]: null }); await invoke(command, { [argument]: value });
  }
  const catalog = await invoke('dictionary_settings', { order: null });
  await invoke('dictionary_settings', { order: catalog.map((entry) => entry.id) });
  const before = readState(destination);
  const rich = original.find((record) => record.headword === 'école');
  const audio = await invoke('speech_cached', { vocabularyId: rich.id });
  await page.evaluate(async () => {
    window.interchangeBackupEvents = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress']) await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
      handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.interchangeBackupEvents.push({ event, payload: value.payload }), false) });
  });
  const latest = (event) => page.evaluate((event) => window.interchangeBackupEvents.filter((value) => value.event === event).at(-1)?.payload, event);
  const filename = join(destination, 'interchange.lexique-backup'); await picker('Create backup', filename, true);
  await expect.poll(async () => (await latest('lexique-backup-progress'))?.state).toBe('completed');
  const exported = await latest('lexique-backup-progress'), archive = inspectPackage(destination, filename);
  assert.equal(archive.snapshot.snapshot_version, 4); assert.ok(archive.snapshot.records.some((record) => record.import_source?.format === 'csv'));
  assert.ok(archive.snapshot.records.some((record) => record.import_source?.format === 'zip'));
  assert.ok(archive.snapshot.grammar_records.length); assert.ok(archive.entries.some((name) => name.startsWith('audio/')));
  const extra = (await invoke('grammar_save', { request: { operation: 'native-after-backup', id: null, revision: null, title: 'After backup', body: '', category_ids: [] } })).record;
  await picker('Choose backup', filename); await expect.poll(async () => (await latest('lexique-backup-preview-progress'))?.state).toBe('preview_ready');
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore' }); await dialog.getByRole('checkbox').check();
  page = await fixture.followRestart(() => dialog.getByRole('button', { name: 'Confirm and restore', exact: true }).click()); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await expect.poll(async () => (await invoke('startup_status')).state).toBe('ready'); assert.deepEqual(await words(), original);
  assert.ok(!(await invoke('grammar_list', { query: '', sort: 'created-asc' })).some((record) => record.id === extra.id));
  const after = readState(destination); compareLearning(after, before);
  assert.deepEqual((await invoke('speech_cached', { vocabularyId: rich.id })).bytes, audio.bytes);
  evidence.counts = exported.result.counts; evidence.checks.push('complete real backup, external provenance, ordinary/rich and grammar notes, categories, progress, selected audio and controlled restore/restart');
}
try {
  for (const name of ['conjugation', 'frequency']) await cp(join(resolve(resources), name), join(destination, name), { recursive: true });
  if (source) {
    const owned = relative(resolve('../../target/issue-189'), resolve(source)); assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
    await cp(join(resolve(source), 'Lexique-data'), join(destination, 'Lexique-data'), { recursive: true, filter: (path) => !path.includes('webview') });
    evidence.initialCsv = JSON.parse(await readFile(join(resolve(source), 'results.json'), 'utf8')).initialCsv;
  }
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true); await expect(page.getByRole('banner')).toBeVisible();
  await ({ seed, csv, markdown, zip, consent, enroll, backup })[scenario]();
  if (scenario !== 'seed') { evidence.records = await words(); await page.screenshot({ path: join(destination, 'native-interchange.png') }); await fixture.stop(true); }
  evidence.state = readState(destination);
  await writeFile(join(destination, 'results.json'), JSON.stringify(evidence, null, 2)); console.log(JSON.stringify({ scenario, passed: true, checks: evidence.checks.length }));
} catch (error) {
  await writeFile(join(destination, 'failure.json'), JSON.stringify({ ...evidence, error: String(error), stack: error.stack }, null, 2));
  if (page && !page.isClosed()) {
    await writeFile(join(destination, 'failure.html'), await page.content()).catch(() => {});
    await page.screenshot({ path: join(destination, 'failure.png') }).catch(() => {});
  }
  throw error;
} finally { await fixture.finish(); }
