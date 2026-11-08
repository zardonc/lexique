import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { nativeFixture, record } from './native-fixture';
import { notebookMessages } from '../src/notebook-content';

const audio = { bytes: [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))], voice: 'female' };
const progress = { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } };
const entry = (id: number, kind: 'recognition' | 'listening') => ({ card: { id, vocabulary_id: id, kind },
  record: { ...record, id, headword: id === 1 ? 'école' : 'bonjour' }, progress,
  version: kind === 'listening' ? 'a'.repeat(64) : null, content_hash: kind === 'listening' ? 'b'.repeat(64) : null,
  audio: kind === 'listening' ? audio : null });
const entries = [entry(1, 'listening'), entry(2, 'recognition')];
function payload(revision: number, changed = entries, before: number | null = null) {
  return { queue: { cards: entries.map((entry) => entry.card), reason: null, available_at: null }, record: entries[0].record,
    audio_pending: false, audio_error: null, audio_vocabulary_id: null, window: changed,
    summary: { admitted_count: 20 + revision, audio_pending_count: 0, missing_vocabulary_ids: [], verification_pending: false,
      learning_day: '2026-11-04', timezone: 'UTC', revision },
    delta: { before_revision: before, after_revision: revision, removed: [], order: [1, 2] },
    metrics: { metadata_candidates: 20, strict_probes: 0, compressed_bytes: audio.bytes.length, payload_bytes: changed.reduce((sum, entry) => sum + (entry.audio?.bytes.length ?? 0), 0),
      validation_wait_ms: 0, metadata_loaded_rows: 0, parsed_progress: 0, metadata_rebuilt: false, rebuild_reason: null, database_ms: 0 } };
}

test('unverified audio is distinguished from audio awaiting generation', async ({ page }) => {
  const view = payload(1);
  const empty = { ...view, queue: { cards: [], reason: null, available_at: null }, record: null,
    audio_pending: true, window: [], delta: { ...view.delta, order: [] },
    summary: { ...view.summary, verification_pending: true } };
  await nativeFixture(page, { review_queue: [{ value: empty }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByText('听音待校验', { exact: true })).toBeVisible();
  const language = page.locator('header select').first();
  await language.selectOption('en');
  await expect(page.getByText('Audio awaiting verification', { exact: true })).toBeVisible();
  await language.selectOption('fr');
  await expect(page.getByText('Audio en attente de vérification', { exact: true })).toBeVisible();
});

test('unknown legacy repair discloses the voice change in each language', async ({ page }) => {
  const view = payload(1);
  await nativeFixture(page, { review_queue: [{ value: { ...view, summary: { ...view.summary, unknown_legacy: true } } }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByText(notebookMessages.zh.unknownLegacyRepair, { exact: true })).toBeVisible();
  const language = page.locator('header select').first();
  for (const locale of ['en', 'fr'] as const) {
    await language.selectOption(locale);
    await expect(page.getByText(notebookMessages[locale].unknownLegacyRepair, { exact: true })).toBeVisible();
  }
});

test('a revision delta reuses verified bytes without replaying the current answer', async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, 'windowPlays', 0);
    HTMLMediaElement.prototype.play = async () => { Reflect.set(window, 'windowPlays', Number(Reflect.get(window, 'windowPlays')) + 1); };
  });
  await nativeFixture(page, { review_queue: [{ value: payload(1) }, { value: payload(2, [], 1) }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  await expect(page.getByRole('button', { name: '记得', exact: true })).toBeEnabled();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect(page.getByText('待复习：22 张', { exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await page.evaluate(() => Reflect.get(window, 'windowPlays'))).toBe(1);
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'speech_cached')).toHaveLength(0);
  expect(calls.filter((call) => call.command === 'review_queue')[1].args?.cursor).toMatchObject({ revision: 1, window: [{ card: entries[0].card }, { card: entries[1].card }] });
});

test('a mismatched delta stays visible as an error and a full response recovers it', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: payload(1) }, { value: payload(2, [], 0) }, { value: payload(3) }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect(page.getByRole('alert')).toContainText('Queue delta revision mismatch');
  await page.getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByText('待复习：23 张', { exact: true })).toBeVisible();
});

test('a preloaded next card appears while the refill response is still blocked', async ({ page }) => {
  const first = [entry(1, 'recognition'), entry(2, 'recognition')];
  const ready = { ...payload(1, first), queue: { cards: first.map((entry) => entry.card), reason: null, available_at: null }, record: first[0].record };
  const rated = { status: 'committed', result: { card: progress.card, elapsed_days: 0, step_seconds: 60, interval_days: null,
    model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } };
  await nativeFixture(page, { review_queue: [{ value: ready }, { value: ready, gate: 'refill' }], review_rate: [{ value: rated }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案' }).click(); await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue').length)).toBe(2);
  await page.evaluate(() => window.testNative.release('refill'));
});

test('prefetched advancement updates undo and preserves a lost-response retry identity', async ({ page }) => {
  const first = [entry(1, 'recognition'), entry(2, 'recognition')];
  const ready = { ...payload(1, first), queue: { cards: first.map((item) => item.card), reason: null, available_at: null }, record: first[0].record };
  const rated = { status: 'committed', result: { card: progress.card, elapsed_days: 0, step_seconds: 60, interval_days: null,
    model: { version: 'test', parameters: [] }, retention_percent: 90, timezone_database: 'test' } };
  await nativeFixture(page, { review_queue: [{ value: ready }], review_rate: [{ value: rated }],
    review_undo_target: [{ value: null }, { value: null }, { value: 'latest-rating' }],
    review_undo: [{ error: 'Response lost' }, { value: { status: 'committed', queue_rebuilt: false } }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const undo = page.getByRole('button', { name: '撤销最近评分', exact: true });
  await expect(undo).toBeDisabled();
  await page.getByRole('button', { name: '显示答案' }).click(); await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible();
  await expect(undo).toBeEnabled(); await undo.click();
  await expect(page.getByRole('alert')).toContainText('Response lost');
  await undo.click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_undo').length)).toBe(2);
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_undo'));
  expect(calls[0].args).toMatchObject({ ratingOperation: 'latest-rating' });
  expect(calls[1].args).toEqual(calls[0].args);
});

test('a changed progress revision cannot authorize a retained played presentation', async ({ page }) => {
  await page.addInitScript(() => { HTMLMediaElement.prototype.play = async () => {}; });
  await nativeFixture(page, { review_queue: [{ value: payload(1) }, { value: payload(2, [], 1) }],
    review_availability: [{ value: { progress, reason: null, available_at: null } }, { value: { progress, reason: null, available_at: null } },
      { value: { progress: { ...progress, revision: 1 }, reason: null, available_at: null } }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案' }).click();
  const rate = page.getByRole('button', { name: '记得', exact: true });
  await expect(rate).toBeEnabled();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_availability').length)).toBeGreaterThan(1);
  await expect(rate).toBeDisabled();
  await expect(page.getByText('卡片已被其他操作修改', { exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate'))).toHaveLength(0);
});

test('a transient audio error can be retried while a review card remains ready', async ({ page }) => {
  await page.addInitScript(() => { HTMLMediaElement.prototype.play = async () => {}; });
  await nativeFixture(page, { review_queue: [{ value: { ...payload(1), audio_error: 'Access denied' } }, { value: payload(2, [], 1) }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Access denied');
  await page.getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByText('待复习：22 张', { exact: true })).toBeVisible();
});
