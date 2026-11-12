import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';
import { readFileSync } from 'node:fs';
import type { Page } from '@playwright/test';

const audio = { bytes: [...readFileSync(new URL('../../../docs/evidence/issue-15/audio/native-ecole-female.opus', import.meta.url))], voice: 'female' };
const saved = { ...record, example_fr: 'Une école.', example_zh: '学校。', example_sense: 0,
  more_examples: [{ fr: 'Autre phrase.', zh: '另一句。' }] };
declare global { interface Window { examplePlayers: HTMLMediaElement[] } }
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.examplePlayers = [];
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () { this.loop = true; window.examplePlayers.push(this); return play.call(this); };
  });
});
async function open(page: Page) {
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
}

test('draft example speech normalizes only its spoken copy without saving', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], speech_preview: [{ error: 'model missing' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  const original = '  Une e\u0301cole.\n\t Une phrase.  ';
  await page.getByLabel('法语例句', { exact: true }).fill(original);
  await page.getByRole('button', { name: /播放例句:/ }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'model missing' })).toBeVisible();
  await expect(page.getByLabel('法语例句', { exact: true })).toHaveValue(original);
  expect(await page.evaluate(() => window.testNative.calls.filter((c) => c.command === 'speech_preview').map((c) => c.args)))
    .toEqual([{ text: 'Une école. Une phrase.' }]);
  expect(await page.evaluate(() => window.testNative.calls.filter((c) => ['vocabulary_save', 'speech_generate', 'review_rate', 'review_add'].includes(c.command)))).toEqual([]);
});

test('speech limits preserve full save input and reject missing French and controls', async ({ page }) => {
  const long = 'é'.repeat(201);
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], speech_preview: [{ error: 'boundary accepted' }],
    vocabulary_save: [{ value: { outcome: 'updated', record: { ...record, example_fr: long, example_sense: 0, revision: 2 } } }] });
  await open(page);
  await page.getByLabel('例句中文译文', { exact: true }).fill('只有中文');
  await expect(page.getByRole('button', { name: /播放例句:/ })).toHaveCount(0);
  const input = page.getByLabel('法语例句', { exact: true });
  await input.fill(' \n\t ');
  await expect(page.getByRole('button', { name: /播放例句:/ })).toHaveCount(0);
  await input.fill('é'.repeat(200));
  await page.getByRole('button', { name: /播放例句:/ }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'boundary accepted' })).toBeVisible();
  await input.fill('\u000bBonjour');
  await expect(page.getByRole('button', { name: /播放例句:/ })).toBeDisabled();
  await expect(page.getByText('例句含有无法朗读的控制字符。')).toBeVisible();
  await input.fill(long);
  await expect(page.getByRole('button', { name: /播放例句:/ })).toBeDisabled();
  await expect(page.getByText(/朗读仅支持规范化后最多 200/)).toBeVisible();
  await page.getByLabel('例句对应释义').selectOption('0');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(input).toHaveValue(long);
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((c) => c.command === 'speech_preview').map((c) => c.args)).toEqual([{ text: 'é'.repeat(200) }]);
  expect(calls.find((c) => c.command === 'vocabulary_save')?.args?.request).toMatchObject({ example_fr: long });
});

