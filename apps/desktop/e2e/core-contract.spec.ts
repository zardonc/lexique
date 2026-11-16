import { expect, test } from '@playwright/test';
import { dictionarySchema } from '../src/platform/dictionary';
import { snapshotFromLookup } from '../src/platform/vocabulary';

const source = { title: 'Contract fixture', version: '1', attribution: 'Fixture author',
  source_url: 'https://example.org/etre', license: 'CC BY-SA 4.0',
  license_url: 'https://creativecommons.org/licenses/by-sa/4.0/',
  span: 'selected sense', modification: 'Scoped fixture explanation' };
const sense = { sense_id: 'etre-ver-identity', frozen_row: 3, headword: 'être', source_headword: 'être',
  pos: 'VER', usage: 'Subject + être + subject complement.',
  definition_fr: 'Relier le sujet à une qualité ou à une identité.', ipa: 'ɛtʁ',
  meaning_en: 'to be, linking a subject to a quality or identity', meaning_zh: '是；将主语与性质或身份相联系。',
  field_sources: { definition_fr: source, ipa: source, meaning_en: source, meaning_zh: source } };
const lookup = { dictionary_id: 'builtin-core', source: { title: source.title, version: source.version,
  attribution: source.attribution, license: source.license, source_url: source.source_url,
  license_url: source.license_url }, sha256: 'a'.repeat(64), entries: [
  { ordinal: 1, key: 'être', pronunciations: ['ɛtʁ'], grammar: ['AUX'], senses: [],
    core_senses: [{ ...sense, sense_id: 'etre-aux-compound', frozen_row: 1, pos: 'AUX',
      usage: 'Compound-tense auxiliary.', definition_fr: 'Former les temps composés de certains verbes.',
      meaning_en: 'compound-tense auxiliary', meaning_zh: '复合时态助动词' }] },
  { ordinal: 3, key: 'être', pronunciations: ['ɛtʁ'], grammar: ['VER'], senses: [],
    core_senses: [sense, { ...sense, sense_id: 'etre-ver-location', usage: 'Subject + être + location.',
      definition_fr: 'Se trouver dans un lieu.', meaning_en: 'to be in a place', meaning_zh: '位于某处' }] },
] };

test('the core draft freezes each selected sense in display order with separate fields and sources', async ({ page }) => {
  await page.goto('/');
  const draft = snapshotFromLookup(dictionarySchema.parse(lookup));
  expect(draft).toMatchObject({ headword: 'être', reference: { dictionary_id: 'builtin-core' }, digest: [
    { translation: null, core: { sense_id: 'etre-aux-compound', pos: 'AUX', meaning_zh: '复合时态助动词' } },
    { translation: null, core: sense },
    { translation: null, core: { sense_id: 'etre-ver-location', pos: 'VER', meaning_zh: '位于某处' } },
  ] });
});

test('core provenance accepts web links and rejects executable links', () => {
  const malicious = structuredClone(lookup);
  malicious.entries[1].core_senses[0].field_sources.meaning_zh.source_url = 'javascript:alert(1)';
  expect(dictionarySchema.safeParse(malicious).success).toBe(false);
  expect(dictionarySchema.safeParse(lookup).success).toBe(true);
});
