import { useEffect, useRef, useState } from 'react';
import type { NotebookText } from './notebook-content';
import type { RuntimeAdapter } from './platform/runtime';
import type { GrammarImportMapping, GrammarImportPreview, GrammarImportResult } from './platform/grammar-import';
import type { NoteImportSelection } from './platform/note-import';
import type { ReviewRecord } from './platform/notes';
import { errorMessage } from './platform/vocabulary';

type Decision = NoteImportSelection['action'] | '';
function useGrammarCardStart(runtime: RuntimeAdapter, onSaved: () => void) {
  const [started, setStarted] = useState<readonly number[]>([]), [startErrors, setStartErrors] = useState<Readonly<Record<number, string>>>({});
  async function start(id: number) {
    try {
      const paused = await runtime.reviewPause(id, false);
      if (paused !== false) throw new Error('Could not start reviewing the saved grammar card');
      setStarted((old) => [...old, id]); setStartErrors((old) => ({ ...old, [id]: '' })); onSaved();
    } catch (cause) { setStartErrors((old) => ({ ...old, [id]: errorMessage(cause) })); }
  }
  return { started, startErrors, start, reset: () => { setStarted([]); setStartErrors({}); } };
}
function useGrammarImport(runtime: RuntimeAdapter, onSaved: () => void, text: NotebookText) {
  const [preview, setPreview] = useState<GrammarImportPreview | null>(null), [result, setResult] = useState<GrammarImportResult | null>(null);
  const [busy, setBusy] = useState(false), [error, setError] = useState(''), [status, setStatus] = useState('');
  const [delimiter, setDelimiter] = useState<'comma' | 'semicolon'>('comma'), [dirty, setDirty] = useState(false);
  const [format, setFormat] = useState<'csv' | 'markdown' | 'zip'>('csv');
  const [mapping, setMapping] = useState<GrammarImportMapping>({ exchange_id: null, title: null, body: null, question: null, answer: null });
  const [decisions, setDecisions] = useState<readonly Decision[]>([]), [submitted, setSubmitted] = useState(false);
  const cards = useGrammarCardStart(runtime, onSaved);
  const operation = useRef<string | null>(null), working = useRef(false);
  useEffect(() => () => { if (operation.current) void runtime.cancelNoteImport(operation.current).catch(console.error); }, [runtime]);
  async function work(action: () => Promise<void>) {
    if (working.current) return;
    working.current = true; setBusy(true); setError('');
    try { await action(); } catch (cause) { setError(errorMessage(cause)); }
    finally { working.current = false; setBusy(false); }
  }
  function accept(selected: GrammarImportPreview | null) {
    setPreview(selected); setDirty(false); setSubmitted(false);
    if (selected) { setMapping(selected.mapping); setDecisions(selected.groups.map((group) => group.status === 'identical' ? 'skip' : group.status === 'create' ? 'create' : '')); }
    else setStatus(text.interchange.cancelled);
  }
  function select(selectedFormat: 'csv' | 'markdown' | 'zip' = 'csv') { void work(async () => {
    setFormat(selectedFormat);
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    const id = crypto.randomUUID(); operation.current = id;
    setPreview(null); setResult(null); setStatus(''); cards.reset();
    accept(await runtime.previewGrammarImport(id, { delimiter, mapping: null, ...(selectedFormat !== 'csv' ? { format: selectedFormat } : {}) }));
  }); }
  function remap() { void work(async () => {
    if (operation.current) accept(await runtime.remapGrammarImport(operation.current, { delimiter, mapping }));
  }); }
  function confirm() { void work(async () => {
    if (!preview || preview.error || dirty || decisions.some((action) => !action)) return;
    setSubmitted(true);
    const saved = await runtime.commitGrammarImport(preview.operation, decisions.map((action, index) => ({ index, action: action as NoteImportSelection['action'], allow_unlinked: false })));
    setResult(saved); setPreview(null); setSubmitted(false); setStatus(text.grammarImport.saved); onSaved();
  }); }
  function cancel() { void work(async () => {
    if (operation.current) await runtime.cancelNoteImport(operation.current);
    operation.current = null; setPreview(null); setError(''); setStatus(text.interchange.cancelled); setSubmitted(false);
  }); }
  function start(id: number) { void work(async () => {
    if (result?.new_card_ids.includes(id)) await cards.start(id);
  }); }
  return { preview, result, busy, error, status, started: cards.started, startErrors: cards.startErrors, delimiter, format, mapping, dirty, submitted, decisions, select, remap, confirm, cancel, start,
    separate: (value: 'comma' | 'semicolon') => { setDelimiter(value); if (preview) setDirty(true); },
    map: (field: keyof GrammarImportMapping, value: number | null) => { setMapping((old) => ({ ...old, [field]: value })); setDirty(true); },
    decide: (index: number, action: Decision) => setDecisions((old) => old.map((value, at) => at === index ? action : value)) };
}
type ImportState = ReturnType<typeof useGrammarImport>;

