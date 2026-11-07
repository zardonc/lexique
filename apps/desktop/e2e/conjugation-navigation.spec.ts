import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { dictionary, nativeFixture, record } from './native-fixture';
import coloring from './fixtures/conjugation-coloring.json' with { type: 'json' };
import defaultOrder from './fixtures/conjugation-order.json' with { type: 'json' };
import pronominal from './fixtures/conjugation-pronominal.json' with { type: 'json' };
import simple from './fixtures/conjugation.json' with { type: 'json' };

const laver = { ...dictionary, entries: [{ ordinal: 6448, key: 'laver', pronunciations: ['la.ve'], grammar: ['v'],
  senses: [{ translations: ['洗'], definitions: [], senses: [] }] }] };
const customOrder = ['GPP', ...defaultOrder.slice(0, -1)];
const learning = { ...record, headword: 'laver', notes: 'existing notes', reference: { ...record.reference, lookup_key: 'laver' },
  digest: [{ pronunciations: ['la.ve'], grammar: ['v'], translation: '洗' }] };

async function lookupResponses(page: Page, dictionaries: Record<string, typeof dictionary>, conjugations: Record<string, { tables: readonly unknown[] }>) {
  await page.addInitScript(({ dictionaries, conjugations, empty }) => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: { key: string }) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = (command, args) => {
      const key = args?.key ?? '';
      if (command === 'dictionary_lookup') return Promise.resolve(dictionaries[key] ?? empty);
      if (command === 'conjugation_lookup') return Promise.resolve({ ...conjugations[key], query: key, tables: conjugations[key]?.tables ?? [] });
      return invoke(command, args);
    };
  }, { dictionaries, conjugations, empty: { ...dictionary, entries: [] } });
}

test('dictionary and conjugation share the selected verb while preserving coloring and block order', async ({ page }) => {
  await nativeFixture(page, {
    dictionary_lookup: [{ value: laver }],
    conjugation_lookup: [{ value: { query: 'laver', tables: [coloring.laver] } }],
    conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: customOrder }],
  });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByLabel('French word or phrase').fill('laver');
  await page.getByRole('button', { name: 'Look up', exact: true }).click();
  const conjugation = page.getByRole('button', { name: 'Open conjugation: laver', exact: true });
  await expect(conjugation).toBeEnabled();
  await conjugation.focus(); await conjugation.press('Enter');
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  await expect(page.locator('.conjugation-table h3')).toBeFocused();
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('.conjugation-block').first()).toHaveAttribute('data-tense', 'GPP');
  const back = page.getByRole('button', { name: 'Open dictionary: laver', exact: true });
  await expect(back).toBeEnabled(); await back.focus(); await back.press('Enter');
  await expect(page.getByLabel('French word or phrase')).toHaveValue('laver');
  await expect(page.locator('.dictionary-entry h3')).toHaveText('laver');
  await expect(page.locator('.dictionary-entry h3')).toBeFocused();
  await expect(page.locator('.dictionary-entry')).toContainText('洗');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('.conjugation-block').first()).toHaveAttribute('data-tense', 'GPP');
  await expect(page.getByLabel('Personal notes')).toHaveCount(0);
});

test('ambiguous dictionary spellings require a choice and keep the chosen entry identity', async ({ page }) => {
  const curly = { ...laver, entries: [{ ...laver.entries[0], ordinal: 1, key: 's’abstenir' }] };
  const straight = { ...laver, entries: [{ ...laver.entries[0], ordinal: 2, key: "s'abstenir" }] };
  await nativeFixture(page, { conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }] });
  await lookupResponses(page, { 's’abstenir': curly, "s'abstenir": straight }, { 'abstenues': pronominal['elles se sont abstenues'] });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill('abstenues');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await expect(page.locator('.dictionary-link')).toContainText('Choose the corresponding dictionary entry.');
  await expect(page.locator('.dictionary-link button')).toHaveCount(2);
  await expect(page.getByLabel('French word or phrase')).toHaveCount(0);
  await page.getByRole('button', { name: /^Open dictionary: s'abstenir/ }).click();
  await expect(page.getByLabel('French word or phrase')).toHaveValue("s'abstenir");
  await expect(page.locator('.dictionary-entry h3')).toHaveText("s'abstenir");
  await expect(page.locator('.dictionary-entry h3')).toBeFocused();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'vocabulary_save' || call.command.startsWith('review_add')))).toBe(false);
});

