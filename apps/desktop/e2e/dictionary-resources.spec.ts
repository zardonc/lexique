import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const bundled = [{ id: 'builtin', title: 'FreeDict fra-zho', content_hash: null, availability: 'available' },
  { id: 'builtin-fra-eng', title: 'FreeDict fra-eng 0.4.1', content_hash: null, availability: 'available' }];

test('bundled dictionaries stay registered and fifteen custom MDX slots remain available', async ({ page }) => {
  const catalog = [...bundled, ...Array.from({ length: 15 }, (_, index) => ({ id: `mdx-${index + 1}`, title: `custom-${index + 1}.mdx`, content_hash: null, availability: 'available' }))];
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_remove: [{ value: catalog.slice(0, -1) }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const settings = page.getByRole('region', { name: '查询词典', exact: true });
  await expect(settings.locator('li')).toHaveCount(17);
  await expect(settings.getByLabel('添加 MDX 词典', { exact: true })).toBeDisabled();
  await expect(settings).toContainText('2018-08-11');
  for (const resource of bundled) {
    const row = settings.locator('li').filter({ hasText: resource.title });
    await expect(row.getByRole('button', { name: '移出词典列表', exact: true })).toHaveCount(0);
    await expect(row.getByText('随包', { exact: true })).toBeVisible();
  }
  await settings.locator('li').filter({ hasText: 'custom-15.mdx' }).getByRole('button', { name: '移出词典列表', exact: true }).click();
  await expect(settings.getByLabel('添加 MDX 词典', { exact: true })).toBeEnabled();
  await expect(settings.locator('li')).toHaveCount(16);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_remove').map((call) => call.args))).toEqual([{ id: 'mdx-15' }]);
});

test('optional links dispatch a known resource ID and leave launcher failures visible', async ({ page }) => {
  const external: string[] = [];
  page.on('request', (request) => { if (!['127.0.0.1', 'localhost'].includes(new URL(request.url()).hostname)) external.push(request.url()); });
  await nativeFixture(page, { dictionary_settings: [{ value: bundled }], dictionary_resource_open: [
    { error: { code: 'query_failed', message: 'test launcher failed' } }, { value: null },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const directory = page.getByRole('region', { name: '额外词典与例句', exact: true });
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_resource_open'))).toEqual([]);
  const link = directory.getByRole('link', { name: '查看来源／下载页: Collins French–English', exact: true });
  await expect(link).toHaveAttribute('href', 'https://forum.freemdict.com/t/topic/31704');
  await link.click();
  await expect(directory.getByRole('alert')).toContainText('test launcher failed');
  await link.click();
  await expect(directory.getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_resource_open').map((call) => call.args))).toEqual([{ id: 'collins-fr-en' }, { id: 'collins-fr-en' }]);
  expect(external).toEqual([]);
});

test('the directory shows eight categories, format limits and three languages at narrow width', async ({ page }, testInfo) => {
  await nativeFixture(page, { dictionary_settings: [{ value: bundled }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const directory = page.locator('.dictionary-resources');
  await expect(directory.locator('details')).toHaveCount(8);
  await expect(directory.locator('a')).toHaveCount(12);
  await directory.getByText('法语—中文', { exact: true }).click();
  await expect(directory).toContainText('Casio 数据的具体版次未确认');
  await expect(directory.getByText('StarDict', { exact: true })).toBeVisible();
  await directory.getByText('法语例句', { exact: true }).click();
  await expect(directory.getByText('TSV', { exact: true }).first()).toBeVisible();
  await directory.getByText('英语单语', { exact: true }).click();
  await expect(directory).toContainText('154.8 MiB 超限');
  await expect(directory).toContainText('MDD 媒体暂不支持');
  await page.locator('.topbar select').first().selectOption('en');
  await expect(directory.getByRole('heading', { name: 'Additional dictionaries and examples', exact: true })).toBeVisible();
  await expect(directory).toContainText('MDD media are unavailable');
  await page.locator('.topbar select').first().selectOption('fr');
  await expect(directory.getByRole('heading', { name: 'Dictionnaires et exemples supplémentaires', exact: true })).toBeVisible();
  await expect(directory).toContainText('médias MDD indisponibles');
  await page.setViewportSize({ width: 640, height: 900 });
  await expect(directory).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth)).toBe(false);
  await directory.screenshot({ path: testInfo.outputPath('resources-narrow.png'), style: '.skip-link:not(:focus) { visibility: hidden; }' });
});
