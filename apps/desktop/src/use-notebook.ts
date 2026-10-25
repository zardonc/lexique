import { useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage, snapshotFromLookup } from './platform/vocabulary';
import type { SaveRequest, SaveResult, VocabularyRecord, VocabularySnapshot } from './platform/vocabulary';
import type { DictionaryLookup } from './platform/dictionary';

interface Editor {
  readonly snapshot: VocabularySnapshot;
  readonly record: VocabularyRecord | null;
  readonly notes: string;
}

function acceptRecord(record: VocabularyRecord): Editor {
  return { snapshot: record, record, notes: record.notes };
}

function savePayload(editor: Editor) {
  return { headword: editor.snapshot.headword, notes: editor.notes,
    id: editor.record?.id ?? null, revision: editor.record?.revision ?? null };
}

export function useNotebook(runtime: RuntimeAdapter) {
  const [editor, setEditor] = useState<Editor | null>(null);
  const [conflict, setConflict] = useState<VocabularyRecord | null>(null);
  const [existing, setExisting] = useState<VocabularyRecord | null>(null);
  const [error, setError] = useState('');
  const [saved, setSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  const [version, setVersion] = useState(0);
  const operation = useRef<SaveRequest | null>(null);
  const saving = useRef(false);
  // A failed response can hide a committed update, even when input returns to its original text.
  const dirty = editor !== null && (editor.record === null || editor.notes !== editor.record.notes || operation.current !== null);
  function clear() {
    setEditor(null); setConflict(null); setExisting(null); setError(''); setSaved(false); operation.current = null;
  }
  function open(record: VocabularyRecord) { clear(); setEditor(acceptRecord(record)); }
  function collect(lookup: DictionaryLookup) {
    clear(); setEditor({ snapshot: snapshotFromLookup(lookup), record: null, notes: '' });
  }
  function applyResult(result: SaveResult, notes: string) {
    setVersion((value) => value + 1);
    if (result.outcome === 'existing') { setExisting(result.record); return false; }
    if (result.outcome === 'conflict' || result.record.notes !== notes) {
      setConflict(result.record); return false;
    }
    setEditor(acceptRecord(result.record)); setSaved(true); operation.current = null;
    return true;
  }
  async function save() {
    if (!editor || saving.current) return false;
    const payload = savePayload(editor);
    const previous = operation.current;
    if (!previous || Object.entries(payload).some(([key, value]) => previous[key as keyof SaveRequest] !== value)) {
      operation.current = { ...payload, operation: crypto.randomUUID() };
    }
    saving.current = true; setBusy(true); setError(''); setSaved(false);
    try {
      const result = await runtime.saveVocabulary(operation.current!);
      if (!result) throw new Error('Native notebook storage unavailable');
      return applyResult(result, payload.notes);
    } catch (cause) { setError(errorMessage(cause)); return false; }
    finally { saving.current = false; setBusy(false); }
  }
  return { editor, conflict, existing, error, saved, busy, dirty, version, save, clear, open, collect,
    edit: (notes: string) => { setEditor((value) => value && { ...value, notes }); setSaved(false); setExisting(null); },
    adopt: () => { if (conflict) open(conflict); },
    back: () => setConflict(null),
  };
}

export type Notebook = ReturnType<typeof useNotebook>;
