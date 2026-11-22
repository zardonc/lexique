import { expect, test } from '@playwright/test';
import { nativeFixture, nativeFixtureSetup, record } from './native-fixture';
import type { ReviewScope } from '../src/platform/review-scope';

const scope: ReviewScope = { kind: 'vocabulary', category_ids: [7], include_uncategorized: false };
const selection = { kind: 'needs_selection' };
const progress = { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } };
const card = { id: 1, vocabulary_id: 1, kind: 'recognition' };
const rated = { status: 'committed', result: { card: { phase: 'learning_first', memory: { stability: 1, difficulty: 5 }, last_review_at: 1000, due_at: 1060 }, elapsed_days: 0, step_seconds: 60, interval_days: null,
  model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } };

function view(applied: ReviewScope = scope, cards = [card], revision = 1) {
  return { queue: { cards, reason: cards.length ? null : applied.kind === 'needs_selection' ? 'needs_selection' : 'empty_scope', available_at: null }, record: cards.length ? record : null,
    audio_pending: false, audio_error: null, audio_vocabulary_id: null,
    summary: { scope: applied, revision, admitted_count: cards.length, audio_pending_count: 0, missing_vocabulary_ids: [], verification_pending: false, learning_day: '2026-11-21', timezone: 'UTC', unknown_legacy: false },
    window: cards.map((card) => ({ card, record: { ...record, id: card.vocabulary_id, headword: card.vocabulary_id === 1 ? 'école' : 'voyage', category_ids: [7] }, progress, version: null, content_hash: null, audio: null })),
    delta: { before_revision: null, after_revision: revision, removed: [], order: cards.map((card) => card.id) },
    metrics: { metadata_candidates: cards.length, strict_probes: 0, compressed_bytes: 0, payload_bytes: 0, validation_wait_ms: 0, metadata_loaded_rows: cards.length, parsed_progress: 0, metadata_rebuilt: true, rebuild_reason: 'collection', database_ms: 0 } };
}

test('a committed rename refreshes the selected label without interrupting the presentation', async ({ page }) => {
  await nativeFixtureSetup(page, ({ scope, queue }) => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => {
      const response = await invoke(command, args);
      if (command === 'review_scope') return scope;
      if (command === 'review_queue') return queue;
      if (command === 'learning_categories') return [{ id: 7, name: Reflect.get(window, 'categoryEditCommitted') ? 'Travel' : 'Voyage' }];
      return response;
    };
  }, { scope, queue: view() });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByText('当前范围：Voyage', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await page.evaluate(() => { Reflect.set(window, 'categoryEditCommitted', true); window.testNative.emit('lexique-review-changed'); });
  await expect(page.getByText('当前范围：Travel', { exact: true })).toBeVisible();
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: 'Travel' })).toBeChecked();
  await expect(page.getByRole('region', { name: '复习卡片' }).getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByRole('button', { name: '记得', exact: true })).toBeEnabled();
});

test('deleting the last condition preserves the current answer until rating then requires selection', async ({ page }) => {
  await nativeFixtureSetup(page, ({ scope, selection, before, after, rated }) => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => {
      const response = await invoke(command, args);
      const deleted = Reflect.get(window, 'categoryEditCommitted');
      if (command === 'review_scope') return deleted ? selection : scope;
      if (command === 'review_queue') return deleted ? after : before;
      if (command === 'learning_categories') return deleted ? [] : [{ id: 7, name: 'Voyage' }];
      if (command === 'review_rate') return rated;
      return response;
    };
  }, { scope, selection, before: view(), after: view({ kind: 'needs_selection' }, [], 2), rated });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByText('当前范围：Voyage', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await page.evaluate(() => { Reflect.set(window, 'categoryEditCommitted', true); window.testNative.emit('lexique-review-changed'); });
  await expect(page.getByText('当前范围：请选择复习范围', { exact: true })).toBeVisible();
  await expect(page.getByRole('region', { name: '复习卡片' }).getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByRole('button', { name: '记得', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toHaveCount(0);
  await expect(page.getByText('请选择复习范围', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'review_rate')?.args?.presentationRevision)).toBe(0);
  await expect(page.getByRole('group', { name: '复习范围', exact: true }).getByRole('checkbox', { name: '全部复习' })).not.toBeChecked();
});

test('saved membership removal leaves the current card finishable with no following scoped cards', async ({ page }) => {
  await nativeFixture(page, { review_scope: [{ value: scope }], learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_queue: [{ value: view() }, { value: view(scope, [], 2) }], review_rate: [{ error: 'save unavailable' }, { value: rated }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect(page.getByText('此范围内没有已加入复习的生词', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '记得', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByText(/save unavailable/)).toBeVisible();
  await page.getByRole('region', { name: '复习卡片' }).getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toHaveCount(0);
  const requests = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate').map((call) => call.args));
  expect(requests).toHaveLength(2);
  expect(requests[1]).toEqual(requests[0]);
});

test('a response from before a membership commit cannot promote an excluded following card', async ({ page }) => {
  const before = view(scope, [card, { id: 3, vocabulary_id: 2, kind: 'recognition' }]);
  await nativeFixture(page, { review_scope: [{ value: scope }], learning_categories: [{ value: [{ id: 7, name: 'Voyage' }] }],
    review_queue: [{ value: before }, { value: before, gate: 'before-membership-save' }, { value: view(scope, [], 2) }], review_rate: [{ value: rated }] });
  await page.goto('/');
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue').length)).toBe(2);
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByRole('region', { name: '复习卡片' })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('before-membership-save'));
  await expect(page.getByText('此范围内没有已加入复习的生词', { exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'voyage', exact: true })).toHaveCount(0);
});
