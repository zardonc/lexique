import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const all = { kind: 'all' };
const voyage = { kind: 'vocabulary', category_ids: [7], include_uncategorized: false };
const combined = { kind: 'vocabulary', category_ids: [7, 8], include_uncategorized: true };
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

test('several categories and Uncategorized apply together after skip and restore together', async ({ page }) => {
  const categories = [{ id: 7, name: 'Voyage' }, { id: 8, name: 'École' }];
  await nativeFixture(page, { learning_categories: [{ value: categories }], review_queue: [{ value: first }, { value: second }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const selector = page.getByRole('group', { name: '复习范围', exact: true });
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await selector.getByRole('checkbox', { name: 'Voyage', exact: true }).check();
  await selector.getByRole('checkbox', { name: 'École', exact: true }).check();
  await selector.getByRole('checkbox', { name: '未分类', exact: true }).check();
  await expect(selector.getByRole('checkbox', { name: '全部复习', exact: true })).not.toBeChecked();
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage, École, 未分类')).toBeVisible();
  await expect(page.getByRole('region', { name: '复习卡片' }).getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope))).toHaveLength(0);
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(page.getByText('当前范围：Voyage, École, 未分类')).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope))).toEqual([combined]);
  const reopened = await page.context().newPage();
  await nativeFixture(reopened, { learning_categories: [{ value: categories }], review_scope: [{ value: combined }], review_queue: [{ value: second }] });
  await reopened.goto('/');
  await reopened.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const restored = reopened.getByRole('group', { name: '复习范围', exact: true });
  for (const name of ['Voyage', 'École', '未分类']) await expect(restored.getByRole('checkbox', { name, exact: true })).toBeChecked();
  await expect(restored.getByRole('checkbox', { name: '全部复习', exact: true })).not.toBeChecked();
  await reopened.close();
});

test('category selection waits for skip, saves once, and restores on re-entry', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_queue: [{ value: first }, { value: second }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true }).check();
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
  await expect(reopened.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true })).toBeChecked();
  await reopened.close();
});

test('clearing the last condition requires an explicit choice after skip and restart', async ({ page }) => {
  const needsSelection = { kind: 'needs_selection' };
  const empty = { queue: { cards: [], reason: 'needs_selection', available_at: null }, record: null, audio_pending: false, audio_error: null };
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_scope: [{ value: voyage }, { value: voyage }, { value: needsSelection }], review_queue: [{ value: second }, { value: empty }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const selector = page.getByRole('group', { name: '复习范围', exact: true });
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await selector.getByRole('checkbox', { name: 'Voyage', exact: true }).uncheck();
  await expect(page.getByText('将在当前卡片评分或跳过后应用：请选择复习范围')).toBeVisible();
  await expect(selector.getByRole('checkbox', { name: '全部复习', exact: true })).not.toBeChecked();
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(page.getByText('当前范围：请选择复习范围')).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByRole('status').filter({ hasText: /^请选择复习范围$/ })).toBeVisible();
  await expect(page.getByText('本轮完成', { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope))).toEqual([needsSelection]);
  const reopened = await page.context().newPage();
  await nativeFixture(reopened, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_scope: [{ value: needsSelection }], review_queue: [{ value: empty }] });
  await reopened.goto('/');
  await reopened.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(reopened.getByText('当前范围：请选择复习范围')).toBeVisible();
  await expect(reopened.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '全部复习', exact: true })).not.toBeChecked();
  await reopened.close();
});

test('failed rating retains the answer and pending choice; scope-save retry cannot repeat a rating', async ({ page }) => {
  const choice = { ...voyage, include_uncategorized: true };
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_scope: [{ value: all }, { value: all }, { error: 'disk unavailable' }, { value: choice }],
    review_queue: [{ value: first }, { value: second }], review_rate: [{ error: 'rating connection lost' }, { value: rated }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true }).check();
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '未分类', exact: true }).check();
  await card.getByRole('button', { name: '记得', exact: true }).click();
  await expect(card.getByRole('alert')).toContainText('rating connection lost');
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage, 未分类')).toBeVisible();
  await card.getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('disk unavailable');
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true })).toBeChecked();
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '未分类', exact: true })).toBeChecked();
  await page.getByRole('button', { name: '重试保存复习范围' }).click();
  await expect(page.getByText('当前范围：Voyage, 未分类')).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  const ratings = calls.filter((call) => call.command === 'review_rate');
  expect(ratings).toHaveLength(2);
  expect(ratings[1].args?.request).toEqual(ratings[0].args?.request);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope)).toHaveLength(2);
  expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope)).toEqual([choice, choice]);
});

