import { useState } from 'react';
import type { FormEvent } from 'react';
import type { Messages } from './content';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure, DictionaryLookup, DictionarySense } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';

function Senses({ senses }: { readonly senses: readonly DictionarySense[] }) {
  return <ol className="dictionary-senses">{senses.map((sense, index) => <li key={index}>
    {sense.translations.map((translation, item) => <p key={`t${item}`} lang="zh">{translation}</p>)}
    {sense.definitions.map((definition, item) => <p key={`d${item}`} lang="fr">{definition}</p>)}
    {sense.senses.length > 0 && <Senses senses={sense.senses} />}
  </li>)}</ol>;
}

function Results({ result, text }: { readonly result: DictionaryLookup; readonly text: Messages['dictionary'] }) {
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

export function DictionaryPanel({ runtime, text }: {
  readonly runtime: RuntimeAdapter; readonly text: Messages['dictionary'];
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
    <form onSubmit={(event) => { void search(event); }}>
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
    {result && <Results result={result} text={text} />}
  </div>;
}
