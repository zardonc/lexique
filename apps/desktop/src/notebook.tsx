import { useEffect, useMemo, useState } from 'react';
import { FrequencyBadge, FrequencyState, useFrequency } from './frequency';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage, sortSchema } from './platform/vocabulary';
import type { VocabularyRecord, VocabularySort } from './platform/vocabulary';
import type { NotebookText } from './notebook-content';
import type { BatchReview } from './use-batch-review';

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

function toggleSelection(current: readonly number[], id: number, checked: boolean) {
  return checked ? [...current, id] : current.filter((value) => value !== id);
}

function BatchControls({ records, selected, setSelected, batch, text }: {
  readonly records: readonly VocabularyRecord[]; readonly selected: readonly number[];
  readonly setSelected: (value: readonly number[]) => void; readonly batch: BatchReview; readonly text: NotebookText;
}) {
  if (!records.length) return null;
  return <><label><input type="checkbox" aria-label={text.selectAll}
    checked={records.every((record) => selected.includes(record.id))}
    onChange={(event) => setSelected(event.target.checked ? records.map((record) => record.id) : [])} />{text.selectAll}</label>
    <div className="actions">{([['add', text.batchAdd], ['pause', text.batchPause], ['resume', text.batchResume]] as const).map(([action, label]) =>
      <button key={action} disabled={batch.working || !!batch.pending || !selected.length} onClick={() => {
        const snapshot = records.filter((record) => selected.includes(record.id)); setSelected([]); void batch.run(snapshot, action);
      }}>{label}</button>)}</div></>;
}

function PageControls({ page, count, capacity, changePage, changeCapacity, text }: {
  readonly page: number; readonly count: number; readonly capacity: number; readonly text: NotebookText;
  readonly changePage: (value: number) => void; readonly changeCapacity: (value: number) => void;
}) {
  const pages = Math.max(1, Math.ceil(count / capacity));
  return <div className="actions"><label>{text.pageSize}<select value={capacity} onChange={(event) => changeCapacity(Number(event.target.value))}>
    {[20, 50, 100].map((value) => <option key={value} value={value}>{value}</option>)}</select></label>
    <button disabled={page === 0} onClick={() => changePage(page - 1)}>{text.previousPage}</button>
    <span>{page + 1}/{pages}</span><button disabled={page + 1 >= pages} onClick={() => changePage(page + 1)}>{text.nextPage}</button></div>;
}

function ReadingState({ list, text }: { readonly list: ReturnType<typeof useVocabularyList>; readonly text: NotebookText }) {
  if (list.error) return <p role="alert">{text.readError}: {list.error} <button onClick={list.retry}>{text.retry}</button></p>;
  if (list.records === undefined) return <p role="status">{text.loading}</p>;
  return list.records?.length === 0 && <p role="status">{text.empty}</p>;
}

function visiblePage(page: number, capacity: number, records: readonly VocabularyRecord[] | null | undefined) {
  const count = records?.length ?? 0;
  return Math.min(page, Math.max(0, Math.ceil(count / capacity) - 1));
}

export function NotebookPanel({ runtime, version, text, onOpen, batch }: {
  readonly runtime: RuntimeAdapter; readonly version: number; readonly text: NotebookText;
  readonly onOpen: (record: VocabularyRecord) => void; readonly batch: BatchReview;
}) {
  const list = useVocabularyList(runtime, version);
  const [selected, setSelected] = useState<readonly number[]>([]);
  const [page, setPage] = useState(0);
  const [capacity, setCapacity] = useState(20);
  useEffect(() => { setSelected([]); }, [list.records]);
  const currentPage = visiblePage(page, capacity, list.records);
  const records = useMemo(() => list.records?.slice(currentPage * capacity, (currentPage + 1) * capacity) ?? [], [list.records, currentPage, capacity]);
  const keys = useMemo(() => records.map((record) => record.headword), [records]);
  const frequencies = useFrequency(runtime, keys);
  function reset() { setSelected([]); setPage(0); }
  if (list.records === null) return <p role="status">{text.preview}</p>;
  return <section className="notebook" aria-label={text.search}>
    <div className="notebook-controls"><label>{text.search}<input type="search" value={list.query}
      onChange={(event) => { reset(); list.setQuery(event.target.value); }} /></label>
      <label>{text.sort}<select value={list.sort ?? 'updated-desc'} disabled={list.busy || !list.sort}
        onChange={(event) => { reset(); void list.changeSort(sortSchema.parse(event.target.value)); }}>
        {sortSchema.options.map((value, index) => <option key={value} value={value}>{text.sorts[index]}</option>)}
      </select></label></div>
    <ReadingState list={list} text={text} />
    {!!records.length && <FrequencyState state={frequencies} text={text.frequency} />}
    <PageControls page={currentPage} count={list.records?.length ?? 0} capacity={capacity} text={text}
      changePage={(value) => { setSelected([]); setPage(value); }} changeCapacity={(value) => { reset(); setCapacity(value); }} />
    <BatchControls records={records} selected={selected} setSelected={setSelected} batch={batch} text={text} />
    {!list.error && <ul className="vocabulary-list">{records.map((record) => <li key={record.id}>
      <label><input type="checkbox" aria-label={`${text.select} ${record.headword}`} checked={selected.includes(record.id)}
        onChange={(event) => { const checked = event.target.checked; setSelected((current) => toggleSelection(current, record.id, checked)); }} /></label>
      <button onClick={() => onOpen(record)}><span className="headword-line"><strong lang="fr">{record.headword}</strong>
        <FrequencyBadge entry={frequencies.lookup?.entries.find((entry) => entry.key === record.headword)} text={text.frequency} /></span>
        <span>{record.digest.map((entry) => entry.translation || text.missing).join(' · ')}</span>
        <span className="note-excerpt">{record.notes}</span></button>
    </li>)}</ul>}
  </section>;
}
