import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';
import type { Page } from '@playwright/test';

const empty = { queue: { cards: [], reason: null, available_at: null }, record: null, audio_pending: false, audio_error: null };
const ready = { queue: { cards: [{ id: 1, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
const other = { ...ready, queue: { ...ready.queue, cards: [{ id: 3, vocabulary_id: 2, kind: 'recognition' }] }, record: { ...record, id: 2, headword: 'bonjour' } };
const count = (page: Page) => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_queue').length);
const open = async (page: Page) => { await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click(); };

test('T01: stable idle has zero extra reads; overlapping resume notifications coalesce', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-02T10:00:00Z') });
  await nativeFixture(page, { review_queue: [{ value: empty }, { value: ready }] });
  await open(page); await expect(page.getByText('本轮完成', { exact: true })).toBeVisible();
  expect(await count(page)).toBe(1);
  await page.clock.runFor(3000); expect(await count(page)).toBe(1);
  await page.evaluate(() => { window.dispatchEvent(new Event('focus')); window.dispatchEvent(new Event('pageshow')); document.dispatchEvent(new Event('visibilitychange')); window.testNative.emit('lexique-review-changed'); });
  await page.clock.runFor(51); await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
  await expect(page.getByRole('button', { name: '刷新队列' })).toHaveCount(0);
});

test('T01: events during a blocked read yield one trailing read and latest state', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: empty, gate: 'read' }, { value: ready }] });
  await open(page); await expect.poll(() => count(page)).toBe(1);
  await page.evaluate(() => { for (let i = 0; i < 1000; i++) window.testNative.emit('lexique-review-changed'); });
  expect(await count(page)).toBe(1);
  await page.evaluate(() => window.testNative.release('read'));
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
});

test('T01: a continuous event stream cannot postpone refresh past the maximum wait', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-05T09:59:00Z') });
  await nativeFixture(page, { review_queue: [{ value: empty }, { value: empty }, { value: other }] });
  await open(page); await expect(page.getByText('本轮完成', { exact: true })).toBeVisible();
  await page.clock.pauseAt(new Date('2026-11-05T10:00:01Z'));
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  for (let index = 0; index < 10; index++) {
    await page.clock.runFor(20);
    await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  }
  expect(await count(page)).toBe(2);
  await page.clock.runFor(51);
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible();
  expect(await count(page)).toBe(3);
  await page.clock.runFor(60_000);
  expect(await count(page)).toBe(3);
});

test('T03: a stale background reply cannot restore a skipped card', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: ready }, { value: ready, gate: 'old' }, { value: other }] });
  await open(page); await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed')); await expect.poll(() => count(page)).toBe(2);
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await page.evaluate(() => window.testNative.release('old'));
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible();
  expect(await count(page)).toBe(3);
});

test('T03: background changes keep the current answer and do not replace the card', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: ready }, { value: other }] });
  await open(page); await page.getByRole('button', { name: '显示答案' }).click();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed')); await expect.poll(() => count(page)).toBe(2);
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByText('学校 · 流派', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '跳过', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible();
});

test('T03: background summary updates while the current answer stays visible', async ({ page }) => {
  const added = { ...other, queue: { ...other.queue, cards: [...other.queue.cards, { id: 5, vocabulary_id: 3, kind: 'recognition' }] } };
  await nativeFixture(page, { review_queue: [{ value: ready }, { value: added }] });
  await open(page); await page.getByRole('button', { name: '显示答案' }).click();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed')); await expect.poll(() => count(page)).toBe(2);
  await expect(page.getByText('待复习：2 张', { exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(page.getByText('学校 · 流派', { exact: true })).toBeVisible();
});

test('T01/T07: hidden notifications stop reads and resume merges them', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-02T10:00:00Z') });
  await nativeFixture(page, { review_queue: [{ value: empty }, { value: ready }] });
  await open(page); await expect(page.getByText('本轮完成', { exact: true })).toBeVisible();
  await page.evaluate(() => { Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'hidden' }); document.dispatchEvent(new Event('visibilitychange')); for (let i = 0; i < 100; i++) window.testNative.emit('lexique-review-changed'); });
  await page.clock.runFor(60_000); expect(await count(page)).toBe(1);
  await page.evaluate(() => { Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'visible' }); document.dispatchEvent(new Event('visibilitychange')); window.dispatchEvent(new Event('focus')); });
  await page.clock.runFor(51); await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
});

