import { Fragment, useEffect, useMemo, useState } from 'react';
import { NoteImport } from './note-import';
import { FrequencyBadge, FrequencyState, useFrequency } from './frequency';
import type { RuntimeAdapter } from './platform/runtime';
import { digestMeaning, errorMessage, sortSchema } from './platform/vocabulary';
import type { VocabularyRecord, VocabularySort } from './platform/vocabulary';
import type { NotebookText } from './notebook-content';
import type { BatchReview } from './use-batch-review';
import { matchesCategory } from './platform/notes';
import type { CategoryFilter, ReviewRecord } from './platform/notes';
import { CategoryBulkControls, CategoryManager } from './learning-categories';
import type { LearningCategories, NotebookMutation } from './learning-categories';
import { NotebookExport } from './note-export';

type NoteKind = 'vocabulary' | 'grammar';
function useNotesList(runtime: RuntimeAdapter, version: number, kind: NoteKind) {
  const [query, setQuery] = useState('');
  const [sorts, setSorts] = useState<Partial<Record<NoteKind, VocabularySort>>>({});
  const sort = sorts[kind];
  const [records, setRecords] = useState<readonly ReviewRecord[] | null>();
  const [recordKind, setRecordKind] = useState(kind);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setError(''); setRecords(undefined); setRecordKind(kind);
    const read = async () => {
      if (!sort) {
        const preference = await (kind === 'grammar' ? runtime.grammarSort() : runtime.vocabularySort());
        if (cancelled) return;
        if (preference === null) setRecords(null); else setSorts((current) => ({ ...current, [kind]: preference }));
        return;
      }
      const rows = await (kind === 'grammar' ? runtime.listGrammar(query, sort) : runtime.listVocabulary(query, sort));
      if (!cancelled) setRecords(rows);
    };
    void read().catch((cause: unknown) => { if (!cancelled) setError(errorMessage(cause)); });
    return () => { cancelled = true; };
  }, [runtime, version, query, sort, retry, kind]);
  async function changeSort(value: VocabularySort) {
    setBusy(true); setError('');
    try {
      const saved = await (kind === 'grammar' ? runtime.grammarSort(value) : runtime.vocabularySort(value));
      if (saved === null) throw new Error('Native storage unavailable');
      setSorts((current) => ({ ...current, [kind]: saved }));
    } catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return { query, setQuery, sort, records: recordKind === kind ? records : undefined, error, busy, changeSort, retry: () => setRetry((value) => value + 1) };
}

function toggleSelection(current: readonly number[], id: number, checked: boolean) {
  return checked ? [...current, id] : current.filter((value) => value !== id);
}

function BatchControls({ records, selected, setSelected, batch, text }: {
  readonly records: readonly VocabularyRecord[]; readonly selected: readonly number[];
  readonly setSelected: (value: readonly number[]) => void; readonly batch: BatchReview; readonly text: NotebookText;
}) {
  if (!records.length) return null;
  return <div className="actions">{([['add', text.batchAdd], ['pause', text.batchPause], ['resume', text.batchResume]] as const).map(([action, label]) =>
      <button key={action} disabled={batch.working || !!batch.pending || !selected.length} onClick={() => {
        const snapshot = records.filter((record) => selected.includes(record.id)); setSelected([]); void batch.run(snapshot, action);
      }}>{label}</button>)}</div>;
}

