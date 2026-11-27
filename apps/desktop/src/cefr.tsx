import { useEffect, useMemo, useState } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import type { CefrLookup, CefrQuery, CefrSettings } from './platform/cefr';
import { CEFR_QUERY_LIMIT } from './platform/cefr';
import type { DigestEntry } from './platform/vocabulary';
import { dictionaryFailure } from './platform/dictionary';

const messages = {
  en: { title: 'Optional CEFR grading (FLELex TT-Beacco)', download: 'Open official download page', import: 'Import .tsv', replace: 'Replace .tsv', remove: 'Remove CEFR resource', missing: 'No CEFR resource imported', installed: 'Imported rows', retry: 'Retry',
    terms: 'Download FleLex_TT_Beacco.tsv from UCLouvain. CC BY-NC-SA 4.0: personal/non-commercial use. Imported grades stay on this device and are excluded from notes and backups.', preview: 'Import requires the desktop app.', error: 'Could not load CEFR grades' },
  fr: { title: 'Niveaux CECR facultatifs (FLELex TT-Beacco)', download: 'Ouvrir la page officielle', import: 'Importer un .tsv', replace: 'Remplacer le .tsv', remove: 'Retirer la ressource CECR', missing: 'Aucune ressource CECR importée', installed: 'Lignes importées', retry: 'Réessayer',
    terms: 'Téléchargez FleLex_TT_Beacco.tsv auprès de l’UCLouvain. CC BY-NC-SA 4.0 : usage personnel/non commercial. Les niveaux restent sur cet appareil et sont exclus des notes et sauvegardes.', preview: 'L’importation nécessite l’application de bureau.', error: 'Impossible de lire les niveaux CECR' },
  zh: { title: '可选 CEFR 分级（FLELex TT-Beacco）', download: '打开官方下载页', import: '导入 .tsv', replace: '替换 .tsv', remove: '移除 CEFR 资源', missing: '尚未导入 CEFR 资源', installed: '已导入行数', retry: '重试',
    terms: '请从 UCLouvain 下载 FleLex_TT_Beacco.tsv。CC BY-NC-SA 4.0：个人／非商业使用。分级仅保存在本机，不进入笔记或学习备份。', preview: '导入需使用桌面应用。', error: '无法读取 CEFR 分级' },
} as const;

export function digestCefrQueries(headword: string, digest: readonly DigestEntry[]): readonly CefrQuery[] {
  const core = digest.flatMap((entry) => entry.core ? [{ key: headword, sense: {
    sense_id: entry.core.sense_id, pos: entry.core.pos, source_headword: entry.core.source_headword,
  } }] : []);
  return core.length ? core : [{ key: headword, sense: null }];
}

export function useCefr(runtime: RuntimeAdapter | undefined, queries: readonly CefrQuery[]) {
  const [lookup, setLookup] = useState<CefrLookup | null>(null);
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setLookup(null); setError('');
    if (runtime && queries.length) {
      const batches = [];
      for (let index = 0; index < queries.length; index += CEFR_QUERY_LIMIT) {
        batches.push(runtime.lookupCefr(queries.slice(index, index + CEFR_QUERY_LIMIT)));
      }
      void Promise.all(batches).then((values) => {
        if (!cancelled) setLookup(values.every((value) => value !== null)
          ? { imported: values.every((value) => value.imported), entries: values.flatMap((value) => value.entries) } : null);
      }).catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause).message); });
    }
    return () => { cancelled = true; };
  }, [runtime, queries, attempt]);
  return { lookup, error, retry: () => setAttempt((value) => value + 1) };
}

export function CefrBadge({ level }: { readonly level?: CefrLookup['entries'][number]['level'] }) {
  return level && <span className="cefr-badge" title="FLELex TT-Beacco · CEFR">{level}</span>;
}

export function cefrRecordLevel(headword: string, lookup: CefrLookup | null) {
  return lookup?.entries.filter((entry) => entry.key === headword).flatMap((entry) => entry.level ? [entry.level] : []).sort()[0];
}

export function CefrRecordBadge({ headword, digest, runtime, text }: {
  readonly headword: string; readonly digest: readonly DigestEntry[]; readonly runtime?: RuntimeAdapter; readonly text: { readonly cefrError: string };
}) {
  const queries = useMemo(() => digestCefrQueries(headword, digest), [headword, digest]);
  const { lookup, error } = useCefr(runtime, queries);
  return error ? <span role="alert">{text.cefrError}: {error}</span> : <CefrBadge level={cefrRecordLevel(headword, lookup)} />;
}

export function CefrState({ error, retry, text }: { readonly error: string; readonly retry: () => void; readonly text: { readonly cefrError: string; readonly retry: string } }) {
  return error && <p role="alert">{text.cefrError}: {error} <button onClick={retry}>{text.retry}</button></p>;
}

export function CefrSettingsPanel({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = messages[locale];
  const [settings, setSettings] = useState<CefrSettings | null>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let cancelled = false;
    void runtime.cefrSettings().then((value) => { if (!cancelled) { setSettings(value); setError(''); } })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause).message); });
    return () => { cancelled = true; };
  }, [runtime, attempt]);
  async function change(file?: File) {
    setBusy(true); setError('');
    try { setSettings(await (file ? runtime.importCefr(file) : runtime.removeCefr())); }
    catch (cause) { setError(dictionaryFailure(cause).message); }
    finally { setBusy(false); }
  }
  return <section className="cefr-settings" aria-label={text.title}><h3>{text.title}</h3><p>{text.terms}</p>
    <button disabled={busy} onClick={() => void runtime.openDictionaryResource('flelex-tt-beacco').catch((cause: unknown) => setError(dictionaryFailure(cause).message))}>{text.download}</button>
    <p role="status">{settings === null ? text.preview : settings?.row_count ? `${text.installed}: ${settings.row_count.toLocaleString(locale)}` : text.missing}</p>
    {settings !== null && <label>{settings?.row_count ? text.replace : text.import}<input type="file" accept=".tsv" disabled={busy} onChange={(event) => {
      const file = event.target.files?.[0]; event.target.value = ''; if (file) void change(file);
    }} /></label>}
    {settings?.row_count && <button disabled={busy} onClick={() => void change()}>{text.remove}</button>}
    <CefrState error={error} retry={() => setAttempt((value) => value + 1)} text={{ cefrError: text.error, retry: text.retry }} />
  </section>;
}
