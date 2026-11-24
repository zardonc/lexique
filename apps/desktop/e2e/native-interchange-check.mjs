/* global process, window, console, setTimeout, clearTimeout */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { cp, readFile, writeFile, rename } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { inspectPackage, readState, seedAudio, compareLearning, fixtureSql } from './native-backup-state.mjs';
import { grammarImportCheck } from './native-grammar-import.mjs';

const [resources, binary, destinationPath, scenario, source] = process.argv.slice(2);
const grammarImportScenarios = ['grammar-markdown', 'grammar-markdown-restart', 'grammar-markdown-validation', 'grammar-batch-mapping', 'grammar-batch-learning', 'grammar-batch-zip-learning', 'grammar-batch-stale', 'grammar-batch-occupied', 'grammar-batch-failure', 'grammar-batch-zip-failure', 'grammar-batch-replay', 'grammar-batch-restart', 'grammar-batch-selection', 'grammar-import', 'grammar-import-restart', 'grammar-import-identifiers', 'grammar-import-validation', 'grammar-import-failure', 'grammar-import-replay', 'grammar-import-known'];
assert.ok(['seed', 'csv', 'markdown', 'zip', 'consent', 'enroll', 'backup', 'device-names', 'grammar-seed', 'grammar-csv', 'grammar-backup', 'grammar-legacy', 'grammar-vocabulary', 'grammar-vocabulary-markdown', 'grammar-zip-source', 'grammar-zip-destination', 'grammar-zip-backup', 'grammar-zip-invalid', 'grammar-zip-reopen', 'grammar-zip-paths', ...grammarImportScenarios].includes(scenario));
const fixture = await native24Fixture(resources, binary, destinationPath);
const { destination } = fixture;
const revision = spawnSync('git', ['rev-parse', 'HEAD', 'HEAD^{tree}'], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
const status = spawnSync('git', ['status', '--porcelain'], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
assert.equal(revision.status, 0, revision.stderr); assert.equal(status.status, 0, status.stderr);
const [sourceSha, sourceTree] = revision.stdout.trim().split(/\r?\n/);
await writeFile(join(destination, 'run-metadata.json'), JSON.stringify({ ...JSON.parse(await readFile(join(destination, 'run-metadata.json'), 'utf8')),
  sourceSha, sourceTree, dirty: Boolean(status.stdout.trim()), platform: process.platform, architecture: process.arch }, null, 2));
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
  evidence.pickers.push({ button, filename, filenameId: id, pid: controls.pid, defaultFilename: selected.defaultFilename });
  return selected;
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
async function deviceNames() {
  const headwords = ['CON.txt', 'con.TXT', 'COM1.txt', 'lPt9.data'];
  const entries = headwords.map((word) => ({ text: `# Lexique note v1\n## headword\n\`\`\`text\n${word}\n\`\`\`\n## notes\n\`\`\`text\n  中文 \`notes\`\r\n尾行\n\`\`\`\n## examples\n### 1\n#### fr\n\`\`\`text\n À l’école.\r\n\`\`\`\n#### zh\n\`\`\`text\n 单侧\n\`\`\`\n` }));
  const input = join(destination, 'device-input.zip');
  python("import zipfile,json,sys\nwith zipfile.ZipFile(sys.argv[1],'w') as z:\n for i,e in enumerate(json.loads(sys.argv[2])): z.writestr(str(i)+'.md',e['text'])", [input, JSON.stringify(entries)]);
  await notebook(); await picker('导入 Markdown ZIP', input); await confirm();
  await page.getByLabel('排序方式').selectOption('headword-asc');
  const before = await words(); assert.equal(before.length, 4);
  const search = page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' });
  const filenames = before.map((record) => `_${record.headword}-${record.id}.md`);
  for (const word of headwords.slice(0, 2)) {
    const record = before.find((item) => item.headword === word);
    await search.fill(word);
    await page.locator('.vocabulary-list button').filter({ has: page.getByText(word, { exact: true }) }).click();
    await page.locator('.vocabulary-editor').getByRole('button', { name: '导出 Markdown', exact: true }).click();
    const filename = `_${word}-${record.id}.md`, file = join(destination, filename);
    const selected = await picker('保存 Markdown', file, true); assert.equal(selected.defaultFilename, filename);
    const text = await readFile(file, 'utf8'); assert.ok(text.includes(`\n${word}\n`));
    assert.ok(text.includes('  中文 `notes`\r\n尾行')); assert.ok(text.includes(' À l’école.\r\n'));
    await page.getByRole('dialog', { name: '导出笔记' }).getByRole('button', { name: '关闭', exact: true }).click();
  }
  await search.fill('COM1.txt');
  await page.getByRole('button', { name: '导出 CSV', exact: true }).first().click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' }), all = join(destination, 'devices.zip'), filtered = join(destination, 'filtered.zip');
  await expect(dialog).toContainText('1 条生词'); await picker('保存 Markdown ZIP', filtered, true);
  await dialog.getByLabel('导出范围').selectOption('all'); await expect(dialog).toContainText('4 条生词');
  await picker('保存 Markdown ZIP', all, true); await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  const output = python("import zipfile,json,sys,pathlib\nwith zipfile.ZipFile(sys.argv[1]) as z:\n z.extractall(pathlib.Path(sys.argv[1]).parent/'extracted')\n print(json.dumps([{'name':i.filename,'text':z.read(i).decode('utf-8')} for i in z.infolist()],ensure_ascii=False))", [all]);
  assert.deepEqual(output.map((entry) => entry.name), filenames);
  assert.equal(new Set(filenames.map((name) => name.toLowerCase())).size, 4);
  for (const record of before) {
    const text = output.find((entry) => entry.name === `_${record.headword}-${record.id}.md`).text;
    assert.ok(text.includes(`\n${record.headword}\n`)); assert.ok(text.includes(record.notes));
    assert.equal(await readFile(join(destination, 'extracted', `_${record.headword}-${record.id}.md`), 'utf8'), text);
  }
  assert.equal(python("import zipfile,json,sys\nprint(json.dumps(zipfile.ZipFile(sys.argv[1]).namelist()))", [filtered]).length, 1);
  await picker('导入 Markdown ZIP', all); await expect(page.getByRole('region', { name: 'Markdown ZIP 导入预览', exact: true })).toContainText('内容相同，将跳过');
  await confirm(); assert.deepEqual(await words(), before);
  evidence.checks.push('observed default Windows Markdown names and actual saved files for dotted device case variants', 'ZIP extraction on Windows, exact body identity/text, unique names, all/search counts/order and unchanged saved records on reimport');
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
  assert.equal(archive.snapshot.snapshot_version, 5); assert.ok(archive.snapshot.records.some((record) => record.import_source?.format === 'csv'));
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

const grammars = () => invoke('grammar_list', { query: '', sort: 'created-asc' });
async function grammarSeed() {
  const category = await invoke('category_create', { name: 'Private grammar category' });
  const records = [];
  for (let index = 0; index < 3; index++) {
    records.push((await invoke('grammar_save', { request: { operation: `grammar-seed-${index}`, id: null, revision: null,
      title: 'Une règle', body: '  été, "中文"\r\nencore\n  ', category_ids: index === 0 ? [category.id] : [] } })).record);
  }
  const identities = readState(destination).grammar_exchange_identities;
  const renamed = records[2];
  records[2] = (await invoke('grammar_save', { request: { operation: 'grammar-renamed', id: renamed.id, revision: renamed.revision,
    title: 'Autre règle', body: renamed.body, category_ids: renamed.category_ids } })).record;
  assert.deepEqual(readState(destination).grammar_exchange_identities, identities);
  for (const [index, record] of records.slice(0, 2).entries()) {
    records[index] = (await invoke('grammar_card_save', { request: { operation: `grammar-card-${index}`, id: record.id, revision: record.revision,
      question: '  Pourquoi, "中文" ?\n  ', answer: '  Réponse indépendante\r\n  ' } })).record;
  }
  await invoke('review_queue', { timezone: 'UTC', skipped: [] });
  await invoke('review_rate', { request: { operation: 'grammar-rating', card_id: records[0].card.id, expected_revision: 0, rating: 'easy', timezone: 'UTC' } });
  await invoke('review_pause', { vocabularyId: records[0].id, paused: true });
  await invoke('vocabulary_save', { request: request('grammar-vocabulary', 'or', 'Preserved vocabulary') });
  evidence.checks.push('independent duplicate-title/content grammar records, edited paired cards, category, learned progress/history, paused and active cards, vocabulary');
}

function grammarRows(filename) {
  return python("import csv,json,pathlib,sys\np=pathlib.Path(sys.argv[1]); b=p.read_bytes(); assert b[:3]==b'\\xef\\xbb\\xbf'\nwith p.open(encoding='utf-8-sig',newline='') as f:\n r=csv.DictReader(f); assert r.fieldnames==['exchange_id','title','body','question','answer']; print(json.dumps(list(r),ensure_ascii=False))", [filename]);
}

async function grammarCsv() {
  const records = await grammars(), before = readState(destination);
  assert.equal(before.grammar_exchange_identities.length, 3);
  assert.equal(new Set(before.grammar_exchange_identities.map((item) => item.exchange_id)).size, 3);
  if (evidence.beforeMigration) {
    const { grammar_exchange_identities: ignoredIdentities, ...previous } = evidence.beforeMigration;
    const { grammar_exchange_identities: newIdentities, ...migrated } = before;
    assert.equal(ignoredIdentities.length, 3); assert.equal(newIdentities.length, 3);
    assert.deepEqual(migrated, previous);
  }
  await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByLabel('排序方式').selectOption('headword-desc');
  await page.getByRole('searchbox', { name: '搜索语法标题或正文', exact: true }).fill('une regle');
  await expect(page.locator('.vocabulary-list li')).toHaveCount(2);
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await expect(dialog).toContainText('2 条语法记录');
  await picker('保存 CSV', '', true); await expect(dialog).toContainText('已取消');
  const filename = join(destination, 'grammar-search.csv');
  await picker('保存 CSV', filename, true); await expect(dialog.getByRole('status').last()).toHaveText('已导出');
  const rows = grammarRows(filename);
  assert.deepEqual(rows.map((row) => row.exchange_id), before.grammar_exchange_identities.slice(0, 2).reverse().map((item) => item.exchange_id));
  assert.equal(rows[0].body, records[0].body); assert.equal(rows[0].question, records[1].card.question); assert.equal(rows[0].answer, records[1].card.answer);
  await dialog.getByLabel('导出范围').selectOption('all'); await expect(dialog).toContainText('3 条语法记录');
  const allFile = join(destination, 'grammar-all.csv'); await picker('保存 CSV', allFile, true);
  const all = grammarRows(allFile); assert.equal(all.length, 3); assert.equal(all.filter((row) => !row.question && !row.answer).length, 1);
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  await page.locator('.vocabulary-list button').first().click();
  const selected = records[1];
  await page.getByLabel('语法标题', { exact: true }).fill('Unsaved title');
  await page.getByLabel('语法正文', { exact: true }).fill('Unsaved body');
  await page.locator('.grammar-editor').getByRole('button', { name: '导出 CSV', exact: true }).click();
  const singleFile = join(destination, 'grammar-single.csv'); await picker('保存 CSV', singleFile, true);
  const single = grammarRows(singleFile); assert.equal(single.length, 1); assert.equal(single[0].title, selected.title); assert.equal(single[0].body, selected.body);
  assert.equal(single[0].exchange_id, rows[0].exchange_id);
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  compareLearning(readState(destination), { ...before, preferences: readState(destination).preferences });
  evidence.csv = { rows, all, single }; evidence.checks.push('T01–T05/T08/T37: real schema-20 migration, all/search/single saved CSV, counts/order, raw paired text, absent cards, unsaved drafts excluded, cancellation then save');
}

async function grammarZipSource() {
  const input = join(destination, 'incoming.csv');
  const rows = [['exchange_id', 'title', 'body', 'question', 'answer'],
    ['zip-217-one', 'Même titre', 'été 中文', 'Pourquoi ?', 'Parce que'],
    ['zip-217-two', 'Même titre', 'Deuxième', 'Question', 'Réponse'],
    ['zip-217-three', 'Autre', 'Troisième', '', '']];
  python("import csv,json,sys\nwith open(sys.argv[1],'w',encoding='utf-8-sig',newline='') as f: csv.writer(f).writerows(json.loads(sys.argv[2]))", [input, JSON.stringify(rows)]);
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  await picker('导入语法 CSV', input);
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('导入已保存');
  const state = readState(destination);
  assert.deepEqual(state.grammar_exchange_identities.map((row) => row.exchange_id).sort(), ['zip-217-one', 'zip-217-three', 'zip-217-two']);
  const paused = fixtureSql(destination, "print(json.dumps([r[0] for r in c.execute('SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id')]))", {}, true);
  assert.equal(paused.length, 2);
  await page.getByRole('searchbox', { name: '搜索语法标题或正文', exact: true }).fill('Autre');
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await expect(dialog).toContainText('1 条语法记录');
  await picker('保存 Markdown ZIP', '', true); await expect(dialog).toContainText('已取消');
  const searchZip = join(destination, 'grammar-search.zip');
  await picker('保存 Markdown ZIP', searchZip, true);
  const searchNames = python("import json,sys,zipfile\nwith zipfile.ZipFile(sys.argv[1]) as z: print(json.dumps(z.namelist()))", [searchZip]);
  assert.equal(searchNames.length, 1);
  await dialog.getByLabel('导出范围').selectOption('all');
  const zip = join(destination, 'grammar-all.zip');
  await picker('保存 Markdown ZIP', zip, true); await expect(dialog.getByRole('status').last()).toHaveText('已导出');
  const members = python("import json,sys,zipfile\nwith zipfile.ZipFile(sys.argv[1]) as z: print(json.dumps([{'name':n,'content':z.read(n).decode('utf-8')} for n in z.namelist()],ensure_ascii=False))", [zip]);
  assert.equal(members.length, 3); assert.equal(new Set(members.map((entry) => entry.name.toLowerCase())).size, 3);
  for (const id of ['zip-217-one', 'zip-217-two', 'zip-217-three']) assert.equal(members.filter((entry) => entry.content.includes(id)).length, 1);
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  evidence.checks.push('CSV import, SQLite exchange identities and paused cards, saved ZIP export with three distinct members');
}

async function grammarZipDestination() {
  const zip = resolve(process.env.LEXIQUE_TICKET_217_ZIP || '');
  const owned = relative(resolve('../../target/ticket-217'), zip);
  assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
  assert.equal((await grammars()).length, 0);
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  await picker('导入语法 Markdown ZIP', zip);
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('导入已保存');
  const identities = readState(destination).grammar_exchange_identities.map((row) => row.exchange_id).sort();
  assert.deepEqual(identities, ['zip-217-one', 'zip-217-three', 'zip-217-two']);
  const paused = fixtureSql(destination, "print(json.dumps([r[0] for r in c.execute('SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id')]))", {}, true);
  assert.equal(paused.length, 2);
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await dialog.getByLabel('导出范围').selectOption('all');
  const output = join(destination, 'roundtrip.csv'); await picker('保存 CSV', output, true);
  const rows = grammarRows(output); assert.deepEqual(rows.map((row) => row.exchange_id).sort(), identities);
  assert.equal(rows.find((row) => row.exchange_id === 'zip-217-one').body, 'été 中文');
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  evidence.checks.push('independent empty library, ZIP import, actual SQLite identity and paused-card readback, real CSV export');
}

async function grammarZipInvalid() {
  const input = resolve(process.env.LEXIQUE_TICKET_217_ZIP || '');
  const owned = relative(resolve('../../target/ticket-217'), input);
  assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
  const mixed = join(destination, 'mixed.zip');
  python("import sys,zipfile\nwith zipfile.ZipFile(sys.argv[1]) as src, zipfile.ZipFile(sys.argv[2],'w') as out:\n for name in src.namelist():\n  data=src.read(name); out.writestr(name,data)\n  if b'zip-217-one' in data: out.writestr('conflict.md',data.replace('Même titre'.encode(),b'Changed title'))\n out.writestr('vocabulary.md','# Lexique note v1\\n## headword\\n```text\\nmot\\n```\\n')", [input, mixed]);
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  await picker('导入语法 Markdown ZIP', mixed);
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await expect(region).toContainText('conflict.md'); await expect(region).toContainText('vocabulary.md');
  await region.getByRole('checkbox', { name: '排除无效组：Même titre' }).check();
  await region.getByRole('checkbox', { name: '排除无效组：vocabulary.md' }).check();
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('导入已保存');
  assert.deepEqual(readState(destination).grammar_exchange_identities.map((row) => row.exchange_id).sort(), ['zip-217-three', 'zip-217-two']);
  evidence.checks.push('conflicting identity group and vocabulary template located and excluded while valid records commit');
}

async function grammarZipReopen() {
  const saved = await grammars();
  assert.equal(saved.length, 3);
  assert.deepEqual(readState(destination).grammar_exchange_identities.map((row) => row.exchange_id).sort(), ['zip-217-one', 'zip-217-three', 'zip-217-two']);
  assert.deepEqual((await invoke('review_queue', { timezone: 'UTC', skipped: [] })).queue.cards, []);
  const paused = fixtureSql(destination, "print(json.dumps([r[0] for r in c.execute('SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id')]))", {}, true);
  assert.equal(paused.length, 2);
  evidence.checks.push('packaged application reopened copied library; three identities and two paused cards persist, with empty review queue');
}

async function grammarZipPaths() {
  const input = resolve(process.env.LEXIQUE_TICKET_217_ZIP || '');
  const owned = relative(resolve('../../target/ticket-217'), input);
  assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
  const zip = join(destination, 'renamed.zip');
  python("import sys,zipfile\nwith zipfile.ZipFile(sys.argv[1]) as src, zipfile.ZipFile(sys.argv[2],'w') as out:\n for i,name in enumerate(src.namelist()):\n  data=src.read(name); out.writestr('../renamed-'+str(i)+'.md',data)\n  if b'zip-217-one' in data: out.writestr('nested/duplicate.md',data)", [input, zip]);
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  const before = readState(destination);
  await picker('导入语法 Markdown ZIP', zip);
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  compareLearning(readState(destination), before);
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('导入已保存');
  assert.deepEqual(readState(destination).grammar_exchange_identities.map((row) => row.exchange_id).sort(), ['zip-217-one', 'zip-217-three', 'zip-217-two']);
  evidence.checks.push('renamed traversal and nested archive paths use content identities; identical duplicate coalesces to three records');
}

async function grammarBackup() {
  await invoke('interface_preferences', { preferences: { locale: 'en', theme: 'white' } });
  await page.reload(); await expect(page.getByRole('banner')).toBeVisible();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await invoke('review_auto_enroll', { enabled: false });
  for (const [command, argument] of [['review_settings', 'settings'], ['speech_preferences', 'preferences'], ['conjugation_preferences', 'preferences'], ['conjugation_block_order', 'order'], ['vocabulary_sort', 'sort'], ['grammar_sort', 'sort']]) {
    const value = await invoke(command, { [argument]: null }); await invoke(command, { [argument]: value });
  }
  const catalog = await invoke('dictionary_settings', { order: null });
  await invoke('dictionary_settings', { order: catalog.map((entry) => entry.id) });
  await page.evaluate(async () => {
    window.grammarBackupEvents = [];
    for (const event of ['lexique-backup-progress', 'lexique-backup-preview-progress']) await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event, target: { kind: 'Any' },
      handler: window.__TAURI_INTERNALS__.transformCallback((value) => window.grammarBackupEvents.push({ event, payload: value.payload }), false) });
  });
  const latest = (event) => page.evaluate((event) => window.grammarBackupEvents.filter((value) => value.event === event).at(-1)?.payload, event);
  const before = readState(destination), original = await grammars();
  const filename = join(destination, 'grammar.lexique-backup');
  await picker('Create backup', filename, true);
  await expect.poll(async () => (await latest('lexique-backup-progress'))?.state).toBe('completed');
  const archive = inspectPackage(destination, filename); assert.equal(archive.snapshot.snapshot_version, 5);
  assert.deepEqual(archive.snapshot.grammar_exchange_identities, before.grammar_exchange_identities);
  let restoreFile = filename;
  if (scenario === 'grammar-legacy') {
    restoreFile = join(destination, 'legacy-v4.lexique-backup');
    python("import zipfile,json,sys\nwith zipfile.ZipFile(sys.argv[1]) as src, zipfile.ZipFile(sys.argv[2],'w',compression=zipfile.ZIP_STORED) as out:\n for name in src.namelist():\n  data=src.read(name)\n  if name in ['manifest.json','snapshot.json']:\n   value=json.loads(data); value['snapshot_version']=4; value.pop('grammar_exchange_identities',None); data=json.dumps(value,ensure_ascii=False).encode()\n  out.writestr(name,data)", [filename, restoreFile]);
  }
  await invoke('grammar_save', { request: { operation: 'after-grammar-backup', id: null, revision: null, title: 'After backup', body: '', category_ids: [] } });
  await picker('Choose backup', restoreFile);
  await expect.poll(async () => (await latest('lexique-backup-preview-progress'))?.state).toBe('preview_ready');
  await page.getByRole('button', { name: 'Restore the whole backup', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Confirm whole-backup restore' }); await dialog.getByRole('checkbox').check();
  page = await fixture.followRestart(() => dialog.getByRole('button', { name: 'Confirm and restore', exact: true }).click()); page.setDefaultTimeout(8000); await page.context().setOffline(true);
  await expect.poll(async () => (await invoke('startup_status')).state).toBe('ready');
  assert.deepEqual(await grammars(), original);
  const after = readState(destination);
  compareLearning(after, scenario === 'grammar-legacy' ? { ...before, grammar_exchange_identities: after.grammar_exchange_identities } : before);
  assert.equal(after.grammar_exchange_identities.length, 3);
  const snapshot = await invoke('note_export_snapshot', { scope: { kind: 'all', sort: 'created-asc' }, recordKind: 'grammar' }); assert.equal(snapshot.records, 3);
  evidence.checks.push(scenario === 'grammar-legacy' ? 'T07: real v4 backup preview, complete restore/restart, assigned identities with unchanged learning state' : 'T06/T38: real v5 backup/restore/restart preserves grammar/exchange identities, vocabulary, classification, cards, progress, history and pause');
  if (scenario === 'grammar-zip-backup') {
    const zip = resolve(process.env.LEXIQUE_TICKET_217_ZIP || '');
    const owned = relative(resolve('../../target/ticket-217'), zip);
    assert.ok(owned && !owned.startsWith('..') && !isAbsolute(owned));
    await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
    await notebook(); await page.getByRole('tab', { name: '语法', exact: true }).click();
    await picker('导入语法 Markdown ZIP', zip);
    const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
    await expect(region.getByText('内容相同，将跳过', { exact: true })).toHaveCount(3);
    await region.getByRole('button', { name: '确认导入', exact: true }).click();
    await expect(region.getByRole('status')).toContainText('导入已保存');
    assert.deepEqual(readState(destination).grammar_exchange_identities, before.grammar_exchange_identities);
    evidence.checks.push('restored library reimports original ZIP as existing identities');
  }
}
async function grammarVocabulary() {
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  const before = readState(destination), filename = join(destination, 'vocabulary.csv');
  await notebook(); await exportCsv(filename);
  const rows = python("import csv,json,sys\nwith open(sys.argv[1],encoding='utf-8-sig',newline='') as f: print(json.dumps(list(csv.DictReader(f)),ensure_ascii=False))", [filename]);
  assert.deepEqual(rows, [{ headword: 'or', notes: 'Preserved vocabulary', example_fr: '', example_zh: '' }]);
  compareLearning(readState(destination), before);
  const original = await words(), originalGrammar = await grammars(), importFile = join(destination, 'vocabulary-import.csv');
  await writeFile(importFile, 'headword,notes,example_fr,example_zh\r\nnative-grammar-preserved-214,Imported vocabulary,,\r\n');
  await picker('导入 CSV', importFile); await confirm();
  const imported = (await words()).find((record) => record.headword === 'native-grammar-preserved-214');
  assert.equal(imported.notes, 'Imported vocabulary'); assert.equal(imported.import_source.filename, 'vocabulary-import.csv');
  assert.deepEqual((await words()).filter((record) => record.id !== imported.id), original);
  assert.deepEqual(await grammars(), originalGrammar);
  assert.deepEqual(await invoke('review_cards', { vocabularyId: imported.id }), []);
  const after = readState(destination);
  compareLearning({ ...after, records: before.records, saves: before.saves }, before);
  evidence.checks.push('T38: real vocabulary CSV export/import retains its four fields, provenance and no-card behavior while preserving existing vocabulary, grammar and all learning data');
}
async function grammarVocabularyMarkdown() {
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  const before = readState(destination), original = await words(), filename = join(destination, 'vocabulary.md');
  await notebook();
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('or');
  await expect(page.locator('.vocabulary-list button')).toHaveCount(1);
  await page.locator('.vocabulary-list button').click();
  await page.locator('.vocabulary-editor').getByRole('button', { name: '导出 Markdown', exact: true }).click();
  await picker('保存 Markdown', filename, true);
  await page.getByRole('dialog', { name: '导出笔记' }).getByRole('button', { name: '关闭', exact: true }).click();
  await picker('导入 Markdown', filename);
  await expect(page.getByRole('region', { name: 'Markdown 导入预览', exact: true })).toContainText('内容相同，将跳过');
  await confirm(); assert.deepEqual(await words(), original); compareLearning(readState(destination), before);
  evidence.checks.push('T38: real vocabulary Markdown export/import remains intact and preserves grammar and learning state');
}
try {
  for (const name of ['conjugation', 'frequency']) await cp(join(resolve(resources), name), join(destination, name), { recursive: true });
  if (source) {
    const roots = scenario.startsWith('grammar-') ? ['../../target/ticket-213', '../../target/ticket-214', '../../target/ticket-215', '../../target/ticket-216', '../../target/ticket-217'] : ['../../target/issue-189'];
    assert.ok(roots.some((root) => { const owned = relative(resolve(root), resolve(source)); return owned && !owned.startsWith('..') && !isAbsolute(owned); }));
    await cp(join(resolve(source), 'Lexique-data'), join(destination, 'Lexique-data'), { recursive: true, filter: (path) => !path.includes('webview') });
    if (scenario.startsWith('grammar-')) evidence.sourceIdentities = readState(resolve(source)).grammar_exchange_identities;
    if (!scenario.startsWith('grammar-')) evidence.initialCsv = JSON.parse(await readFile(join(resolve(source), 'results.json'), 'utf8')).initialCsv;
    if (scenario === 'grammar-csv') {
      evidence.beforeMigration = readState(destination);
      fixtureSql(destination, "c.executescript('DROP TRIGGER grammar_exchange_identity_insert; DROP TABLE grammar_exchange_identities; PRAGMA user_version=20;'); c.commit()", {});
    }
  }
  page = await fixture.start(); page.setDefaultTimeout(8000); await page.context().setOffline(true); await expect(page.getByRole('banner')).toBeVisible();
  if (evidence.sourceIdentities && scenario !== 'grammar-csv') assert.deepEqual(readState(destination).grammar_exchange_identities, evidence.sourceIdentities);
  if (grammarImportScenarios.includes(scenario)) await grammarImportCheck({ scenario, page, destination, invoke, picker, evidence, python });
  else await ({ seed, csv, markdown, zip, consent, enroll, backup, 'device-names': deviceNames, 'grammar-seed': grammarSeed, 'grammar-csv': grammarCsv, 'grammar-backup': grammarBackup, 'grammar-legacy': grammarBackup, 'grammar-vocabulary': grammarVocabulary, 'grammar-vocabulary-markdown': grammarVocabularyMarkdown, 'grammar-zip-source': grammarZipSource, 'grammar-zip-destination': grammarZipDestination, 'grammar-zip-backup': grammarBackup, 'grammar-zip-invalid': grammarZipInvalid, 'grammar-zip-reopen': grammarZipReopen, 'grammar-zip-paths': grammarZipPaths })[scenario]();
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
