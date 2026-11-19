import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { coreCatalog, coreLookup, coreRecord, coreSenses } from './core-fixture';
import { nativeFixture } from './native-fixture';
import simple from './fixtures/conjugation.json' with { type: 'json' };
import defaultOrder from './fixtures/conjugation-order.json' with { type: 'json' };

const audio = { bytes: [...readFileSync(new URL('../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus', import.meta.url))], voice: 'female' };

for (const labels of [
  { locale: 'en', query: 'French word or phrase', collect: 'Add to notebook', edit: 'View and edit', summary: 'Word summary', auxiliary: 'Auxiliary (AUX)',
    coverage: '154 / 4,087 POS groups', fields: ['French definition', 'IPA', 'English meaning', 'Chinese meaning', 'Usage'], sources: 'Field sources' },
  { locale: 'fr', query: 'Mot ou expression en français', collect: 'Ajouter au carnet', edit: 'Voir et modifier', summary: 'Résumé du mot', auxiliary: 'Auxiliaire (AUX)',
    coverage: '154 / 4 087 groupes de catégories grammaticales', fields: ['Définition française', 'API', 'Sens anglais', 'Sens chinois', 'Usage'], sources: 'Sources des champs' },
  { locale: 'zh', query: '法语词或词组', collect: '加入生词本', edit: '查看与编辑', summary: '基本词汇信息', auxiliary: '助动词 (AUX)',
    coverage: '154 / 4,087 个核心词性组', fields: ['法语定义', '国际音标', '英语译义', '中文译义', '用法'], sources: '字段来源' },
]) {
  test(`core fields retain their languages with localized labels and coverage in ${labels.locale}`, async ({ page }) => {
    await nativeFixture(page, { interface_preferences: [{ value: { locale: labels.locale, theme: 'white' } }],
      dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: coreLookup }] });
    await page.goto('/');
    await expect(page.locator('.dictionary-hint')).toContainText(labels.coverage);
    await page.getByLabel(labels.query, { exact: true }).fill('être');
    await page.getByLabel(labels.query, { exact: true }).press('Enter');
    const auxiliary = page.locator('.dictionary-entry').getByRole('region', { name: labels.auxiliary, exact: true });
    await expect(auxiliary.locator(':scope > dl > dt')).toHaveText(labels.fields);
    for (const [value, language] of [
      ['Auxiliaire de certains temps composés et de la voix passive.', 'fr'], ['ɛtʁ', 'und'],
      ['Auxiliary for selected compound tenses and the passive voice.', 'en'], ['构成部分复合时态和被动语态的助动词。', 'zh'],
      [coreSenses[0].usage, 'en'],
    ]) await expect(auxiliary.getByText(value, { exact: true })).toHaveAttribute('lang', language);
    await auxiliary.getByText(labels.sources, { exact: true }).click();
    await expect(auxiliary.locator('.core-field-source h5')).toHaveText(labels.fields.slice(0, 4));
    await expect(auxiliary.getByText('Lexique test authors', { exact: true })).toHaveCount(4);
    await page.getByRole('button', { name: labels.collect, exact: true }).click();
    const draft = page.getByRole('region', { name: labels.edit, exact: true }).getByRole('region', { name: labels.summary, exact: true })
      .getByRole('region', { name: labels.auxiliary, exact: true });
    await expect(draft.locator(':scope > dl > dt')).toHaveText(labels.fields);
    await draft.getByText(labels.sources, { exact: true }).click();
    await expect(draft.locator('.core-field-source h5')).toHaveText(labels.fields.slice(0, 4));
  });
}