function Mapping({ state, text }: { readonly state: ImportState; readonly text: NotebookText }) {
  const messages = text.interchange;
  if (state.format !== 'csv') return null;
  return <fieldset disabled={state.busy || state.submitted}>
    <label>{messages.delimiter}<select aria-label={messages.delimiter} value={state.delimiter} onChange={(event) => state.separate(event.target.value as 'comma' | 'semicolon')}>
      <option value="comma">{messages.comma}</option><option value="semicolon">{messages.semicolon}</option>
    </select></label>
    {state.preview && <>{(['exchange_id', 'title', 'body', 'question', 'answer'] as const).map((field) => <label key={field}>{field}
      <select aria-label={field} value={state.mapping[field] ?? ''} onChange={(event) => state.map(field, event.target.value === '' ? null : Number(event.target.value))}>
        <option value="">{messages.unmapped}</option>{state.preview!.columns.map((column, index) => <option key={index} value={index}>{index + 1}: {column}</option>)}
      </select></label>)}<button onClick={state.remap}>{messages.remap}</button></>}
  </fieldset>;
}
function Incoming({ state, index, text }: { readonly state: ImportState; readonly index: number; readonly text: NotebookText }) {
  const group = state.preview!.groups[index], labels = text.grammarImport, messages = text.interchange;
  return <section aria-label={group.content.title}><h3>{group.content.title}</h3><p>{messages.rows}: {group.rows.join(', ')}</p>{group.files && <p>{group.files.join(', ')}</p>}
    <p>{labels.identity}: {group.content.exchange_id || labels.assigned}</p><p>{messages[group.status]}</p>
    <IncomingContent group={group} text={text} />
    {group.error && <p role="alert">{group.error}</p>}
    {group.existing && <><p>{messages.current}</p><h4>{group.existing.title}</h4><pre>{group.existing.body}</pre>
      <pre>{group.existing.card?.question}</pre><pre>{group.existing.card?.answer}</pre></>}
    <ImportDecision state={state} index={index} text={text} />
  </section>;
}
function IncomingContent({ group, text }: { readonly group: GrammarImportPreview['groups'][number]; readonly text: NotebookText }) {
  const { content } = group;
  return <><p>{text.interchange.incoming}</p><pre>{content.body ?? text.interchange.preserved}</pre>
    {content.question?.trim() || content.answer?.trim() ? <><pre>{content.question}</pre><pre>{content.answer}</pre></> : <p>{text.grammarImport.pairPreserved}</p>}</>;
}
function ImportDecision({ state, index, text }: { readonly state: ImportState; readonly index: number; readonly text: NotebookText }) {
  const group = state.preview!.groups[index], action = state.decisions[index], messages = text.interchange;
  return group.status === 'invalid' ? <label><input type="checkbox" disabled={state.busy || state.submitted} checked={action === 'skip'}
      aria-label={messages.exclude + '：' + group.content.title} onChange={(event) => state.decide(index, event.target.checked ? 'skip' : '')} />{messages.exclude}</label>
      : group.status !== 'identical' ? <label>{messages.decision}<select aria-label={messages.decision + '：' + group.content.title} disabled={state.busy || state.submitted}
        value={action ?? ''} onChange={(event) => state.decide(index, event.target.value as Decision)}>
        <option value="">{messages.decide}</option><option value={group.status === 'create' ? 'create' : 'update'}>{group.status === 'create' ? messages.create : messages.update}</option>
        <option value="skip">{messages.skip}</option>
      </select></label> : null;
}
function Preview({ state, text }: { readonly state: ImportState; readonly text: NotebookText }) {
  const preview = state.preview, messages = text.interchange;
  if (!preview) return null;
  return <><p>{messages.source}: {preview.source.filename}</p>
    {preview.error && <p role="alert">{preview.error}</p>}
    {preview.groups.map((_group, index) => <Incoming key={index} state={state} index={index} text={text} />)}
    <p>{messages.selected}: {state.decisions.filter((action) => action === 'create' || action === 'update').length}</p>
    <button disabled={state.busy || state.dirty || !!preview.error || !preview.groups.length || state.decisions.some((action) => !action)} onClick={state.confirm}>{messages.confirm}</button>
    <button disabled={state.busy} onClick={state.cancel}>{messages.cancel}</button>
  </>;
}
function Saved({ state, text, onOpen }: { readonly state: ImportState; readonly text: NotebookText; readonly onOpen: (record: ReviewRecord) => void }) {
  const { result, busy } = state, labels = text.grammarImport, messages = text.interchange;
  return <>{state.status && <p role="status">{state.status}{result && <> {messages.createdCount} {result.created} · {messages.updatedCount} {result.updated} · {messages.skippedCount} {result.skipped}</>}</p>}
    {result?.records.map((record) => <section key={record.id} aria-label={record.title}><h4>{record.title}</h4><button disabled={busy} onClick={() => onOpen(record)}>{messages.openRecord}</button>
      {result.new_card_ids.includes(record.id) && <><p>{state.started.includes(record.id) ? labels.started : labels.paused}</p>
        {state.startErrors[record.id] && <p role="alert">{state.startErrors[record.id]}</p>}
        {!state.started.includes(record.id) && <button disabled={busy} onClick={() => state.start(record.id)}>{labels.start}</button>}</>}
    </section>)}</>;
}
export function GrammarImport({ runtime, text, onSaved, onOpen, onPrepare }: {
  readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onSaved: () => void;
  readonly onOpen: (record: ReviewRecord) => void; readonly onPrepare: (action: () => void) => void;
}) {
  const state = useGrammarImport(runtime, onSaved, text), labels = text.grammarImport;
  return <section aria-label={labels.open}>
    <button disabled={state.busy} onClick={() => onPrepare(() => state.select())}>{labels.open}</button>
    <button disabled={state.busy} onClick={() => onPrepare(() => state.select('markdown'))}>{labels.markdown}</button>
    <button disabled={state.busy} onClick={() => onPrepare(() => state.select('zip'))}>{labels.zip}</button><p>{labels.columns}</p>
    <Mapping state={state} text={text} />
    {state.busy && <p>{text.interchange.busy}</p>}
    {state.error && <p role="alert">{state.error}</p>}
    <Preview state={state} text={text} /><Saved state={state} text={text} onOpen={onOpen} />
  </section>;
}
