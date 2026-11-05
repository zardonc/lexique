import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { nativeFixture, record } from './native-fixture';

const audio = [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))];
const ready = { queue: { cards: [{ id: 11, vocabulary_id: 1, kind: 'listening' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };

test('failed queue audio can be regenerated in place; replacement must play before rating', async ({ page }) => {
  await page.addInitScript(() => {
    let count = 0;
    HTMLMediaElement.prototype.play = () => ++count === 1 ? Promise.reject(new Error('invalid audio'))
      : new Promise<void>((resolve) => { Reflect.set(window, 'finishReplacement', resolve); });
  });
  await nativeFixture(page, { review_queue: [{ value: ready }], speech_cached: [{ value: audio }],
    speech_generate: [{ error: 'worker unavailable' }, { value: audio, gate: 'replacement' }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await expect(card.getByRole('alert')).toContainText('invalid audio');
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '重新生成语音' }).click();
  await expect(card.getByRole('alert').filter({ hasText: 'worker unavailable' })).toBeVisible();
  await expect(card.getByRole('button', { name: '忘记' })).toBeDisabled();
  await card.getByRole('button', { name: '重新生成语音' }).click();
  await expect(card.getByRole('button', { name: '重新生成语音' })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('replacement'));
  await expect.poll(() => page.evaluate(() => typeof Reflect.get(window, 'finishReplacement'))).toBe('function');
  await expect(card.getByRole('heading', { name: 'école' })).toBeVisible();
  await expect(card.getByRole('button', { name: '忘记' })).toBeDisabled();
  await page.evaluate(() => Reflect.get(window, 'finishReplacement')());
  await expect(card.getByRole('button', { name: '忘记' })).toBeEnabled();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate').map((call) => call.args));
  expect(calls.map((args) => ({ vocabularyId: args?.vocabularyId, regenerate: args?.regenerate })))
    .toEqual([{ vocabularyId: 1, regenerate: true }, { vocabularyId: 1, regenerate: true }]);
  expect(calls[0]?.operation).toBeTruthy();
  expect(calls[1]?.operation).toBe(calls[0]?.operation);
});

test('pending audio retry sends only the missing word identity', async ({ page }) => {
  await nativeFixture(page, { review_queue: [{ value: { ...ready, queue: { ...ready.queue, cards: [] }, record: null,
    audio_pending: true, audio_vocabulary_id: 2 } }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '重试 · 听音卡：待生成语音' }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_retry_missing').map((call) => call.args)))
    .toEqual([{ vocabularyId: 2 }]);
});

test('leaving a card while audio regenerates does not play its late replacement', async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, 'playCount', 0);
    HTMLMediaElement.prototype.play = () => {
      Reflect.set(window, 'playCount', Number(Reflect.get(window, 'playCount')) + 1);
      return Promise.reject(new Error('invalid audio'));
    };
  });
  await nativeFixture(page, { review_queue: [{ value: ready }, { value: { ...ready, queue: { ...ready.queue, cards: [] }, record: null } }],
    speech_cached: [{ value: audio }], speech_generate: [{ value: audio, gate: 'left' }] });
  await page.goto('/'); await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const card = page.getByRole('region', { name: '复习卡片' });
  await card.getByRole('button', { name: '重新生成语音' }).click();
  await card.getByRole('button', { name: '跳过' }).click();
  await expect(card).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('left'));
  await expect(page.getByText('本轮完成', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => Reflect.get(window, 'playCount'))).toBe(1);
});