for (const completion of ['rating', 'skip'] as const) {
  test(`failed scope save after ${completion} holds the completed boundary with another eligible old-scope card`, async ({ page }) => {
    const oldWindow = windowView(first, all, { revision: 1 });
    const nextWindow = windowView(second, all, { revision: 2 });
    const oldCards = { ...oldWindow, summary: { ...oldWindow.summary, admitted_count: 2 },
      queue: { ...oldWindow.queue, cards: [...first.queue.cards, ...second.queue.cards] },
      window: [...oldWindow.window, ...nextWindow.window], delta: { ...oldWindow.delta, order: [1, 3] } };
    await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
      review_scope: [{ value: all }, { value: all }, { error: 'scope disk unavailable' }, { value: voyage }],
      review_queue: [{ value: oldCards }, { value: nextWindow }, { value: windowView(second, voyage, { revision: 3 }) }],
      review_rate: [{ value: rated }] });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    const card = page.getByRole('region', { name: '复习卡片' });
    const selector = page.getByRole('group', { name: '复习范围', exact: true });
    await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
    await selector.getByRole('checkbox', { name: 'Voyage', exact: true }).check();
    if (completion === 'rating') {
      await card.getByRole('button', { name: '显示答案' }).click();
      await card.getByRole('button', { name: '记得', exact: true }).click();
    } else await card.getByRole('button', { name: '跳过', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText('scope disk unavailable');
    await expect(page.getByText('当前范围：全部复习')).toBeVisible();
    await expect(selector.getByRole('checkbox', { name: 'Voyage', exact: true })).toBeChecked();
    await expect(card).toHaveCount(0);
    await page.getByRole('button', { name: '重试保存复习范围' }).click();
    await expect(page.getByText('当前范围：Voyage')).toBeVisible();
    await expect(card.getByRole('heading', { name: 'voyage' })).toBeVisible();
    const calls = await page.evaluate(() => window.testNative.calls);
    expect(calls.filter((call) => call.command === 'review_rate')).toHaveLength(completion === 'rating' ? 1 : 0);
    expect(calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope)).toEqual([voyage, voyage]);
  });
}

test('scope read failures remain visible and prevent mixed queue reads until retry', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ error: 'scope read failed' }, { error: 'scope read failed' }, { value: all }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('scope read failed');
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '全部复习', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue'))).toHaveLength(0);
  await page.getByRole('button', { name: '重读复习范围' }).click();
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
});

