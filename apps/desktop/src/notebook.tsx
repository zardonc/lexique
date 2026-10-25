import { useEffect, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage, sortSchema } from './platform/vocabulary';
import type { VocabularyRecord, VocabularySort } from './platform/vocabulary';
import type { NotebookText } from './notebook-content';

function useVocabularyList(runtime: RuntimeAdapter, version: number) {
  const [query, setQuery] = useState('');
  const [sort, setSort] = useState<VocabularySort>();
  const [records, setRecords] = useState<readonly VocabularyRecord[] | null>();
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setError(''); setRecords(undefined);
    const read = async () => {
      if (!sort) {
        const preference = await runtime.vocabularySort();
        if (cancelled) return;
        if (preference === null) setRecords(null); else setSort(preference);
        return;
      }
      const rows = await runtime.listVocabulary(query, sort);
      if (!cancelled) setRecords(rows);
    };
    void read().catch((cause: unknown) => { if (!cancelled) setError(errorMessage(cause)); });
    return () => { cancelled = true; };
  }, [runtime, version, query, sort, retry]);
  async function changeSort(value: VocabularySort) {
    setBusy(true); setError('');
    try {
      const saved = await runtime.vocabularySort(value);
      if (saved === null) throw new Error('Native storage unavailable');
      setSort(saved);
    } catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return { query, setQuery, sort, records, error, busy, changeSort, retry: () => setRetry((value) => value + 1) };
}

export function NotebookPanel({ runtime, version, text, onOpen }: {
  readonly runtime: RuntimeAdapter; readonly version: number; readonly text: NotebookText;
  readonly onOpen: (record: VocabularyRecord) => void;
}) {
  const list = useVocabularyList(runtime, version);
  if (list.records === null) return <p role="status">{text.preview}</p>;
  return <section className="notebook" aria-label={text.search}>
    <div className="notebook-controls"><label>{text.search}<input type="search" value={list.query}
      onChange={(event) => list.setQuery(event.target.value)} /></label>
      <label>{text.sort}<select value={list.sort ?? 'updated-desc'} disabled={list.busy || !list.sort}
        onChange={(event) => { void list.changeSort(sortSchema.parse(event.target.value)); }}>
        {sortSchema.options.map((value, index) => <option key={value} value={value}>{text.sorts[index]}</option>)}
      </select></label></div>
    {list.error ? <p role="alert">{text.readError}: {list.error} <button onClick={list.retry}>{text.retry}</button></p>
      : list.records === undefined ? <p role="status">{text.loading}</p>
        : list.records.length === 0 && <p role="status">{text.empty}</p>}
    {!list.error && <ul className="vocabulary-list">{list.records?.map((record) => <li key={record.id}>
      <button onClick={() => onOpen(record)}><strong lang="fr">{record.headword}</strong>
        <span>{record.digest.map((entry) => entry.translation || text.missing).join(' · ')}</span>
        <span className="note-excerpt">{record.notes}</span></button>
    </li>)}</ul>}
  </section>;
}
