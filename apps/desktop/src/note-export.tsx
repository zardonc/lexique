import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import type { NoteExportScope, NoteExportSnapshot } from './platform/note-export';
import type { NoteExportText } from './note-export-content';
import { errorMessage } from './platform/vocabulary';
import type { VocabularyRecord, VocabularySort } from './platform/vocabulary';
import type { CategoryFilter } from './platform/notes';

function ExportScope({ selection, scope, busy, text, change }: {
  readonly selection: NoteExportScope; readonly scope: NoteExportScope; readonly busy: boolean; readonly text: NoteExportText;
  readonly change: (value: NoteExportScope) => void;
}) {
  return <label>{text.scope}<select value={selection.kind} disabled={busy || scope.kind === 'single'} onChange={(event) => {
    if (scope.kind !== 'single') change(event.target.value === 'all' ? { kind: 'all', sort: scope.sort } : scope);
  }}>{scope.kind === 'single' ? <option value="single">{text.single}</option> : <><option value="search">{text.search}</option><option value="all">{text.all}</option></>}</select></label>;
}

function ExportCounts({ snapshot, text }: { readonly snapshot: NoteExportSnapshot; readonly text: NoteExportText }) {
  return <p role="status">{snapshot.records} {text.records} · {snapshot.rows} {text.rows}{snapshot.records === 0 && <> — {text.empty}</>}</p>;
}

function MarkdownSave({ single, disabled, text, save, available }: {
  readonly single: boolean; readonly disabled: boolean; readonly text: NoteExportText; readonly save: (format: 'markdown' | 'zip') => void;
  readonly available: boolean;
}) {
  if (!available) return null;
  return <button disabled={disabled} onClick={() => save(single ? 'markdown' : 'zip')}>{single ? text.saveMarkdown : text.saveZip}</button>;
}

function markdownAvailable(recordKind: 'grammar' | undefined, scope: NoteExportScope) {
  return !recordKind || scope.kind === 'single';
}

function ExportDialog({ runtime, scope, text, close, recordKind }: {
  readonly runtime: RuntimeAdapter; readonly scope: NoteExportScope; readonly text: NoteExportText; readonly close: () => void;
  readonly recordKind?: 'grammar';
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [selection, setSelection] = useState<NoteExportScope>(scope);
  const [snapshot, setSnapshot] = useState<NoteExportSnapshot>();
  const [error, setError] = useState(''), [result, setResult] = useState(''), [busy, setBusy] = useState(false), [retry, setRetry] = useState(0);
  useEffect(() => { dialog.current?.showModal(); }, []);
  useEffect(() => {
    let cancelled = false;
    setSnapshot(undefined); setError(''); setResult('');
    void runtime.noteExportSnapshot(selection, recordKind).then((value) => { if (!cancelled) setSnapshot(value); })
      .catch((cause: unknown) => { if (!cancelled) setError(errorMessage(cause)); });
    return () => { cancelled = true; };
  }, [runtime, selection, retry, recordKind]);
  async function save(format: 'csv' | 'markdown' | 'zip' = 'csv') {
    if (!snapshot || busy) return;
    setBusy(true); setError(''); setResult('');
    try {
      const value = await (format === 'zip' ? runtime.exportNoteZip(snapshot.token) : format === 'markdown' ? runtime.exportNoteMarkdown(snapshot.token) : runtime.exportNoteCsv(snapshot.token));
      setResult(value.state === 'saved' ? text.saved : text.cancelled);
    }
    catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return <dialog ref={dialog} aria-label={text.heading} onCancel={(event) => { event.preventDefault(); if (!busy) close(); }}>
    <h3>{text.heading}</h3><p>{text.detail}</p>
    <ExportScope selection={selection} scope={scope} busy={busy} text={text} change={setSelection} />
    {snapshot ? <ExportCounts snapshot={snapshot} text={text} />
      : !error && <p role="status">{text.loading}</p>}
    {error && <p role="alert">{error} <button disabled={busy} onClick={() => setRetry((value) => value + 1)}>{text.retry}</button></p>}
    {result && <p role="status">{result}</p>}
    <div className="actions"><button disabled={busy || !snapshot?.records} onClick={() => { void save(); }}>{busy ? text.busy : text.save}</button>
      <MarkdownSave available={markdownAvailable(recordKind, scope)} single={scope.kind === 'single'} disabled={busy || !snapshot?.records} text={text} save={(format) => { void save(format); }} />
      <button disabled={busy} onClick={close}>{text.close}</button></div>
  </dialog>;
}

export function NoteExport({ runtime, scope, text, disabled = false, recordKind }: {
  readonly runtime: RuntimeAdapter; readonly scope: NoteExportScope; readonly text: NoteExportText; readonly disabled?: boolean;
  readonly recordKind?: 'grammar';
}) {
  const [open, setOpen] = useState(false);
  return <><button disabled={disabled} onClick={() => setOpen(true)}>{text.open}</button>
    {!recordKind && scope.kind === 'single' && <button disabled={disabled} onClick={() => setOpen(true)}>{text.openMarkdown}</button>}
    {open && <ExportDialog runtime={runtime} scope={scope} text={text} recordKind={recordKind} close={() => setOpen(false)} />}</>;
}

export function NotebookExport({ runtime, kind, sort, query, category, disabled, loaded, text, grammarText }: {
  readonly runtime: RuntimeAdapter; readonly kind: string; readonly sort?: VocabularySort; readonly query: string;
  readonly category: CategoryFilter; readonly disabled: boolean; readonly loaded: boolean; readonly text: NoteExportText;
  readonly grammarText: NoteExportText;
}) {
  return sort && <NoteExport key={kind} runtime={runtime} text={kind === 'grammar' ? grammarText : text} recordKind={kind === 'grammar' ? 'grammar' : undefined}
    scope={{ kind: 'search', query, sort, category }} disabled={disabled || !loaded} />;
}

export function SavedNoteExport({ runtime, record, disabled, text }: {
  readonly runtime: RuntimeAdapter; readonly record: VocabularyRecord | null; readonly disabled: boolean; readonly text: NoteExportText;
}) {
  return record && <NoteExport key={record.id} runtime={runtime} text={text} scope={{ kind: 'single', id: record.id }} disabled={disabled} />;
}
