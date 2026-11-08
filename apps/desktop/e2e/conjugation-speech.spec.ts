import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { nativeFixture } from './native-fixture';
import coloring from './fixtures/conjugation-coloring.json' with { type: 'json' };
import defaultOrder from './fixtures/conjugation-order.json' with { type: 'json' };
import ordinary from './fixtures/conjugation-ordinary.json' with { type: 'json' };
import pronominal from './fixtures/conjugation-pronominal.json' with { type: 'json' };
import simple from './fixtures/conjugation.json' with { type: 'json' };

const audio = { bytes: [...await readFile(resolve(process.cwd(), '../../docs/evidence/issue-15/audio/native-ecole-female.opus'))], voice: 'female' };
const defaults = {
  conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
  speech_preview: [{ value: audio }],
};
const laverLookup = { query: 'nous avions lavé', tables: [coloring.laver] };

declare global { interface Window { testExpressionAudio: HTMLMediaElement[] } }

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.testExpressionAudio = [];
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () { this.loop = true; window.testExpressionAudio.push(this); return play.call(this); };
  });
});

async function showTable(page: Page, query: string) {
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill(query);
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
}

async function playExpression(page: Page, expression: string) {
  const index = await page.evaluate(() => window.testExpressionAudio.length);
  const button = page.getByRole('button', { name: `Play expression: ${expression} (Machine-generated speech)`, exact: true }).first();
  await button.focus(); await button.press('Space');
  await expect.poll(() => page.evaluate((index) => window.testExpressionAudio[index]?.currentTime ?? 0, index)).toBeGreaterThan(0);
  return index;
}

test('expression playback sends the complete displayed text and exposes generation failures without learning writes', async ({ page }) => {
  await nativeFixture(page, {
    ...defaults,
    conjugation_lookup: [{ value: laverLookup }],
    speech_preview: [{ error: 'voice unavailable' }],
  });
  await showTable(page, 'nous avions lavé');
  const expression = page.locator('[data-tense=PQP] .conjugation-expression').filter({ hasText: 'nous avions lavé' });
  const play = expression.getByRole('button', { name: 'Play expression: nous avions lavé (Machine-generated speech)', exact: true });
  await expect(play).toBeVisible();
  await play.focus(); await play.press('Enter');
  await expect(expression.getByRole('alert')).toContainText('voice unavailable');
  await expect(play).toBeEnabled();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)).toEqual([{ text: 'nous avions lavé' }]);
  expect(calls.filter((call) => ['vocabulary_save', 'review_add', 'review_add_batch', 'speech_generate', 'review_rate'].includes(call.command))).toHaveLength(0);
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
});

for (const [query, result, expressions] of [
  ['paie', simple.paie, ['je paie', 'je paye']],
  ['je suis allée', ordinary['je suis allée'], ['je suis allé', 'je suis allée']],
  ['se laver', pronominal['se laver'], ['je me lave', 'lave-toi', 'elle s’est lavée']],
] as const) test(`each ${query} alternative plays separately without slash notation or whole-table generation`, async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: result }] });
  await showTable(page, query);
  const displayed = page.locator('.conjugation-expression');
  await expect(displayed.locator('button')).toHaveCount(await displayed.count());
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview'))).toHaveLength(0);
  for (const expression of expressions) await playExpression(page, expression);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)))
    .toEqual(expressions.map((text) => ({ text })));
  await expect(page.locator('.conjugation').getByRole('alert')).toHaveCount(0);
});

for (const [voice, wpm, rate] of [
  ['female', 80, 0.535989389], ['female', 180, 1.205976126], ['female', 240, 1.607968168],
  ['male', 80, 0.536376731], ['male', 180, 1.206847645], ['male', 240, 1.609130193],
] as const) {
  test(`expression playback reads saved ${voice}/${wpm} WPM and stops on navigation`, async ({ page }) => {
    await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: pronominal['elles se sont abstenues'] }],
      speech_preferences: [{ value: { voice, wpm } }], speech_preview: [{ value: { ...audio, voice } }] });
    await showTable(page, 'elles se sont abstenues');
    const index = await playExpression(page, 'elles se sont abstenues');
    expect(await page.evaluate((index) => window.testExpressionAudio[index].playbackRate, index)).toBeCloseTo(rate, 7);
    expect(await page.evaluate((index) => window.testExpressionAudio[index].preservesPitch, index)).toBe(true);
    expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preferences').map((call) => call.args)))
      .toEqual([{ preferences: null }]);
    await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
    expect(await page.evaluate((index) => window.testExpressionAudio[index].paused, index)).toBe(true);
  });
}

test('unplayable returned audio remains visible and the same expression can retry', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: laverLookup }],
    speech_preview: [{ value: { bytes: [79, 103, 103, 83], voice: 'female' } }, { value: audio }] });
  await showTable(page, 'nous avions lavé');
  const play = page.getByRole('button', { name: 'Play expression: nous avions lavé (Machine-generated speech)', exact: true });
  await play.click();
  await expect(page.locator('.conjugation').getByRole('alert')).toContainText('Playback failed');
  await expect(play).toBeEnabled();
  await playExpression(page, 'nous avions lavé');
  await expect(page.locator('.conjugation').getByRole('alert')).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)))
    .toEqual([{ text: 'nous avions lavé' }, { text: 'nous avions lavé' }]);
});