test('returning through dictionary preserves access to all conjugation lookup candidates', async ({ page }) => {
  const suivre = { ...laver, entries: [{ ...laver.entries[0], key: 'suivre' }] };
  await nativeFixture(page, { conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: customOrder }] });
  await lookupResponses(page, { suivre }, { suis: simple.suis, suivre: { tables: [simple.suis.tables[0]] } });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill('suis');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await expect(page.getByRole('group', { name: 'Matching verbs' }).getByRole('button')).toHaveCount(2);
  await page.getByRole('group', { name: 'Matching verbs' }).getByRole('button', { name: /^suivre/ }).click();
  await page.getByRole('button', { name: 'Open dictionary: suivre', exact: true }).click();
  await expect(page.locator('.dictionary-entry h3')).toHaveText('suivre');
  await page.getByRole('button', { name: 'Open conjugation: suivre', exact: true }).click();
  await expect(page.getByRole('group', { name: 'Matching verbs' }).getByRole('button')).toHaveCount(2);
  await expect(page.locator('.conjugation-table h3')).toHaveText('suivre');
  await expect(page.getByLabel('French verb or simple form')).toHaveValue('suis');
  await expect(page.locator('.conjugation-block').first()).toHaveAttribute('data-tense', 'GPP');
});

test('noun homographs, unmatched verbs and ambiguous conjugation lookup stay explicit', async ({ page }) => {
  const noun = { ...laver, entries: [{ ...laver.entries[0], key: 'été', grammar: ['n'] }] };
  const other = { ...laver, entries: [{ ...laver.entries[0], key: 'être', grammar: ['n'] }] };
  await nativeFixture(page, { conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }] });
  await lookupResponses(page, { 'été': noun, laver, 'être': other }, { suis: simple.suis });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  const input = page.getByLabel('French word or phrase');
  await input.fill('été'); await input.press('Enter');
  await expect(page.locator('.dictionary-entry')).toContainText('été');
  await expect(page.locator('.conjugation-link')).toHaveCount(0);
  await input.fill('laver'); await input.press('Enter');
  await expect(page.locator('.conjugation-link')).toHaveText('No verified conjugation link for this dictionary verb.');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill('suis');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await expect(page.getByText('Choose a verb to open its table.', { exact: true })).toBeVisible();
  await expect(page.locator('.conjugation-table')).toHaveCount(0);
  await page.getByRole('group', { name: 'Matching verbs' }).getByRole('button', { name: /^être/ }).click();
  await expect(page.locator('.dictionary-link')).toHaveText('No corresponding dictionary verb entry.');
});

test('dictionary links protect edits through stay, failed save, save and leave, and discard', async ({ page }) => {
  const saved = { ...learning, notes: 'saved changes', revision: 2 };
  await nativeFixture(page, {
    vocabulary_find: [{ value: learning }], vocabulary_list: [{ value: [saved] }],
    vocabulary_save: [{ error: 'disk full' }, { value: { outcome: 'updated', record: saved } }],
    conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: customOrder }],
  });
  await lookupResponses(page, { laver }, { laver: { tables: [coloring.laver] } });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByLabel('French word or phrase').fill('laver'); await page.getByRole('button', { name: 'Look up', exact: true }).click();
  await page.getByRole('button', { name: 'View existing record', exact: true }).click();
  await page.getByLabel('Personal notes', { exact: true }).fill(saved.notes);
  const link = page.getByRole('button', { name: 'Open conjugation: laver', exact: true });
  await link.click();
  const dialog = page.getByRole('dialog', { name: 'You have unsaved changes' });
  await dialog.getByRole('button', { name: 'Keep editing' }).click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(saved.notes);
  await expect(page.locator('.conjugation-table')).toHaveCount(0);
  await link.click(); await dialog.getByRole('button', { name: 'Save and leave' }).click();
  await expect(dialog).toContainText('disk full');
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(saved.notes);
  await dialog.getByRole('button', { name: 'Save and leave' }).click();
  await expect(dialog).toBeHidden(); await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click(); await page.locator('.vocabulary-list button').click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(saved.notes);
  await page.getByLabel('Personal notes', { exact: true }).fill('discard this edit');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await dialog.getByRole('button', { name: 'Discard changes' }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('laver');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click(); await page.locator('.vocabulary-list button').click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(saved.notes);
});

