import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const external = { ...record, id: 2, headword: 'mot externe', reference: null, digest: [], example_sense: null,
  import_source: { format: 'csv', filename: 'notes.csv' } };
const group = { rows: [2], status: 'create', error: null, existing: null,
  content: { headword: external.headword, notes: external.notes, examples: [] },
  enrichment: { status: 'error', snapshot: null, warnings: [{ dictionary_id: 'builtin', code: 'missing', message: 'FreeDict: dictionary.sqlite3 is missing' }] } };
const preview = { operation: 'dictionary-preview', source: external.import_source, columns: ['headword', 'notes'],
  mapping: { headword: 0, notes: 1, example_fr: null, example_zh: null }, groups: [group], error: null };

test('dictionary failure requires explicit continuation before unlinked import', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }],
    note_import_remap: [{ value: preview }],
    note_import_commit: [{ value: { created: 1, updated: 0, skipped: 0, records: [external] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('dictionary.sqlite3 is missing');
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  await expect(confirm).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await expect(confirm).toBeEnabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).uncheck();
  await expect(confirm).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await expect(confirm).toBeEnabled();
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' })).not.toBeChecked();
  await expect(confirm).toBeDisabled();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await confirm.click();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls).toHaveLength(1);
  expect(calls[0].args).toMatchObject({ operation: 'dictionary-preview', selections: [{ index: 0, action: 'create', allow_unlinked: true }] });
});

test('each affected creation needs consent while skipped groups and saved updates need none', async ({ page }) => {
  const second = { ...group, content: { ...group.content, headword: 'deuxième' } };
  const retained = { ...group, content: { ...group.content, headword: record.headword }, existing: record, status: 'different',
    enrichment: { status: 'retained', snapshot: null, warnings: [] } };
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, groups: [group, second, retained] } }],
    note_import_commit: [{ value: { created: 1, updated: 1, skipped: 1, records: [external, record] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await expect(page.getByText(`保留已有词典信息 · ${record.reference.dictionary} · ${record.reference.version}`)).toBeVisible();
  await page.getByRole('combobox', { name: `导入决定：${record.headword}`, exact: true }).selectOption('update');
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  const consent = page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' });
  await expect(consent).toHaveCount(2); await consent.nth(0).check();
  await expect(confirm).toBeDisabled();
  await page.getByRole('combobox', { name: `导入决定：${external.headword}`, exact: true }).selectOption('skip');
  await expect(consent.nth(0)).toBeDisabled(); await expect(confirm).toBeDisabled();
  await consent.nth(1).check(); await expect(confirm).toBeEnabled(); await confirm.click();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls).toHaveLength(1);
  expect(calls[0].args?.selections).toEqual([{ index: 0, action: 'skip', allow_unlinked: true },
    { index: 1, action: 'create', allow_unlinked: true }, { index: 2, action: 'update', allow_unlinked: false }]);
});

test('exact match displays its source and earlier warnings without unlinked consent', async ({ page }) => {
  const linked = { ...preview, groups: [{ ...group, enrichment: { status: 'linked', snapshot: { reference: record.reference, digest: record.digest }, warnings: group.enrichment.warnings } }] };
  await nativeFixture(page, { note_import_preview: [{ value: linked }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  const panel = page.getByRole('region', { name: 'CSV 导入预览', exact: true });
  await expect(panel).toContainText(`已精确关联本地词典 · ${record.reference.dictionary}`);
  await expect(panel.getByRole('alert')).toContainText('missing');
  await expect(panel.getByRole('checkbox')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeEnabled();
});

test('true absence is shown separately and cancelled consent is cleared on a new preview', async ({ page }) => {
  const absent = { ...preview, groups: [{ ...group, enrichment: { status: 'not_found', snapshot: null, warnings: [] } }] };
  await nativeFixture(page, { note_import_preview: [{ value: absent }, { value: preview }, { value: preview }], note_import_cancel: [{ value: null }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await expect(page.getByText('已安装词典未收录，保存为外部笔记。')).toBeVisible();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' }).check();
  await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await expect(page.getByRole('checkbox', { name: '继续保存这条未关联词典的笔记' })).not.toBeChecked();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'note_import_commit'))).toBe(false);
});