test('core lookup keeps POS groups, complete fields and field sources readable', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: coreLookup }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  await expect(page.getByRole('tab', { name: 'Lexique core pilot' })).toHaveAttribute('aria-selected', 'true');
  const original = page.getByRole('region', { name: 'Dictionary text', exact: true });
  await expect(original.locator('.dictionary-entry')).toHaveCount(3);
  await expect(original.getByRole('heading', { name: 'Auxiliary (AUX)' })).toBeVisible();
  await expect(original.getByRole('heading', { name: 'Noun (NOM)' })).toBeVisible();
  await expect(original.getByRole('heading', { name: 'Verb (VER)' })).toBeVisible();
  const auxiliary = original.getByRole('region', { name: 'Auxiliary (AUX)', exact: true });
  await expect(auxiliary.locator('dd[lang="fr"]')).toHaveText('Auxiliaire de certains temps composés et de la voix passive.');
  await expect(auxiliary.locator('dd[lang="und"]')).toHaveText('ɛtʁ');
  await expect(auxiliary.getByText('Auxiliary for selected compound tenses and the passive voice.', { exact: true })).toHaveAttribute('lang', 'en');
  await expect(auxiliary.getByText('构成部分复合时态和被动语态的助动词。', { exact: true })).toHaveAttribute('lang', 'zh');
  await expect(auxiliary.getByText(coreSenses[0].usage, { exact: true })).toHaveAttribute('lang', 'en');
  await auxiliary.getByText('Field sources', { exact: true }).click();
  for (const span of ['French definition span', 'IPA span', 'English meaning span', 'Chinese meaning span']) {
    await expect(auxiliary.getByText(span, { exact: true })).toBeVisible();
  }
  await expect(auxiliary.getByText('Lexique test authors', { exact: true })).toHaveCount(4);
  await expect(auxiliary.getByText('Selected for the UI fixture.', { exact: true })).toHaveCount(4);
});

test('core lookup keeps the frozen coeur headword and identifies its source spelling', async ({ page }) => {
  const sense = { ...coreSenses[1], sense_id: 'coeur-noun', frozen_row: 483, headword: 'coeur', source_headword: 'cœur',
    definition_fr: 'Organe qui fait circuler le sang.', ipa: 'kœʁ', meaning_en: 'The heart, which circulates blood.',
    meaning_zh: '心脏；推动血液循环的器官。', usage: 'The bodily organ.' };
  const lookup = { ...coreLookup, entries: [{ ordinal: 483, key: 'coeur', pronunciations: ['kœʁ'], grammar: ['NOM'], senses: [], core_senses: [sense] }] };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: lookup }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('cœur');
  await page.getByLabel('French word or phrase').press('Enter');
  const entry = page.locator('.dictionary-entry');
  await expect(entry.locator('h3')).toHaveText('coeur');
  await expect(entry.getByText('Source spelling', { exact: true })).toBeVisible();
  await expect(entry.getByText('cœur', { exact: true })).toHaveAttribute('lang', 'fr');
});

test('core conjugation links keep auxiliary and lexical verb choices distinct from the noun', async ({ page }) => {
  const conjugation = { query: 'être', tables: simple.sommes.tables.filter((table) => table.lemma === 'être') };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: coreLookup }],
    conjugation_lookup: [{ value: conjugation }], conjugation_preferences: [{ value: { mode: 'ending' } }],
    conjugation_block_order: [{ value: defaultOrder }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  const auxiliary = page.locator('.dictionary-entry[data-ordinal="1"]');
  const noun = page.locator('.dictionary-entry[data-ordinal="2"]');
  const verb = page.locator('.dictionary-entry[data-ordinal="3"]');
  await expect(auxiliary.getByRole('button', { name: 'Open conjugation: être', exact: true })).toBeEnabled();
  await expect(noun.getByRole('button', { name: /Open conjugation/ })).toHaveCount(0);
  await expect(verb.getByRole('button', { name: 'Open conjugation: être', exact: true })).toBeEnabled();
  await auxiliary.getByRole('button', { name: 'Open conjugation: être', exact: true }).focus();
  await auxiliary.getByRole('button', { name: 'Open conjugation: être', exact: true }).press('Enter');
  await expect(page.locator('.conjugation-table h3')).toHaveText('être');
  const links = page.locator('.dictionary-link button');
  await expect(links).toHaveCount(2);
  await expect(links.filter({ hasText: 'AUX · 构成部分复合时态和被动语态的助动词。' })).toBeVisible();
  const lexical = links.filter({ hasText: 'VER · 是；将主语与性质或身份相联系。' });
  await lexical.focus(); await lexical.press('Enter');
  await expect(page.locator('.dictionary-entry[data-ordinal="3"] h3')).toBeFocused();
});

