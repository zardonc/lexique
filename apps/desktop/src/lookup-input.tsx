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
import { useOcrInput } from './use-ocr-input';
import { OcrInput } from './ocr-input';
import { TranslationPanel, useTranslation } from './translation-panel';
import { ExampleSpeech } from './example-speech';
import { notebookMessages } from './notebook-content';

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
    else setRaw(selection.kind === 'captured' ? selection.text : '');
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

function sourceLanguage(translation: ReturnType<typeof useTranslation>) {
  if (translation.busy || (translation.result && !translation.complete)) return undefined;
  const language = translation.settings?.preferences.source_language;
  return language === 'auto' ? (translation.complete && translation.result?.source !== 'auto' ? translation.result?.source : undefined) : language;
}

function SourceSpeech({ language, raw, runtime, locale, playbackKey, visible }: { readonly language?: string; readonly raw: string; readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly playbackKey: string; readonly visible: boolean }) {
  return visible && language === 'fr' ? <ExampleSpeech key={playbackKey} value={raw} runtime={runtime} text={notebookMessages[locale]} /> : null;
}

function useQuerySelection(raw: string, selection?: SelectionEvent) {
  const [target, setTarget] = useState('');
  const [source, setSource] = useState('');
  useEffect(() => { setSource(''); setTarget(''); }, [raw, selection]);
  const selected = target || source || raw;
  return { selected, valid: dictionaryQuery.safeParse(selected.trim()).success,
    clear: () => { setTarget(''); setSource(''); },
    source: (node: HTMLTextAreaElement) => { setTarget(''); setSource(raw.slice(node.selectionStart, node.selectionEnd)); },
    target: (value: string) => { if (value) setSource(''); setTarget(value); }, setTarget };
}

function lookupDisabled(loading: boolean, capturing: boolean, valid: boolean) {
  return loading || capturing || !valid;
}

export function LookupInputPanel({ runtime, locale, selection, onCancel, onDictionary, onSettings, visible }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly selection?: SelectionEvent;
  readonly onCancel: () => void; readonly onDictionary: (view: DictionaryView) => void; readonly onSettings: () => void; readonly visible: boolean;
}) {
  const text = lookupInputMessages[locale], dictionaryText = messages[locale].dictionary;
  const input = useInputLookup(runtime, selection), field = useRef<HTMLTextAreaElement>(null);
  const translation = useTranslation(runtime, locale, { raw: input.raw, visible });
  useEffect(() => { if (!visible && translation.busy) translation.cancel(); }, [visible]);
  const query = useQuerySelection(input.raw, selection);
  function edit(value: string) { translation.cancel(); input.edit(value); }
  const ocr = useOcrInput(runtime, selection, edit);
  useEffect(() => { if (selection) translation.cancel(); }, [selection]);
  const raw = input.raw;
  const capturing = selection?.kind === 'pending';
  const selectedLanguage = translation.settings?.preferences.source_language;
  const resolvedSource = sourceLanguage(translation);
  useEffect(() => { query.setTarget(''); }, [translation.result, translation.busy, selectedLanguage]);
  return <div className="lookup-input" onPaste={(event) => {
    const file = event.clipboardData.files[0];
    if (file) { event.preventDefault(); translation.cancel(); void ocr.importImage(file); }
  }}>
    <OcrInput ocr={ocr} locale={locale} onCancel={onCancel} onStart={translation.cancel} />
    <form onSubmit={(event) => { event.preventDefault(); void input.lookup(query.selected); }}>
      <label htmlFor="lookup-input-raw">{text.raw}</label>
      <textarea ref={field} id="lookup-input-raw" lang={resolvedSource} rows={4} value={raw} required
        autoCapitalize="none" autoCorrect="off" spellCheck={false}
        onSelect={(event) => query.source(event.currentTarget)}
        onChange={(event) => { query.clear(); ocr.stop(); if (selection) onCancel(); edit(event.target.value); }} />
      <SourceSpeech language={resolvedSource} visible={visible} playbackKey={`${selectedLanguage}-${translation.busy}-${translation.saving}`} raw={raw} runtime={runtime} locale={locale} />
      <p className="dictionary-hint">{text.hint}</p>
      {raw && !query.valid && <p>{text.long}</p>}
      <div className="actions"><button type="submit" disabled={lookupDisabled(input.loading, capturing, query.valid)}>{input.loading ? dictionaryText.loading : text.query}</button>
        {(input.loading || capturing) && <button type="button" onClick={() => { input.cancel(); onCancel(); }}>{text.cancel}</button>}</div>
    </form>
    <TranslationPanel translation={translation} locale={locale} raw={raw} onSettings={onSettings} runtime={runtime} onSelection={query.target} visible={visible} />
    {capturing && <p role="status">{text.capturing}</p>}
    <InputResults input={input} locale={locale} onDictionary={onDictionary} />
  </div>;
}
