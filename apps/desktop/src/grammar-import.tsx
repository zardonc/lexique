import { useEffect, useRef, useState } from 'react';
import type { NotebookText } from './notebook-content';
import type { RuntimeAdapter } from './platform/runtime';
import type { GrammarImportPreview, GrammarImportResult } from './platform/grammar-import';
import type { ReviewRecord } from './platform/notes';
import { errorMessage } from './platform/vocabulary';

function useGrammarImport(runtime: RuntimeAdapter, onSaved: () => void, text: NotebookText) {
  const [preview, setPreview] = useState<GrammarImportPreview | null>(null), [result, setResult] = useState<GrammarImportResult | null>(null);
  const [busy, setBusy] = useState(false), [error, setError] = useState(''), [status, setStatus] = useState(''), [started, setStarted] = useState(false);
  const operation = useRef<string | null>(null), working = useRef(false);
  useEffect(() => () => { if (operation.current) void runtime.cancelNoteImport(operation.current).catch(console.error); }, [runtime]);
  async function work(action: () => Promise<void>) {
    if (working.current) return;
    working.current = true; setBusy(true); setError('');
    try { await action(); } catch (cause) { setError(errorMessage(cause)); }
    finally { working.current = false; setBusy(false); }
  }
  function select() { void work(async () => {
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    const id = crypto.randomUUID(); operation.current = id;
    setPreview(null); setResult(null); setStatus(''); setStarted(false);
    const selected = await runtime.previewGrammarImport(id); setPreview(selected);
    if (!selected) setStatus(text.interchange.cancelled);
  }); }
  function confirm() { void work(async () => {
    if (!preview?.group || preview.error || preview.group.error || !['create', 'identical'].includes(preview.group.status)) return;
    const saved = await runtime.commitGrammarImport(preview.operation, preview.group.status === 'create' ? 'create' : 'skip');
    setResult(saved); setPreview(null); setStatus(text.grammarImport.saved); onSaved();
  }); }
  function cancel() { void work(async () => {
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    operation.current = null; setPreview(null); setStatus(text.interchange.cancelled);
  }); }
  function start() { void work(async () => {
    const record = result?.records[0];
    if (!record?.card) return;
    const paused = await runtime.reviewPause(record.id, false);
    if (paused !== false) throw new Error('Could not start reviewing the saved grammar card');
    setStarted(true); onSaved();
  }); }
  return { preview, result, busy, error, status, started, select, confirm, cancel, start };
}

type ImportState = ReturnType<typeof useGrammarImport>;

function Incoming({ group, text }: { readonly group: NonNullable<GrammarImportPreview['group']>; readonly text: NotebookText }) {
  const labels = text.grammarImport, messages = text.interchange;
  return <><h3>{group.content.title}</h3><p>{messages.rows}: {group.row}</p>
    <p>{labels.identity}: {group.content.exchange_id || labels.assigned}</p><p>{group.status === 'different' ? labels.different : messages[group.status]}</p>
    <pre>{group.content.body}</pre><pre>{group.content.question}</pre><pre>{group.content.answer}</pre>
    {group.error && <p role="alert">{group.error}</p>}
    {group.existing && <><p>{messages.current}</p><h4>{group.existing.title}</h4><pre>{group.existing.body}</pre>
      <pre>{group.existing.card?.question}</pre><pre>{group.existing.card?.answer}</pre></>}
  </>;
}

function Preview({ state, text }: { readonly state: ImportState; readonly text: NotebookText }) {
  const preview = state.preview;
  if (!preview) return null;
  const group = preview.group, messages = text.interchange;
  const valid = !preview.error && group && !group.error && ['create', 'identical'].includes(group.status);
  return <><p>{messages.source}: {preview.source.filename}</p>
    {preview.error && <p role="alert">{preview.error}</p>}
    {group && <Incoming group={group} text={text} />}
    <button disabled={state.busy || !valid} onClick={state.confirm}>{messages.confirm}</button>
    <button disabled={state.busy} onClick={state.cancel}>{messages.cancel}</button>
  </>;
}

function Saved({ state, text, onOpen }: { readonly state: ImportState; readonly text: NotebookText; readonly onOpen: (record: ReviewRecord) => void }) {
  const { result, busy, started } = state, labels = text.grammarImport, messages = text.interchange;
  return <>{state.status && <p role="status">{state.status}{result && <> {messages.createdCount} {result.created} · {messages.skippedCount} {result.skipped}</>}</p>}
    {result?.records.map((record) => <div key={record.id}><button disabled={busy} onClick={() => onOpen(record)}>{messages.openRecord}</button>
      {record.card && <><p>{started ? labels.started : labels.paused}</p>
        {!started && <button disabled={busy} onClick={state.start}>{labels.start}</button>}</>}
    </div>)}</>;
}

export function GrammarImport({ runtime, text, onSaved, onOpen, onPrepare }: {
  readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onSaved: () => void;
  readonly onOpen: (record: ReviewRecord) => void; readonly onPrepare: (action: () => void) => void;
}) {
  const state = useGrammarImport(runtime, onSaved, text), labels = text.grammarImport;
  return <section aria-label={labels.open}>
    <button disabled={state.busy} onClick={() => onPrepare(state.select)}>{labels.open}</button><p>{labels.columns}</p>
    {state.busy && <p>{text.interchange.busy}</p>}
    {state.error && <p role="alert">{state.error}</p>}
    <Preview state={state} text={text} /><Saved state={state} text={text} onOpen={onOpen} />
  </section>;
}
