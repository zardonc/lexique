import { useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import type { GrammarCardRequest, GrammarRecord, GrammarSaveRequest } from './platform/notes';

interface GrammarEditor {
  readonly record: GrammarRecord | null;
  readonly title: string;
  readonly body: string;
  readonly category_ids: readonly number[];
  readonly card: { readonly question: string; readonly answer: string } | null;
}
interface SaveCallbacks {
  readonly accept: (editor: GrammarEditor) => void;
  readonly conflict: (record: GrammarRecord) => void;
}
function acceptRecord(record: GrammarRecord): GrammarEditor {
  return { record, title: record.title, body: record.body, category_ids: record.category_ids, card: record.card };
}
const categoryKey = (ids: readonly number[]) => JSON.stringify([...ids].sort((a, b) => a - b));
function sameNote(record: Pick<GrammarRecord, 'title' | 'body' | 'category_ids'>, editor: GrammarEditor) {
  return record.title === editor.title.trim() && record.body === editor.body && categoryKey(record.category_ids) === categoryKey(editor.category_ids);
}
function changedNote(editor: GrammarEditor) { return !editor.record || !sameNote(editor.record, editor); }
function sameCardContent(left: GrammarEditor['card'], right: GrammarEditor['card']) {
  return left?.question === right?.question && left?.answer === right?.answer;
}
function changedCard(editor: GrammarEditor) {
  return editor.card !== null && !sameCardContent(editor.card, editor.record?.card ?? null);
}
function conflictingCard(editor: GrammarEditor, record: GrammarRecord) {
  return changedCard(editor) && editor.record !== null && (editor.record.card?.id !== record.card?.id
    || !sameCardContent(editor.record.card, record.card));
}
function noteRequest(state: GrammarEditor, previous: GrammarSaveRequest | null): GrammarSaveRequest {
  const id = state.record?.id ?? null;
  if (previous && previous.id === id && sameNote(previous, state)) return previous;
  return { operation: crypto.randomUUID(), id, revision: state.record?.revision ?? null,
    title: state.title.trim(), body: state.body, category_ids: state.category_ids };
}
function cardRequest(state: GrammarEditor, previous: GrammarCardRequest | null): GrammarCardRequest {
  const { question, answer } = state.card!, record = state.record!;
  if (previous && previous.id === record.id && previous.question === question && previous.answer === answer) return previous;
  return { operation: crypto.randomUUID(), id: record.id, revision: record.revision, question, answer };
}

function useGrammarNoteSave(runtime: RuntimeAdapter, callbacks: SaveCallbacks) {
  const operation = useRef<GrammarSaveRequest | null>(null);
  async function save(state: GrammarEditor) {
    operation.current = noteRequest(state, operation.current);
    const result = await runtime.saveGrammar(operation.current);
    if (!result) throw new Error('Native notebook storage unavailable');
    if (result.outcome === 'conflict' || !sameNote(result.record, state) || conflictingCard(state, result.record)) { callbacks.conflict(result.record); return null; }
    const next = { ...acceptRecord(result.record), card: changedCard(state) ? state.card : result.record.card };
    operation.current = null; callbacks.accept(next); return next;
  }
  return { save, pending: operation.current !== null, clear: () => { operation.current = null; } };
}
function matchesSavedCard(state: GrammarEditor, result: GrammarRecord) {
  const card = result.card, request = state.card!;
  return card !== null && sameCardContent(card, request) && sameNote(result, acceptRecord(state.record!));
}
function useGrammarCardSave(runtime: RuntimeAdapter, callbacks: SaveCallbacks) {
  const operation = useRef<GrammarCardRequest | null>(null);
  async function save(state: GrammarEditor) {
    if (!state.record || !state.card) return false;
    operation.current = cardRequest(state, operation.current);
    const result = await runtime.saveGrammarCard(operation.current);
    if (!result) throw new Error('Native notebook storage unavailable');
    if (result.outcome === 'conflict' || !matchesSavedCard(state, result.record)) { callbacks.conflict(result.record); return false; }
    const card = result.record.card!;
    if (state.record.card && card.id !== state.record.card.id) throw new Error('Grammar card identity changed');
    operation.current = null; callbacks.accept({ ...state, record: result.record, card }); return true;
  }
  return { save, pending: operation.current !== null, clear: () => { operation.current = null; } };
}
function useGrammarOperation() {
  const [error, setError] = useState(''), [busy, setBusy] = useState(false), working = useRef(false);
  async function run(action: () => Promise<boolean>) {
    if (working.current) return false;
    working.current = true; setBusy(true); setError('');
    try { return await action(); }
    catch (cause) { setError(errorMessage(cause)); return false; }
    finally { working.current = false; setBusy(false); }
  }
  return { error, busy, run, reset: () => setError('') };
}
async function saveDraft(part: 'note' | 'card' | 'all', state: GrammarEditor, saves: {
  readonly note: ReturnType<typeof useGrammarNoteSave>; readonly card: ReturnType<typeof useGrammarCardSave>;
}) {
  if (part === 'note') return await saves.note.save(state) !== null;
  if (part === 'card') return saves.card.save(state);
  let current = state;
  if (changedNote(current) || saves.note.pending) {
    const next = await saves.note.save(current);
    if (!next) return false;
    current = next;
  }
  if (changedCard(current) || saves.card.pending) return saves.card.save(current);
  return true;
}

export function useGrammarNotebook(runtime: RuntimeAdapter) {
  const [editor, setEditor] = useState<GrammarEditor | null>(null), [conflict, setConflict] = useState<GrammarRecord | null>(null);
  const [saved, setSaved] = useState(false), [version, setVersion] = useState(0), action = useGrammarOperation();
  const changed = () => setVersion((value) => value + 1);
  const callbacks: SaveCallbacks = { accept: (value) => { setEditor(value); setSaved(true); setConflict(null); changed(); },
    conflict: (record) => { setConflict(record); setSaved(false); changed(); } };
  const noteSave = useGrammarNoteSave(runtime, callbacks), cardSave = useGrammarCardSave(runtime, callbacks);
  function clear() { setEditor(null); setConflict(null); setSaved(false); action.reset(); noteSave.clear(); cardSave.clear(); }
  function open(record: GrammarRecord) { clear(); setEditor(acceptRecord(record)); }
  function create() { clear(); setEditor({ record: null, title: '', body: '', category_ids: [], card: null }); }
  function change(fields: Partial<Pick<GrammarEditor, 'title' | 'body' | 'category_ids' | 'card'>>) {
    setEditor((value) => value && { ...value, ...fields }); setSaved(false); action.reset();
  }
  function commit(part: 'note' | 'card' | 'all') {
    return editor ? action.run(() => saveDraft(part, editor, { note: noteSave, card: cardSave })) : Promise.resolve(false);
  }
  function makeCard() {
    if (!editor?.record || changedNote(editor)) return;
    change({ card: { question: editor.record.title, answer: editor.record.body } });
  }
  function refresh() {
    return action.run(async () => {
      if (!editor?.record) return true;
      const record = await runtime.getGrammar(editor.record.id);
      if (!record) throw new Error('Grammar record unavailable');
      open(record); return true;
    });
  }
  function remove() {
    return action.run(async () => {
      if (!editor?.record) return false;
      if (await runtime.deleteGrammar(editor.record.id, editor.record.revision) === null) throw new Error('Native notebook storage unavailable');
      clear(); changed(); return true;
    });
  }
  const noteDirty = editor !== null && (changedNote(editor) || noteSave.pending), cardDirty = editor !== null && (changedCard(editor) || cardSave.pending);
  return { editor, conflict, saved, busy: action.busy, error: action.error, dirty: noteDirty || cardDirty, noteDirty, cardDirty, version,
    clear, open, create, change, makeCard, refresh, remove, save: () => commit('all'), saveNote: () => commit('note'), saveCard: () => commit('card'),
    adopt: () => { if (conflict) open(conflict); }, back: () => setConflict(null) };
}

export type GrammarNotebook = ReturnType<typeof useGrammarNotebook>;
