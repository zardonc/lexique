import { expect, test } from '@playwright/test';
import type { Locator } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { coreCatalog, coreLookup, coreRecord, coreSenses } from './core-fixture';
import { dictionary, nativeFixture, record } from './native-fixture';

const audio = { bytes: [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))], voice: 'female' };
const longSense = {
  ...coreSenses[1],
  definition_fr: 'Définition française. '.padEnd(19_990, 'é') + ' fin-fr-80',
  meaning_en: 'Complete English meaning. '.padEnd(19_990, 'e') + ' fin-en-80',
  meaning_zh: '完整中文译义。'.padEnd(19_990, '字') + ' fin-zh-80',
  usage: 'Complete selected use. '.padEnd(19_990, 'u') + ' fin-us-80',
  field_sources: { ...coreSenses[1].field_sources,
    definition_fr: { ...coreSenses[1].field_sources.definition_fr, span: 'Selected source span. '.padEnd(19_990, 's') + ' fin-sp-80' } },
};
const longLookup = { ...coreLookup, entries: coreLookup.entries.map((entry) => entry.ordinal === 2 ? { ...entry, core_senses: [longSense] } : entry) };
const longRecord = { ...coreRecord, example_sense: 1, example_fr: 'Un être vivant.', example_zh: '一个生物。',
  digest: coreRecord.digest.map((entry) => entry.core?.frozen_row === 2 ? { ...entry, core: longSense } : entry) };

async function completeContent(scope: Locator) {
  const sense = scope.getByRole('region', { name: 'Noun (NOM)', exact: true });
  await expect(sense.locator(':scope > dl > dd[lang="fr"]')).toHaveText(longSense.definition_fr);
  await expect(sense.locator(':scope > dl > dd[lang="und"]')).toHaveText(longSense.ipa);
  await expect(sense.locator(':scope > dl > dd[lang="en"]')).toHaveText([longSense.meaning_en, longSense.usage]);
  await expect(sense.locator(':scope > dl > dd[lang="zh"]')).toHaveText(longSense.meaning_zh);
  await sense.getByText('Field sources', { exact: true }).click();
  const span = sense.getByText(longSense.field_sources.definition_fr.span, { exact: true });
  await span.scrollIntoViewIfNeeded();
  await expect(span).toBeVisible();
  expect(await sense.evaluate((element) => element.scrollWidth <= element.clientWidth + 1)).toBe(true);
}

test('complete 20,000-character core fields remain readable in lookup, saved fallback and both review backs', async ({ page }) => {
  for (const text of [longSense.definition_fr, longSense.meaning_en, longSense.meaning_zh, longSense.usage, longSense.field_sources.definition_fr.span]) {
    expect(Array.from(text)).toHaveLength(20_000);
  }
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: longLookup }, { error: { code: 'missing', message: 'Core resource removed' } }],
    vocabulary_list: [{ value: [longRecord] }], review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }], speech_cached: [{ value: audio }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase', { exact: true }).fill('être');
  await page.getByLabel('French word or phrase', { exact: true }).press('Enter');
  await completeContent(page.getByRole('region', { name: 'Dictionary text', exact: true }));
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.locator('.dictionary-original [role="alert"]')).toContainText('Core resource removed');
  await completeContent(page.getByRole('region', { name: 'Word summary', exact: true }));
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue('1');
  for (const [open, name, close] of [['Open recognition card', 'Recognition card', 'Close card'], ['Open listening card', 'Listening card', 'Skip']]) {
    await page.getByRole('button', { name: open, exact: true }).click();
    const card = page.getByRole('dialog', { name, exact: true });
    await card.getByRole('button', { name: 'Show answer', exact: true }).click();
    await completeContent(card);
    await expect(card.getByText('Un être vivant.', { exact: true })).toBeVisible();
    await card.getByRole('button', { name: close, exact: true }).click();
  }
});

const newerLookup = { ...coreLookup, sha256: 'd'.repeat(64), source: { ...coreLookup.source, version: 'pilot-ui-2' },
  entries: coreLookup.entries.map((entry) => entry.ordinal === 1 ? { ...entry,
    core_senses: [{ ...coreSenses[0], definition_fr: 'Nouvelle définition française.', meaning_zh: '当前资源中的新释义。' }] } : entry) };

test('a late newer core response identifies both versions and preserves unsaved notes and frozen meanings', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], vocabulary_list: [{ value: [coreRecord] }],
    dictionary_lookup: [{ gate: 'newer-core-1', value: newerLookup }, { gate: 'newer-core-2', value: newerLookup }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.locator('.dictionary-original [role="status"]')).toBeVisible();
  await page.getByLabel('Personal notes', { exact: true }).fill('Keep this unsaved note.');
  await page.evaluate(() => { window.testNative.release('newer-core-1'); window.testNative.release('newer-core-2'); });
  const current = page.locator('.dictionary-original');
  await expect(current.getByText('The dictionary version has changed. The saved digest keeps its original content.', { exact: true })).toBeVisible();
  await expect(current).toContainText('pilot-ui-2');
  await expect(current.getByText('当前资源中的新释义。', { exact: true })).toBeVisible();
  const frozen = page.getByRole('region', { name: 'Word summary', exact: true });
  await expect(frozen.getByText(coreSenses[0].meaning_zh, { exact: true })).toBeVisible();
  await expect(page.locator('.vocabulary-editor .dictionary-source')).toContainText('pilot-ui-1');
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('Keep this unsaved note.');
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByRole('dialog', { name: 'You have unsaved changes' }).getByRole('button', { name: 'Keep editing', exact: true }).click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('Keep this unsaved note.');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save'))).toEqual([]);
});

test('leaving a pending core lookup cannot replace the next record or its unsaved notes', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], vocabulary_list: [{ value: [coreRecord, { ...record, id: 2 }] }],
    dictionary_lookup: [{ gate: 'old-core-1', value: newerLookup }, { gate: 'old-core-2', value: newerLookup }, { value: dictionary }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').first().click();
  await expect(page.locator('.dictionary-original [role="status"]')).toBeVisible();
  await page.getByLabel('Personal notes', { exact: true }).fill('Previous draft.');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.getByRole('dialog', { name: 'You have unsaved changes' }).getByRole('button', { name: 'Discard changes', exact: true }).click();
  await page.locator('.vocabulary-list button').last().click();
  await page.getByLabel('Personal notes', { exact: true }).fill('Keep the next record draft.');
  await expect(page.locator('.dictionary-original')).toContainText('学校');
  await page.evaluate(async () => {
    window.testNative.release('old-core-1'); window.testNative.release('old-core-2');
    await new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
  });
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('Keep the next record draft.');
  await expect(page.getByRole('region', { name: 'Word summary', exact: true }).locator('h3')).toContainText('école');
  await expect(page.locator('.dictionary-original')).not.toContainText('当前资源中的新释义。');
});
