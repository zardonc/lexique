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

function useBatchReview({ runtime, text, onWorkingChange, onReviewChanged }: {
  readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onWorkingChange: (working: boolean) => void; readonly onReviewChanged: () => void;
}) {
  const [selected, setSelected] = useState<readonly number[]>([]);
  const [results, setResults] = useState<readonly { readonly record: VocabularyRecord; readonly action: 'add' | 'pause' | 'resume'; readonly error: string; readonly added: boolean }[]>([]);
  const [working, setWorking] = useState(false);
  async function perform(record: VocabularyRecord, action: 'add' | 'pause' | 'resume') {
    let error = '';
    let added = false;
    try {
      if (action === 'add') {
        const cards = await runtime.addReview(record.id);
        if (!cards) throw new Error(text.preview);
        added = true;
        const cached = await runtime.cachedSpeech(record.id);
        if (!cached && !await runtime.generateSpeech(record.id)) throw new Error(text.listeningPending);
      } else if (await runtime.reviewPause(record.id, action === 'pause') === null) throw new Error(text.preview);
    } catch (cause) { error = errorMessage(cause); }
    setResults((current) => [...current.filter((item) => item.record.id !== record.id), { record, action, error, added }]);
  }
  async function run(records: readonly VocabularyRecord[], action: 'add' | 'pause' | 'resume') {
    setWorking(true); onWorkingChange(true);
    try { for (const record of records) await perform(record, action); }
    finally { setWorking(false); onWorkingChange(false); onReviewChanged(); }
  }
  return { selected, setSelected, results, setResults, working, run };
}

function BatchResults({ results, working, run, text }: {
  readonly results: ReturnType<typeof useBatchReview>['results']; readonly working: boolean;
  readonly run: ReturnType<typeof useBatchReview>['run']; readonly text: NotebookText;
}) {
  return results.length > 0 && <ul>{results.map(({ record, action, error, added }) => <li key={record.id}>
    <span>{record.headword}：{error ? added ? text.audioFailed : text.failed : text.success}</span>{error && <>: {error} <button disabled={working} onClick={() => void run([record], action)}>{text.retry} {record.headword}</button></>}
  </li>)}</ul>;
}

function toggleSelection(current: readonly number[], id: number, checked: boolean) {
  return checked ? [...current, id] : current.filter((value) => value !== id);
}

function BatchControls({ records, batch, text }: { readonly records: readonly VocabularyRecord[] | null | undefined; readonly batch: ReturnType<typeof useBatchReview>; readonly text: NotebookText }) {
  if (!records?.length) return null;
  const { selected, setSelected, setResults, working, run } = batch;
  return <><label><input type="checkbox" disabled={working} aria-label={text.selectAll}
    checked={records.every((record) => selected.includes(record.id))}
    onChange={(event) => setSelected(event.target.checked ? records.map((record) => record.id) : [])} />{text.selectAll}</label>
    <div className="actions">{([['add', text.batchAdd], ['pause', text.batchPause], ['resume', text.batchResume]] as const).map(([action, label]) =>
      <button key={action} disabled={working || !selected.length} onClick={() => { setResults([]); void run(records.filter((record) => selected.includes(record.id)), action); }}>{label}</button>)}</div></>;
}

export function NotebookPanel({ runtime, version, text, onOpen, onWorkingChange, onReviewChanged }: {
  readonly runtime: RuntimeAdapter; readonly version: number; readonly text: NotebookText;
  readonly onOpen: (record: VocabularyRecord) => void;
  readonly onWorkingChange: (working: boolean) => void;
  readonly onReviewChanged: () => void;
}) {
  const list = useVocabularyList(runtime, version);
  const batch = useBatchReview({ runtime, text, onWorkingChange, onReviewChanged });
  const { selected, setSelected, results, setResults, working, run } = batch;
  if (list.records === null) return <p role="status">{text.preview}</p>;
  return <section className="notebook" aria-label={text.search}>
    <div className="notebook-controls"><label>{text.search}<input type="search" value={list.query} disabled={working}
      onChange={(event) => { setSelected([]); setResults([]); list.setQuery(event.target.value); }} /></label>
      <label>{text.sort}<select value={list.sort ?? 'updated-desc'} disabled={working || list.busy || !list.sort}
        onChange={(event) => { void list.changeSort(sortSchema.parse(event.target.value)); }}>
        {sortSchema.options.map((value, index) => <option key={value} value={value}>{text.sorts[index]}</option>)}
      </select></label></div>
    {list.error ? <p role="alert">{text.readError}: {list.error} <button onClick={list.retry}>{text.retry}</button></p>
      : list.records === undefined ? <p role="status">{text.loading}</p>
        : list.records.length === 0 && <p role="status">{text.empty}</p>}
    <BatchControls records={list.records} batch={batch} text={text} />
    <BatchResults results={results} working={working} run={run} text={text} />
    {!list.error && <ul className="vocabulary-list">{list.records?.map((record) => <li key={record.id}>
      <label><input type="checkbox" disabled={working} aria-label={`${text.select} ${record.headword}`} checked={selected.includes(record.id)}
        onChange={(event) => { const checked = event.target.checked; setSelected((current) => toggleSelection(current, record.id, checked)); }} /></label>
      <button disabled={working} onClick={() => onOpen(record)}><strong lang="fr">{record.headword}</strong>
        <span>{record.digest.map((entry) => entry.translation || text.missing).join(' · ')}</span>
        <span className="note-excerpt">{record.notes}</span></button>
    </li>)}</ul>}
  </section>;
}
