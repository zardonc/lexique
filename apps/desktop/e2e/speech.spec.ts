import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('speech settings save both choices and restore them when reopened', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('siwis');
  await page.getByLabel('法语声线').selectOption('tom');
  await page.getByLabel('朗读语速').selectOption('slow');
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'speech_preferences' && call.args?.preferences)))
    .toEqual({ command: 'speech_preferences', args: { preferences: { voice: 'tom', speed: 'slow' } } });
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('tom');
  await expect(page.getByLabel('朗读语速')).toHaveValue('slow');
  await page.screenshot({ path: 'test-results/speech-settings.png', fullPage: true });
});

test('failed speech save retains choices and permits a successful retry', async ({ page }) => {
  await nativeFixture(page, { speech_preferences: [
    { value: { voice: 'siwis', speed: 'normal' } },
    { value: { voice: 'siwis', speed: 'normal' } },
    { error: 'disk is read-only' },
    { value: { voice: 'tom', speed: 'slow' } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toBeEnabled();
  await page.getByLabel('法语声线').selectOption('tom');
  await page.getByLabel('朗读语速').selectOption('slow');
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByRole('alert')).toContainText('disk is read-only');
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toHaveCount(0);
  await expect(page.getByLabel('法语声线')).toHaveValue('tom');
  await expect(page.getByLabel('朗读语速')).toHaveValue('slow');
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
});

test('invalid saved preferences stay visible as an error and can be reloaded', async ({ page }) => {
  await nativeFixture(page, { speech_preferences: [
    { value: { voice: 'external', speed: 'normal' } },
    { value: { voice: 'external', speed: 'normal' } },
    { value: { voice: 'tom', speed: 'slow' } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('语音偏好操作失败');
  await expect(page.getByRole('button', { name: '保存语音偏好' })).toBeDisabled();
  await page.getByRole('button', { name: '重新读取' }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('tom');
  await expect(page.getByLabel('朗读语速')).toBeEnabled();
});

test('browser preview explains persistence limits in all languages at mobile width', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByText('浏览器预览：请在桌面应用中保存语音偏好。')).toBeVisible();
  await expect(page.getByRole('button', { name: '保存语音偏好' })).toBeDisabled();
  await page.getByLabel('界面语言').selectOption('en');
  await expect(page.getByLabel('French voice')).toBeDisabled();
  await page.getByLabel('Interface language').selectOption('fr');
  await expect(page.getByLabel('Vitesse de lecture')).toBeDisabled();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: 'test-results/speech-settings-mobile.png', fullPage: true });
});
