import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('grammar search export selects its own saved scope and releases cancellation', async ({ page }) => {
  await nativeFixture(page, {
    grammar_list: [{ value: [] }],
    note_export_snapshot: [{ value: { token: 'grammar-search', records: 2, rows: 2 } }],
    note_export_csv: [{ value: { state: 'cancelled', records: 2, rows: 2 } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await expect(dialog).toContainText('2 条语法记录');
  await expect(dialog.getByRole('button', { name: '保存 Markdown', exact: true })).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args?.recordKind)).toBe('grammar');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args?.scope))
    .toEqual({ kind: 'search', query: '', sort: 'updated-desc', category: 'all' });
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog).toContainText('已取消');
  await expect(dialog.getByRole('button', { name: '保存 CSV', exact: true })).toBeEnabled();
});

test('single grammar export reads saved identity while a draft is edited and shows write failure', async ({ page }) => {
  const record = { id: 42, title: 'Saved title', body: 'Saved body', category_ids: [], card: null, revision: 1, created_at: 1000, updated_at: 1000 };
  await nativeFixture(page, {
    grammar_list: [{ value: [record] }],
    note_export_snapshot: [{ value: { token: 'saved-grammar', records: 1, rows: 1 } }],
    note_export_csv: [{ error: 'Could not write CSV file: access denied' }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('语法标题', { exact: true }).fill('Unsaved title');
  await page.locator('.grammar-editor').getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await expect(dialog.getByLabel('导出范围')).toHaveValue('single');
  await expect(dialog.getByLabel('导出范围')).toBeDisabled();
  await dialog.getByRole('button', { name: '保存 CSV', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('access denied');
  await expect(dialog).not.toContainText('已导出');
  await dialog.getByRole('button', { name: '关闭', exact: true }).click();
  await expect(page.getByLabel('语法标题', { exact: true })).toHaveValue('Unsaved title');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args))
    .toEqual({ scope: { kind: 'single', id: 42 }, recordKind: 'grammar' });
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'grammar_save'))).toEqual([]);
});

test('grammar all scope retains list order and shows an empty saved selection', async ({ page }) => {
  await nativeFixture(page, {
    grammar_list: [{ value: [] }], grammar_sort: [{ value: 'review-due-asc' }],
    note_export_snapshot: [{ value: { token: 'empty', records: 0, rows: 0 } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '导出语法笔记' });
  await expect(dialog.getByRole('button', { name: '保存 CSV', exact: true })).toBeDisabled();
  await dialog.getByLabel('导出范围').selectOption('all');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_export_snapshot').at(-1)?.args?.scope))
    .toEqual({ kind: 'all', sort: 'review-due-asc' });
  await expect(dialog).toContainText('没有已保存的语法记录');
});
