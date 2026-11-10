import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';
import { readFileSync } from 'node:fs';

declare global { interface Window { testSpeechAudio: HTMLAudioElement[] } }

test('WPM controls support all steps, keyboard endpoints, presets and persisted choices', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('female');
  const speed = page.getByRole('slider', { name: '朗读语速' });
  await expect(speed).toHaveValue('180');
  await speed.focus();
  await speed.press('Home');
  for (let wpm = 80; wpm <= 240; wpm += 20) {
    await expect(speed).toHaveValue(String(wpm));
    await expect(page.locator('output')).toHaveText(`${wpm} WPM`);
    if (wpm < 240) await speed.press('ArrowRight');
  }
  await speed.press('Home');
  await expect(speed).toHaveValue('80');
  await speed.press('End');
  await expect(speed).toHaveValue('240');
  await page.getByRole('button', { name: '正常 180 WPM', exact: true }).click();
  await expect(speed).toHaveValue('180');
  await page.getByRole('button', { name: '快速 240 WPM', exact: true }).click();
  await expect(speed).toHaveValue('240');
  await page.getByLabel('法语声线').selectOption('male');
  await page.getByRole('button', { name: '慢速 100 WPM', exact: true }).click();
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'speech_preferences' && call.args?.preferences)))
    .toEqual({ command: 'speech_preferences', args: { preferences: { voice: 'male', wpm: 100 } } });
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('male');
  await expect(page.getByLabel('朗读语速')).toHaveValue('100');
  await page.screenshot({ path: 'test-results/speech-settings.png', fullPage: true });
});

test('failed speech save retains choices and permits a successful retry', async ({ page }) => {
  await nativeFixture(page, { speech_preferences: [
    { value: { voice: 'female', wpm: 180 } },
    { value: { voice: 'female', wpm: 180 } },
    { error: 'disk is read-only' },
    { value: { voice: 'male', wpm: 100 } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('法语声线')).toBeEnabled();
  await page.getByLabel('法语声线').selectOption('male');
  await page.getByRole('button', { name: '慢速 100 WPM', exact: true }).click();
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByRole('alert')).toContainText('disk is read-only');
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toHaveCount(0);
  await expect(page.getByLabel('法语声线')).toHaveValue('male');
  await expect(page.getByLabel('朗读语速')).toHaveValue('100');
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
});

test('invalid saved preferences stay visible as an error and can be reloaded', async ({ page }) => {
  await nativeFixture(page, { speech_preferences: [
    { value: { voice: 'external', wpm: 180 } },
    { value: { voice: 'external', wpm: 180 } },
    { value: { voice: 'male', wpm: 100 } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('语音偏好操作失败');
  await expect(page.getByRole('button', { name: '保存语音偏好' })).toBeDisabled();
  await page.locator('.speech-settings').getByRole('button', { name: '重新读取' }).click();
  await expect(page.getByLabel('法语声线')).toHaveValue('male');
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

test('sentence preview uses draft WPM and voice; saving changes only the next playback', async ({ page }) => {
  const bytes = [...readFileSync(new URL('../../../docs/evidence/issue-56/female-1.ogg', import.meta.url))];
  await page.addInitScript(() => {
    const Audio = window.Audio;
    window.testSpeechAudio = [];
    window.Audio = class extends Audio {
      constructor(src?: string) { super(src); window.testSpeechAudio.push(this); }
    };
  });
  await nativeFixture(page, { speech_preview: [{ value: { bytes, voice: 'female' } }, { value: { bytes, voice: 'male' } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const speed = page.getByRole('slider', { name: '朗读语速' });
  await speed.focus(); await speed.press('Home');
  const preview = page.getByRole('button', { name: /^试听短句:/ });
  await preview.click();
  await expect.poll(() => page.evaluate(() => window.testSpeechAudio.length)).toBe(1);
  expect(await page.evaluate(() => window.testSpeechAudio[0].playbackRate)).toBeCloseTo(0.535989389, 7);
  expect(await page.evaluate(() => window.testSpeechAudio[0].preservesPitch)).toBe(true);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preferences' && call.args?.preferences))).toHaveLength(0);
  await page.getByLabel('法语声线').selectOption('male');
  await page.getByRole('button', { name: '快速 240 WPM', exact: true }).click();
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testSpeechAudio[0].paused)).toBe(false);
  expect(await page.evaluate(() => window.testSpeechAudio[0].playbackRate)).toBeCloseTo(0.535989389, 7);
  await preview.click();
  await expect.poll(() => page.evaluate(() => window.testSpeechAudio.length)).toBe(2);
  expect(await page.evaluate(() => window.testSpeechAudio[0].paused)).toBe(true);
  expect(await page.evaluate(() => window.testSpeechAudio[1].playbackRate)).toBeCloseTo(1.609130193, 7);
  await expect(page.getByRole('alert')).toHaveCount(0);
});
