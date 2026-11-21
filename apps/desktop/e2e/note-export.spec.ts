import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

test('notebook exports a retained saved snapshot and reports cancellation', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    note_export_snapshot: [{ value: { token: 'export-1', records: 1, rows: 1 } }],
    note_export_csv: [{ value: { state: 'cancelled', records: 1, rows: 1 } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog).toContainText('1 条生词');
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog).toContainText('已取消');
  await expect(dialog).not.toContainText('已导出');
});

test('search export keeps notebook sort and includes matches beyond the current page', async ({ page }) => {
  const records = Array.from({ length: 27 }, (_, index) => ({ ...record, id: index + 1, headword: `mot-${index}` }));
  await nativeFixture(page, {
    vocabulary_list: [{ value: records }], vocabulary_sort: [{ value: 'review-due-asc' }],
    note_export_snapshot: [{ value: { token: 'search', records: 27, rows: 30 } }, { value: { token: 'search', records: 27, rows: 30 } }, { value: { token: 'all', records: 40, rows: 44 } }],
    note_export_csv: [{ value: { state: 'saved', records: 40, rows: 44 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('sujet');
  await expect(page.locator('.vocabulary-list li')).toHaveCount(20);
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog).toContainText('27 条生词');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args?.scope))
    .toEqual({ kind: 'search', query: 'sujet', sort: 'review-due-asc', category: 'all' });
  await dialog.getByLabel('导出范围').selectOption('all');
  await expect(dialog).toContainText('40 条生词');
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog.getByRole('status').last()).toHaveText('已导出');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_csv').map((call) => call.args))).toEqual([{ token: 'all' }]);
});

test('empty search disables file saving while all vocabulary can still be selected', async ({ page }) => {
  await nativeFixture(page, {
    note_export_snapshot: [{ value: { token: 'empty', records: 0, rows: 0 } }, { value: { token: 'empty', records: 0, rows: 0 } }, { value: { token: 'all', records: 1, rows: 1 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog).toContainText('此范围没有已保存的生词');
  await expect(dialog.getByRole('button', { name: '保存 CSV', exact: true })).toBeDisabled();
  await dialog.getByLabel('导出范围').selectOption('all');
  await expect(dialog.getByRole('button', { name: '保存 CSV', exact: true })).toBeEnabled();
});

test('a write error remains visible and retrying the saved snapshot can succeed', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    note_export_snapshot: [{ value: { token: 'saved', records: 1, rows: 1 } }],
    note_export_csv: [{ error: 'Could not write CSV file: Access is denied.' }, { value: { state: 'saved', records: 1, rows: 1 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('Access is denied.');
  await expect(dialog).not.toContainText('已导出');
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog.getByRole('alert')).toHaveCount(0);
  await expect(dialog.getByRole('status').last()).toHaveText('已导出');
});

test('single-record export excludes an unsaved editing draft', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    note_export_snapshot: [{ value: { token: 'single', records: 1, rows: 1 } }],
    note_export_csv: [{ value: { state: 'cancelled', records: 1, rows: 1 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('独立笔记').fill('unsaved draft');
  await page.locator('.vocabulary-editor').getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出笔记' });
  await expect(dialog.getByLabel('导出范围')).toHaveValue('single');
  await expect(dialog.getByLabel('导出范围')).toBeDisabled();
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog).toContainText('已取消');
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  await expect(page.getByLabel('独立笔记')).toHaveValue('unsaved draft');
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args).toEqual({ scope: { kind: 'single', id: 1 } });
  expect(calls.filter((call) => call.command === 'vocabulary_save')).toHaveLength(0);
});
