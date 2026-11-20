import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const source = { format: 'zip', filename: 'notes.zip' };
const group = { content: { headword: record.headword, notes: '更新\n`````', examples: null }, rows: [1], files: ['nested/école.md'], existing: record, status: 'different', error: null };
const invalid = { ...group, content: { headword: 'conflict', notes: null, examples: null }, existing: null, status: 'invalid', rows: [2, 3], files: ['one.md', 'two.md'], error: 'Same headword has conflicting provided fields or content' };
const unlinked = { ...group, content: { headword: 'new-note', notes: '', examples: [] }, existing: null, status: 'create', rows: [4], files: ['new.md'],
  enrichment: { status: 'error', snapshot: null, warnings: [{ dictionary_id: 'builtin', code: 'missing', message: 'Dictionary file missing' }] } };
const preview = { operation: 'zip-preview', source, columns: [], mapping: { headword: null, notes: null, example_fr: null, example_zh: null }, groups: [group, invalid, unlinked], error: null };

test('ZIP file groups require exclusion and explicit consent again after a stale preview', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }], note_import_remap: [{ value: preview }],
    note_import_commit: [{ error: 'Record changed after preview; preview again' }, { gate: 'zip-commit', value: { created: 1, updated: 1, skipped: 1, records: [record] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 Markdown ZIP', exact: true }).click();
  const panel = page.getByRole('region', { name: 'Markdown ZIP 导入预览', exact: true });
  await expect(panel).toContainText('one.md, two.md'); await expect(panel).toContainText('nested/école.md');
  const confirm = panel.getByRole('button', { name: '确认导入' });
  const update = panel.getByLabel(`导入决定：${record.headword}`, { exact: true });
  const exclude = panel.getByLabel('排除无效组：conflict', { exact: true });
  const consent = panel.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' });
  await update.selectOption('update'); await exclude.check(); await expect(confirm).toBeDisabled();
  await consent.check(); await confirm.click(); await expect(page.getByRole('alert').filter({ hasText: 'Record changed' })).toBeVisible();
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(update).toHaveValue(''); await expect(exclude).not.toBeChecked(); await expect(consent).not.toBeChecked();
  await update.selectOption('update'); await exclude.check(); await consent.check(); await confirm.click();
  await expect(confirm).toBeDisabled(); await expect(panel.getByRole('button', { name: '取消导入' })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('zip-commit'));
  await expect(panel).not.toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'note_import_commit')).toHaveLength(2);
  expect(calls.find((call) => call.command === 'note_import_preview')?.args?.options).toMatchObject({ format: 'zip' });
});

test('ZIP cancellation and fatal container errors cannot submit an earlier selection', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }, { error: 'notes.zip: invalid ZIP container' }, { value: null }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  const open = page.getByRole('button', { name: '导入 Markdown ZIP', exact: true });
  await open.click(); await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await open.click(); await expect(page.getByRole('alert')).toContainText('invalid ZIP container');
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toHaveCount(0);
  await open.click(); await expect(page.getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'note_import_commit'))).toBe(false);
});

test('ZIP export shares saved search and all scopes and disables an empty range', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    note_export_snapshot: [{ value: { token: 'query', records: 1, rows: 2 } }, { value: { token: 'query', records: 1, rows: 2 } }, { value: { token: 'all', records: 3, rows: 4 } }, { value: { token: 'empty', records: 0, rows: 0 } }],
    note_export_zip: [{ value: { state: 'cancelled', records: 1, rows: 2 } }, { value: { state: 'saved', records: 3, rows: 4 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('école');
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  const save = dialog.getByRole('button', { name: '保存 Markdown ZIP', exact: true });
  await expect(dialog).toContainText('1 条生词 · 2 行例句'); await save.click();
  await expect(dialog).toContainText('已取消，未导出文件。');
  await dialog.getByLabel('导出范围').selectOption('all');
  await expect(dialog).toContainText('3 条生词 · 4 行例句'); await save.click();
  await expect(dialog.getByRole('status').last()).toHaveText('已导出');
  await dialog.getByLabel('导出范围').selectOption('search');
  await expect(save).toBeDisabled(); await expect(dialog).toContainText('此范围没有已保存的生词');
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'note_export_zip').map((call) => call.args)).toEqual([{ token: 'query' }, { token: 'all' }]);
  expect(calls.filter((call) => call.command === 'note_export_snapshot').map((call) => call.args?.scope)).toEqual([
    { kind: 'search', query: 'école', sort: 'updated-desc', category: 'all' },
    { kind: 'search', query: 'école', sort: 'updated-desc', category: 'all' }, { kind: 'all', sort: 'updated-desc' },
    { kind: 'search', query: 'école', sort: 'updated-desc', category: 'all' },
  ]);
});
