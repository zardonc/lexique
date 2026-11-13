import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const missing = { id: 'mdx-7', title: 'original.mdx', content_hash: 'a'.repeat(64), availability: 'missing' };
const bundled = [{ id: 'builtin', title: 'FreeDict fra-zho', content_hash: null, availability: 'available' },
  { id: 'builtin-fra-eng', title: 'FreeDict fra-eng 0.4.1', content_hash: null, availability: 'available' }];
const catalog = [missing, ...bundled];
const restored = [{ ...missing, availability: 'available' }, ...bundled];
const file = { name: 'same-file.mdx', mimeType: 'application/octet-stream', buffer: Buffer.from([77, 68, 88]) };

test('relinks a missing dictionary with the original identity and order', async ({ page }) => {
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_relink: [{ value: { status: 'linked', catalog: restored } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.getByRole('region', { name: '查询词典', exact: true });
  const row = settings.locator('li').filter({ hasText: 'original.mdx' });
  await expect(row).toContainText('待补齐 MDX', { timeout: 5000 });
  await row.getByLabel('选择 MDX 文件：original.mdx', { exact: true }).setInputFiles(file);
  await expect(settings.getByRole('status')).toHaveText('词典设置已保存。');
  await expect(row).not.toContainText('待补齐 MDX');
  await expect(settings.locator('li').first()).toContainText('original.mdx');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_relink').map((call) => ({
    bytes: Object.values(call.args ?? {}), headers: call.options?.headers,
  })))).toEqual([{ bytes: [77, 68, 88], headers: { 'x-lexique-dictionary-name': 'c2FtZS1maWxlLm1keA==', 'x-lexique-dictionary-id': 'mdx-7' } }]);
});

test('different content waits for explicit confirmation and cancellation keeps the registration', async ({ page }) => {
  const confirmation = { status: 'confirmation_required', dictionary_id: 'mdx-7', expected_hash: missing.content_hash, content_hash: 'b'.repeat(64) };
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_relink: [{ value: confirmation }, { value: confirmation },
    { value: { status: 'linked', catalog: [{ ...restored[0], content_hash: 'b'.repeat(64) }, ...bundled] }, gate: 'relink-commit' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.getByRole('region', { name: '查询词典', exact: true });
  const row = settings.locator('li').filter({ hasText: 'original.mdx' });
  const picker = row.getByLabel('选择 MDX 文件：original.mdx', { exact: true });
  await picker.setInputFiles(file);
  const dialog = page.getByRole('dialog', { name: '确认词典内容', exact: true });
  await expect(dialog).toContainText('所选 MDX 的内容与原词典不同。', { timeout: 5000 });
  await expect(dialog).toContainText('same-file.mdx');
  await expect(picker).toBeDisabled();
  await dialog.getByRole('button', { name: '取消', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await expect(row).toContainText('待补齐 MDX');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_relink').length)).toBe(1);
  await picker.setInputFiles(file);
  await dialog.getByRole('button', { name: '确认使用此 MDX', exact: true }).click();
  await expect(dialog.getByRole('button', { name: '确认使用此 MDX', exact: true })).toBeDisabled();
  await page.keyboard.press('Escape');
  await expect(dialog).toBeVisible();
  await page.evaluate(() => window.testNative.release('relink-commit'));
  await expect(dialog).not.toBeVisible();
  await expect(row).not.toContainText('待补齐 MDX');
  await expect(settings.locator('li').first()).toContainText('original.mdx');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_relink').map((call) => call.options?.headers))).toEqual([
    { 'x-lexique-dictionary-name': 'c2FtZS1maWxlLm1keA==', 'x-lexique-dictionary-id': 'mdx-7' },
    { 'x-lexique-dictionary-name': 'c2FtZS1maWxlLm1keA==', 'x-lexique-dictionary-id': 'mdx-7' },
    { 'x-lexique-dictionary-name': 'c2FtZS1maWxlLm1keA==', 'x-lexique-dictionary-id': 'mdx-7', 'x-lexique-dictionary-confirmed-hash': 'b'.repeat(64) },
  ]);
});

test('unknown content can be supplied at capacity and a failed confirmation keeps the old dictionary', async ({ page }) => {
  const fullCatalog = [{ ...missing, content_hash: null }, ...bundled,
    ...Array.from({ length: 14 }, (_, index) => ({ id: `mdx-${index + 20}`, title: `other-${index}.mdx`, content_hash: null, availability: 'available' }))];
  await nativeFixture(page, { dictionary_settings: [{ value: fullCatalog }], dictionary_relink: [
    { value: { status: 'confirmation_required', dictionary_id: 'mdx-7', expected_hash: null, content_hash: 'b'.repeat(64) } },
    { error: { code: 'query_failed', message: 'synthetic registration failure' } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.getByRole('region', { name: '查询词典', exact: true });
  const row = settings.locator('li').filter({ hasText: 'original.mdx' });
  await expect(settings.getByLabel('添加 MDX 词典', { exact: true })).toBeDisabled();
  const picker = row.getByLabel('选择 MDX 文件：original.mdx', { exact: true });
  await expect(picker).toBeEnabled();
  await picker.setInputFiles(file);
  const dialog = page.getByRole('dialog', { name: '确认词典内容', exact: true });
  await expect(dialog).toContainText('原词典的内容身份未知，无法确认所选 MDX 是同一版本。');
  await dialog.getByRole('button', { name: '确认使用此 MDX', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('synthetic registration failure');
  await dialog.getByRole('button', { name: '取消', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await expect(row).toContainText('待补齐 MDX');
  await expect(settings.locator('li')).toHaveCount(17);
  await expect(picker).toBeEnabled();
  await expect(settings.getByRole('alert')).toContainText('synthetic registration failure');
});

test('invalid uploads and a mismatched IPC identity leave the missing registration visible', async ({ page }) => {
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_relink: [
    { error: { code: 'corrupt', message: 'damaged test MDX' } },
    { value: { status: 'linked', catalog: [{ ...restored[0], id: 'mdx-8' }, ...bundled] } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.getByRole('region', { name: '查询词典', exact: true });
  const row = settings.locator('li').filter({ hasText: 'original.mdx' });
  const picker = row.getByLabel('选择 MDX 文件：original.mdx', { exact: true });
  await picker.setInputFiles({ ...file, name: 'wrong.txt' });
  await expect(settings.getByRole('alert')).toContainText('MDX: non-empty, at most 128 MiB');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_relink'))).toEqual([]);
  await picker.setInputFiles(file);
  await expect(settings.getByRole('alert')).toContainText('damaged test MDX');
  await expect(row).toContainText('待补齐 MDX');
  await picker.setInputFiles(file);
  await expect(settings.getByRole('alert')).toContainText('Dictionary reattachment response differs from the request');
  await expect(row).toContainText('待补齐 MDX');
  await expect(settings.getByRole('status')).toHaveCount(0);
});
