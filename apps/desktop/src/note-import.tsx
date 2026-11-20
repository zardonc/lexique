import { useEffect, useRef, useState } from 'react';
import type { NotebookText } from './notebook-content';
import type { RuntimeAdapter } from './platform/runtime';
import type { NoteImportPreview } from './platform/note-import';
import { errorMessage } from './platform/vocabulary';
import type { VocabularyRecord } from './platform/vocabulary';

function useNoteImport(runtime: RuntimeAdapter, messages: NotebookText['interchange'], onSaved: () => void) {
  const [preview, setPreview] = useState<NoteImportPreview | null>(null);
  const [result, setResult] = useState<VocabularyRecord | null>(null);
  const [error, setError] = useState(''), [status, setStatus] = useState(''), [busy, setBusy] = useState(false);
  const working = useRef(false), operation = useRef<string | null>(null);
  useEffect(() => () => { if (operation.current) void runtime.cancelNoteImport(operation.current).catch(console.error); }, [runtime]);
  async function select() {
    if (working.current) return;
    working.current = true; setBusy(true); setPreview(null); setResult(null); setError(''); setStatus('');
    const id = crypto.randomUUID(); operation.current = id;
    try {
      const selected = await runtime.previewNoteImport(id);
      if (operation.current !== id) return;
      setPreview(selected);
      if (!selected) setStatus(messages.cancelled);
    } catch (cause) { setError(errorMessage(cause)); }
    finally { working.current = false; setBusy(false); }
  }
  async function confirm() {
    if (!preview || working.current) return;
    working.current = true; setBusy(true); setError('');
    try {
      const saved = await runtime.commitNoteImport(preview.operation);
      setResult(saved.record); setPreview(null);
      setStatus(saved.outcome === 'existing' ? messages.skipped : messages.imported); onSaved();
    } catch (cause) { setError(errorMessage(cause)); }
    finally { working.current = false; setBusy(false); }
  }
  async function cancel() {
    if (busy || !operation.current) return;
    try {
      await runtime.cancelNoteImport(operation.current); operation.current = null;
      setPreview(null); setError(''); setStatus(messages.cancelled);
    } catch (cause) { setError(errorMessage(cause)); }
  }
  return { preview, result, error, status, busy, select, confirm, cancel };
}

function ImportPreview({ state, messages }: { readonly state: ReturnType<typeof useNoteImport>; readonly messages: NotebookText['interchange'] }) {
  const { preview, busy, confirm, cancel } = state;
  if (!preview) return null;
  return <section aria-label={messages.title}>
    <h3>{messages.title}</h3><p>{messages.source}: {preview.source.filename}</p>
    <p>{messages.rows}: {preview.rows.join(', ')}</p><p>{messages[preview.status]}</p>
    {preview.error && <p role="alert">{preview.error}</p>}
    {preview.content && <><h4 lang="fr">{preview.content.headword}</h4><p>{messages.incoming}</p><pre>{preview.content.notes}</pre>
      {preview.content.examples?.map((example, index) => <div key={index}><pre lang="fr">{example.fr}</pre><pre lang="zh">{example.zh}</pre></div>)}</>}
    {preview.existing && <><p>{messages.current}</p><pre>{preview.existing.notes}</pre><pre>{preview.existing.example_fr}{'\n'}{preview.existing.example_zh}</pre>
      {preview.existing.more_examples.map((example, index) => <pre key={index}>{example.fr}{'\n'}{example.zh}</pre>)}</>}
    <button disabled={busy || !['create', 'identical'].includes(preview.status)} onClick={() => { void confirm(); }}>{messages.confirm}</button>
    <button disabled={busy} onClick={() => { void cancel(); }}>{messages.cancel}</button>
  </section>;
}

export function NoteImport({ runtime, text, onSaved, onOpen, onPrepare }: {
  readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onSaved: () => void;
  readonly onOpen: (record: VocabularyRecord) => void; readonly onPrepare: (action: () => void) => void;
}) {
  const messages = text.interchange;
  const state = useNoteImport(runtime, messages, onSaved);
  const { busy, error, status, result } = state;
  return <section aria-label={messages.open}>
    <button disabled={busy} onClick={() => onPrepare(() => { void state.select(); })}>{messages.open}</button>
    <p>{messages.columns}</p>
    {busy && <p role="status">{messages.busy}</p>}
    <ImportPreview state={state} messages={messages} />
    {error && <p role="alert">{messages.error}: {error}</p>}
    {status && <p role="status">{status}</p>}
    {result && <button onClick={() => onOpen(result)}>{messages.openRecord}</button>}
  </section>;
}
