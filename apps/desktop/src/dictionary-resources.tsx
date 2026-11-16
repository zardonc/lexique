import { useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import { dictionaryFailure } from './platform/dictionary';
import resources from './dictionary-resources.json';

const text = {
  zh: { title: '额外词典与例句', open: '查看来源／下载页', notices: '查看随包来源／许可', error: '无法打开来源资源。',
    hint: '核心词典的来源与许可已随包提供。其他资源可自行选择并下载，再通过上方入口导入。仅 MDX 可直接导入，单文件最多 128 MiB；MDD 媒体暂不支持。获取和使用须符合来源授权。',
    groups: { bundled: '随包核心词典与来源', 'fr-en': '法语—英语', 'fr-fr': '法语单语', 'fr-zh': '法语—中文', 'en-zh': '英语—中文', 'en-en': '英语单语', 'fr-sentences': '法语例句', 'fr-synonyms': '法语同近反义', 'en-sentences': '英语例句' } },
  en: { title: 'Additional dictionaries and examples', open: 'View source / download page', notices: 'Open bundled sources / licenses', error: 'Could not open the source resource.',
    hint: 'The core sources and licenses are already bundled. Choose and download other resources, then import them above. Only MDX can be imported directly, up to 128 MiB per file; MDD media are unavailable. Follow the source’s terms of use.',
    groups: { bundled: 'Bundled core dictionary and sources', 'fr-en': 'French–English', 'fr-fr': 'French monolingual', 'fr-zh': 'French–Chinese', 'en-zh': 'English–Chinese', 'en-en': 'English monolingual', 'fr-sentences': 'French examples', 'fr-synonyms': 'French synonyms and opposites', 'en-sentences': 'English examples' } },
  fr: { title: 'Dictionnaires et exemples supplémentaires', open: 'Voir la source / le téléchargement', notices: 'Ouvrir les sources / licences intégrées', error: 'Impossible d’ouvrir la ressource source.',
    hint: 'Les sources et licences du dictionnaire de base sont intégrées. Choisissez et téléchargez les autres ressources, puis importez-les ci-dessus. Seul MDX est directement importable, jusqu’à 128 Mio par fichier ; médias MDD indisponibles. Respectez les conditions de la source.',
    groups: { bundled: 'Dictionnaire de base et sources intégrées', 'fr-en': 'Français–anglais', 'fr-fr': 'Français monolingue', 'fr-zh': 'Français–chinois', 'en-zh': 'Anglais–chinois', 'en-en': 'Anglais monolingue', 'fr-sentences': 'Exemples français', 'fr-synonyms': 'Synonymes et contraires français', 'en-sentences': 'Exemples anglais' } },
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
    {Object.entries(labels.groups).map(([category, title]) => <details key={category} open={category === 'bundled' || category === 'fr-en'}>
      <summary>{title}</summary>
      <ul>{resources.filter((resource) => resource.category === category).map((resource) => <li key={resource.id}>
        <div><strong>{resource.title}</strong> <span className="resource-format">{resource.format}</span></div>
        <p>{resource.detail[locale]}</p>
        <a href={resource.url} target="_blank" rel="noopener noreferrer" aria-label={`${resource.category === 'bundled' ? labels.notices : labels.open}: ${resource.title}`} onClick={(event) => { event.preventDefault(); void open(resource.id); }}>{resource.category === 'bundled' ? labels.notices : labels.open} ↗</a>
      </li>)}</ul>
    </details>)}
  </section>;
}
