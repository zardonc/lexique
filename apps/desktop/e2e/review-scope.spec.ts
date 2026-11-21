import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const all = { kind: 'all' };
const voyage = { kind: 'vocabulary', category_ids: [7], include_uncategorized: false };
const first = { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
const second = { queue: { cards: [{ id: 3, vocabulary_id: 2, kind: 'recognition' }], reason: null, available_at: null }, record: { ...record, id: 2, headword: 'voyage', category_ids: [7] }, audio_pending: false, audio_error: null };
const rated = { status: 'committed', result: { card: { phase: 'learning_first', memory: { stability: 1, difficulty: 5 }, last_review_at: 1000, due_at: 1060 }, elapsed_days: 0, step_seconds: 60, interval_days: null,
  model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } };

function windowView(view: typeof first, scope: typeof all | typeof voyage, context: { readonly revision: number; readonly before?: number }) {
  const { revision, before = null } = context;
  return { ...view, summary: { scope, revision, admitted_count: 1, audio_pending_count: 0, missing_vocabulary_ids: [], verification_pending: false, learning_day: '2026-11-21', timezone: 'UTC', unknown_legacy: false },
    window: view.queue.cards.map((card) => ({ card, record: view.record, progress: { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } }, version: null, content_hash: null, audio: null })),
    delta: { before_revision: before, after_revision: revision, removed: [], order: view.queue.cards.map((card) => card.id) },
    metrics: { metadata_candidates: 1, strict_probes: 0, compressed_bytes: 0, payload_bytes: 0, validation_wait_ms: 0, metadata_loaded_rows: 1, parsed_progress: 0, metadata_rebuilt: true, rebuild_reason: 'context', database_ms: 0 } };
}

test('category selection waits for skip, saves once, and restores on re-entry', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_queue: [{ value: first }, { value: second }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  await page.getByRole('combobox', { name: '复习范围', exact: true }).selectOption('7');
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage')).toBeVisible();
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope))).toHaveLength(0);
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(card.getByRole('heading', { name: 'voyage' })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope)).toEqual([voyage]);
  expect(calls.filter((call) => call.command === 'review_rate')).toHaveLength(0);
  const reopened = await page.context().newPage();
  await nativeFixture(reopened, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_scope: [{ value: voyage }], review_queue: [{ value: second }] });
  await reopened.goto('/');
  await reopened.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(reopened.getByRole('combobox', { name: '复习范围', exact: true })).toHaveValue('7');
  await reopened.close();
});

test('failed rating retains the answer and pending choice; scope-save retry cannot repeat a rating', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_scope: [{ value: all }, { value: all }, { error: 'disk unavailable' }, { value: voyage }],
    review_queue: [{ value: first }, { value: second }], review_rate: [{ error: 'rating connection lost' }, { value: rated }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('combobox', { name: '复习范围', exact: true }).selectOption('7');
  await card.getByRole('button', { name: '记得', exact: true }).click();
  await expect(card.getByRole('alert')).toContainText('rating connection lost');
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage')).toBeVisible();
  await card.getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('disk unavailable');
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
  await expect(page.getByRole('combobox', { name: '复习范围', exact: true })).toHaveValue('7');
  await page.getByRole('button', { name: '重试保存复习范围' }).click();
  await expect(page.getByText('当前范围：Voyage')).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  const ratings = calls.filter((call) => call.command === 'review_rate');
  expect(ratings).toHaveLength(2);
  expect(ratings[1].args?.request).toEqual(ratings[0].args?.request);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(2);
});

test('scope read failures remain visible and prevent mixed queue reads until retry', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ error: 'scope read failed' }, { error: 'scope read failed' }, { value: all }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('scope read failed');
  await expect(page.getByRole('combobox', { name: '复习范围', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue'))).toHaveLength(0);
  await page.getByRole('button', { name: '重读复习范围' }).click();
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
});

test('pending scope remains transient after restart', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_queue: [{ value: first }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await page.getByRole('combobox', { name: '复习范围', exact: true }).selectOption('7');
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage')).toBeVisible();
  await page.reload();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('combobox', { name: '复习范围', exact: true })).toHaveValue('all');
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage')).toHaveCount(0);
});

test('late old-scope response is discarded and the new scope starts without its cursor', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_queue: [
    { value: windowView(first, all, { revision: 10 }) }, { value: windowView(first, all, { revision: 11, before: 10 }), gate: 'old-scope' }, { value: windowView(second, voyage, { revision: 12 }) },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue').length)).toBe(2);
  await page.getByRole('combobox', { name: '复习范围', exact: true }).selectOption('7');
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(page.getByText('当前范围：Voyage')).toBeVisible();
  await page.evaluate(() => window.testNative.release('old-scope'));
  await expect(page.getByRole('region', { name: '复习卡片' }).getByRole('heading', { name: 'voyage' })).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);
  const reads = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue'));
  expect(reads[1].args?.cursor).toMatchObject({ scope: all, revision: 10 });
  expect(reads[2].args?.cursor).toBeNull();
});

test('an obsolete delta cannot replace a card in the applied scope', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_scope: [{ value: voyage }], review_queue: [
    { value: windowView(second, voyage, { revision: 20 }) }, { value: windowView(first, all, { revision: 21, before: 20 }) },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await expect(card.getByRole('heading', { name: 'voyage' })).toBeVisible();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect(page.getByRole('alert')).toContainText('Queue delta scope mismatch');
  await expect(card.getByRole('heading', { name: 'voyage' })).toBeVisible();
  await expect(card.getByRole('heading', { name: 'école' })).toHaveCount(0);
});

test('applied and pending scopes follow the English and French interface language', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_queue: [{ value: first }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await page.getByRole('combobox', { name: '复习范围', exact: true }).selectOption('7');
  await page.getByRole('combobox', { name: '界面语言' }).selectOption('en');
  await expect(page.getByRole('combobox', { name: 'Review scope', exact: true })).toHaveValue('7');
  await expect(page.getByText('Applied scope: All reviews')).toBeVisible();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage')).toBeVisible();
  await page.getByRole('combobox', { name: 'Interface language' }).selectOption('fr');
  await expect(page.getByRole('combobox', { name: 'Périmètre de révision', exact: true })).toHaveValue('7');
  await expect(page.getByText('Périmètre actuel : Toutes les révisions')).toBeVisible();
  await expect(page.getByText('S’appliquera après l’évaluation ou le passage de la carte actuelle : Voyage')).toBeVisible();
});

test('malformed saved scope fails contract validation before any queue read', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ value: { ...voyage, category_ids: [7, 7] } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('复习范围读取失败');
  await expect(page.getByRole('combobox', { name: '复习范围', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue'))).toHaveLength(0);
});

test('scoped listening playback failure retains the normal rating restriction', async ({ page }) => {
  await page.addInitScript(() => { HTMLMediaElement.prototype.play = () => Promise.reject(new Error('speaker unavailable')); });
  const listening = { ...second, queue: { ...second.queue, cards: [{ id: 4, vocabulary_id: 2, kind: 'listening' }] } };
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_scope: [{ value: voyage }], review_queue: [{ value: listening }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await expect(card.getByRole('alert')).toContainText('speaker unavailable');
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByRole('button', { name: '记得', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate'))).toHaveLength(0);
});
