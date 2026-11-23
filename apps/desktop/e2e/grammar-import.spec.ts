import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const record = { id: 42, title: 'Une règle', body: 'été, 中文\n', category_ids: [], card: { id: 77, question: 'Pourquoi ?', answer: 'Réponse' }, revision: 1, created_at: 1000, updated_at: 1000 };
const preview = { operation: 'grammar-preview', source: { format: 'csv', filename: 'grammar.csv' }, error: null,
  group: { row: 2, status: 'create', error: null, existing: null, content: { exchange_id: 'external-214', title: record.title, body: record.body, question: record.card.question, answer: record.card.answer } } };

test('grammar CSV shows content and identity, saves first and reports start-review failure without losing the note', async ({ page }) => {
  await nativeFixture(page, {
    grammar_list: [{ value: [] }], note_import_preview: [{ value: preview }],
    note_import_commit: [{ value: { created: 1, skipped: 0, records: [record] } }],
    review_pause: [{ error: 'start write failed' }, { value: false }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  for (const value of ['external-214', 'Une règle', 'été, 中文', 'Pourquoi ?', 'Réponse', 'grammar.csv', '2']) await expect(region).toContainText(value);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'))).toEqual([]);
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('导入已保存');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_pause'))).toEqual([]);
  await region.getByRole('button', { name: '开始复习', exact: true }).click();
  await expect(region.getByRole('alert')).toContainText('start write failed');
  await expect(region.getByRole('status')).toContainText('导入已保存');
  await expect(region.getByRole('button', { name: '打开记录', exact: true })).toBeVisible();
  await region.getByRole('button', { name: '开始复习', exact: true }).click();
  await expect(region).toContainText('已开始复习');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_preview')[0]?.args?.options))
    .toEqual({ record_kind: 'grammar', delimiter: 'comma', mapping: null });
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_pause').map((call) => call.args)))
    .toEqual([{ vocabularyId: 42, paused: false }, { vocabularyId: 42, paused: false }]);
});

test('failed confirmation keeps the preview and the same submission available for retry', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }],
    note_import_commit: [{ error: 'uncertain response' }, { value: { created: 1, skipped: 0, records: [{ ...record, card: null }] }, gate: 'commit' }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('alert')).toContainText('uncertain response');
  await expect(region).not.toContainText('导入已保存');
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  await expect(region.getByRole('button', { name: '取消导入', exact: true })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('commit'));
  await expect(region.getByRole('status')).toContainText('导入已保存');
  await expect(region.getByRole('button', { name: '开始复习', exact: true })).toHaveCount(0);
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit').map((call) => call.args));
  expect(calls).toHaveLength(2); expect(calls[0]).toEqual(calls[1]);
});

test('known changed identity and one-sided pair remain visible and cannot overwrite saved content', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [
    { value: { ...preview, group: { ...preview.group, status: 'different', existing: { ...record, body: 'Saved body' } } } },
    { value: { ...preview, group: { ...preview.group, status: 'invalid', error: 'grammar.csv row 2: Question and answer must be supplied together' } } },
    { value: null },
  ], note_import_cancel: [{ value: null }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  await expect(region).toContainText('当前导入保留原内容'); await expect(region).toContainText('Saved body');
  await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  await region.getByRole('button', { name: '取消导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('已取消');
  await region.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  await expect(region.getByRole('alert')).toContainText('row 2');
  await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  await region.getByRole('button', { name: '取消导入', exact: true }).click();
  await region.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('已取消');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'))).toEqual([]);
});

test('unchanged identity can skip without creating a record or offering enrollment', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, group: { ...preview.group, status: 'identical', existing: record } } }],
    note_import_commit: [{ value: { created: 0, skipped: 1, records: [] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导入语法 CSV', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  await region.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(region.getByRole('status')).toContainText('跳过 1');
  await expect(region.getByRole('button', { name: '开始复习', exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'note_import_commit')?.args?.selections))
    .toEqual([{ index: 0, action: 'skip', allow_unlinked: false }]);
});

test('browser preview reports that grammar import needs the desktop application', async ({ page }) => {
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await expect(page.getByRole('main')).toContainText('浏览器');
  await expect(page.getByRole('button', { name: '导入语法 CSV', exact: true })).toHaveCount(0);
});
