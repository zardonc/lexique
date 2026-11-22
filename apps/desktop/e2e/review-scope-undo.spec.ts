import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const scope = { kind: 'vocabulary', category_ids: [7], include_uncategorized: false };
const voyage = { ...record, id: 2, headword: 'voyage', category_ids: [7] };
const train = { ...record, id: 3, headword: 'train', category_ids: [7] };

function queue(words: readonly (typeof voyage)[]) {
  return { queue: { cards: words.map((word) => ({ id: word.id * 2 - 1, vocabulary_id: word.id, kind: 'recognition' })), reason: null, available_at: null },
    record: words[0] ?? null, audio_pending: false, audio_error: null };
}

test('a rejected global undo stays visible and never substitutes an older scoped target', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ value: scope }], review_queue: [{ value: queue([voyage]) }],
    review_undo_target: [{ value: 'global-latest' }, { value: 'global-latest' }, { value: null }],
    review_undo: [{ value: { status: 'rejected', detail: 'Rating revision conflict' } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分', exact: true });
  await expect(undo).toBeEnabled();
  await undo.click();
  await expect(page.getByRole('alert')).toContainText('Rating revision conflict');
  await expect(undo).toBeDisabled();
  await expect(page.getByRole('heading', { name: 'voyage', exact: true })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'review_undo').map((call) => call.args?.ratingOperation)).toEqual(['global-latest']);
  expect(calls.filter((call) => call.command === 'review_queue')).toHaveLength(1);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(0);
});

test('a lost global undo response reopens with the applied scope and retries the original request', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ value: scope }], review_queue: [{ value: queue([voyage]) }],
    review_undo_target: [{ value: 'other-category-rating' }], review_undo: [{ error: 'connection lost' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '撤销最近评分', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('connection lost');
  await expect(page.getByRole('heading', { name: 'voyage', exact: true })).toBeVisible();
  const request = await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'review_undo')?.args);
  const reopened = await page.context().newPage();
  await nativeFixture(reopened, { review_scope: [{ value: scope }], review_queue: [{ value: queue([voyage]) }], review_undo_target: [{ value: null }] });
  await reopened.goto('/');
  await reopened.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await reopened.getByRole('button', { name: '撤销最近评分', exact: true }).click();
  await expect.poll(() => reopened.evaluate(() => localStorage.getItem('lexique-pending-undo'))).toBeNull();
  await expect(reopened.getByRole('heading', { name: 'voyage', exact: true })).toBeVisible();
  const calls = await reopened.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'review_undo').map((call) => call.args)).toEqual([request]);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(0);
  await reopened.close();
});

test('global undo keeps a pending checkbox choice when the applied queue becomes empty', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    review_scope: [{ value: scope }], learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_queue: [{ value: queue([voyage]) }, { value: queue([]) }], review_undo_target: [{ value: 'all-reviews-grammar-rating' }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Flashcards', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'voyage', exact: true })).toBeVisible();
  await page.getByRole('checkbox', { name: 'Uncategorized', exact: true }).check();
  const pending = page.getByText('Applies after rating or skipping the current card: Voyage, Uncategorized', { exact: true });
  await expect(pending).toBeVisible();
  await page.getByRole('button', { name: 'Undo last rating', exact: true }).click();
  await expect(page.getByText('Review complete', { exact: true })).toBeVisible();
  await expect(page.getByText('Applied scope: Voyage', { exact: true })).toBeVisible();
  await expect(pending).toBeVisible();
  await expect(page.getByRole('checkbox', { name: 'Uncategorized', exact: true })).toBeChecked();
  await expect(page.getByRole('region', { name: 'Review card', exact: true })).toHaveCount(0);
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'review_undo').map((call) => call.args?.ratingOperation)).toEqual(['all-reviews-grammar-rating']);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(0);
  expect(calls.filter((call) => call.command === 'review_rate')).toHaveLength(0);
});

for (const target of ['other-category-rating', 'all-reviews-grammar-rating']) {
  test(`global undo of ${target} preserves the applied scope and skipped cards`, async ({ page }) => {
    await nativeFixture(page, { review_scope: [{ value: scope }], learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
      review_queue: [{ value: queue([voyage]) }, { value: queue([train]) }], review_undo_target: [{ value: target }] });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    const card = page.getByRole('region', { name: '复习卡片' });
    await expect(card.getByRole('heading', { name: 'voyage', exact: true })).toBeVisible();
    await page.getByRole('button', { name: '跳过', exact: true }).click();
    await expect(card.getByRole('heading', { name: 'train', exact: true })).toBeVisible();
    await page.getByRole('button', { name: '撤销最近评分', exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue').length)).toBe(3);
    await expect(card.getByRole('heading', { name: 'train', exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'école', exact: true })).toHaveCount(0);
    const calls = await page.evaluate(() => window.testNative.calls);
    expect(calls.filter((call) => call.command === 'review_undo').map((call) => call.args?.ratingOperation)).toEqual([target]);
    expect(calls.filter((call) => call.command === 'review_queue').at(-1)?.args?.skipped).toEqual([3]);
    expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(0);
    expect(calls.filter((call) => call.command === 'review_rate')).toHaveLength(0);
  });
}
