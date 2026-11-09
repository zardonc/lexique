import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { nativeFixture } from './native-fixture';

const imagePath = new URL('./fixtures/ocr-french.png', import.meta.url);
const image = { width: 948, height: 159, data_url: `data:image/png;base64,${readFileSync(imagePath).toString('base64')}` };

test('imported image is previewed and locally recognized before explicit dictionary lookup', async ({ page }) => {
  await nativeFixture(page, { ocr_image: [{ value: { kind: 'image', request: 1, image } }] });
  await page.context().route('**/*', (route) => new URL(route.request().url()).hostname === '127.0.0.1' ? route.continue() : route.abort());
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('导入图片', { exact: true }).setInputFiles(fileURLToPath(imagePath));
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('');
  await page.getByRole('button', { name: '识别文字', exact: true }).click();
  const field = page.getByLabel('法语原文', { exact: true });
  await expect(field).toHaveValue(/L'école d'aujourd'hui\.\nOù est le café \?\nÇa coûte 10 euros/);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
  await field.evaluate((node) => { (node as HTMLTextAreaElement).setSelectionRange(2, 7); });
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
});

test('screen region is automatically recognized into the editable translation input', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.evaluate((image) => window.testNative.emit('selection-input', { kind: 'image', request: 7, image, recognize: true }), image);
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(/Où est le café \?/);
  await page.getByLabel('法语原文', { exact: true }).fill('Où est le café ?');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
});

test('missing local language data is visible and recognition can be retried', async ({ page }) => {
  await nativeFixture(page);
  await page.route('**/ocr/fra.traineddata', (route) => route.abort());
  await page.goto('/');
  await page.evaluate((image) => window.testNative.emit('selection-input', { kind: 'image', request: 1, image, recognize: true }), image);
  await expect(page.getByRole('alert').filter({ hasText: '图片或 OCR 处理失败' })).toBeVisible();
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  await page.unroute('**/ocr/fra.traineddata');
  await page.getByRole('button', { name: '识别文字', exact: true }).click();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(/Où est le café \?/);
});

test('pasting a screenshot imports the image and blank recognition stays editable', async ({ page }) => {
  await nativeFixture(page, { ocr_image: [{ value: { kind: 'image', request: 1, image } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('法语原文', { exact: true }).evaluate((field, dataUrl) => {
    const bytes = Uint8Array.from(atob(dataUrl.split(',')[1]!), (character) => character.charCodeAt(0));
    const clipboard = new DataTransfer(); clipboard.items.add(new File([bytes], 'screenshot.png', { type: 'image/png' }));
    field.dispatchEvent(new ClipboardEvent('paste', { clipboardData: clipboard, bubbles: true, cancelable: true }));
  }, image.data_url);
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'ocr_image').length)).toBe(1);
  await page.evaluate(() => {
    const canvas = document.createElement('canvas'); canvas.width = 200; canvas.height = 100;
    const context = canvas.getContext('2d')!; context.fillStyle = 'white'; context.fillRect(0, 0, 200, 100);
    window.testNative.emit('selection-input', { kind: 'image', request: 2, recognize: true,
      image: { width: 200, height: 100, data_url: canvas.toDataURL() } });
  });
  await expect(page.getByRole('alert').filter({ hasText: '没有识别到文字' })).toBeVisible();
  await expect(page.getByRole('alert').filter({ hasText: '识别结果不确定' })).toBeVisible();
  await page.getByLabel('法语原文', { exact: true }).fill('école');
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
});

test('cancelling blocked OCR protects later text edits and another request can complete', async ({ page }) => {
  await nativeFixture(page);
  let blocked = false, release = () => {};
  const gate = new Promise<void>((resolve) => { release = resolve; });
  await page.route('**/ocr/fra.traineddata', async () => { blocked = true; await gate; });
  await page.goto('/');
  await page.evaluate((image) => window.testNative.emit('selection-input', { kind: 'image', request: 1, image, recognize: true }), image);
  await expect.poll(() => blocked).toBe(true);
  await page.getByRole('button', { name: '取消图片', exact: true }).click();
  await page.getByLabel('法语原文', { exact: true }).fill('Une nouvelle phrase.');
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toHaveCount(0);
  release(); await page.unroute('**/ocr/fra.traineddata');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('heading', { name: '设置', exact: true })).toBeVisible();
  await page.evaluate((image) => window.testNative.emit('selection-input', { kind: 'image', request: 2, image, recognize: true }), image);
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(/Où est le café \?/);
  await page.getByLabel('法语原文', { exact: true }).fill('Une nouvelle phrase.');
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('Une nouvelle phrase.');
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
});
