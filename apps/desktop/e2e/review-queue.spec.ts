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

for (const phase of ['new', 'learning_first', 'review'] as const) {
  test(`undo restores a ${phase} card and permits continued rating`, async ({ page }) => {
    const card = { phase, memory: phase === 'new' ? null : { stability: 1, difficulty: 5 },
      last_review_at: phase === 'new' ? null : 1700000000, due_at: phase === 'new' ? null : 1700000060 };
    const empty = { queue: { cards: [], reason: null, available_at: null }, record: null, audio_pending: false, audio_error: null };
    const restored = { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
    await nativeFixture(page, {
      review_queue: [{ value: empty }, { value: empty }, { value: restored }, { value: empty }],
      review_undo_target: [{ value: 'last-rating' }],
      review_progress: [{ value: { revision: 2, card } }],
      review_rate: [{ value: { status: 'committed', result: { card, elapsed_days: 0, step_seconds: null, interval_days: 1,
        model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } } }],
    });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    await page.getByRole('button', { name: '撤销最近评分' }).click();
    await page.getByRole('button', { name: '查看识义卡' }).click();
    await page.getByRole('button', { name: '显示答案' }).click();
    await page.getByRole('button', { name: '记得', exact: true }).click();
    await expect(page.getByText('本轮完成', { exact: true })).toBeVisible();
    const rating = (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_rate');
    expect(rating).toHaveLength(1);
    expect(rating[0].args?.request).toMatchObject({ expected_revision: 2, card_id: 1 });
  });
}

test('pending undo survives page reload and retries its original identity', async ({ page }) => {
  await nativeFixture(page, { review_undo_target: [{ value: null }] });
  await page.goto('/');
  await page.evaluate(() => localStorage.setItem('lexique-pending-undo', JSON.stringify({ operation: 'lost-response', ratingOperation: 'prior-day-rating' })));
  await page.reload();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '撤销最近评分' }).click();
  await expect.poll(() => page.evaluate(() => localStorage.getItem('lexique-pending-undo'))).toBeNull();
  const calls = (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_undo');
  expect(calls.map((call) => call.args)).toEqual([{ operation: 'lost-response', ratingOperation: 'prior-day-rating' }]);
});

test('undo retries the same operation after a lost response', async ({ page }) => {
  await nativeFixture(page, { review_undo_target: [{ value: 'rating-1' }], review_undo: [{ error: 'connection lost' }, { value: { status: 'committed', queue_rebuilt: false } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分' });
  await expect(undo).toBeEnabled();
  await undo.click();
  await expect(page.getByRole('alert')).toContainText('connection lost');
  await undo.click();
  await expect.poll(async () => (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_undo').length).toBe(2);
  const calls = (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_undo');
  expect(calls[0].args).toEqual(calls[1].args);
});

test('a rejected undo refreshes the target and permits a new attempt', async ({ page }) => {
  await nativeFixture(page, { review_undo_target: [{ value: 'old' }, { value: 'old' }, { value: 'new' }], review_undo: [{ value: { status: 'rejected', detail: 'stale' } }, { value: { status: 'committed', queue_rebuilt: false } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分' });
  await expect(undo).toBeEnabled();
  await undo.click();
  await expect(page.getByRole('alert')).toContainText('stale');
  await expect(undo).toBeEnabled();
  await undo.click();
  await expect.poll(async () => (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_undo').length).toBe(2);
  const calls = (await page.evaluate(() => window.testNative.calls)).filter((call) => call.command === 'review_undo');
  expect(calls[0].args?.ratingOperation).toBe('old');
  expect(calls[1].args?.ratingOperation).toBe('new');
});

test('legacy undo explains that the queue was rebuilt', async ({ page }) => {
  await nativeFixture(page, { review_undo_target: [{ value: 'legacy' }], review_undo: [{ value: { status: 'committed', queue_rebuilt: true } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分' });
  await expect(undo).toBeEnabled();
  await undo.click();
  await expect(page.getByText('旧版评分已撤销，队列顺序可能变化。')).toBeVisible();
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