for (const diagnostic of [
  { reason: 'empty_scope', pending: false, error: null, label: '此范围内没有已加入复习的生词' },
  { reason: 'paused', pending: false, error: null, label: '复习已暂停' },
  { reason: 'due', pending: false, error: null, label: '尚未到期' },
  { reason: 'quota', pending: false, error: null, label: '今日额度已用完' },
  { reason: null, pending: true, error: null, label: '听音卡：待生成语音' },
  { reason: null, pending: false, error: 'audio unavailable', label: 'audio unavailable' },
]) {
  test(`combined empty scope retains its selection and ${diagnostic.reason ?? diagnostic.label} diagnostic`, async ({ page }) => {
    await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }, { id: 8, name: 'École' }] }], review_scope: [{ value: combined }],
      review_queue: [{ value: { queue: { cards: [], reason: diagnostic.reason, available_at: diagnostic.reason === 'due' ? 2_000_000_000 : null }, record: null, audio_pending: diagnostic.pending, audio_error: diagnostic.error } }] });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    const selector = page.getByRole('group', { name: '复习范围', exact: true });
    for (const name of ['Voyage', 'École', '未分类']) await expect(selector.getByRole('checkbox', { name, exact: true })).toBeChecked();
    await expect(selector.getByRole('checkbox', { name: '全部复习', exact: true })).not.toBeChecked();
    await expect(page.getByText('当前范围：Voyage, École, 未分类')).toBeVisible();
    await expect(page.getByRole(diagnostic.error ? 'alert' : 'status').filter({ hasText: diagnostic.label })).toBeVisible();
    if (diagnostic.reason === 'empty_scope' || diagnostic.reason === 'paused') {
      await page.getByRole('combobox', { name: '界面语言' }).selectOption('en');
      await expect(page.getByText('Applied scope: Voyage, École, Uncategorized')).toBeVisible();
      await expect(page.getByText(diagnostic.reason === 'paused' ? 'Review paused' : 'No enrolled vocabulary in this scope', { exact: true })).toBeVisible();
      await page.getByRole('combobox', { name: 'Interface language' }).selectOption('fr');
      await expect(page.getByText('Périmètre actuel : Voyage, École, Sans catégorie')).toBeVisible();
      await expect(page.getByText(diagnostic.reason === 'paused' ? 'Révisions suspendues' : 'Aucun vocabulaire inscrit dans ce périmètre', { exact: true })).toBeVisible();
    }
    expect(await page.evaluate(() => window.testNative.calls.filter((call) => ['review_add', 'review_add_batch', 'review_rate'].includes(call.command) || call.command === 'review_scope' && call.args?.scope))).toHaveLength(0);
  });
}

test('All reviews replaces category conditions and clearing it never widens the scope', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const selector = page.getByRole('group', { name: '复习范围', exact: true });
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
  await selector.getByRole('checkbox', { name: '未分类', exact: true }).check();
  await expect(page.getByText('当前范围：未分类', { exact: true })).toBeVisible();
  await selector.getByRole('checkbox', { name: 'Voyage', exact: true }).check();
  await expect(page.getByText('当前范围：Voyage, 未分类')).toBeVisible();
  await selector.getByRole('checkbox', { name: '全部复习', exact: true }).check();
  await expect(page.getByText('当前范围：全部复习')).toBeVisible();
  for (const name of ['Voyage', '未分类']) await expect(selector.getByRole('checkbox', { name, exact: true })).not.toBeChecked();
  await selector.getByRole('checkbox', { name: '全部复习', exact: true }).uncheck();
  await expect(page.getByText('当前范围：请选择复习范围')).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope).map((call) => call.args?.scope))).toEqual([
    { kind: 'vocabulary', category_ids: [], include_uncategorized: true }, { ...voyage, include_uncategorized: true }, all, { kind: 'needs_selection' },
  ]);
});

test('pending scope remains transient after restart', async ({ page }) => {
  await nativeFixture(page, { learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }], review_queue: [{ value: first }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toBeVisible();
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true }).check();
  await expect(page.getByText('将在当前卡片评分或跳过后应用：Voyage')).toBeVisible();
  await page.reload();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '全部复习', exact: true })).toBeChecked();
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
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true }).check();
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
  await page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true }).check();
  await page.getByRole('combobox', { name: '界面语言' }).selectOption('en');
  await expect(page.getByRole('group', { name: 'Review scope', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true })).toBeChecked();
  await expect(page.getByText('Applied scope: All reviews')).toBeVisible();
  await expect(page.getByText('Applies after rating or skipping the current card: Voyage')).toBeVisible();
  await page.getByRole('combobox', { name: 'Interface language' }).selectOption('fr');
  await expect(page.getByRole('group', { name: 'Périmètre de révision', exact: true }).getByRole('checkbox', { name: 'Voyage', exact: true })).toBeChecked();
  await expect(page.getByText('Périmètre actuel : Toutes les révisions')).toBeVisible();
  await expect(page.getByText('S’appliquera après l’évaluation ou le passage de la carte actuelle : Voyage')).toBeVisible();
});

test('malformed saved scope fails contract validation before any queue read', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ value: { ...voyage, category_ids: [7, 7] } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('复习范围读取失败');
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '全部复习', exact: true })).toBeDisabled();
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