test('choosing another expression stops the previous clip on the same table', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: simple.paie }] });
  await showTable(page, 'paie');
  const first = await playExpression(page, 'je paie');
  await playExpression(page, 'je paye');
  expect(await page.evaluate((index) => window.testExpressionAudio[index].paused, first)).toBe(true);
});

test('choosing another expression ignores the previous pending response on the same table', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: simple.paie }],
    speech_preview: [{ value: audio, gate: 'old-expression' }, { value: audio }] });
  await showTable(page, 'paie');
  const first = page.getByRole('button', { name: 'Play expression: je paie (Machine-generated speech)', exact: true }).first();
  await first.click(); await expect(first).toBeDisabled();
  await playExpression(page, 'je paye');
  await expect(first).toBeEnabled();
  await page.evaluate(async () => {
    window.testNative.release('old-expression');
    await new Promise<void>((done) => requestAnimationFrame(() => done()));
  });
  expect(await page.evaluate(() => window.testExpressionAudio.length)).toBe(1);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)))
    .toEqual([{ text: 'je paie' }, { text: 'je paye' }]);
});

test('speech preference errors prevent playback and recover on retry', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: laverLookup }],
    speech_preferences: [{ error: 'saved speech preferences corrupt' }, { value: { voice: 'female', wpm: 180 } }] });
  await showTable(page, 'nous avions lavé');
  const play = page.getByRole('button', { name: 'Play expression: nous avions lavé (Machine-generated speech)', exact: true });
  await play.click();
  await expect(page.locator('.conjugation').getByRole('alert')).toContainText('saved speech preferences corrupt');
  expect(await page.evaluate(() => window.testExpressionAudio)).toHaveLength(0);
  await playExpression(page, 'nous avions lavé');
  await expect(page.locator('.conjugation').getByRole('alert')).toHaveCount(0);
});

test('changing the selected verb cancels a pending response even when the displayed expression is shared', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: simple.suis }],
    speech_preview: [{ value: audio, gate: 'old-verb' }] });
  await showTable(page, 'suis');
  await page.getByRole('button', { name: /^être ·/u }).click();
  const play = page.getByRole('button', { name: 'Play expression: je suis (Machine-generated speech)', exact: true }).first();
  await play.click(); await expect(play).toBeDisabled();
  await page.getByRole('button', { name: /^suivre ·/u }).click();
  await page.evaluate(async () => {
    window.testNative.release('old-verb');
    await new Promise<void>((done) => requestAnimationFrame(() => done()));
  });
  expect(await page.evaluate(() => window.testExpressionAudio)).toHaveLength(0);
});

test('leaving pending expression speech preserves concurrent dictionary playback', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: laverLookup }],
    speech_preview: [{ value: audio, gate: 'left-table' }, { value: audio }] });
  await showTable(page, 'nous avions lavé');
  const expression = page.getByRole('button', { name: 'Play expression: nous avions lavé (Machine-generated speech)', exact: true });
  await expression.click(); await expect(expression).toBeDisabled();
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByLabel('French word or phrase').fill('école');
  await page.getByRole('button', { name: 'Look up', exact: true }).click();
  await page.getByRole('button', { name: 'Play word: école (Machine-generated speech)', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testExpressionAudio[0]?.currentTime ?? 0)).toBeGreaterThan(0);
  await page.evaluate(async () => {
    window.testNative.release('left-table');
    await new Promise<void>((done) => requestAnimationFrame(() => done()));
  });
  expect(await page.evaluate(() => window.testExpressionAudio.length)).toBe(1);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').map((call) => call.args)))
    .toEqual([{ text: 'nous avions lavé' }, { text: 'école' }]);
});

test('expression controls retain keyboard access and machine-speech disclosure in all languages and themes', async ({ page }) => {
  await nativeFixture(page, { ...defaults, conjugation_lookup: [{ value: laverLookup }], speech_preview: [{ error: 'voice unavailable' }] });
  await showTable(page, 'nous avions lavé');
  const language = page.getByRole('combobox').filter({ has: page.getByRole('option', { name: 'English', exact: true }) });
  for (const [locale, label, disclosure] of [['en', 'Play expression', 'Machine-generated speech'],
    ['fr', 'Écouter l’expression', 'Voix de synthèse'], ['zh', '播放表达', '机器合成语音']] as const) {
    await language.selectOption(locale);
    await expect(page.locator('.conjugation-table')).toContainText(`${disclosure} · Supertonic 3`);
    for (const theme of ['white', 'sepia', 'dark']) {
      await page.getByLabel(/显示主题|Display theme|Thème d’affichage/u).selectOption(theme);
      const play = page.getByRole('button', { name: `${label}: nous avions lavé (${disclosure})`, exact: true });
      expect(await play.evaluate((button) => button.closest('[lang]')?.getAttribute('lang'))).toBe(locale === 'zh' ? 'zh-CN' : locale);
      const box = await play.boundingBox();
      expect(box?.width).toBeGreaterThanOrEqual(44); expect(box?.height).toBeGreaterThanOrEqual(44);
      await play.focus(); await play.press('Space');
      await expect(page.locator('.conjugation').getByRole('alert')).toContainText('voice unavailable');
      await expect(play).toBeEnabled();
    }
  }
});