test('core conflicts identify the saved meaning while retaining the current draft', async ({ page }) => {
  const original = { ...coreRecord, example_sense: 1, example_fr: 'Un être vivant.', example_zh: '一个生物。' };
  const current = { ...coreRecord, revision: 2, notes: 'Other edit', example_sense: 0,
    example_fr: 'Elle est arrivée.', example_zh: '她到了。' };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], vocabulary_list: [{ value: [original] }],
    vocabulary_save: [{ value: { outcome: 'conflict', record: current } }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('Personal notes', { exact: true }).fill('Keep my draft');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  const conflict = page.getByRole('alert', { name: 'Another editor changed this record' });
  await expect(conflict.getByText('AUX · 构成部分复合时态和被动语态的助动词。', { exact: true })).toBeVisible();
  await expect(conflict.getByText('AUX · 构成部分复合时态和被动语态的助动词。', { exact: true })).toHaveAttribute('lang', 'zh');
  await conflict.getByRole('button', { name: 'Return to editing' }).click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('Keep my draft');
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue('1');
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  const leave = page.getByRole('dialog', { name: 'You have unsaved changes' });
  await leave.getByRole('button', { name: 'Keep editing' }).click();
  await expect(page.getByLabel('Personal notes', { exact: true })).toHaveValue('Keep my draft');
});

