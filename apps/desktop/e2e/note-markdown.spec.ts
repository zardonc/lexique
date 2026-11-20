import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const source = { format: 'markdown', filename: 'école-1.md' };
const group = { content: { headword: record.headword, notes: '  **原文**\r\n', examples: null }, rows: [1], existing: record, status: 'different', error: null };
const preview = { operation: 'markdown-fixture', source, columns: [], mapping: { headword: null, notes: null, example_fr: null, example_zh: null }, groups: [group], error: null };

test('single Markdown template help and preview require an explicit update and allow cancellation', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByText('Markdown 模板与示例', { exact: true }).click();
  await expect(page.getByText('# Lexique note v1', { exact: false })).toBeVisible();
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
