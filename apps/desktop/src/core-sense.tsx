import type { CoreSense } from './platform/dictionary';
import { digestMeaning } from './platform/vocabulary';
import type { DigestEntry } from './platform/vocabulary';

export const coreMessages = {
  en: { definitionFr: 'French definition', ipa: 'IPA', meaningEn: 'English meaning', meaningZh: 'Chinese meaning', usage: 'Usage', sourceSpelling: 'Source spelling',
    sources: 'Field sources', source: 'Source', version: 'Version', attribution: 'Attribution', license: 'Licence',
    sourceUrl: 'Source link', licenseUrl: 'Licence link', span: 'Source span', modification: 'Modifications',
    pos: { AUX: 'Auxiliary', NOM: 'Noun', VER: 'Verb', ADV: 'Adverb', ADJ: 'Adjective',
      'PRO:per': 'Personal pronoun', 'PRE': 'Preposition', 'ART:def': 'Definite article', 'CON': 'Conjunction', 'ADJ:num': 'Numeral adjective', 'ART:ind': 'Indefinite article', 'PRO:ind': 'Indefinite pronoun', 'PRO:int': 'Interrogative pronoun', 'PRO:rel': 'Relative pronoun', 'PRO:dem': 'Demonstrative pronoun', 'ADJ:ind': 'Indefinite adjective', 'ADJ:dem': 'Demonstrative adjective', 'ADJ:pos': 'Possessive adjective' } },
  fr: { definitionFr: 'Définition française', ipa: 'API', meaningEn: 'Sens anglais', meaningZh: 'Sens chinois', usage: 'Usage', sourceSpelling: 'Graphie de la source',
    sources: 'Sources des champs', source: 'Source', version: 'Version', attribution: 'Attribution', license: 'Licence',
    sourceUrl: 'Lien de source', licenseUrl: 'Lien de licence', span: 'Passage source', modification: 'Modifications',
    pos: { AUX: 'Auxiliaire', NOM: 'Nom', VER: 'Verbe', ADV: 'Adverbe', ADJ: 'Adjectif',
      'PRO:per': 'Pronom personnel', 'PRE': 'Préposition', 'ART:def': 'Article défini', 'CON': 'Conjonction', 'ADJ:num': 'Adjectif numéral', 'ART:ind': 'Article indéfini', 'PRO:ind': 'Pronom indéfini', 'PRO:int': 'Pronom interrogatif', 'PRO:rel': 'Pronom relatif', 'PRO:dem': 'Pronom démonstratif', 'ADJ:ind': 'Adjectif indéfini', 'ADJ:dem': 'Adjectif démonstratif', 'ADJ:pos': 'Adjectif possessif' } },
  zh: { definitionFr: '法语定义', ipa: '国际音标', meaningEn: '英语译义', meaningZh: '中文译义', usage: '用法', sourceSpelling: '来源拼写',
    sources: '字段来源', source: '来源', version: '版本', attribution: '署名', license: '许可',
    sourceUrl: '来源链接', licenseUrl: '许可链接', span: '来源片段', modification: '修改说明',
    pos: { AUX: '助动词', NOM: '名词', VER: '动词', ADV: '副词', ADJ: '形容词',
      'PRO:per': '人称代词', 'PRE': '介词', 'ART:def': '定冠词', 'CON': '连词', 'ADJ:num': '数词形容词', 'ART:ind': '不定冠词', 'PRO:ind': '不定代词', 'PRO:int': '疑问代词', 'PRO:rel': '关系代词', 'PRO:dem': '指示代词', 'ADJ:ind': '不定形容词', 'ADJ:dem': '指示形容词', 'ADJ:pos': '物主形容词' } },
} as const;
type CoreText = (typeof coreMessages)[keyof typeof coreMessages];

export function digestLabel(entry: DigestEntry | undefined, missing: string) {
  if (!entry) return missing;
  const meaning = digestMeaning(entry) || missing;
  return entry.core ? `${entry.core.pos} · ${meaning}` : meaning;
}

function CoreFieldSource({ source, label, text }: {
  readonly source: CoreSense['field_sources']['definition_fr']; readonly label: string; readonly text: CoreText;
}) {
  return <section className="core-field-source"><h5>{label}</h5><dl>
    <dt>{text.source}</dt><dd>{source.title}</dd>
    <dt>{text.version}</dt><dd>{source.version}</dd>
    <dt>{text.attribution}</dt><dd>{source.attribution}</dd>
    <dt>{text.license}</dt><dd>{source.license}</dd>
    <dt>{text.sourceUrl}</dt><dd>{source.source_url}</dd>
    <dt>{text.licenseUrl}</dt><dd>{source.license_url}</dd>
    <dt>{text.span}</dt><dd>{source.span}</dd>
    <dt>{text.modification}</dt><dd>{source.modification}</dd>
  </dl></section>;
}

export function CoreSenseContent({ sense, text }: { readonly sense: CoreSense; readonly text: CoreText }) {
  const pos = `${text.pos[sense.pos]} (${sense.pos})`;
  return <section className="core-sense" aria-label={pos} data-sense-id={sense.sense_id} data-pos={sense.pos}>
    <h4>{pos}</h4><dl>
      <dt>{text.definitionFr}</dt><dd lang="fr">{sense.definition_fr}</dd>
      <dt>{text.ipa}</dt><dd lang="und">{sense.ipa}</dd>
      <dt>{text.meaningEn}</dt><dd lang="en">{sense.meaning_en}</dd>
      <dt>{text.meaningZh}</dt><dd lang="zh">{sense.meaning_zh}</dd>
      <dt>{text.usage}</dt><dd lang="en">{sense.usage}</dd>
      {sense.source_headword !== sense.headword && <><dt>{text.sourceSpelling}</dt><dd lang="fr">{sense.source_headword}</dd></>}
    </dl>
    <details className="core-sources"><summary>{text.sources}</summary>
      {([['definition_fr', text.definitionFr], ['ipa', text.ipa], ['meaning_en', text.meaningEn], ['meaning_zh', text.meaningZh]] as const)
        .map(([field, label]) => <CoreFieldSource key={field} source={sense.field_sources[field]} label={label} text={text} />)}
    </details>
  </section>;
}
