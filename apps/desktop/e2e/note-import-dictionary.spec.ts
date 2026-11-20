import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const external = { ...record, headword: 'mot externe', reference: null, digest: [], example_sense: null,
  import_source: { format: 'csv', filename: 'notes.csv' } };
const preview = { operation: 'dictionary-preview', source: external.import_source, rows: [2], status: 'create', error: null, existing: null,
  content: { headword: external.headword, notes: external.notes, examples: [] },
  enrichment: { status: 'error', snapshot: null, warnings: [{ dictionary_id: 'builtin', code: 'missing', message: 'FreeDict: dictionary.sqlite3 is missing' }] } };

test('dictionary failure requires explicit continuation before unlinked import', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }],
    note_import_commit: [{ value: { outcome: 'created', record: external } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('dictionary.sqlite3 is missing');
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  await expect(confirm).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await expect(confirm).toBeEnabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).uncheck();
  await expect(confirm).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await expect(confirm).toBeEnabled(); await confirm.click();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls[0].args).toMatchObject({ operation: 'dictionary-preview', allowUnlinked: true });
});

test('exact match displays its source and earlier warnings without unlinked consent', async ({ page }) => {
  const linked = { ...preview, enrichment: { status: 'linked', snapshot: { reference: record.reference, digest: record.digest }, warnings: preview.enrichment.warnings } };
  await nativeFixture(page, { note_import_preview: [{ value: linked }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  const panel = page.getByRole('region', { name: '单词 CSV 导入预览', exact: true });
  await expect(panel).toContainText(`已精确关联本地词典 · ${record.reference.dictionary}`);
  await expect(panel.getByRole('alert')).toContainText('missing');
  await expect(panel.getByRole('checkbox')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeEnabled();
});

test('true absence is shown separately and cancelled consent is cleared on a new preview', async ({ page }) => {
  const absent = { ...preview, enrichment: { status: 'not_found', snapshot: null, warnings: [] } };
  await nativeFixture(page, { note_import_preview: [{ value: absent }, { value: preview }, { value: preview }], note_import_cancel: [{ value: null }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByText('已安装词典未收录，保存为外部笔记。')).toBeVisible();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' })).not.toBeChecked();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'note_import_commit'))).toBe(false);
});
