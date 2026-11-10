import { useEffect, useState } from 'react';
import type { Locale } from './content';
import { messages } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { DictionaryCatalog } from './platform/dictionary';
import { dictionaryFailure } from './platform/dictionary';

const text = {
  zh: { title: '查询词典', add: '添加 MDX 词典', earlier: '向前移动', later: '向后移动', remove: '移出词典列表', saved: '词典设置已保存。',
    hint: '按列表顺序查询，其他词典正文点击标签后加载。MDX 副本随便携数据目录保存；最多 16 个词典，每个文件最多 128 MiB。移出列表会保留词典文件和学习记录。' },
  en: { title: 'Query dictionaries', add: 'Add an MDX dictionary', earlier: 'Move earlier', later: 'Move later', remove: 'Remove from dictionary list', saved: 'Dictionary settings saved.',
    hint: 'Query in list order; select other tabs to load their bodies. MDX copies stay in the portable data directory. Maximum: 16 dictionaries, 128 MiB per file. Removing a dictionary keeps its file and learning records.' },
  fr: { title: 'Dictionnaires consultés', add: 'Ajouter un dictionnaire MDX', earlier: 'Déplacer avant', later: 'Déplacer après', remove: 'Retirer de la liste', saved: 'Réglages des dictionnaires enregistrés.',
    hint: 'Consultez les dictionnaires dans cet ordre ; les autres contenus se chargent en cliquant sur les onglets. Les copies MDX restent dans les données portables. Maximum : 16 dictionnaires, 128 Mio par fichier. Le retrait conserve le fichier et les données d’apprentissage.' },
} as const;

export function DictionarySettings({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const labels = text[locale], dictionary = messages[locale].dictionary;
  const [catalog, setCatalog] = useState<DictionaryCatalog | null>();
  const [error, setError] = useState(''), [busy, setBusy] = useState(false), [saved, setSaved] = useState(false);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setCatalog(undefined); setError(''); setSaved(false);
    void runtime.dictionarySettings().then((value) => { if (!cancelled) setCatalog(value); }).catch((cause: unknown) => {
      if (!cancelled) { const failure = dictionaryFailure(cause); setError(`${dictionary[failure.code]} ${failure.message}`); }
    });
    return () => { cancelled = true; };
  }, [runtime, retry, dictionary]);
  async function change(operation: () => Promise<DictionaryCatalog | null>) {
    setBusy(true); setError(''); setSaved(false);
    try { const value = await operation(); setCatalog(value); setSaved(value !== null); }
    catch (cause) { const failure = dictionaryFailure(cause); setError(`${dictionary[failure.code]} ${failure.message}`); }
    finally { setBusy(false); }
  }
  function move(index: number, offset: number) {
    if (!catalog) return;
    const ids = catalog.map((item) => item.id);
    [ids[index], ids[index + offset]] = [ids[index + offset], ids[index]];
    void change(() => runtime.dictionarySettings(ids));
  }
  return <section className="dictionary-settings" aria-label={labels.title}><h3>{labels.title}</h3><p>{labels.hint}</p>
    {catalog === undefined && !error && <p role="status">{dictionary.loading}</p>}
    {catalog === null && <p role="status">{dictionary.preview}</p>}
    {catalog && <><label htmlFor="dictionary-import">{labels.add}</label><input id="dictionary-import" type="file" accept=".mdx" disabled={busy || catalog.length >= 16}
      onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ''; if (file) void change(() => runtime.importDictionary(file)); }} />
      <ol>{catalog.map((item, index) => <li key={item.id}><span>{item.title}</span><div className="actions">
        <button disabled={busy || index === 0} aria-label={`${labels.earlier}: ${item.title}`} onClick={() => move(index, -1)}>↑</button>
        <button disabled={busy || index === catalog.length - 1} aria-label={`${labels.later}: ${item.title}`} onClick={() => move(index, 1)}>↓</button>
        {item.id !== 'builtin' && <button disabled={busy} onClick={() => { void change(() => runtime.removeDictionary(item.id)); }}>{labels.remove}</button>}
      </div></li>)}</ol></>}
    {saved && <p role="status">{labels.saved}</p>}
    {error && <p role="alert">{error} <button disabled={busy} onClick={() => setRetry((value) => value + 1)}>{dictionary.retry}</button></p>}
  </section>;
}
