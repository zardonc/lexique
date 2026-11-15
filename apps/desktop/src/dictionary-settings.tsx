import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import { messages } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { DictionaryCatalog } from './platform/dictionary';
import { dictionaryFailure, MAX_DICTIONARIES } from './platform/dictionary';

const text = {
  zh: { title: '查询词典', add: '添加 MDX 词典', earlier: '向前移动', later: '向后移动', remove: '移出词典列表', saved: '词典设置已保存。',
    missing: '待补齐 MDX', choose: '选择 MDX 文件：',
    confirmTitle: '确认词典内容', different: '所选 MDX 的内容与原词典不同。', unknown: '原词典的内容身份未知，无法确认所选 MDX 是同一版本。',
    selected: '所选文件：', confirm: '确认使用此 MDX', cancel: '取消',
    replacement: '确认后将把所选文件关联到原词典，保持词典身份、顺序、摘要、笔记及例句出处。旧文件仍会保留。',
    bundled: '随包', supplemental: 'FreeDict 法英 0.4.1 随包提供，原内容日期为 2018-08-11，适合作为补充词典。',
    hint: '按列表顺序查询，其他词典正文点击标签后加载。最多 17 个词典，包含两项内置和 15 份自选 MDX；每个 MDX 最多 128 MiB。副本随便携数据目录保存，移出列表会保留文件和学习记录。' },
  en: { title: 'Query dictionaries', add: 'Add an MDX dictionary', earlier: 'Move earlier', later: 'Move later', remove: 'Remove from dictionary list', saved: 'Dictionary settings saved.',
    missing: 'MDX file needed', choose: 'Choose an MDX file: ',
    confirmTitle: 'Confirm dictionary content', different: 'The selected MDX has different content from the original dictionary.', unknown: 'The original dictionary content is unknown, so this MDX cannot be verified as the same version.',
    selected: 'Selected file: ', confirm: 'Use this MDX', cancel: 'Cancel',
    replacement: 'Confirmation associates the selected file with the original dictionary and keeps its identity, order, summaries, notes and example sources. Previous files are retained.',
    bundled: 'Bundled', supplemental: 'FreeDict French–English 0.4.1 is bundled. Its content dates from 2018-08-11; use it as a supplemental dictionary.',
    hint: 'Query in list order; select other tabs to load their bodies. Maximum: 17 dictionaries, including two bundled dictionaries and 15 optional MDX files, up to 128 MiB each. Copies stay in the portable data directory; removal keeps files and learning records.' },
  fr: { title: 'Dictionnaires consultés', add: 'Ajouter un dictionnaire MDX', earlier: 'Déplacer avant', later: 'Déplacer après', remove: 'Retirer de la liste', saved: 'Réglages des dictionnaires enregistrés.',
    missing: 'Fichier MDX à fournir', choose: 'Choisir un fichier MDX : ',
    confirmTitle: 'Confirmer le contenu du dictionnaire', different: 'Le contenu du MDX choisi diffère du dictionnaire original.', unknown: 'Le contenu original est inconnu ; ce MDX ne peut pas être vérifié comme étant la même version.',
    selected: 'Fichier choisi : ', confirm: 'Utiliser ce MDX', cancel: 'Annuler',
    replacement: 'La confirmation associe ce fichier au dictionnaire original et conserve son identité, son ordre, les résumés, les notes et les sources des exemples. Les anciens fichiers sont conservés.',
    bundled: 'Inclus', supplemental: 'FreeDict français–anglais 0.4.1 est inclus. Son contenu date du 11 août 2018 ; utilisez-le comme dictionnaire complémentaire.',
    hint: 'Consultez les dictionnaires dans cet ordre ; les autres contenus se chargent en cliquant sur les onglets. Maximum : 17 dictionnaires, dont deux intégrés et 15 fichiers MDX facultatifs de 128 Mio chacun. Les copies restent dans les données portables ; le retrait conserve les fichiers et les données d’apprentissage.' },
} as const;

interface ListProps {
  readonly catalog: DictionaryCatalog; readonly labels: typeof text[Locale]; readonly busy: boolean;
  readonly onMove: (index: number, offset: number) => void; readonly onRemove: (id: string) => void;
  readonly onRelink: (id: string, file: File) => void;
}

function DictionaryList({ catalog, labels, busy, onMove, onRemove, onRelink }: ListProps) {
  return <ol>{catalog.map((item, index) => <li key={item.id}><span>{item.title}
    {item.id.startsWith('builtin') && <small>{labels.bundled}</small>}
    {item.availability === 'missing' && <small>{labels.missing}</small>}</span><div className="actions">
    <button disabled={busy || index === 0} aria-label={`${labels.earlier}: ${item.title}`} onClick={() => onMove(index, -1)}>↑</button>
    <button disabled={busy || index === catalog.length - 1} aria-label={`${labels.later}: ${item.title}`} onClick={() => onMove(index, 1)}>↓</button>
    {item.id.startsWith('mdx-') && <><label htmlFor={`dictionary-relink-${item.id}`}>{labels.choose}{item.title}</label>
      <input id={`dictionary-relink-${item.id}`} type="file" accept=".mdx" disabled={busy} onChange={(event) => {
        const file = event.target.files?.[0]; event.target.value = ''; if (file) onRelink(item.id, file);
      }} /><button disabled={busy} onClick={() => onRemove(item.id)}>{labels.remove}</button></>}
  </div></li>)}</ol>;
}

