import { useEffect, useRef, useState } from 'react';
import type { NotebookText } from './notebook-content';
import { NoteImportDictionary } from './note-import-dictionary';
import type { RuntimeAdapter } from './platform/runtime';
import type { NoteImportMapping, NoteImportPreview, NoteImportResult, NoteImportSelection } from './platform/note-import';
import { errorMessage } from './platform/vocabulary';
import type { VocabularyRecord } from './platform/vocabulary';
import type { BatchReview } from './use-batch-review';
import template from '../../../docs/examples/lexique-note-v1.md?raw';

function useNoteImport(runtime: RuntimeAdapter, messages: NotebookText['interchange'], onSaved: () => void) {
  const [preview, setPreview] = useState<NoteImportPreview | null>(null), [result, setResult] = useState<NoteImportResult | null>(null);
  const [error, setError] = useState(''), [status, setStatus] = useState(''), [busy, setBusy] = useState(false), [dirty, setDirty] = useState(false);
  const [delimiter, setDelimiter] = useState<'comma' | 'semicolon'>('comma'), [format, setFormat] = useState<'csv' | 'markdown' | 'zip'>('csv');
  const [mapping, setMapping] = useState<NoteImportMapping>({ headword: null, notes: null, example_fr: null, example_zh: null });
  const [decisions, setDecisions] = useState<readonly (NoteImportSelection['action'] | '')[]>([]);
  const [allowUnlinked, setAllowUnlinked] = useState<readonly boolean[]>([]);
  const needsConsent = !!preview?.groups.some((group, index) => decisions[index] === 'create' && group.enrichment?.status === 'error' && !allowUnlinked[index]);
  const working = useRef(false), operation = useRef<string | null>(null);
  useEffect(() => () => { if (operation.current) void runtime.cancelNoteImport(operation.current).catch(console.error); }, [runtime]);
  function accept(selected: NoteImportPreview | null) {
    setPreview(selected); setDirty(false); setAllowUnlinked(selected?.groups.map(() => false) ?? []);
    if (selected) { setMapping(selected.mapping); setDecisions(selected.groups.map((group) => group.status === 'create' ? 'create' : group.status === 'identical' ? 'skip' : '')); }
    else setStatus(messages.cancelled);
  }
  async function work(action: () => Promise<void>) {
    if (working.current) return;
    working.current = true; setBusy(true); setError(''); setStatus('');
    try { await action(); } catch (cause) { setError(errorMessage(cause)); }
    finally { working.current = false; setBusy(false); }
  }
  function select(format: 'csv' | 'markdown' | 'zip' = 'csv') { void work(async () => {
    setFormat(format);
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    setPreview(null); setResult(null);
    const id = crypto.randomUUID(); operation.current = id;
    accept(await runtime.previewNoteImport(id, { delimiter, mapping: null, ...(format !== 'csv' ? { format } : {}) }));
  }); }
  function remap() { void work(async () => {
    if (operation.current) accept(await runtime.remapNoteImport(operation.current, { delimiter, mapping, ...(format !== 'csv' ? { format } : {}) }));
  }); }
  function confirm() { void work(async () => {
    if (!preview || dirty || needsConsent || decisions.some((action) => !action)) return;
    const selections = decisions.map((action, index) => ({ index, action: action as NoteImportSelection['action'], allow_unlinked: allowUnlinked[index] ?? false }));
    const saved = await runtime.commitNoteImport(preview.operation, selections);
    setResult(saved); setPreview(null); setStatus(messages.imported); onSaved();
  }); }
  function cancel() { void work(async () => {
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    operation.current = null; setPreview(null); setError(''); setStatus(messages.cancelled);
  }); }
  function exportTemplate() { void work(async () => {
    const exported = await runtime.exportNoteMarkdown(null); setStatus(exported.state === 'saved' ? messages.markdown.exported : messages.markdown.cancelled);
  }); }
  return { preview, result, error, status, busy, dirty, delimiter, format, mapping, decisions, allowUnlinked, needsConsent, select, remap, confirm, cancel, exportTemplate,
    continueUnlinked: (index: number, value: boolean) => setAllowUnlinked((old) => old.map((allowed, at) => at === index ? value : allowed)),
    map: (field: keyof NoteImportMapping, value: number | null) => { setMapping((old) => ({ ...old, [field]: value })); setDirty(true); },
    separate: (value: 'comma' | 'semicolon') => { setDelimiter(value); if (preview) setDirty(true); },
    decide: (index: number, action: NoteImportSelection['action'] | '') => setDecisions((old) => old.map((value, at) => at === index ? action : value)) };
}
type ImportState = ReturnType<typeof useNoteImport>;
type Messages = NotebookText['interchange'];

function Mapping({ state, messages }: { readonly state: ImportState; readonly messages: Messages }) {
  if (state.format !== 'csv') return state.preview && <button disabled={state.busy} onClick={state.remap}>{messages.remap}</button>;
  return <fieldset disabled={state.busy}>
    <label>{messages.delimiter}<select aria-label={messages.delimiter} value={state.delimiter} onChange={(event) => state.separate(event.target.value as 'comma' | 'semicolon')}>
      <option value="comma">{messages.comma}</option><option value="semicolon">{messages.semicolon}</option>
    </select></label>
    {state.preview && <>{(['headword', 'notes', 'example_fr', 'example_zh'] as const).map((field) => <label key={field}>{field}
      <select aria-label={field} value={state.mapping[field] ?? ''} onChange={(event) => state.map(field, event.target.value === '' ? null : Number(event.target.value))}>
        <option value="">{messages.unmapped}</option>{state.preview!.columns.map((column, index) => <option key={index} value={index}>{index + 1}: {column}</option>)}
      </select></label>)}<button onClick={state.remap}>{messages.remap}</button></>}
  </fieldset>;
}

