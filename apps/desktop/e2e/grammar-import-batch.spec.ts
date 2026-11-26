import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const content = { exchange_id: 'known', title: 'Renamed', body: null, question: null, answer: null };
const record = { id: 42, title: 'Original', body: 'Saved body', category_ids: [], card: null, revision: 1, created_at: 1000, updated_at: 1000 };
const mapping = { exchange_id: 0, title: 1, body: null, question: null, answer: null };
const preview = { operation: 'batch-preview', source: { format: 'csv', filename: 'batch.csv' }, columns: ['ID', 'Heading'], mapping, error: null,
  groups: [{ content, rows: [2], existing: record, status: 'different', error: null },
    { content: { ...content, title: 'Invalid pair' }, rows: [3, 4], existing: null, status: 'invalid', error: 'batch.csv rows 3, 4: conflicting identity' }] };

test('batch requires update choice and invalid group exclusion; uncertain retry keeps identical selection', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }], note_import_commit: [
    { error: 'Committed response lost' }, { value: { created: 0, updated: 1, skipped: 1, new_card_ids: [], records: [{ ...record, title: 'Renamed' }] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await expect(region).toContainText('Saved body'); await expect(region).toContainText('3, 4');
  const confirm = region.getByRole('button', { name: '确认导入', exact: true });
  await expect(confirm).toBeDisabled();
  await region.getByRole('combobox', { name: '导入决定：Renamed', exact: true }).selectOption('update');
  await expect(confirm).toBeDisabled();
  await region.getByRole('checkbox', { name: '排除无效组：Invalid pair', exact: true }).check();
  await confirm.click(); await expect(region.getByRole('alert').filter({ hasText: 'Committed response lost' })).toBeVisible();
  await confirm.click(); await expect(region.getByRole('status')).toContainText('导入已保存');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit').map((call) => call.args));
  expect(calls).toHaveLength(2); expect(calls[0]).toEqual(calls[1]);
  expect(calls[0]?.selections).toEqual([{ index: 0, action: 'update', allow_unlinked: false }, { index: 1, action: 'skip', allow_unlinked: false }]);
});

test('external mapping must be previewed again before committing and cancellation releases the session', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, groups: [], error: 'Map the required title column' } }],
    note_import_remap: [{ value: { ...preview, error: null, groups: [{ ...preview.groups[0], existing: null, status: 'create' }] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('combobox', { name: '分隔符', exact: true }).selectOption('semicolon');
  await region.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  await expect(region.getByRole('alert')).toContainText('title');
  await region.getByRole('combobox', { name: 'title', exact: true }).selectOption('1');
  await region.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeEnabled();
  await region.getByRole('combobox', { name: 'body', exact: true }).selectOption('0');
  await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  await region.getByRole('button', { name: '取消导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('已取消');
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.find((call) => call.command === 'note_import_remap')?.args?.options).toEqual({ record_kind: 'grammar', delimiter: 'semicolon', mapping: null, grammar_mapping: mapping });
  expect(calls.filter((call) => call.command === 'note_import_commit')).toHaveLength(0);
  expect(calls.find((call) => call.command === 'note_import_cancel')?.args?.operation).toBe(calls.find((call) => call.command === 'note_import_preview')?.args?.operation);
});

test('new cards offer independent start actions with visible per-card failures and retries', async ({ page }) => {
  const second = { ...record, id: 43, title: 'Second' }, third = { ...record, id: 44, title: 'Existing card' };
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, groups: [{ ...preview.groups[0], existing: null, status: 'create' }] } }],
    note_import_commit: [{ value: { created: 2, updated: 1, skipped: 0, new_card_ids: [42, 43], records: [record, second, third] } }],
    review_pause: [{ error: 'first card is still paused' }, { value: false }, { value: false }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click(); await page.getByRole('tab', { name: '语法', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  const first = region.getByRole('region', { name: 'Original', exact: true }), other = region.getByRole('region', { name: 'Second', exact: true });
  await first.getByRole('button', { name: '开始复习', exact: true }).click();
  await expect(first.getByRole('alert')).toContainText('first card is still paused');
  await other.getByRole('button', { name: '开始复习', exact: true }).click();
  await expect(other).toContainText('已开始复习'); await expect(first.getByRole('alert')).toBeVisible();
  await expect(region.getByRole('region', { name: 'Existing card', exact: true }).getByRole('button', { name: '开始复习', exact: true })).toHaveCount(0);
  await first.getByRole('button', { name: '开始复习', exact: true }).click();
  await expect(first).toContainText('已开始复习'); await expect(first.getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_pause').map((call) => call.args?.vocabularyId))).toEqual([42, 43, 42]);
});
