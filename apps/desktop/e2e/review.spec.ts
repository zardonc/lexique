import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { nativeFixture, record } from './native-fixture';

const audio = [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))];

test('a saved word can be added to review without claiming audio is available', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_add: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByText('识义卡', { exact: true })).toBeVisible();
  await expect(page.getByText('听音卡：待生成语音', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_add')
    .map((call) => call.args))).toEqual([{ vocabularyId: 1 }]);
});

test('existing cards are restored and a failed retry remains visible', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    review_add: [{ error: 'database unavailable' }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByText('识义卡', { exact: true })).toBeVisible();
  await expect(page.getByText('听音卡：待生成语音', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('database unavailable');
  await expect(page.getByText('识义卡', { exact: true })).toBeVisible();
});

test('existing audio is restored without regenerating after reopening a saved word', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    review_add: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    speech_cached: [{ value: audio }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByText('听音卡：语音已生成', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate'))).toHaveLength(0);
});

test('missing cached audio can be regenerated for existing cards', async ({ page }) => {
  const cards = [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }];
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: cards }],
    review_add: [{ value: cards }],
    speech_cached: [{ value: null }],
    speech_generate: [{ value: audio }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByText('听音卡：待生成语音', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByText('听音卡：语音已生成', { exact: true })).toBeVisible();
  await expect(page.getByText('识义卡', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate')
    .map((call) => call.args))).toEqual([{ vocabularyId: 1 }]);
});

test('recognition card reveals the saved meaning only after flipping', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await expect(card).toContainText('école');
  await expect(card).not.toContainText('学校');
  await page.screenshot({ path: 'test-results/recognition-front.png' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card).toContainText('学校');
  await expect(card).toContainText('待补例句');
});

test('recognition rating appears after reveal and preserves a failed request for retry', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    review_progress: [{ value: { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } } }],
    review_rate: [{ error: 'database unavailable' }, { value: { status: 'committed', result: { card: { phase: 'learning_first', memory: { stability: 1, difficulty: 5 }, last_review_at: 1700000000, due_at: 1700000060 }, elapsed_days: 0, step_seconds: 60, interval_days: null, model: { version: 'v', parameters: [] }, retention_percent: 90, timezone_database: 'v' } } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await expect(card.getByRole('button', { name: '忘记' })).toHaveCount(0);
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '忘记' }).click();
  await expect(card.getByRole('alert')).toContainText('database unavailable');
  await card.getByRole('button', { name: '重试' }).click();
  const requests = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate').map((call) => call.args));
  expect(requests).toHaveLength(2);
  expect(requests[0]).toEqual(requests[1]);
});

test('an undued card shows its next time instead of rating controls', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_progress: [{ value: { revision: 1, card: { phase: 'learning_first', memory: { stability: 1, difficulty: 5 }, last_review_at: 1700000000, due_at: 4102444800 } } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByText('尚未到期', { exact: false })).toBeVisible();
  await expect(card.getByRole('button', { name: '忘记' })).toHaveCount(0);
  await card.getByRole('button', { name: '关闭卡片' }).click();
});

test('a deferred card shows when it can be reviewed instead of rating buttons', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_availability: [{ value: { progress: { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } }, reason: 'deferred', available_at: 4102444800 } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByText('同词卡片暂缓', { exact: false })).toBeVisible();
  await expect(card.getByRole('button', { name: '忘记' })).toHaveCount(0);
});

test('a waiting card becomes rateable after its due time', async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_progress: [{ value: { revision: 1, card: { phase: 'learning_first', memory: { stability: 1, difficulty: 5 }, last_review_at: now - 58, due_at: now + 2 } } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByRole('button', { name: '忘记' })).toBeEnabled({ timeout: 10000 });
});

test('a failed availability refresh can be retried', async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const progress = { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } };
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_availability: [{ value: { progress, reason: 'deferred', available_at: now + 1 } }, { error: 'temporary failure' }, { value: { progress, reason: null, available_at: null } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByRole('alert')).toContainText('temporary failure', { timeout: 10000 });
  await card.getByRole('button', { name: '重试' }).click();
  await expect(card.getByRole('button', { name: '忘记' })).toBeEnabled();
});

test('a successful timed refresh clears an earlier rejection', async ({ page }) => {
  const progress = { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } };
  const available = { progress, reason: null, available_at: null };
  const now = Math.floor(Date.now() / 1000);
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_availability: [{ value: available }, { value: available }, { value: { progress, reason: 'due', available_at: now + 2 } }, { value: available }],
    review_rate: [{ value: { status: 'rejected', code: 'due', detail: 'not due' } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '忘记' }).click();
  await expect(card.getByRole('alert')).toContainText('not due');
  await expect(card.getByRole('button', { name: '忘记' })).toBeEnabled({ timeout: 10000 });
  await expect(card.getByRole('alert')).toHaveCount(0);
});

test('a rejected stale rating can be abandoned without leaving a pending retry', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_rate: [{ value: { status: 'rejected', code: 'conflict', detail: 'Review card revision conflict' } }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '忘记' }).click();
  await expect(card.getByRole('alert')).toContainText('revision conflict');
  await expect(card.getByRole('button', { name: '重试' })).toHaveCount(0);
});