interface PendingRelink { readonly id: string; readonly file: File; readonly expectedHash: string | null; readonly contentHash: string }
interface DialogProps {
  readonly pending: PendingRelink | null; readonly labels: typeof text[Locale]; readonly busy: boolean; readonly error: string;
  readonly onConfirm: (pending: PendingRelink) => void; readonly onCancel: () => void;
}

function RelinkDialog({ pending, labels, busy, error, onConfirm, onCancel }: DialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { if (pending) dialog.current?.showModal(); else dialog.current?.close(); }, [pending]);
  return <dialog ref={dialog} aria-labelledby="dictionary-relink-heading" onCancel={(event) => { event.preventDefault(); if (!busy) onCancel(); }}>
    <h3 id="dictionary-relink-heading">{labels.confirmTitle}</h3>
    {pending && <><p>{pending.expectedHash === null ? labels.unknown : labels.different}</p>
      <p>{labels.selected}<strong>{pending.file.name}</strong></p><p>{labels.replacement}</p>
      {error && <p role="alert">{error}</p>}
      <div className="actions"><button disabled={busy} onClick={() => onConfirm(pending)}>{labels.confirm}</button>
        <button disabled={busy} onClick={onCancel}>{labels.cancel}</button></div></>}
  </dialog>;
}

function movedOrder(catalog: DictionaryCatalog, index: number, offset: number) {
  const ids = catalog.map((item) => item.id);
  [ids[index], ids[index + offset]] = [ids[index + offset], ids[index]];
  return ids;
}

export function DictionarySettings({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const labels = text[locale], dictionary = messages[locale].dictionary;
  const [catalog, setCatalog] = useState<DictionaryCatalog | null>();
  const [error, setError] = useState(''), [busy, setBusy] = useState(false), [saved, setSaved] = useState(false);
  const [retry, setRetry] = useState(0);
  const [pending, setPending] = useState<PendingRelink | null>(null);
  const disabled = busy || pending !== null;
  useEffect(() => {
    let cancelled = false;
    setCatalog(undefined); setError(''); setSaved(false);
    void runtime.dictionarySettings().then((value) => { if (!cancelled) setCatalog(value); }).catch((cause: unknown) => {
      if (!cancelled) { const failure = dictionaryFailure(cause); setError(`${dictionary[failure.code]} ${failure.message}`); }
    });
    return () => { cancelled = true; };
  }, [runtime, retry, dictionary]);
  async function change(operation: () => Promise<DictionaryCatalog | null | undefined>) {
    setBusy(true); setError(''); setSaved(false);
    try { const value = await operation(); if (value !== undefined) { setCatalog(value); setSaved(value !== null); } }
    catch (cause) { const failure = dictionaryFailure(cause); setError(`${dictionary[failure.code]} ${failure.message}`); }
    finally { setBusy(false); }
  }
  function relink(id: string, file: File, confirmedHash?: string) {
    void change(async () => {
      const result = await runtime.relinkDictionary(id, file, confirmedHash);
      if (result?.status === 'confirmation_required') {
        setPending({ id, file, expectedHash: result.expected_hash, contentHash: result.content_hash }); return;
      }
      setPending(null);
      return result?.catalog ?? null;
    });
  }
  return <section className="dictionary-settings" aria-label={labels.title}><h3>{labels.title}</h3><p>{labels.hint}</p>
    <p>{labels.supplemental}</p>
    {catalog === undefined && !error && <p role="status">{dictionary.loading}</p>}
    {catalog === null && <p role="status">{dictionary.preview}</p>}
    {catalog && <><label htmlFor="dictionary-import">{labels.add}</label><input id="dictionary-import" type="file" accept=".mdx" disabled={disabled || catalog.length >= MAX_DICTIONARIES}
      onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ''; if (file) void change(() => runtime.importDictionary(file)); }} />
      <DictionaryList catalog={catalog} labels={labels} busy={disabled} onMove={(index, offset) => { void change(() => runtime.dictionarySettings(movedOrder(catalog, index, offset))); }}
        onRemove={(id) => { void change(() => runtime.removeDictionary(id)); }} onRelink={relink} /></>}
    {saved && <p role="status">{labels.saved}</p>}
    {error && <p role="alert">{error} <button disabled={disabled} onClick={() => setRetry((value) => value + 1)}>{dictionary.retry}</button></p>}
    <RelinkDialog pending={pending} labels={labels} busy={busy} error={error} onConfirm={(value) => relink(value.id, value.file, value.contentHash)} onCancel={() => setPending(null)} />
  </section>;
}