function SelectAll({ records, selected, setSelected, text }: {
  readonly records: readonly ReviewRecord[]; readonly selected: readonly number[];
  readonly setSelected: (value: readonly number[]) => void; readonly text: NotebookText;
}) {
  if (!records.length) return null;
  return <label><input type="checkbox" aria-label={text.selectAll}
    checked={records.every((record) => selected.includes(record.id))}
    onChange={(event) => {
      const ids = records.map((record) => record.id);
      setSelected(event.target.checked ? [...new Set([...selected, ...ids])] : selected.filter((id) => !ids.includes(id)));
    }} />{text.selectAll}</label>;
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

function ReadingState({ list, text, kind }: { readonly list: Pick<ReturnType<typeof useNotesList>, 'records' | 'error' | 'retry'>; readonly text: NotebookText; readonly kind: NoteKind }) {
  if (list.error) return <p role="alert">{text.readError}: {list.error} <button onClick={list.retry}>{text.retry}</button></p>;
  if (list.records === undefined) return <p role="status">{text.loading}</p>;
  return list.records?.length === 0 && <p role="status">{kind === 'grammar' ? text.notesText.empty : text.empty}</p>;
}

function visiblePage(page: number, capacity: number, records: readonly ReviewRecord[] | null | undefined) {
  const count = records?.length ?? 0;
  return Math.min(page, Math.max(0, Math.ceil(count / capacity) - 1));
}

function recordTitle(record: ReviewRecord) { return 'title' in record ? record.title : record.headword; }

function RecordContent({ record, frequency, text }: {
  readonly record: ReviewRecord; readonly frequency: ReturnType<typeof useFrequency>; readonly text: NotebookText;
}) {
  if ('title' in record) return <><strong>{record.title}</strong><span className="note-excerpt">{record.body}</span></>;
  return <><span className="headword-line"><strong lang="fr">{record.headword}</strong>
    <FrequencyBadge entry={frequency.lookup?.entries.find((entry) => entry.key === record.headword)} text={text.frequency} /></span>
    <span>{record.digest.map((entry, index) => {
      const meaning = digestMeaning(entry) || text.missing;
      return <Fragment key={index}>{index > 0 && ' · '}{entry.core ? <span lang="zh">{meaning}</span> : meaning}</Fragment>;
    })}</span>{record.import_source && <span>{text.interchange.external}: {record.import_source.filename}</span>}
    {!record.reference && <span>{text.interchange.missing}</span>}<span className="note-excerpt">{record.notes}</span></>;
}

function NotesTabs({ kind, onChange, text }: { readonly kind: NoteKind; readonly onChange: (value: NoteKind) => void; readonly text: NotebookText }) {
  return <div role="tablist" aria-label={text.notesText.tabs}>{(['vocabulary', 'grammar'] as const).map((value) => <button key={value} id={`notes-tab-${value}`} type="button"
    role="tab" aria-selected={kind === value} aria-controls="notes-panel" tabIndex={kind === value ? 0 : -1} onClick={() => onChange(value)} onKeyDown={(event) => {
      if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
      event.preventDefault(); onChange(event.key === 'Home' ? 'vocabulary' : event.key === 'End' ? 'grammar' : kind === 'grammar' ? 'vocabulary' : 'grammar');
    }}>{text.notesText[value]}</button>)}</div>;
}

function NotebookFilters({ list, kind, categories, filter, onFilter, reset, text }: {
  readonly list: ReturnType<typeof useNotesList>; readonly kind: NoteKind; readonly categories: LearningCategories; readonly filter: CategoryFilter;
  readonly onFilter: (value: CategoryFilter) => void; readonly reset: () => void; readonly text: NotebookText;
}) {
  return <div className="notebook-controls"><label>{kind === 'grammar' ? text.notesText.search : text.search}<input type="search" value={list.query}
    onChange={(event) => { reset(); list.setQuery(event.target.value); }} /></label>
    <label>{text.sort}<select value={list.sort ?? 'updated-desc'} disabled={list.busy || !list.sort}
      onChange={(event) => { void list.changeSort(sortSchema.parse(event.target.value)); }}>
      {sortSchema.options.map((value, index) => <option key={value} value={value}>{(kind === 'grammar' ? text.notesText.sorts : text.sorts)[index]}</option>)}
    </select></label><label>{text.notesText.filter}<select value={filter} disabled={!categories.categories} onChange={(event) => {
      const value = event.target.value; onFilter(value === 'all' || value === 'uncategorized' ? value : Number(value));
    }}><option value="all">{text.notesText.all}</option><option value="uncategorized">{text.notesText.uncategorized}</option>
      {categories.categories?.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}</select></label></div>;
}

function useActiveCategory(filter: CategoryFilter, categories: LearningCategories['categories'], reset: () => void) {
  useEffect(() => {
    if (typeof filter === 'number' && categories && !categories.some((category) => category.id === filter)) reset();
  }, [categories, filter]);
}

function NotesActions({ kind, runtime, text, onOpen, onSaved, onPrepare, onCreateGrammar, batch }: {
  readonly kind: NoteKind; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onOpen: (record: ReviewRecord) => void;
  readonly onSaved: () => void; readonly onPrepare: (action: () => void) => void; readonly onCreateGrammar: () => void;
  readonly batch: BatchReview;
}) {
  return kind === 'grammar' ? <button onClick={onCreateGrammar}>{text.notesText.create}</button>
    : <NoteImport runtime={runtime} text={text} onSaved={onSaved} onOpen={onOpen} onPrepare={onPrepare} batch={batch} />;
}

export function NotebookPanel({ runtime, version, text, onOpen, batch, categories, onMutation, onScopeChange, onCreateGrammar }: {
  readonly runtime: RuntimeAdapter; readonly version: number; readonly text: NotebookText;
  readonly onOpen: (record: ReviewRecord) => void; readonly batch: BatchReview; readonly categories: LearningCategories;
  readonly onMutation: NotebookMutation; readonly onScopeChange: (action: () => void) => void; readonly onCreateGrammar: () => void;
}) {
  const [kind, setKind] = useState<NoteKind>('vocabulary'), [filter, setFilter] = useState<CategoryFilter>('all');
  const list = useNotesList(runtime, version, kind);
  const [selected, setSelected] = useState<readonly number[]>([]);
  const [page, setPage] = useState(0);
  const [capacity, setCapacity] = useState(20);
  const filtered = useMemo(() => list.records?.filter((record) => matchesCategory(record, filter)), [list.records, filter]);
  useEffect(() => {
    if (!filtered) return;
    const ids = new Set(filtered.map((record) => record.id));
    setSelected((current) => current.filter((id) => ids.has(id)));
  }, [filtered]);
  const currentPage = visiblePage(page, capacity, filtered);
  const records = useMemo(() => filtered?.slice(currentPage * capacity, (currentPage + 1) * capacity) ?? [], [filtered, currentPage, capacity]);
  const words = (filtered ?? []).filter((record): record is VocabularyRecord => 'headword' in record);
  const keys = useMemo(() => records.flatMap((record) => 'headword' in record ? [record.headword] : []), [records]);
  const frequencies = useFrequency(runtime, keys);
  function reset() { setSelected([]); setPage(0); }
  useActiveCategory(filter, categories.categories, () => { setFilter('all'); reset(); });
  if (list.records === null) return <p role="status">{text.preview}</p>;
  return <section className="notebook" aria-label={text.search}>
    <NotesTabs kind={kind} text={text} onChange={(value) => { if (value !== kind) onScopeChange(() => {
      setKind(value); reset(); list.setQuery(''); document.getElementById(`notes-tab-${value}`)?.focus();
    }); }} />
    <div id="notes-panel" role="tabpanel" aria-labelledby={`notes-tab-${kind}`}>
    <NotesActions kind={kind} runtime={runtime} text={text} onSaved={list.retry} onOpen={onOpen} onPrepare={onScopeChange} onCreateGrammar={onCreateGrammar} batch={batch} />
    <CategoryManager state={categories} runtime={runtime} onMutation={onMutation} text={text.notesText} />
    <NotebookFilters list={list} kind={kind} categories={categories} filter={filter} onFilter={(value) => { reset(); setFilter(value); }} reset={reset} text={text} />
    <NotebookExport runtime={runtime} text={text.exportText} grammarText={text.grammarExportText} kind={kind} sort={list.sort} query={list.query}
      category={filter} disabled={list.busy} loaded={!!list.records} />
    <ReadingState list={{ ...list, records: filtered }} text={text} kind={kind} />
    {!!words.length && <FrequencyState state={frequencies} text={text.frequency} />}
    <PageControls page={currentPage} count={filtered?.length ?? 0} capacity={capacity} text={text}
      changePage={(value) => { setSelected([]); setPage(value); }} changeCapacity={(value) => { reset(); setCapacity(value); }} />
    <SelectAll records={records} selected={selected} setSelected={setSelected} text={text} />
    {kind === 'vocabulary' && <BatchControls records={words} selected={selected} setSelected={setSelected} batch={batch} text={text} />}
    <CategoryBulkControls state={categories} records={(filtered ?? []).map((record) => ({ id: record.id, title: recordTitle(record) }))} selected={selected}
      kind={kind} runtime={runtime} onMutation={onMutation} text={text} />
    {!list.error && <ul className="vocabulary-list">{records.map((record) => <li key={record.id}>
      <label><input type="checkbox" aria-label={`${text.select} ${recordTitle(record)}`} checked={selected.includes(record.id)}
        onChange={(event) => { const checked = event.target.checked; setSelected((current) => toggleSelection(current, record.id, checked)); }} /></label>
      <button onClick={() => onOpen(record)}><RecordContent record={record} frequency={frequencies} text={text} /></button>
    </li>)}</ul>}</div>
  </section>;
}