test('a corrupt pending rating blocks new ratings', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('lexique-pending-rating-10', '{invalid'));
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByRole('alert')).toBeVisible();
  await expect(card.getByRole('button', { name: '忘记' })).toBeDisabled();
});

test('rating submission keeps the card open until failure is shown', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }],
    review_rate: [{ error: 'database unavailable', delay: 500 }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看识义卡' }).click();
  const card = page.getByRole('region', { name: '识义卡' });
  await card.getByRole('button', { name: '显示答案' }).click();
  await card.getByRole('button', { name: '忘记' }).click();
  await expect(card.getByRole('button', { name: '关闭卡片' })).toBeDisabled();
  await expect(card.getByRole('alert')).toContainText('database unavailable');
  await expect(card.getByRole('button', { name: '关闭卡片' })).toBeEnabled();
});

test('speech generation failure keeps both cards and allows retry', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_add: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    speech_generate: [{ error: 'model missing' }, { error: 'model missing' }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('model missing');
  await expect(page.getByText('识义卡', { exact: true })).toBeVisible();
  await expect(page.getByText('听音卡：待生成语音', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate').length)).toBe(2);
});

test('listening card plays once, hides the word and reports playback failure', async ({ page }) => {
  await page.addInitScript(() => {
    let count = 0;
    HTMLMediaElement.prototype.play = function () {
      count += 1;
      return count === 1 ? Promise.reject(new Error('speaker unavailable')) : Promise.resolve();
    };
  });
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_add: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    speech_generate: [{ value: audio }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const card = page.getByRole('region', { name: '听音卡' });
  await expect(card).not.toContainText('école');
  await expect(card.getByRole('alert')).toContainText('speaker unavailable');
  await card.getByRole('button', { name: '显示答案' }).click();
  await expect(card.getByRole('button', { name: '忘记' })).toBeDisabled();
  await card.getByRole('button', { name: '重播' }).click();
  await expect(card.getByRole('alert')).toHaveCount(0);
  await expect(card.getByRole('button', { name: '忘记' })).toBeEnabled();
  await card.locator('audio').evaluate((audioElement) => audioElement.dispatchEvent(new Event('error')));
  await expect(card.getByRole('button', { name: '忘记' })).toBeDisabled();
  await expect(card).toContainText('école');
  await expect(card).toContainText('待补例句');
  await expect(card.locator('details summary')).toBeVisible();
  await card.locator('details summary').click();
  await expect(card.locator('details')).toContainText(record.reference.dictionary);
  await card.getByRole('button', { name: '跳过' }).click();
  await expect(card).not.toBeVisible();
});

test('adding again regenerates audio after playback failure', async ({ page }) => {
  await page.addInitScript(() => {
    HTMLMediaElement.prototype.play = () => Promise.reject(new Error('invalid audio'));
  });
  const cards = [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }];
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_add: [{ value: cards }, { value: cards }],
    speech_generate: [{ value: audio }, { value: audio }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const card = page.getByRole('region', { name: '听音卡' });
  await expect(card.getByRole('alert')).toContainText('播放失败');
  await card.getByRole('button', { name: '跳过' }).click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate')
    .map((call) => call.args))).toEqual([{ vocabularyId: 1 }, { vocabularyId: 1, regenerate: true }]);
});

test('successful replay preserves usable audio when adding again', async ({ page }) => {
  await page.addInitScript(() => {
    let first = true;
    HTMLMediaElement.prototype.play = () => {
      if (first) { first = false; return Promise.reject(new Error('speaker unavailable')); }
      return Promise.resolve();
    };
  });
  const cards = [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }];
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }], review_cards: [{ value: cards }],
    review_add: [{ value: cards }], speech_cached: [{ value: audio }],
  });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const card = page.getByRole('region', { name: '听音卡' });
  await expect(card.getByRole('alert')).toContainText('speaker unavailable');
  await card.getByRole('button', { name: '重播' }).click();
  await expect(card.getByRole('alert')).toHaveCount(0);
  await card.getByRole('button', { name: '跳过' }).click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_generate'))).toHaveLength(0);
});

test('reopening a listening card cancels its previous pending autoplay', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_list: [{ value: [record] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }],
    speech_cached: [{ value: audio }],
  });
  await page.goto('/');
  await page.evaluate(() => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as {
      invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
    const invoke = native.invoke;
    const pending: (() => void)[] = [];
    native.invoke = (command, args) => command === 'speech_preferences'
      ? new Promise((done) => pending.push(() => { void invoke(command, args).then(done); }))
      : invoke(command, args);
    Reflect.set(window, 'releasePreferences', () => pending.forEach((done) => done()));
    Reflect.set(window, 'playCount', 0);
    HTMLMediaElement.prototype.play = () => {
      Reflect.set(window, 'playCount', Number(Reflect.get(window, 'playCount')) + 1);
      return Promise.resolve();
    };
  });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  const card = page.getByRole('region', { name: '听音卡' });
  await card.getByRole('button', { name: '跳过' }).click();
  await page.getByRole('button', { name: '查看听音卡' }).click();
  await page.evaluate(() => { Reflect.get(window, 'releasePreferences')(); });
  await expect.poll(() => page.evaluate(() => Reflect.get(window, 'playCount'))).toBe(1);
  await card.getByRole('button', { name: '显示答案' }).click();
  expect(await page.evaluate(() => Reflect.get(window, 'playCount'))).toBe(1);
});