for (const outcome of ['existing', 'conflict'] as const) {
  test(`${outcome} save outcomes keep draft edits and block dictionary-to-conjugation navigation`, async ({ page }) => {
    await nativeFixture(page, {
      ...(outcome === 'conflict' ? { vocabulary_find: [{ value: learning }] } : {}),
      vocabulary_save: [{ value: { outcome, record: { ...learning, notes: 'other saved content', revision: 2 } } }],
      conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
    });
    await lookupResponses(page, { laver }, { laver: { tables: [coloring.laver] } });
    await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
    await page.getByLabel('French word or phrase').fill('laver'); await page.getByRole('button', { name: 'Look up', exact: true }).click();
    await page.getByRole('button', { name: outcome === 'existing' ? 'Add to notebook' : 'View existing record', exact: true }).click();
    await page.getByLabel('Personal notes', { exact: true }).fill('my pending edit');
    await page.getByRole('button', { name: 'Open conjugation: laver', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'You have unsaved changes' });
    await dialog.getByRole('button', { name: 'Save and leave' }).click();
    await expect(dialog).toContainText(outcome === 'existing' ? 'Already in notebook' : 'Another editor changed this record');
    await expect(page.locator('.conjugation-table')).toHaveCount(0);
    await dialog.getByRole('button', { name: 'Keep editing' }).click();
    await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('my pending edit');
  });
}

test('a conjugation lemma can return to its verified straight-apostrophe dictionary spelling', async ({ page }) => {
  const straight = { ...laver, entries: [{ ...laver.entries[0], key: "s'abstenir", senses: [{ translations: ['避免'], definitions: [], senses: [] }] }] };
  await nativeFixture(page, {
    conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
  });
  await lookupResponses(page, { "s'abstenir": straight }, { "S'ABSTENIR": pronominal['elles se sont abstenues'] });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill("S'ABSTENIR");
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('s’abstenir');
  const back = page.getByRole('button', { name: "Open dictionary: s'abstenir", exact: true });
  await expect(back).toBeVisible(); await back.click();
  await expect(page.getByLabel('French word or phrase')).toHaveValue("s'abstenir");
  await expect(page.locator('.dictionary-entry h3')).toHaveText("s'abstenir");
  await expect(page.getByLabel('Personal notes')).toHaveCount(0);
});

test('return navigation keeps the originating case and apostrophe-distinct learning record', async ({ page }) => {
  const original = "S'ABSTENIR", canonical = 's’abstenir';
  const lookup = { ...laver, entries: [{ ...laver.entries[0], key: original }] };
  const saved = { ...learning, headword: original, notes: 'exact original record', reference: { ...learning.reference, lookup_key: original } };
  const other = { ...saved, id: 2, headword: canonical, notes: 'separate record' };
  await nativeFixture(page, {
    vocabulary_find: [{ value: saved }], vocabulary_list: [{ value: [saved, other] }],
    conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
  });
  await lookupResponses(page, { [original]: lookup }, { [original]: pronominal['elles se sont abstenues'] });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByLabel('French word or phrase').fill(original); await page.getByRole('button', { name: 'Look up', exact: true }).click();
  await page.getByRole('button', { name: 'Open conjugation: s’abstenir', exact: true }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText(canonical);
  await page.getByRole('button', { name: `Open dictionary: ${original}`, exact: true }).click();
  await page.getByRole('button', { name: 'View existing record', exact: true }).click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue(saved.notes);
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await expect(page.locator('.vocabulary-list li')).toHaveCount(2);
  await expect(page.locator('.vocabulary-list')).toContainText(original);
  await expect(page.locator('.vocabulary-list')).toContainText(canonical);
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'vocabulary_save' || call.command.startsWith('review_add')))).toBe(false);
});

