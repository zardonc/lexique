import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import { messages } from './content';
import { DictionaryResults } from './dictionary';
import type { DictionaryView } from './conjugation-navigation';
import { lookupInputMessages } from './lookup-input-content';
import { dictionaryFailure, dictionaryQuery } from './platform/dictionary';
import type { DictionaryFailure, DictionaryLookup } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';
import { selectionText, type SelectionEvent } from './platform/selection';

function useInputLookup(runtime: RuntimeAdapter, selection?: SelectionEvent) {
  const [raw, setRaw] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DictionaryLookup | null>();
  const [error, setError] = useState<DictionaryFailure>();
  const [failure, setFailure] = useState<Extract<SelectionEvent, { kind: 'failed' }>>();
  const request = useRef(0);
  function cancel() { request.current++; setLoading(false); }
  function edit(value: string) {
    cancel(); setResult(undefined); setError(undefined); setFailure(undefined);
    setRaw(selectionText.parse(value));
  }
  async function lookup(value: string) {
    const key = value.trim(), identity = ++request.current;
    if (!dictionaryQuery.safeParse(key).success) {
      setError({ code: 'invalid_query', message: '' }); setResult(undefined); setLoading(false); return;
    }
    setLoading(true); setResult(undefined); setError(undefined);
    try { const response = await runtime.lookupDictionary(key); if (request.current === identity) setResult(response); }
    catch (cause: unknown) { if (request.current === identity) setError(dictionaryFailure(cause)); }
    finally { if (request.current === identity) setLoading(false); }
  }
  useEffect(() => {
    if (!selection) return;
    cancel(); setResult(undefined); setError(undefined); setFailure(undefined);
    if (selection.kind === 'pending') setRaw('');
    else if (selection.kind === 'failed') setFailure(selection);
    else setRaw(selection.text);
  }, [selection, runtime]);
  useEffect(() => () => { request.current++; }, []);
  return { raw, loading, result, error, failure, edit, lookup, cancel };
}

function InputResults({ input, locale, onDictionary }: {
  readonly input: ReturnType<typeof useInputLookup>; readonly locale: Locale;
  readonly onDictionary: (view: DictionaryView) => void;
}) {
  const text = lookupInputMessages[locale], dictionaryText = messages[locale].dictionary;
  const { result, error, failure } = input;
  return <>
    {failure && <p role="alert">{text[failure.code]} {failure.message}</p>}
    {error && <p role="alert">{dictionaryText[error.code]} {error.message}</p>}
    {result === null && <p role="status">{dictionaryText.preview}</p>}
    {result && <section className="dictionary-original"><DictionaryResults result={result} text={dictionaryText} />
      {result.entries[0] && <button onClick={() => onDictionary({ result, ordinal: result.entries[0]!.ordinal })}>{text.full}</button>}
    </section>}
  </>;
}

export function LookupInputPanel({ runtime, locale, selection, onCancel, onDictionary }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly selection?: SelectionEvent;
  readonly onCancel: () => void; readonly onDictionary: (view: DictionaryView) => void;
}) {
  const text = lookupInputMessages[locale], dictionaryText = messages[locale].dictionary;
  const input = useInputLookup(runtime, selection), field = useRef<HTMLTextAreaElement>(null);
  const raw = input.raw;
  const capturing = selection?.kind === 'pending';
  function selected() { const node = field.current; return node && node.selectionEnd > node.selectionStart ? raw.slice(node.selectionStart, node.selectionEnd) : raw; }
  return <div className="lookup-input">
    <form onSubmit={(event) => { event.preventDefault(); void input.lookup(selected()); }}>
      <label htmlFor="lookup-input-raw">{text.raw}</label>
      <textarea ref={field} id="lookup-input-raw" lang="fr" rows={4} value={raw} required
        autoCapitalize="none" autoCorrect="off" spellCheck={false}
        onChange={(event) => { if (selection) onCancel(); input.edit(event.target.value); }} />
      <p className="dictionary-hint">{text.hint}</p>
      {raw && !dictionaryQuery.safeParse(raw.trim()).success && <p>{text.long}</p>}
      <div className="actions"><button type="submit" disabled={input.loading || capturing}>{input.loading ? dictionaryText.loading : text.query}</button>
        {(input.loading || capturing) && <button type="button" onClick={() => { input.cancel(); onCancel(); }}>{text.cancel}</button>}</div>
    </form>
    {capturing && <p role="status">{text.capturing}</p>}
    <InputResults input={input} locale={locale} onDictionary={onDictionary} />
  </div>;
}