function Group({ state, index, messages }: { readonly state: ImportState; readonly index: number; readonly messages: Messages }) {
  const group = state.preview!.groups[index], action = state.decisions[index];
  return <section aria-label={group.content.headword}>
    <h4 lang="fr">{group.content.headword}</h4><p>{messages.rows}: {group.rows.join(', ')}</p><p>{messages[group.status]}</p>
    {group.error && <p role="alert">{group.error}</p>}
    <GroupDictionary state={state} index={index} messages={messages} />
    <p>{messages.incoming}</p><pre>{group.content.notes ?? messages.preserved}</pre>
    {group.content.examples === null ? <p>{messages.preserved}</p> : <><p>{messages.replacement}</p>
      {group.content.examples.map((example, at) => <div key={at}><pre lang="fr">{example.fr}</pre><pre lang="zh">{example.zh}</pre></div>)}</>}
    {group.existing && <><p>{messages.current}</p><pre>{group.existing.notes}</pre><pre>{group.existing.example_fr}{'\n'}{group.existing.example_zh}</pre>
      {group.existing.more_examples.map((example, at) => <pre key={at}>{example.fr}{'\n'}{example.zh}</pre>)}</>}
    {group.status === 'invalid' ? <label><input type="checkbox" disabled={state.busy} checked={action === 'skip'}
      aria-label={`${messages.exclude}：${group.content.headword}`} onChange={(event) => state.decide(index, event.target.checked ? 'skip' : '')} />{messages.exclude}</label>
      : group.status !== 'identical' && <label>{messages.decision}<select aria-label={`${messages.decision}：${group.content.headword}`} disabled={state.busy}
        value={action ?? ''} onChange={(event) => state.decide(index, event.target.value as NoteImportSelection['action'] | '')}>
        <option value="">{messages.decide}</option><option value={group.status === 'create' ? 'create' : 'update'}>{group.status === 'create' ? messages.create : messages.update}</option>
        <option value="skip">{messages.skip}</option>
      </select></label>}
  </section>;
}

function GroupDictionary({ state, index, messages }: { readonly state: ImportState; readonly index: number; readonly messages: Messages }) {
  const group = state.preview!.groups[index];
  return <>{group.files && <p>{messages.source}: {group.files.join(', ')}</p>}
    {group.enrichment && <NoteImportDictionary enrichment={group.enrichment} existing={group.existing} messages={messages.dictionary}
      allowUnlinked={state.allowUnlinked[index] ?? false} onContinue={(value) => state.continueUnlinked(index, value)} disabled={state.busy || state.decisions[index] !== 'create'} />}</>;
}

function ImportPreview({ state, messages }: { readonly state: ImportState; readonly messages: Messages }) {
  const preview = state.preview;
  if (!preview) return null;
  const selected = state.decisions.filter((action) => action === 'create' || action === 'update').length;
  const title = state.format === 'csv' ? messages.title : messages[state.format].title;
  return <section aria-label={title}>
    <h3>{title}</h3><p>{messages.source}: {preview.source.filename}</p>
    {preview.error && <p role="alert">{preview.error}</p>}
    {preview.groups.map((_group, index) => <Group key={index} state={state} index={index} messages={messages} />)}
    <p>{messages.selected}: {selected}</p>
    <button disabled={state.busy || state.dirty || state.needsConsent || !!preview.error || state.decisions.some((action) => !action)} onClick={state.confirm}>{messages.confirm}</button>
    <button disabled={state.busy} onClick={state.cancel}>{messages.cancel}</button>
  </section>;
}

function ImportFeedback({ state, messages }: { readonly state: ImportState; readonly messages: Messages }) {
  const labels = state.format === 'csv' ? messages : messages[state.format];
  return <>{state.busy && <p role="status">{labels.busy}</p>}
    {state.error && <p role="alert">{labels.error}: {state.error}</p>}</>;
}

export function NoteImport({ runtime, text, onSaved, onOpen, onPrepare, batch }: {
  readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onSaved: () => void;
  readonly onOpen: (record: VocabularyRecord) => void; readonly onPrepare: (action: () => void) => void;
  readonly batch: BatchReview;
}) {
  const messages = text.interchange, state = useNoteImport(runtime, messages, onSaved);
  const { busy, status, result } = state;
  return <section aria-label={messages.open}>
    <button disabled={busy} onClick={() => onPrepare(() => state.select())}>{messages.open}</button>
    <button disabled={busy} onClick={() => onPrepare(() => state.select('markdown'))}>{messages.markdown.open}</button>
    <button disabled={busy} onClick={() => onPrepare(() => state.select('zip'))}>{messages.zip.open}</button><p>{messages.columns}</p>
    <details><summary>{messages.markdown.help}</summary><p>{messages.markdown.rules}</p><pre>{template}</pre>
      <button disabled={busy} onClick={state.exportTemplate}>{messages.markdown.example}</button></details>
    <Mapping state={state} messages={messages} />
    <ImportFeedback state={state} messages={messages} />
    <ImportPreview state={state} messages={messages} />
    {status && <p role="status">{status}{result && <> {messages.createdCount} {result.created} · {messages.updatedCount} {result.updated} · {messages.skippedCount} {result.skipped}</>}</p>}
    {result?.records.map((record) => <button key={record.id} onClick={() => onOpen(record)}>{messages.openRecord}{result.records.length > 1 && `：${record.headword}`}</button>)}
    {!!result?.records.length && <button disabled={batch.working || !!batch.pending} onClick={() => onPrepare(() => { void batch.run(result.records, 'add'); })}>{messages.addBatch}</button>}
  </section>;
}
