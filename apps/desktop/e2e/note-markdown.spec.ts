import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const source = { format: 'markdown', filename: 'école-1.md' };
const group = { content: { headword: record.headword, notes: '  **原文**\r\n', examples: null }, rows: [1], existing: record, status: 'different', error: null };
const preview = { operation: 'markdown-fixture', source, columns: [], mapping: { headword: null, notes: null, example_fr: null, example_zh: null }, groups: [group], error: null };

test('single Markdown template help and preview require an explicit update and allow cancellation', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }], note_export_markdown: [{ value: { state: 'cancelled', records: 1, rows: 1 } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByText('Markdown 模板与示例', { exact: true }).click();
  await expect(page.getByText('# Lexique note v1', { exact: false })).toBeVisible();
  await page.getByRole('button', { name: '导出 Markdown 示例', exact: true }).click();
  await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('已取消，未导出文件。');
  await page.getByRole('button', { name: '导入 Markdown', exact: true }).click();
  const region = page.getByRole('region', { name: 'Markdown 导入预览', exact: true });
  await expect(region).toContainText('école-1.md');
  await expect(region).toContainText('未提供的字段保留已有内容。');
  await expect(region.getByRole('button', { name: '确认导入' })).toBeDisabled();
  await region.getByLabel(`导入决定：${record.headword}`, { exact: true }).selectOption('update');
  await expect(region.getByRole('button', { name: '确认导入' })).toBeEnabled();
  await region.getByRole('button', { name: '取消导入' }).click();
  await expect(region).not.toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command.startsWith('note_import_')));
  expect(calls.find((call) => call.command === 'note_import_preview')?.args?.options).toEqual({ delimiter: 'comma', mapping: null, format: 'markdown' });
  expect(calls.some((call) => call.command === 'note_import_commit')).toBe(false);
  expect(calls.some((call) => call.command === 'note_import_cancel')).toBe(true);
});

test('invalid Markdown shows filename and reason before reselecting and stale updates require fresh decisions', async ({ page }) => {
  const current = { ...record, revision: 2, notes: '同时修改' };
  await nativeFixture(page, {
    note_import_preview: [{ value: { ...preview, groups: [], error: 'école-1.md: Unclosed text fence' } }, { value: preview }],
    note_import_remap: [{ value: { ...preview, groups: [{ ...group, existing: current }] } }],
    note_import_commit: [{ error: 'Record changed after preview; preview again' }, { value: { created: 0, updated: 1, skipped: 0, records: [current] } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  const open = page.getByRole('button', { name: '导入 Markdown', exact: true });
  await open.click(); await expect(page.getByRole('alert')).toContainText('école-1.md: Unclosed text fence');
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  await expect(confirm).toBeDisabled(); await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await open.click();
  const decision = page.getByLabel(`导入决定：${record.headword}`, { exact: true });
  await decision.selectOption('update'); await confirm.click();
  await expect(page.getByRole('alert')).toContainText('Record changed after preview');
  await expect(page.getByRole('region', { name: 'Markdown 导入预览', exact: true })).toBeVisible();
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(decision).toHaveValue(''); await expect(confirm).toBeDisabled();
  await expect(page.getByText('同时修改', { exact: true })).toBeVisible();
  await decision.selectOption('update'); await confirm.click();
  await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('更新 1');
});

test('Markdown reuses dictionary error consent and resets it on re-preview', async ({ page }) => {
  const external = { ...record, reference: null, digest: [], example_sense: null, import_source: source };
  const enrichment = { status: 'error', snapshot: null, warnings: [{ dictionary_id: 'builtin', code: 'missing', message: 'Dictionary file missing' }] };
  const unlinked = { ...preview, groups: [{ ...group, existing: null, status: 'create', enrichment }] };
  await nativeFixture(page, { note_import_preview: [{ value: unlinked }], note_import_remap: [{ value: unlinked }],
    note_import_commit: [{ gate: 'commit', value: { created: 1, updated: 0, skipped: 0, records: [external] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 Markdown', exact: true }).click();
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  const consent = page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' });
  await expect(confirm).toBeDisabled(); await consent.check(); await expect(confirm).toBeEnabled();
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(consent).not.toBeChecked(); await expect(confirm).toBeDisabled();
  await consent.check(); await confirm.click(); await expect(confirm).toBeDisabled();
  await expect(page.getByRole('button', { name: '重新预览', exact: true })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('commit'));
  await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('新建 1');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls).toHaveLength(1);
  expect(calls[0].args?.selections).toEqual([{ index: 0, action: 'create', allow_unlinked: true }]);
});

test('single Markdown export uses saved scope and keeps drafts on write errors and cancellation', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], note_export_snapshot: [{ value: { token: 'single-markdown', records: 1, rows: 1 } }],
    note_export_markdown: [{ error: 'Could not write Markdown file: Access denied' }, { value: { state: 'cancelled', records: 1, rows: 1 } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click(); await page.getByLabel('独立笔记').fill('unsaved draft');
  await page.getByRole('button', { name: '导出 Markdown', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog.getByLabel('导出范围')).toHaveValue('single');
  await dialog.getByRole('button', { name: '保存 Markdown', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('Access denied');
  await dialog.getByRole('button', { name: '保存 Markdown', exact: true }).click();
  await expect(dialog).toContainText('已取消，未导出文件。'); await expect(dialog).not.toContainText('已导出');
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  await expect(page.getByLabel('独立笔记')).toHaveValue('unsaved draft');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot' || call.command === 'note_export_markdown'));
  expect(calls[0].args?.scope).toEqual({ kind: 'single', id: record.id });
  expect(calls.filter((call) => call.command === 'note_export_markdown').map((call) => call.args)).toEqual([{ token: 'single-markdown' }, { token: 'single-markdown' }]);
});
