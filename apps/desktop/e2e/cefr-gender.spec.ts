import { test, expect, type Page } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';
import { coreCatalog, coreLookup, coreRecord, coreSenses } from './core-fixture';

async function importedCefr(page: Page) {
  await page.addInitScript(() => {
    const bridge = (window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> } }).__TAURI_INTERNALS__;
    const invoke = bridge.invoke;
    bridge.invoke = async (command, args) => {
      if (command !== 'cefr_lookup') return invoke(command, args);
      window.testNative.calls.push({ command, args });
      const queries = args?.queries as { key: string; sense: { sense_id: string; pos: string } | null }[];
      return { imported: true, entries: queries.map((query) => ({ key: query.key, sense_id: query.sense?.sense_id ?? null,
        level: query.sense?.pos === 'NOM' ? 'B1' : 'A2' })) };
    };
  });
}

test('gender and local CEFR appear on saved views and review backs while the prompt stays headword-only', async ({ page }) => {
  const senses = coreSenses.map((sense) => sense.pos === 'NOM' ? { ...sense, gender: 'm' as const,
    field_sources: { ...sense.field_sources, gender: sense.field_sources.ipa } } : sense);
  const saved = { ...coreRecord, digest: senses.map((core) => ({ pronunciations: [core.ipa], grammar: [core.pos], translation: null, core })) };
  const lookup = { ...coreLookup, entries: coreLookup.entries.map((entry, index) => ({ ...entry, core_senses: [senses[index]] })) };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: lookup }], vocabulary_list: [{ value: [saved] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }] }] });
  await importedCefr(page);
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  const noun = page.locator('.dictionary-entry[data-ordinal="2"]');
  await expect(noun.getByText('Masculine (m)', { exact: true })).toBeVisible();
  await expect(noun.locator('.cefr-badge')).toHaveText('B1');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'être' }));
  await page.getByRole('button', { name: 'Look up input', exact: true }).click();
  await expect(page.locator('.dictionary-entry[data-ordinal="2"] .cefr-badge')).toHaveText('B1');
  await page.getByRole('button', { name: 'Notes', exact: true }).click();
  await expect(page.locator('.vocabulary-list .cefr-badge')).toHaveText('A2');
  await page.locator('.vocabulary-list button').click();
  await expect(page.locator('.dictionary-original .core-sense[data-pos="NOM"] .cefr-badge')).toHaveText('B1');
  await page.getByRole('button', { name: 'Open recognition card', exact: true }).click();
  const card = page.getByRole('dialog', { name: 'Recognition card', exact: true });
  await expect(card.locator('.cefr-badge')).toHaveCount(0);
  await expect(card.getByText('Masculine (m)', { exact: true })).toHaveCount(0);
  await card.getByRole('button', { name: 'Show answer', exact: true }).click();
  await expect(card.getByText('Masculine (m)', { exact: true })).toBeVisible();
  await expect(card.locator('.core-sense[data-pos="NOM"] .cefr-badge')).toHaveText('B1');
});

test('the unsaved collection editor displays imported CEFR grades', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: coreLookup }] });
  await importedCefr(page);
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  await expect(page.locator('.dictionary-original .core-sense[data-pos="NOM"] .cefr-badge')).toHaveText('B1');
  await page.getByRole('button', { name: 'Add to notebook', exact: true }).click();
  await expect(page.locator('.vocabulary-editor .vocabulary-summary .core-sense[data-pos="NOM"] .cefr-badge')).toHaveText('B1');
});

test('notebook CEFR batches respect the 200-query limit and follow the visible page', async ({ page }) => {
  const rows = Array.from({ length: 20 }, (_, index) => ({ ...coreRecord, id: index + 1, headword: `mot-${index}`,
    digest: Array.from({ length: 11 }, (_, sense) => ({ ...coreRecord.digest[1], core: { ...coreSenses[1],
      headword: `mot-${index}`, sense_id: `mot-${index}-${sense}` } })) }));
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }], vocabulary_list: [{ value: [...rows, { ...record, id: 21 }] }] });
  await importedCefr(page);
  await page.goto('/');
  await page.getByRole('button', { name: 'Notes', exact: true }).click();
  await expect(page.locator('.vocabulary-list .cefr-badge')).toHaveCount(20);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'cefr_lookup').map((call) => (call.args?.queries as unknown[]).length))).toEqual([200, 20]);
  await page.getByRole('button', { name: 'Next page', exact: true }).click();
  await expect(page.locator('.vocabulary-list .cefr-badge')).toHaveText('A2');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'cefr_lookup').at(-1)?.args?.queries)).toEqual([{ key: 'école', sense: null }]);
});

test('the visible notebook page uses one CEFR request and one failure banner with retry', async ({ page }) => {
  const rows = [coreRecord, { ...record, id: 2 }];
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    vocabulary_list: [{ value: rows }], cefr_lookup: [{ error: { code: 'query_failed', message: 'Unreadable local TSV' } }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notes', exact: true }).click();
  await expect(page.locator('.vocabulary-list li')).toHaveCount(2);
  await expect(page.getByRole('alert').filter({ hasText: 'Unreadable local TSV' }).first()).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'cefr_lookup').length)).toBe(1);
  await expect(page.getByRole('alert').filter({ hasText: 'Unreadable local TSV' })).toHaveCount(1);
  await page.getByRole('alert').filter({ hasText: 'Unreadable local TSV' }).getByRole('button', { name: 'Retry', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'cefr_lookup').length)).toBe(2);
});

test('CEFR settings preserve an installed resource on rejected replacement and support remove and the official link', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    cefr_settings: [{ value: { row_count: 2 } }], cefr_import: [{ error: { code: 'invalid_query', message: 'Expected the exact 10-column FleLex_TT_Beacco.tsv header' } }, { value: { row_count: 3 } }],
    cefr_remove: [{ value: { row_count: null } }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  const section = page.getByRole('region', { name: 'Optional CEFR grading (FLELex TT-Beacco)', exact: true });
  await section.getByRole('button', { name: 'Open official download page', exact: true }).click();
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'dictionary_resource_open')?.args?.id)).toBe('flelex-tt-beacco');
  await section.getByLabel('Replace .tsv').setInputFiles({ name: 'wrong.tsv', mimeType: 'text/tab-separated-values', buffer: Buffer.from('word\ttag\tlevel') });
  await expect(section.getByRole('alert')).toContainText('10-column');
  await expect(section.getByRole('status')).toHaveText('Imported rows: 2');
  await section.getByLabel('Replace .tsv').setInputFiles({ name: 'FleLex_TT_Beacco.tsv', mimeType: 'text/tab-separated-values', buffer: Buffer.from('synthetic fixture') });
  await expect(section.getByRole('status')).toHaveText('Imported rows: 3');
  await section.getByRole('button', { name: 'Remove CEFR resource', exact: true }).click();
  await expect(section.getByRole('status')).toHaveText('No CEFR resource imported');
  await expect(section.locator('.cefr-badge')).toHaveCount(0);
});
