import { useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { characterCount, errorMessage, snapshotFromLookup } from './platform/vocabulary';
import type { SaveRequest, SaveResult, VocabularyRecord, VocabularySnapshot } from './platform/vocabulary';
import type { DictionaryLookup } from './platform/dictionary';

interface Editor {
  readonly snapshot: VocabularySnapshot;
  readonly record: VocabularyRecord | null;
  readonly notes: string;
  readonly example_fr: string;
  readonly example_zh: string;
  readonly example_source?: VocabularyRecord['reference'] | null;
  readonly example_sense: number | null;
  readonly more_examples: VocabularyRecord['more_examples'];
}

function acceptRecord(record: VocabularyRecord): Editor {
  return { snapshot: record, record, notes: record.notes, example_fr: record.example_fr, example_zh: record.example_zh, example_source: record.example_source, example_sense: record.example_sense, more_examples: record.more_examples };
}

function savePayload(editor: Editor) {
  return { headword: editor.snapshot.headword, notes: editor.notes, example_fr: editor.example_fr, example_zh: editor.example_zh, example_source: editor.example_source, example_sense: editor.example_sense, more_examples: editor.more_examples,
    id: editor.record?.id ?? null, revision: editor.record?.revision ?? null };
}

function stableJson(value: unknown) {
  return JSON.stringify(value, (_key, item: unknown) => item && typeof item === 'object' && !Array.isArray(item) ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b))) : item);
}

function samePayload(previous: SaveRequest, payload: ReturnType<typeof savePayload>) {
  return Object.entries(payload).every(([key, value]) => stableJson(previous[key as keyof SaveRequest]) === stableJson(value));
}

function needsExampleSense(editor: Editor) {
  const hasContent = [editor.example_fr, editor.example_zh, ...editor.more_examples.flatMap((example) => [example.fr, example.zh])].some((value) => value.trim());
  const original = editor.record;
  const unchangedLegacy = original && original.example_sense === null && editor.example_fr === original.example_fr && editor.example_zh === original.example_zh && JSON.stringify(editor.more_examples) === JSON.stringify(original.more_examples);
  return editor.example_sense === null && hasContent && !unchangedLegacy;
}



function useExcerpt(editor: Editor | null, setEditor: (editor: Editor) => void, { open, collect, changed }: { readonly open: (record: VocabularyRecord) => void; readonly collect: (lookup: DictionaryLookup) => void; readonly changed: () => void }) {
  const [excerpt, setExcerpt] = useState<{ readonly source: VocabularyRecord['reference']; readonly fr: string; readonly zh: string } | null>(null);
  const [excerptStatus, setExcerptStatus] = useState<'duplicate' | 'capacity' | 'limit' | ''>('');
  const [focusExample, setFocusExample] = useState('');
  function beginExcerpt(lookup: DictionaryLookup, record: VocabularyRecord | null) {
    if (editor?.snapshot.headword !== lookup.entries[0]?.key) {
      if (record) open(record); else collect(lookup);
    }
    setExcerpt({ source: snapshotFromLookup(lookup).reference, fr: '', zh: '' }); setExcerptStatus('');
  }
  function transferExcerpt() {
    if (!editor || !excerpt) return;
    if ([excerpt.fr, excerpt.zh].some((text) => characterCount(text) > 20_000)) { setExcerptStatus('limit'); return; }
    const examples = [{ fr: editor.example_fr, zh: editor.example_zh, source: editor.example_source }, ...editor.more_examples];
    const sameSource = (source: VocabularyRecord['example_source']) => source && Object.entries(excerpt.source).every(([key, value]) => source[key as keyof typeof source] === value);
    const duplicate = examples.findIndex((item) => item.fr === excerpt.fr && item.zh === excerpt.zh && sameSource(item.source));
    if (duplicate >= 0) { setExcerptStatus('duplicate'); setFocusExample(duplicate === 0 ? 'example-fr' : `example-fr-${duplicate - 1}`); return; }
    if (!editor.example_fr && !editor.example_zh) setEditor({ ...editor, example_fr: excerpt.fr, example_zh: excerpt.zh, example_source: excerpt.source });
    else {
      if (editor.more_examples.length >= 100) { setExcerptStatus('capacity'); return; }
      setEditor({ ...editor, more_examples: [...editor.more_examples, excerpt] });
    }
    setExcerpt(null); setExcerptStatus(''); changed();
  }
  return { excerpt, excerptStatus, focusExample, beginExcerpt, transferExcerpt, excerptPending: !!excerpt && (!!excerpt.fr || !!excerpt.zh), cancelExcerpt: () => { setExcerpt(null); setExcerptStatus(''); }, editExcerpt: (field: 'fr' | 'zh', value: string) => { setExcerpt((old) => old && { ...old, [field]: value }); setExcerptStatus(''); } };
}

