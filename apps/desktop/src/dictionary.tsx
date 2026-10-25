import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import type { Messages } from './content';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure, DictionaryLookup, DictionarySense } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';
import type { NotebookText } from './notebook-content';
import { errorMessage, snapshotFromLookup } from './platform/vocabulary';
import type { VocabularyRecord } from './platform/vocabulary';
import { VocabularySummary } from './vocabulary-view';

function Senses({ senses }: { readonly senses: readonly DictionarySense[] }) {
  return <ol className="dictionary-senses">{senses.map((sense, index) => <li key={index}>
    {sense.translations.map((translation, item) => <p key={`t${item}`} lang="zh">{translation}</p>)}
    {sense.definitions.map((definition, item) => <p key={`d${item}`} lang="fr">{definition}</p>)}
    {sense.senses.length > 0 && <Senses senses={sense.senses} />}
  </li>)}</ol>;
}

export function DictionaryResults({ result, text }: { readonly result: DictionaryLookup; readonly text: Messages['dictionary'] }) {
  return <>
    <p role="status">{result.entries.length === 0 ? text.noMatch : `${text.records}: ${result.entries.length}`}</p>
    {result.entries.map((entry) => <article className="dictionary-entry" key={entry.ordinal}>
      <h3 lang="fr">{entry.key}</h3>
      <p className="dictionary-grammar">{entry.pronunciations.join(' · ')} {entry.grammar.join(' · ')}</p>
      <Senses senses={entry.senses} />
    </article>)}
    <footer className="dictionary-source">
      <p>{text.source}: {result.source.title} · {result.source.version}</p>
      <p>{result.source.attribution} · {result.source.license}</p>
      <p>{result.source.source_url}</p><p>{result.source.license_url}</p>
    </footer>
  </>;
}

function Collection({ result, runtime, text, version, onCollect, onOpen }: {
  readonly result: DictionaryLookup; readonly runtime: RuntimeAdapter; readonly text: NotebookText;
  readonly version: number; readonly onCollect: (result: DictionaryLookup) => void;
  readonly onOpen: (record: VocabularyRecord) => void;
}) {
  const [record, setRecord] = useState<VocabularyRecord | null>();
  const [error, setError] = useState('');
  const [retry, setRetry] = useState(0);
  const headword = result.entries[0]?.key;
  useEffect(() => {
    let cancelled = false;
    setRecord(undefined); setError('');
    if (headword) void runtime.findVocabulary(headword).then((value) => {
      if (!cancelled) setRecord(value);
    }).catch((cause: unknown) => { if (!cancelled) setError(errorMessage(cause)); });
    return () => { cancelled = true; };
  }, [runtime, headword, version, retry]);
  if (!headword) return null;
  return <><VocabularySummary snapshot={snapshotFromLookup(result)} text={text} />
    {error ? <p role="alert">{text.readError}: {error} <button onClick={() => setRetry((value) => value + 1)}>{text.retry}</button></p>
      : record ? <p role="status">{text.existing} <button onClick={() => onOpen(record)}>{text.view}</button></p>
        : <button disabled={record === undefined} onClick={() => onCollect(result)}>{text.collect}</button>}</>;
}

export function RecordOriginal({ headword, runtime, text, title }: {
  readonly headword: string; readonly runtime: RuntimeAdapter; readonly text: Messages['dictionary']; readonly title: string;
}) {
  const [result, setResult] = useState<DictionaryLookup | null>();
  const [error, setError] = useState<DictionaryFailure>();
  useEffect(() => {
    let cancelled = false;
    setResult(undefined); setError(undefined);
    void runtime.lookupDictionary(headword).then((value) => { if (!cancelled) setResult(value); })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause)); });
    return () => { cancelled = true; };
  }, [runtime, headword]);
  return <section className="dictionary-original" aria-label={title}><h3>{title}</h3>
    {error && <p role="alert">{text[error.code]} {error.message}</p>}
    {result === undefined && !error && <p role="status">{text.loading}</p>}
    {result === null && <p role="status">{text.preview}</p>}
    {result && <DictionaryResults result={result} text={text} />}
  </section>;
}

export function DictionaryPanel({ runtime, text, notebookText, version, onCollect, onOpen, beforeSearch }: {
  readonly runtime: RuntimeAdapter; readonly text: Messages['dictionary'];
  readonly notebookText: NotebookText; readonly version: number;
  readonly onCollect: (result: DictionaryLookup) => void; readonly onOpen: (record: VocabularyRecord) => void;
  readonly beforeSearch: (action: () => void) => void;
}) {
  const [key, setKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DictionaryLookup | null | undefined>();
  const [error, setError] = useState<DictionaryFailure>();
  async function search(event: FormEvent) {
    event.preventDefault();
    setLoading(true); setResult(undefined); setError(undefined);
    try { setResult(await runtime.lookupDictionary(key)); }
    catch (cause) { setError(dictionaryFailure(cause)); }
    finally { setLoading(false); }
  }
  return <div className="dictionary">
    <form onSubmit={(event) => { event.preventDefault(); beforeSearch(() => { void search(event); }); }}>
      <label htmlFor="dictionary-key">{text.label}</label>
      <div className="dictionary-search"><input id="dictionary-key" lang="fr" value={key} required maxLength={512}
        autoCapitalize="none" autoCorrect="off" spellCheck={false} disabled={loading}
        onChange={(event) => setKey(event.target.value)} />
      <button type="submit" disabled={loading}>{loading ? text.loading : text.search}</button></div>
    </form>
    <p className="dictionary-hint">{text.hint}</p>
    {loading && <p role="status">{text.loading}</p>}
    {result === null && <p role="status">{text.preview}</p>}
    {error && <div className="runtime-error" role="alert"><p>{text[error.code]}</p><details>
      <summary>{text.details}</summary><p>{error.message}</p></details></div>}
    {result && <><Collection result={result} runtime={runtime} text={notebookText} version={version}
      onCollect={onCollect} onOpen={onOpen} />
      <section className="dictionary-original" aria-label={notebookText.original}>
        <h3>{notebookText.original}</h3><DictionaryResults result={result} text={text} />
      </section></>}
  </div>;
}