for (const [label, region] of [['查看识义卡', '识义卡'], ['查看听音卡', '听音卡']] as const) {
  test(`${region} examples play manually and stop when collapsed or closed`, async ({ page }) => {
    await nativeFixture(page, { vocabulary_list: [{ value: [saved] }], speech_preview: [{ value: audio }], speech_cached: [{ value: audio }],
      review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }] });
    await open(page);
    await page.getByRole('button', { name: /播放例句: Autre phrase/ }).click();
    await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.currentTime ?? 0)).toBeGreaterThan(0);
    const editorPlayer = await page.evaluate(() => window.examplePlayers.length - 1);
    await page.getByRole('button', { name: label }).click();
    const card = page.getByRole('dialog', { name: region, exact: true });
    await card.getByRole('button', { name: '显示答案' }).click();
    expect(await page.evaluate(() => window.testNative.calls.filter((c) => c.command === 'speech_preview').map((c) => c.args))).toEqual([{ text: 'Autre phrase.' }]);
    await card.getByRole('button', { name: /播放例句: Une école/ }).click();
    expect(await page.evaluate((index) => window.examplePlayers[index].paused, editorPlayer)).toBe(true);
    await card.getByText('更多例句', { exact: true }).click();
    await card.getByRole('button', { name: /播放例句: Autre phrase/ }).click();
    await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.currentTime ?? 0)).toBeGreaterThan(0);
    expect(await page.evaluate(() => window.examplePlayers.at(-2)?.paused)).toBe(true);
    await card.getByText('更多例句', { exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.paused)).toBe(true);
    await card.getByRole('button', { name: /播放例句: Une école/ }).click();
    await card.getByRole('button', { name: region === '识义卡' ? '关闭卡片' : '跳过', exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.paused)).toBe(true);
    expect(await page.evaluate(() => window.testNative.calls.filter((c) => ['review_rate', 'speech_generate', 'vocabulary_save'].includes(c.command)))).toEqual([]);
  });
}

test('example switch cancels pending speech, retry reads new WPM, edits and navigation stop playback', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [saved] }],
    speech_preview: [{ value: audio, gate: 'old' }, { error: 'model unavailable' }, { value: audio }, { value: audio }],
    speech_preferences: [{ value: { voice: 'female', wpm: 80 } }, { value: { voice: 'female', wpm: 80 } },
      { value: { voice: 'female', wpm: 240 } }] });
  await open(page);
  await page.getByRole('button', { name: /播放例句: Une école/ }).click();
  const later = page.getByRole('button', { name: /播放例句: Autre phrase/ });
  await later.click();
  await expect(page.getByRole('alert').filter({ hasText: 'model unavailable' })).toBeVisible();
  await page.evaluate(() => window.testNative.release('old'));
  await expect(page.getByRole('button', { name: /播放例句: Une école/ })).toBeEnabled();
  await later.click();
  await expect.poll(() => page.evaluate(() => window.examplePlayers[0]?.currentTime ?? 0)).toBeGreaterThan(0);
  expect(await page.evaluate(() => window.examplePlayers.length)).toBe(1);
  expect(await page.evaluate(() => window.examplePlayers[0].playbackRate)).toBeCloseTo(1.607968168, 7);
  expect(await page.evaluate(() => window.examplePlayers[0].preservesPitch)).toBe(true);
  await page.locator('#example-fr').fill('Nouvelle phrase.');
  await page.getByRole('button', { name: /播放例句: Nouvelle phrase/ }).click();
  expect(await page.evaluate(() => window.examplePlayers[0].paused)).toBe(true);
  await page.locator('#example-fr').fill(saved.example_fr);
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.paused)).toBe(true);
});

test('example controls are localized and invalid audio can retry without changing the draft', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [saved] }],
    speech_preview: [{ value: { bytes: [79, 103, 103, 83], voice: 'female' } }, { value: audio }] });
  await open(page);
  for (const [locale, label] of [['en', 'Play example'], ['fr', 'Écouter l’exemple'], ['zh', '播放例句']] as const) {
    await page.getByLabel(/界面语言|Interface language|Langue de l’interface/).selectOption(locale);
    await expect(page.getByRole('button', { name: new RegExp(`${label}: Une école`) })).toBeVisible();
  }
  const button = page.getByRole('button', { name: /播放例句: Une école/ });
  await button.focus(); await button.press('Space');
  await expect(page.getByRole('alert').filter({ hasText: '播放失败' })).toBeVisible();
  await button.click();
  await expect.poll(() => page.evaluate(() => window.examplePlayers.at(-1)?.currentTime ?? 0)).toBeGreaterThan(0);
  await expect(page.getByRole('alert').filter({ hasText: '播放失败' })).toHaveCount(0);
  await expect(page.locator('#example-fr')).toHaveValue(saved.example_fr);
});
