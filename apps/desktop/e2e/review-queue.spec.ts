import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

test('review queue displays a card and refreshes after rating', async ({ page }) => {
  const progress = { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } };
  await nativeFixture(page, { review_queue: [
    { value: { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null } },
    { value: { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null } },
    { value: { queue: { cards: [], reason: 'due', available_at: 1_800_000_000 }, record: null, audio_pending: false, audio_error: null } },
  ], review_rate: [{ value: { status: 'committed', result: { card: progress.card, elapsed_days: 0, step_seconds: 60, interval_days: null, model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByText(/尚未到期/)).toBeVisible();
});

test('browser preview reports review queue unavailable', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByText('浏览器预览无法读取复习队列。')).toBeVisible();
});

test('skipping failed listening playback advances without rating', async ({ page }) => {
  await page.addInitScript(() => { HTMLMediaElement.prototype.play = () => Promise.reject(new Error('speaker unavailable')); });
  const listening = { queue: { cards: [{ id: 2, vocabulary_id: 1, kind: 'listening' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
  const recognition = { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
  await nativeFixture(page, { review_queue: [{ value: listening }, { value: listening }, { value: recognition }], speech_cached: [{ value: [79, 103, 103, 83] }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  await expect(page.getByRole('alert')).toContainText('speaker unavailable');
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(page.getByRole('button', { name: '查看识义卡' })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'review_rate')).toHaveLength(0);
  expect(calls.filter((call) => call.command === 'review_queue').at(-1)?.args?.skipped).toEqual([2]);
});