test('saved core content stays readable when its source fails and another dictionary works', async ({ page }) => {
  const saved = { ...coreRecord, example_sense: 1, example_fr: 'Un être vivant.', example_zh: '一个生物。' };
  const unavailable = { code: 'missing', message: 'Core fixture resource unavailable' };
  const supplement = { ...coreLookup, dictionary_id: 'builtin-fra-eng', source: { ...coreLookup.source, title: 'FreeDict fra-eng 0.4.1' },
    entries: [{ ordinal: 1, key: 'être', pronunciations: ['ɛtʁ'], grammar: ['v'], senses: [{ translations: ['to be'], definitions: [], senses: [] }] }] };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], vocabulary_list: [{ value: [saved] }] });
  await page.addInitScript(({ unavailable, supplement }) => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: { key: string; dictionary_id?: string }) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => {
      const value = await invoke(command, args);
      if (command === 'dictionary_lookup' && args?.dictionary_id === 'builtin-core') throw unavailable;
      if (command === 'dictionary_lookup' && args?.dictionary_id === 'builtin-fra-eng') return supplement;
      return value;
    };
  }, { unavailable, supplement });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByRole('region', { name: 'Dictionary text', exact: true }).getByRole('alert')).toContainText('Core fixture resource unavailable');
  const summary = page.getByRole('region', { name: 'View and edit', exact: true }).getByRole('region', { name: 'Word summary', exact: true });
  await expect(summary.locator('.core-sense')).toHaveCount(3);
  const noun = summary.getByRole('region', { name: 'Noun (NOM)', exact: true });
  await expect(noun.getByText('A being, especially a living individual.', { exact: true })).toBeVisible();
  await noun.getByText('Field sources', { exact: true }).click();
  await expect(noun.getByText('Chinese meaning span', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Dictionary', exact: true }).click();
  await page.getByLabel('French word or phrase', { exact: true }).fill('être');
  await page.getByLabel('French word or phrase', { exact: true }).press('Enter');
  await expect(page.locator('.dictionary-entry').first()).toBeVisible();
  await page.getByRole('tab', { name: 'Lexique core pilot', exact: true }).click();
  await expect(page.locator('.dictionary').getByRole('alert')).toContainText('Core fixture resource unavailable');
  const other = page.getByRole('tab', { name: 'FreeDict fra-eng 0.4.1', exact: true });
  await other.focus(); await other.press('Enter');
  await expect(other).toHaveAttribute('aria-selected', 'true');
  await expect(page.locator('.dictionary-entry').getByText('to be', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(noun.getByText('存在的个体，尤指生物。', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue('1');
});

test('both core review backs show four fields, provenance and the associated example meaning', async ({ page }) => {
  const saved = { ...coreRecord, example_sense: 1, example_fr: 'Un être vivant.', example_zh: '一个生物。' };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], vocabulary_list: [{ value: [saved] }],
    review_cards: [{ value: [{ id: 10, kind: 'recognition' }, { id: 11, kind: 'listening' }] }], speech_cached: [{ value: audio }] });
  await page.goto('/');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  for (const [open, name] of [['Open recognition card', 'Recognition card'], ['Open listening card', 'Listening card']]) {
    await page.getByRole('button', { name: open, exact: true }).click();
    const card = page.getByRole('dialog', { name, exact: true });
    await expect(card.getByText('Auxiliaire de certains temps composés et de la voix passive.', { exact: true })).not.toBeVisible();
    await card.getByRole('button', { name: 'Show answer', exact: true }).click();
    const auxiliary = card.getByRole('region', { name: 'Auxiliary (AUX)', exact: true });
    await expect(auxiliary.locator('dd[lang="fr"]')).toHaveText('Auxiliaire de certains temps composés et de la voix passive.');
    await expect(auxiliary.locator('dd[lang="und"]')).toHaveText('ɛtʁ');
    await expect(auxiliary.getByText('Auxiliary for selected compound tenses and the passive voice.', { exact: true })).toHaveAttribute('lang', 'en');
    await expect(auxiliary.getByText('构成部分复合时态和被动语态的助动词。', { exact: true })).toHaveAttribute('lang', 'zh');
    await expect(auxiliary.getByText(coreSenses[0].usage, { exact: true })).toBeVisible();
    await expect(card.getByText('Meaning for examples：NOM · 存在的个体，尤指生物。', { exact: true })).toBeVisible();
    await expect(card.getByText('NOM · 存在的个体，尤指生物。', { exact: true })).toHaveAttribute('lang', 'zh');
    await expect(card.getByText('Un être vivant.', { exact: true })).toBeVisible();
    await auxiliary.getByText('Field sources', { exact: true }).click();
    await expect(auxiliary.getByText('French definition span', { exact: true })).toBeVisible();
    await card.getByRole('button', { name: name === 'Recognition card' ? 'Close card' : 'Skip', exact: true }).click();
  }
});

test('core saving and notebook details keep selected example meaning and complete digest', async ({ page }) => {
  const saved = { ...coreRecord, example_sense: 1, example_fr: 'Un être vivant.', example_zh: '一个生物。' };
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: coreLookup }],
    vocabulary_save: [{ value: { outcome: 'created', record: saved } }], vocabulary_list: [{ value: [saved] }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  await page.getByRole('button', { name: 'Add to notebook', exact: true }).click();
  const summary = page.getByRole('region', { name: 'View and edit' }).getByRole('region', { name: 'Word summary' });
  await expect(summary.getByText('Auxiliaire de certains temps composés et de la voix passive.', { exact: true })).toBeVisible();
  await expect(summary.getByText('To be, linking a subject to a quality or identity.', { exact: true })).toHaveAttribute('lang', 'en');
  await page.getByLabel('French example', { exact: true }).fill('Un être vivant.');
  await page.getByLabel('Chinese translation of example', { exact: true }).fill('一个生物。');
  await page.getByLabel('Meaning for examples', { exact: true }).selectOption({ label: 'NOM · 存在的个体，尤指生物。' });
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByRole('status').filter({ hasText: /^Saved$/ })).toBeVisible();
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue('1');
  await page.getByRole('button', { name: 'Notebook', exact: true }).click();
  const item = page.locator('.vocabulary-list button');
  await expect(item).toContainText('存在的个体，尤指生物。');
  await expect(item.getByText('存在的个体，尤指生物。', { exact: true })).toHaveAttribute('lang', 'zh');
  await expect(item).not.toContainText('Not provided');
  await item.click();
  await expect(page.getByLabel('Meaning for examples', { exact: true })).toHaveValue('1');
  await expect(page.getByRole('region', { name: 'Word summary' }).locator('.core-sense')).toHaveCount(3);
  await expect(page.getByLabel('French example', { exact: true })).toHaveValue('Un être vivant.');
  const savedNoun = page.getByRole('region', { name: 'Word summary' }).getByRole('region', { name: 'Noun (NOM)', exact: true });
  await savedNoun.getByText('Field sources', { exact: true }).click();
  await expect(savedNoun.getByText('Chinese meaning span', { exact: true })).toBeVisible();
});


