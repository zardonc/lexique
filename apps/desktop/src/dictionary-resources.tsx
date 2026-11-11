import { useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import { dictionaryFailure } from './platform/dictionary';
import resources from './dictionary-resources.json';

const text = {
  zh: { title: '额外词典与例句', open: '查看来源／下载页', error: '无法打开来源页。',
    hint: '自行选择并下载资源，再通过上方入口导入。仅 MDX 可直接导入，单文件最多 128 MiB；MDD 媒体暂不支持。获取和使用须符合来源授权。',
    groups: { 'fr-en': '法语—英语', 'fr-fr': '法语单语', 'fr-zh': '法语—中文', 'en-zh': '英语—中文', 'en-en': '英语单语', 'fr-sentences': '法语例句', 'fr-synonyms': '法语同近反义', 'en-sentences': '英语例句' } },
  en: { title: 'Additional dictionaries and examples', open: 'View source / download page', error: 'Could not open the source page.',
    hint: 'Choose and download resources, then import them above. Only MDX can be imported directly, up to 128 MiB per file; MDD media are unavailable. Follow the source’s terms of use.',
    groups: { 'fr-en': 'French–English', 'fr-fr': 'French monolingual', 'fr-zh': 'French–Chinese', 'en-zh': 'English–Chinese', 'en-en': 'English monolingual', 'fr-sentences': 'French examples', 'fr-synonyms': 'French synonyms and opposites', 'en-sentences': 'English examples' } },
  fr: { title: 'Dictionnaires et exemples supplémentaires', open: 'Voir la source / le téléchargement', error: 'Impossible d’ouvrir la page source.',
    hint: 'Choisissez et téléchargez les ressources, puis importez-les ci-dessus. Seul MDX est directement importable, jusqu’à 128 Mio par fichier ; médias MDD indisponibles. Respectez les conditions de la source.',
    groups: { 'fr-en': 'Français–anglais', 'fr-fr': 'Français monolingue', 'fr-zh': 'Français–chinois', 'en-zh': 'Anglais–chinois', 'en-en': 'Anglais monolingue', 'fr-sentences': 'Exemples français', 'fr-synonyms': 'Synonymes et contraires français', 'en-sentences': 'Exemples anglais' } },
} as const;

export function DictionaryResources({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const labels = text[locale], [error, setError] = useState('');
  async function open(id: string) {
    setError('');
    try { await runtime.openDictionaryResource(id); }
    catch (cause) { setError(`${labels.error} ${dictionaryFailure(cause).message}`); }
  }
  return <section className="dictionary-resources" aria-label={labels.title}>
    <h3>{labels.title}</h3><p>{labels.hint}</p>
    {error && <p role="alert">{error}</p>}
    {Object.entries(labels.groups).map(([category, title]) => <details key={category} open={category === 'fr-en'}>
      <summary>{title}</summary>
      <ul>{resources.filter((resource) => resource.category === category).map((resource) => <li key={resource.id}>
        <div><strong>{resource.title}</strong> <span className="resource-format">{resource.format}</span></div>
        <p>{resource.detail[locale]}</p>
        <a href={resource.url} target="_blank" rel="noopener noreferrer" aria-label={`${labels.open}: ${resource.title}`} onClick={(event) => { event.preventDefault(); void open(resource.id); }}>{labels.open} ↗</a>
      </li>)}</ul>
    </details>)}
  </section>;
}
