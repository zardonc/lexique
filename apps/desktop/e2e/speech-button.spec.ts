import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

test('dictionary and notebook expose playback failures without creating review cards', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], speech_preview: [{ error: 'voice unavailable' }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: /播放单词: école/ }).click();
  await expect(page.getByRole('alert')).toContainText('voice unavailable');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: /播放单词: école/ }).click();
  await expect(page.getByRole('alert')).toContainText('voice unavailable');
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)).toEqual([{ text: 'école' }, { text: 'école' }]);
  expect(calls.filter((call) => call.command === 'review_add' || call.command === 'vocabulary_save')).toHaveLength(0);
});
