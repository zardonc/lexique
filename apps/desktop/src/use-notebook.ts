import { useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage, snapshotFromLookup } from './platform/vocabulary';
import type { SaveRequest, SaveResult, VocabularyRecord, VocabularySnapshot } from './platform/vocabulary';
import type { DictionaryLookup } from './platform/dictionary';

interface Editor {
  readonly snapshot: VocabularySnapshot;
  readonly record: VocabularyRecord | null;
  readonly notes: string;
  readonly example_fr: string;
  readonly example_zh: string;
  readonly example_sense: number | null;
  readonly more_examples: VocabularyRecord['more_examples'];
}

function acceptRecord(record: VocabularyRecord): Editor {
  return { snapshot: record, record, notes: record.notes, example_fr: record.example_fr, example_zh: record.example_zh, example_sense: record.example_sense, more_examples: record.more_examples };
}

function savePayload(editor: Editor) {
  return { headword: editor.snapshot.headword, notes: editor.notes, example_fr: editor.example_fr, example_zh: editor.example_zh, example_sense: editor.example_sense, more_examples: editor.more_examples,
    id: editor.record?.id ?? null, revision: editor.record?.revision ?? null };
}

function samePayload(previous: SaveRequest, payload: ReturnType<typeof savePayload>) {
  return Object.entries(payload).every(([key, value]) => JSON.stringify(previous[key as keyof SaveRequest]) === JSON.stringify(value));
}

function needsExampleSense(editor: Editor) {
  const hasContent = [editor.example_fr, editor.example_zh, ...editor.more_examples.flatMap((example) => [example.fr, example.zh])].some((value) => value.trim());
  const original = editor.record;
  const unchangedLegacy = original && original.example_sense === null && editor.example_fr === original.example_fr && editor.example_zh === original.example_zh && JSON.stringify(editor.more_examples) === JSON.stringify(original.more_examples);
  return editor.example_sense === null && hasContent && !unchangedLegacy;
}

async function generateEnrolledAudio(runtime: RuntimeAdapter, id: number) {
  const cards = await runtime.reviewCards(id);
  if (!cards?.some((card) => card.kind === 'listening') || await runtime.cachedSpeech(id)) return false;
  if (!await runtime.generateSpeech(id)) throw new Error('Native speech unavailable');
  return true;
}

function needsAudio(result: SaveResult, id: number | null, accepted: boolean) {
  return accepted && id === null && (result.outcome === 'created' || result.outcome === 'replayed');
}

function useEnrollmentAudio(runtime: RuntimeAdapter, refresh: () => void) {
  const [audioError, setAudioError] = useState('');
  async function generate(id: number) {
    setAudioError('');
    try { if (await generateEnrolledAudio(runtime, id)) refresh(); return true; }
    catch (cause) { setAudioError(errorMessage(cause)); return false; }
  }
  return { audioError, clearAudioError: () => setAudioError(''), generate };
}

export function useNotebook(runtime: RuntimeAdapter) {
  const [editor, setEditor] = useState<Editor | null>(null);
  const [conflict, setConflict] = useState<VocabularyRecord | null>(null);
  const [existing, setExisting] = useState<VocabularyRecord | null>(null);
  const [error, setError] = useState('');
  const [saved, setSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  const [version, setVersion] = useState(0);
  const audio = useEnrollmentAudio(runtime, () => setVersion((value) => value + 1));
  const operation = useRef<SaveRequest | null>(null); const saving = useRef(false);
  // A failed response can hide a committed update, even when input returns to its original text.
  const dirty = editor !== null && (editor.record === null || !samePayload({ ...savePayload(acceptRecord(editor.record)), operation: '' }, savePayload(editor)) || operation.current !== null);
  function clear() { setEditor(null); setConflict(null); setExisting(null); setError(''); audio.clearAudioError(); setSaved(false); operation.current = null; }
  function open(record: VocabularyRecord) { clear(); setEditor(acceptRecord(record)); }
  function collect(lookup: DictionaryLookup) {
    clear(); setEditor({ snapshot: snapshotFromLookup(lookup), record: null, notes: '', example_fr: '', example_zh: '', example_sense: null, more_examples: [] });
  }
  function applyResult(result: SaveResult, payload: ReturnType<typeof savePayload>) {
    setVersion((value) => value + 1);
    if (result.outcome === 'existing') { setExisting(result.record); return false; }
    if (result.outcome === 'conflict' || !samePayload({ ...savePayload(acceptRecord(result.record)), id: payload.id, revision: payload.revision, operation: '' }, payload)) {
      setConflict(result.record); return false;
    }
    setEditor(acceptRecord(result.record)); setSaved(true); operation.current = null;
    return true;
  }
  async function save() {
    if (!editor || saving.current || needsExampleSense(editor)) return false;
    const payload = savePayload(editor);
    const previous = operation.current;
    if (!previous || !samePayload(previous, payload)) {
      operation.current = { ...payload, operation: crypto.randomUUID() };
    }
    saving.current = true; setBusy(true); setError(''); setSaved(false);
    try {
      const result = await runtime.saveVocabulary(operation.current!);
      if (!result) throw new Error('Native notebook storage unavailable');
      const accepted = applyResult(result, payload);
      if (needsAudio(result, payload.id, accepted) && !await audio.generate(result.record.id)) return false;
      return accepted;
    } catch (cause) { setError(errorMessage(cause)); return false; }
    finally { saving.current = false; setBusy(false); }
  }
  return { editor, conflict, existing, error, audioError: audio.audioError, saved, busy, dirty, meaningRequired: editor !== null && needsExampleSense(editor), version, save, clear, open, collect,
    edit: (notes: string) => { setEditor((value) => value && { ...value, notes }); setSaved(false); setExisting(null); },
    editExample: (field: 'example_fr' | 'example_zh', text: string) => { setEditor((value) => value && { ...value, [field]: text }); setSaved(false); setExisting(null); },
    editSense: (example_sense: number | null) => { setEditor((value) => value && { ...value, example_sense }); setSaved(false); setExisting(null); }, editMore: (examples: VocabularyRecord['more_examples']) => { setEditor((value) => value && { ...value, more_examples: examples }); setSaved(false); setExisting(null); },
    adopt: () => { if (conflict) open(conflict); },
    back: () => setConflict(null),
  };
}

export type Notebook = ReturnType<typeof useNotebook>;
