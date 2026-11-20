import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { nativeFixture, record } from './native-fixture';

const external = { ...record, headword: 'école externe', reference: null, digest: [], import_source: { format: 'csv', filename: 'mes-écoles.csv' },
  notes: '学校\n独立笔记', example_fr: 'Une école indépendante.', example_zh: '一所独立学校。', example_sense: null,
  more_examples: [{ fr: '', zh: '尚待法语例句' }] };
const preview = { operation: 'csv-fixture', source: external.import_source, rows: [2, 3], status: 'create', error: null, existing: null, enrichment: null,
  content: { headword: external.headword, notes: external.notes, examples: [{ fr: external.example_fr, zh: external.example_zh }, ...external.more_examples] } };

test('single-word preview confirms content without adding cards or speech and offers the saved record', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: preview }], note_import_commit: [{ value: { outcome: 'created', record: external } }],
    vocabulary_list: [{ value: [] }, { value: [external] }], review_auto_enroll: [{ value: true }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  const panel = page.getByRole('region', { name: '单词 CSV 导入预览', exact: true });
  await expect(panel).toContainText('mes-écoles.csv'); await expect(panel).toContainText('尚待法语例句');
  await page.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(page.getByText('导入已保存。可打开记录，主动加入识义卡和听音卡。')).toBeVisible();
  const commands = await page.evaluate(() => window.testNative.calls.map((call) => call.command));
  expect(commands).not.toContain('review_add'); expect(commands).not.toContain('speech_generate'); expect(commands).not.toContain('review_add_batch');
  await page.getByRole('button', { name: '打开记录', exact: true }).click();
  await expect(page.getByRole('region', { name: '查看与编辑', exact: true })).toContainText('暂无词汇摘要');
  await expect(page.locator('.dictionary-source')).toContainText('mes-écoles.csv');
  await expect(page.locator('#personal-notes')).toHaveValue(external.notes);
});

test('external notes and unassigned examples save and reopen without requiring a dictionary meaning', async ({ page }) => {
  const edited = { ...external, revision: 2, notes: '修改后的笔记', example_fr: 'Une phrase modifiée.' };
  await nativeFixture(page, { vocabulary_list: [{ value: [external] }, { value: [edited] }], vocabulary_save: [{ value: { outcome: 'updated', record: edited } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.locator('#personal-notes').fill(edited.notes); await page.locator('#example-fr').fill(edited.example_fr);
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '词典', exact: true }).click(); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.locator('#personal-notes')).toHaveValue(edited.notes); await expect(page.locator('#example-fr')).toHaveValue(edited.example_fr);
});

test('file-picker and preview cancellation never commit learning data', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: null }, { value: preview }], note_import_cancel: [{ value: null }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByText('已取消，未保存任何记录。')).toBeVisible();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await page.getByRole('button', { name: '取消导入', exact: true }).click();
  await expect(page.getByRole('region', { name: '单词 CSV 导入预览', exact: true })).toBeHidden();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'note_import_commit'))).toBe(false);
});

test('read and write errors stay visible and a failed confirmation retains preview for retry', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ error: 'CSV read denied' }, { value: preview }],
    note_import_commit: [{ error: 'SQLite write failed' }, { value: { outcome: 'created', record: external } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click(); await expect(page.getByRole('alert')).toContainText('CSV read denied');
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await page.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(page.getByRole('alert')).toContainText('SQLite write failed');
  await expect(page.getByRole('region', { name: '单词 CSV 导入预览', exact: true })).toContainText(external.notes);
  await page.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(page.getByRole('button', { name: '打开记录', exact: true })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls.map((call) => call.args?.operation)).toEqual(['csv-fixture', 'csv-fixture']);
});

test('differences and invalid groups block confirmation and display file locations and existing content', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, status: 'different', existing: { ...external, notes: '已有笔记' } } },
    { value: { ...preview, status: 'invalid', error: 'Row 2: Headword exceeds 200 characters' } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled(); await expect(page.getByText('已有笔记', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '导入单词 CSV', exact: true }).click();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled(); await expect(page.getByRole('alert')).toContainText('Row 2');
});

test('external card backs show content and missing dictionary information while listening without audio stays ungradable', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [external] }], review_cards: [{ value: [] }],
    review_add: [{ value: [{ id: 11, kind: 'recognition' }, { id: 12, kind: 'listening' }] }], speech_generate: [{ error: 'Speech model unavailable' }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click(); await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '加入复习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Speech model unavailable');
  await page.getByRole('button', { name: '查看识义卡', exact: true }).click(); await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await expect(page.getByRole('dialog')).toContainText(external.example_fr); await expect(page.getByRole('dialog')).toContainText('暂无词汇摘要');
  await page.keyboard.press('Escape');
  await expect(page.getByText('听音卡：待生成语音', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '查看听音卡', exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_rate'))).toBe(false);
});

test('external listening cards require completed playback before grading', async ({ page }) => {
  await page.addInitScript(() => {
    HTMLMediaElement.prototype.play = () => new Promise<void>((resolve) => { Reflect.set(window, 'finishExternalPlayback', resolve); });
  });
  const audio = { bytes: [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))], voice: 'female' };
  await nativeFixture(page, { vocabulary_list: [{ value: [external] }], review_cards: [{ value: [{ id: 12, kind: 'listening' }] }], speech_cached: [{ value: audio }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click(); await page.locator('.vocabulary-list button').click();
  await page.getByRole('button', { name: '查看听音卡', exact: true }).click(); await page.getByRole('button', { name: '显示答案', exact: true }).click();
  const card = page.getByRole('region', { name: '听音卡', exact: true });
  await expect(card.getByRole('button', { name: '记得', exact: true })).toBeDisabled();
  await expect(card).toContainText(external.example_fr);
  await page.evaluate(() => Reflect.get(window, 'finishExternalPlayback')());
  await expect(card.getByRole('button', { name: '记得', exact: true })).toBeEnabled();
});
