import type { CoreSense, DictionaryCatalog, DictionaryLookup } from '../src/platform/dictionary';
import type { VocabularyRecord } from '../src/platform/vocabulary';

const source = (span: string) => ({ title: 'Lexique UI field fixture', version: 'pilot-ui-1',
  attribution: 'Lexique test authors', license: 'MIT', source_url: 'https://example.com/lexique/fields',
  license_url: 'https://example.com/lexique/license', span, modification: 'Selected for the UI fixture.' });
const fields = { definition_fr: source('French definition span'), ipa: source('IPA span'),
  meaning_en: source('English meaning span'), meaning_zh: source('Chinese meaning span') };

export const coreSenses: readonly CoreSense[] = [
  { sense_id: 'etre-aux', frozen_row: 1, headword: 'être', source_headword: 'être', pos: 'AUX', ipa: 'ɛtʁ',
    definition_fr: 'Auxiliaire de certains temps composés et de la voix passive.',
    meaning_en: 'Auxiliary for selected compound tenses and the passive voice.', meaning_zh: '构成部分复合时态和被动语态的助动词。',
    usage: 'In compound tenses, être combines with the past participle of a verb that selects this auxiliary. The past participle agrees with the subject where that construction requires it. In the passive voice, être combines with the past participle of a transitive verb: the subject receives the action. These constructions do not make être a lexical verb meaning to possess or a noun meaning a living being. Keep the auxiliary choice, the passive construction and their agreement conditions when interpreting the example; this final restriction belongs to the complete selected use.',
    field_sources: fields },
  { sense_id: 'etre-noun', frozen_row: 2, headword: 'être', source_headword: 'être', pos: 'NOM', ipa: 'ɛtʁ',
    definition_fr: 'Individu qui existe.', meaning_en: 'A being, especially a living individual.', meaning_zh: '存在的个体，尤指生物。',
    usage: 'A noun, as in un être vivant.', field_sources: fields },
  { sense_id: 'etre-verb', frozen_row: 3, headword: 'être', source_headword: 'être', pos: 'VER', ipa: 'ɛtʁ',
    definition_fr: 'Relier un sujet à une qualité ou à une identité.', meaning_en: 'To be, linking a subject to a quality or identity.', meaning_zh: '是；将主语与性质或身份相联系。',
    usage: 'A linking verb, as in elle est médecin.', field_sources: fields },
];

export const coreLookup: DictionaryLookup = { dictionary_id: 'builtin-core',
  source: { title: 'Lexique core pilot', version: 'pilot-ui-1', attribution: 'Lexique test authors', license: 'MIT',
    source_url: 'https://example.com/lexique/core', license_url: 'https://example.com/lexique/license' },
  sha256: 'c'.repeat(64), entries: coreSenses.map((sense) => ({ ordinal: sense.frozen_row,
    key: sense.headword, pronunciations: [sense.ipa], grammar: [sense.pos], senses: [], core_senses: [sense] })) };

export const coreCatalog: DictionaryCatalog = [
  { id: 'builtin-core', title: 'Lexique core pilot', content_hash: null, availability: 'available' },
  { id: 'builtin', title: 'FreeDict fra-zho', content_hash: null, availability: 'available' },
  { id: 'builtin-fra-eng', title: 'FreeDict fra-eng 0.4.1', content_hash: null, availability: 'available' },
];

export const coreRecord: VocabularyRecord = { id: 1, headword: 'être', revision: 1, created_at: 1000, updated_at: 1000,
  notes: '', example_fr: '', example_zh: '', example_sense: null, more_examples: [],
  reference: { dictionary_id: 'builtin-core', dictionary: coreLookup.source.title, version: coreLookup.source.version,
    attribution: coreLookup.source.attribution, lookup_key: 'être', license: coreLookup.source.license,
    source_url: coreLookup.source.source_url, license_url: coreLookup.source.license_url, sha256: coreLookup.sha256 },
  digest: coreSenses.map((core) => ({ pronunciations: [core.ipa], grammar: [core.pos], translation: null, core })) };