test('batch grammatical subclasses survive lookup and remain readable', async ({ page }) => {
  const subclasses = ['PRO:per', 'PRE', 'ART:def', 'CON', 'ADJ:num', 'ART:ind', 'PRO:ind', 'PRO:int', 'PRO:rel', 'PRO:dem', 'ADJ:ind', 'ADJ:dem', 'ADJ:pos', 'ONO', 'ADJ:int', 'PRO:pos'];
  const labels = ['Personal pronoun', 'Preposition', 'Definite article', 'Conjunction', 'Numeral adjective', 'Indefinite article', 'Indefinite pronoun', 'Interrogative pronoun', 'Relative pronoun', 'Demonstrative pronoun', 'Indefinite adjective', 'Demonstrative adjective', 'Possessive adjective', 'Interjection / onomatopoeia', 'Interrogative adjective', 'Possessive pronoun'];
  const entries = subclasses.map((pos, index) => {
    const sense = { ...coreSenses[0], sense_id: `batch-${index}`, frozen_row: index + 6, pos };
    return { ...coreLookup.entries[0], ordinal: index + 6, grammar: [pos], core_senses: [sense] };
  });
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'en', theme: 'white' } }],
    dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: { ...coreLookup, entries } }] });
  await page.goto('/');
  await page.getByLabel('French word or phrase').fill('être');
  await page.getByLabel('French word or phrase').press('Enter');
  for (const [index, pos] of subclasses.entries()) {
    await expect(page.locator(`.dictionary-entry .core-sense[data-pos="${pos}"] h4`)).toHaveText(`${labels[index]} (${pos})`);
  }
});

for (const [locale, query, labels] of [
  ['fr', 'Mot ou expression en français', ['Adjectif interrogatif', 'Pronom possessif']],
  ['zh', '法语词或词组', ['疑问形容词', '物主代词']],
] as const) {
  test(`new batch grammatical subclasses are localized in ${locale}`, async ({ page }) => {
    const subclasses = ['ADJ:int', 'PRO:pos'] as const;
    const entries = subclasses.map((pos, index) => ({ ...coreLookup.entries[0], grammar: [pos],
      core_senses: [{ ...coreSenses[0], sense_id: `batch-${index}`, pos }] }));
    await nativeFixture(page, { interface_preferences: [{ value: { locale, theme: 'white' } }],
      dictionary_settings: [{ value: coreCatalog }], dictionary_lookup: [{ value: { ...coreLookup, entries } }] });
    await page.goto('/');
    await page.getByLabel(query, { exact: true }).fill('être');
    await page.getByLabel(query, { exact: true }).press('Enter');
    for (const [index, pos] of subclasses.entries()) {
      await expect(page.locator(`.dictionary-entry .core-sense[data-pos="${pos}"] h4`)).toHaveText(`${labels[index]} (${pos})`);
    }
  });
}