function hasExcerptSource(editor: Editor | null) {
  return editor && [editor.example_source, ...editor.more_examples.map((example) => example.source)].some(Boolean);
}

export function useNotebook(runtime: RuntimeAdapter) {
  const [editor, setEditor] = useState<Editor | null>(null);
  const [conflict, setConflict] = useState<VocabularyRecord | null>(null);
  const [existing, setExisting] = useState<VocabularyRecord | null>(null);
  const [error, setError] = useState('');
  const [saved, setSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  const [version, setVersion] = useState(0);
  const operation = useRef<SaveRequest | null>(null); const saving = useRef(false);
  // A failed response can hide a committed update, even when input returns to its original text.
  function clear() { excerpts.cancelExcerpt(); setEditor(null); setConflict(null); setExisting(null); setError(''); setSaved(false); operation.current = null; }
  function open(record: VocabularyRecord) { clear(); setEditor(acceptRecord(record)); }
  function collect(lookup: DictionaryLookup) {
    clear(); setEditor({ snapshot: snapshotFromLookup(lookup), record: null, notes: '', example_fr: '', example_zh: '', example_sense: null, more_examples: [] });
  }
  const excerpts = useExcerpt(editor, setEditor, { open, collect, changed: () => { setSaved(false); setExisting(null); } });
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
    if (excerpts.excerptPending) return false;
    if (!editor || saving.current || needsExampleSense(editor)) return false;
    const payload = savePayload(editor);
    const previous = operation.current;
    if (!previous || !samePayload(previous, payload)) {
      operation.current = { ...payload, operation: crypto.randomUUID() };
    }
    saving.current = true; setBusy(true); setError(''); setSaved(false);
    try {
      const result = await runtime.saveVocabulary(operation.current!, editor.snapshot.reference.dictionary_id ?? 'builtin');
      if (!result) throw new Error('Native notebook storage unavailable');
      const accepted = applyResult(result, payload);
      return accepted;
    } catch (cause) { setError(errorMessage(cause)); return false; }
    finally { saving.current = false; setBusy(false); }
  }
  const dirty = excerpts.excerptPending || editor !== null && (editor.record === null || !samePayload({ ...savePayload(acceptRecord(editor.record)), operation: '' }, savePayload(editor)) || operation.current !== null);
  return { ...excerpts, preservesExcerpt: Boolean(excerpts.excerpt || hasExcerptSource(editor)), editor, conflict, existing, error, audioError: '', saved, busy, dirty, meaningRequired: editor !== null && needsExampleSense(editor), version, save, clear, open, collect,
    edit: (notes: string) => { setEditor((value) => value && { ...value, notes }); setSaved(false); setExisting(null); },
    editExample: (field: 'example_fr' | 'example_zh', text: string) => { setEditor((value) => value && { ...value, [field]: text }); setSaved(false); setExisting(null); },
    editSense: (example_sense: number | null) => { setEditor((value) => value && { ...value, example_sense }); setSaved(false); setExisting(null); }, editMore: (examples: VocabularyRecord['more_examples']) => { setEditor((value) => value && { ...value, more_examples: examples }); setSaved(false); setExisting(null); },
    adopt: () => { if (conflict) open(conflict); },
    back: () => setConflict(null),
  };
}

export type Notebook = ReturnType<typeof useNotebook>;
