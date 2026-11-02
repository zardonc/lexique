import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const other = { ...record, id: 2, headword: 'bonjour' };
const committed = { operation: 'fixture', cancelled: false, items: [1, 2].map((vocabulary_id) => ({ vocabulary_id, created_cards: 2, error: null, audio_state: 'queued', audio_error: null })) };

test('T05/T06: navigation stays available during commit and cards report before audio', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record, other] }], review_add_batch: [{ value: committed, gate: 'commit' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByRole('button', { name: '批量加入复习', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_add_batch').length)).toBe(1);
  await expect(page.getByText(/已创建卡片/)).not.toBeVisible();
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.evaluate(() => window.testNative.release('commit'));
  await expect(page.getByText(/已创建卡片: 4/)).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate'))).toEqual([]);
});

test('T04: page capacity and context changes clear only unsubmitted selection', async ({ page }) => {
  const records = Array.from({ length: 53 }, (_, index) => ({ ...record, id: index + 1, headword: 'word-' + index }));
  await nativeFixture(page, { vocabulary_list: [{ value: records }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByRole('list').filter({ has: page.getByText('word-0', { exact: true }) }).locator('li')).toHaveCount(20);
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByRole('button', { name: '下一页', exact: true }).click();
  await expect(page.getByRole('button', { name: '批量加入复习' })).toBeDisabled();
  await page.getByLabel('每页词数').selectOption('50');
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByRole('button', { name: '下一页', exact: true }).click();
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByRole('button', { name: '批量加入复习' }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_add_batch').length)).toBe(1);
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.find((call) => call.command === 'review_add_batch')?.args?.request).toMatchObject({ vocabulary_ids: [51, 52, 53] });
});

test('T05: lost commit reply retries the same snapshot after navigation', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record, other] }], review_add_batch: [{ error: 'lost reply' }, { value: committed }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByLabel('选择本页', { exact: true }).check(); await page.getByRole('button', { name: '批量加入复习' }).click();
  await expect(page.getByRole('alert')).toContainText('lost reply');
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByRole('region', { name: '批量任务', exact: true }).getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByText(/已创建卡片: 4/)).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_add_batch'));
  expect(calls[0].args).toEqual(calls[1].args);
});