test('T02: absolute due wake fires at the boundary without early queries', async ({ page }) => {
  const time = new Date('2026-11-02T10:00:00Z');
  await page.clock.install({ time: new Date(time.getTime() - 1000) });
  await page.clock.pauseAt(time);
  await nativeFixture(page, { review_queue: [{ value: { ...empty, queue: { ...empty.queue, reason: 'due', available_at: time.getTime() / 1000 + 5 } } }, { value: ready }] });
  await open(page); await page.clock.runFor(1); await expect(page.getByText(/尚未到期/)).toBeVisible();
  await page.clock.runFor(4998); expect(await count(page)).toBe(1);
  await page.clock.runFor(2); await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
});

test('T02: 04:00 local learning day wakes a quota-blocked queue', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-02T03:59:58Z') });
  await nativeFixture(page, { review_queue: [{ value: { ...empty, queue: { ...empty.queue, reason: 'quota' } } }, { value: ready }] });
  await open(page); await expect(page.getByText('今日额度已用完')).toBeVisible();
  await page.clock.runFor(2000); await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
});

test('T02: resuming after two learning days merges overdue wakes and rechecks the timezone', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-05T03:59:00Z') });
  await nativeFixture(page, { review_queue: [{ value: { ...empty, queue: { ...empty.queue, reason: 'due', available_at: Date.parse('2026-11-05T04:01:00Z') / 1000 } } }, { value: ready }] });
  await open(page); await expect(page.getByText(/尚未到期/)).toBeVisible();
  await page.evaluate(() => {
    Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'hidden' });
    document.dispatchEvent(new Event('visibilitychange'));
  });
  await page.clock.setSystemTime(new Date('2026-11-07T12:00:00Z'));
  expect(await count(page)).toBe(1);
  await page.evaluate(() => {
    Intl.DateTimeFormat = new Proxy(Intl.DateTimeFormat, { apply(target, receiver, args) {
      const formatter = Reflect.apply(target, receiver, args) as Intl.DateTimeFormat;
      const options = formatter.resolvedOptions();
      formatter.resolvedOptions = () => ({ ...options, timeZone: 'America/Vancouver' });
      return formatter;
    } });
    Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'visible' });
    document.dispatchEvent(new Event('visibilitychange'));
    window.dispatchEvent(new Event('focus')); window.dispatchEvent(new Event('pageshow'));
  });
  await page.clock.runFor(51);
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
  expect(await count(page)).toBe(2);
  const request = await page.evaluate(() => window.testNative.calls.filter(call => call.command === 'review_queue').at(-1)?.args);
  expect(request?.timezone).toBe('America/Vancouver');
  await page.clock.runFor(60_000);
  expect(await count(page)).toBe(2);
});

test('T01/T11: failed reads do not loop and offer a targeted retry', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-11-02T10:00:00Z') });
  await nativeFixture(page, { review_queue: [{ error: 'database locked' }, { value: ready }] });
  await open(page); await expect(page.getByRole('alert')).toContainText('database locked');
  await page.clock.runFor(60_000); expect(await count(page)).toBe(1);
  await page.getByRole('button', { name: '重试', exact: true }).click(); await page.clock.runFor(1);
  await expect(page.getByRole('heading', { name: 'école' })).toBeVisible();
});

test('T07: remount waits for the outstanding native read without applying its old reply', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: ready, gate: 'left' }, { value: other }] });
  await open(page); await expect.poll(() => count(page)).toBe(1);
  await page.getByRole('button', { name: '词典', exact: true }).click(); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  expect(await count(page)).toBe(1); await page.evaluate(() => window.testNative.release('left'));
  await expect(page.getByRole('heading', { name: 'bonjour' })).toBeVisible(); expect(await count(page)).toBe(2);
});
