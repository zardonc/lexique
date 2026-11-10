import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { nativeFixture } from './native-fixture';

const screen = { width: 948, height: 159,
  data_url: `data:image/png;base64,${readFileSync(new URL('./fixtures/ocr-french.png', import.meta.url)).toString('base64')}` };

for (const locale of [
  { code: 'en', label: 'Screen selection', image: 'Frozen screen', cancel: 'Cancel capture', empty: 'Select an area.' },
  { code: 'fr', label: 'Sélection de l’écran', image: 'Écran figé', cancel: 'Annuler la capture', empty: 'Sélectionnez une zone.' },
]) test(`capture controls and empty selection use ${locale.code}`, async ({ page }) => {
  await nativeFixture(page, { ocr_screen: [{ value: screen }] });
  await page.goto(`/?capture=7&locale=${locale.code}`);
  await expect(page.getByRole('img', { name: locale.image })).toBeVisible();
  await page.getByRole('application', { name: locale.label }).click({ position: { x: 30, y: 80 } });
  await expect(page.getByRole('alert')).toHaveText(locale.empty);
  await page.getByRole('button', { name: locale.cancel }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.find((call) => call.command === 'ocr_abort')?.args)).toEqual({ request: 7 });
});

test('frozen screen selection converts viewport coordinates to bitmap pixels', async ({ page }) => {
  await nativeFixture(page, { ocr_screen: [{ value: screen }] });
  await page.setViewportSize({ width: 474, height: 318 });
  await page.goto('/?capture=7');
  await expect(page.getByRole('img', { name: '冻结屏幕', exact: true })).toBeVisible();
  await page.mouse.move(50, 100); await page.mouse.down();
  await page.mouse.move(250, 200); await page.mouse.up();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.find((call) => call.command === 'ocr_region')?.args)).toEqual({
    request: 7, region: { x: 100, y: 50, width: 400, height: 50 },
  });
  expect(await page.getByRole('button', { name: '词典', exact: true }).count()).toBe(0);
});

test('keyboard selection submits the chosen region and Escape cancels capture', async ({ page }) => {
  await nativeFixture(page, { ocr_screen: [{ value: screen }] });
  await page.setViewportSize({ width: 474, height: 318 });
  await page.goto('/?capture=7');
  await expect(page.getByRole('img', { name: '冻结屏幕', exact: true })).toBeVisible();
  await page.keyboard.press('Space');
  await page.keyboard.press('Shift+ArrowRight'); await page.keyboard.press('Shift+ArrowDown');
  await page.keyboard.press('Enter');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.find((call) => call.command === 'ocr_region')?.args)).toEqual({
    request: 7, region: { x: 474, y: 79, width: 40, height: 11 },
  });
  await page.keyboard.press('Escape');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.find((call) => call.command === 'ocr_abort')?.args)).toEqual({ request: 7 });
});

test('drag keeps its image origin when the viewport changes during selection', async ({ page }) => {
  await nativeFixture(page, { ocr_screen: [{ value: screen }] });
  await page.setViewportSize({ width: 474, height: 318 });
  await page.goto('/?capture=7');
  await expect(page.getByRole('img', { name: '冻结屏幕', exact: true })).toBeVisible();
  await page.mouse.move(50, 100); await page.mouse.down();
  await page.setViewportSize({ width: 948, height: 159 });
  await page.mouse.move(250, 120); await page.mouse.up();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.find((call) => call.command === 'ocr_region')?.args)).toEqual({
    request: 7, region: { x: 100, y: 50, width: 150, height: 70 },
  });
});