test('ordinary dictionary entries do not invent pronominal relationships', async ({ page }) => {
  await nativeFixture(page, { conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: customOrder }] });
  await lookupResponses(page, { laver }, { laver: { tables: [coloring.laver] }, 'se laver': pronominal['se laver'] });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByLabel('French word or phrase').fill('laver'); await page.getByRole('button', { name: 'Look up', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Open conjugation: se laver', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: 'Open conjugation: laver', exact: true }).click();
  await page.getByLabel('French verb or simple form').fill('se laver');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await expect(page.locator('.conjugation-table h3')).toHaveText('se laver');
  await expect(page.locator('.dictionary-link')).toHaveText('No corresponding dictionary verb entry.');
  await expect(page.getByRole('button', { name: 'Open dictionary: laver', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('.conjugation-block').first()).toHaveAttribute('data-tense', 'GPP');
});

for (const direction of ['dictionary', 'conjugation'] as const) {
  test(`${direction} link read failures remain visible and cannot navigate`, async ({ page }) => {
    await nativeFixture(page, {
      dictionary_lookup: [{ ...(direction === 'dictionary' ? { value: laver } : { error: { code: 'corrupt', message: 'dictionary unreadable' } }) }],
      conjugation_lookup: [{ ...(direction === 'conjugation' ? { value: { query: 'laver', tables: [coloring.laver] } } : { error: { code: 'missing', message: 'conjugation removed' } }) }],
      conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
    });
    await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
    if (direction === 'dictionary') {
      await page.getByLabel('French word or phrase').fill('laver'); await page.getByRole('button', { name: 'Look up', exact: true }).click();
      await expect(page.locator('.dictionary-original').getByRole('alert')).toContainText('conjugation removed');
      await expect(page.getByRole('button', { name: 'Open conjugation: laver', exact: true })).toHaveCount(0);
    } else {
      await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
      await page.getByLabel('French verb or simple form').fill('laver'); await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
      await expect(page.locator('.conjugation-table').getByRole('alert')).toContainText('dictionary unreadable');
      await expect(page.getByRole('button', { name: 'Open dictionary: laver', exact: true })).toHaveCount(0);
    }
  });

  test(`${direction} link responses arriving after leaving cannot reopen a destination`, async ({ page }) => {
    await nativeFixture(page, {
      dictionary_lookup: [{ value: laver, ...(direction === 'conjugation' ? { gate: 'link' } : {}) }],
      conjugation_lookup: [{ value: { query: 'laver', tables: [coloring.laver] }, ...(direction === 'dictionary' ? { gate: 'link' } : {}) }],
      conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: defaultOrder }],
    });
    await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
    if (direction === 'dictionary') {
      await page.getByLabel('French word or phrase').fill('laver'); await page.getByRole('button', { name: 'Look up', exact: true }).click();
      await expect(page.locator('.dictionary-original')).toContainText('Looking up…');
    } else {
      await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
      await page.getByLabel('French verb or simple form').fill('laver'); await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
      await expect(page.locator('.conjugation-table')).toContainText('Looking up…');
    }
    await page.getByRole('button', { name: 'Notebook', exact: true }).click();
    await page.evaluate(() => window.testNative.release('link'));
    await expect(page.getByRole('heading', { name: 'Notebook', level: 1 })).toBeVisible();
    await expect(page.locator('.conjugation-table, .dictionary-entry')).toHaveCount(0);
  });
}

test('navigation labels, keyboard destinations and preferences work in all languages and themes', async ({ page }) => {
  await nativeFixture(page, { conjugation_preferences: [{ value: { mode: 'main' } }], conjugation_block_order: [{ value: customOrder }] });
  await lookupResponses(page, { laver }, { laver: { tables: [coloring.laver] } });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('laver'); await page.getByRole('button', { name: '查词', exact: true }).click();
  for (const [locale, theme, forward, back] of [
    ['zh', 'white', '打开动词变位: laver', '打开词典: laver'],
    ['fr', 'sepia', 'Ouvrir la conjugaison: laver', 'Ouvrir le dictionnaire: laver'],
    ['en', 'dark', 'Open conjugation: laver', 'Open dictionary: laver'],
  ]) {
    await page.getByRole('banner').getByRole('combobox').nth(0).selectOption(locale);
    await page.getByRole('banner').getByRole('combobox').nth(1).selectOption(theme);
    const link = page.getByRole('button', { name: forward, exact: true });
    await link.focus(); await link.press('Space');
    await expect(page.locator('.conjugation-table h3')).toBeFocused();
    await expect(page.locator('.conjugation-block').first()).toHaveAttribute('data-tense', 'GPP');
    await expect(page.locator('.app')).toHaveAttribute('data-theme', theme);
    const returnLink = page.getByRole('button', { name: back, exact: true });
    await returnLink.focus(); await returnLink.press('Enter');
    await expect(page.locator('.dictionary-entry h3')).toBeFocused();
  }
});
