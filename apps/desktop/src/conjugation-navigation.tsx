import { useEffect, useState } from 'react';
import type { Locale } from './content';
import { messages } from './content';
import { conjugationMessages } from './conjugation-content';
import { expressionMatches } from './platform/conjugation';
import type { ConjugationLookup } from './platform/conjugation';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure, DictionaryLookup } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';

export interface ConjugationView {
  readonly key: string;
  readonly selected: string;
  readonly result?: ConjugationLookup | null;
  readonly dictionaryKey?: string;
}
export interface DictionaryView {
  readonly result: DictionaryLookup;
  readonly ordinal: number;
  readonly conjugation?: ConjugationView;
}

export function useLookupNavigation(beforeLeave: (action: () => void) => void) {
  const [page, setPage] = useState(0);
  const [conjugation, setConjugation] = useState<ConjugationView>({ key: '', selected: '' });
  const [dictionary, setDictionary] = useState<DictionaryView>();
  function open(next: number, update: () => void) { beforeLeave(() => { update(); setPage(next); }); }
  return { page, conjugation, dictionary, setConjugation,
    changePage: (next: number) => open(next, () => setDictionary(undefined)),
    openConjugation: (view: ConjugationView) => open(1, () => setConjugation(view)),
    openDictionary: (view: DictionaryView) => open(0, () => setDictionary({ ...view, conjugation })),
  };
}

function correspondingConjugation(previous: ConjugationView | undefined, dictionaryKey: string) {
  return previous?.result && expressionMatches(previous.selected, dictionaryKey) ? previous : undefined;
}

export function DictionaryConjugation({ runtime, locale, dictionaryKey, previous, onOpen }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly dictionaryKey: string;
  readonly previous?: ConjugationView;
  readonly onOpen: (view: ConjugationView) => void;
}) {
  const text = conjugationMessages[locale];
  const context = correspondingConjugation(previous, dictionaryKey);
  const query = context?.result?.query ?? dictionaryKey;
  const [result, setResult] = useState<ConjugationLookup | null>();
  const [error, setError] = useState<DictionaryFailure>();
  useEffect(() => {
    let cancelled = false;
    setResult(undefined); setError(undefined);
    void runtime.lookupConjugation(query).then((value) => { if (!cancelled) setResult(value); })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause)); });
    return () => { cancelled = true; };
  }, [runtime, query]);
  if (error) return <p role="alert">{text[error.code]} {error.message}</p>;
  if (result === undefined) return <p>{text.loading}</p>;
  if (result === null) return <p>{text.linksPreview}</p>;
  const candidates = result.tables.filter((table) => expressionMatches(table.lemma, dictionaryKey));
  return <div className="conjugation-link">
    {candidates.length === 0 && <p>{text.conjugationUnavailable}</p>}
    {candidates.length > 1 && <p>{text.chooseConjugation}</p>}
    {candidates.map((table) => <button key={table.lemma} type="button" onClick={() => onOpen({
      key: context?.key ?? result.query, result, selected: table.lemma, dictionaryKey,
    })}>{text.openConjugation}: <span lang="fr">{table.lemma}</span></button>)}
  </div>;
}

export function ConjugationDictionary({ runtime, locale, lemma, dictionaryKey, onOpen }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly lemma: string; readonly dictionaryKey?: string;
  readonly onOpen: (view: DictionaryView) => void;
}) {
  const text = conjugationMessages[locale], dictionaryText = messages[locale].dictionary;
  const [results, setResults] = useState<readonly DictionaryLookup[] | null>();
  const [error, setError] = useState<DictionaryFailure>();
  useEffect(() => {
    let cancelled = false;
    setResults(undefined); setError(undefined);
    const keys = dictionaryKey && expressionMatches(dictionaryKey, lemma) ? [dictionaryKey]
      : [...new Set([lemma, lemma.replaceAll('’', "'"), lemma.replaceAll("'", '’')])];
    void Promise.all(keys.map((key) => runtime.lookupDictionary(key)))
      .then((values) => { if (!cancelled) setResults(values.includes(null) ? null : values.filter((value) => value !== null)); })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause)); });
    return () => { cancelled = true; };
  }, [runtime, lemma, dictionaryKey]);
  if (error) return <p role="alert">{dictionaryText[error.code]} {error.message}</p>;
  if (results === undefined) return <p>{text.loading}</p>;
  if (results === null) return <p>{text.linksPreview}</p>;
  const candidates = results.flatMap((result) => result.entries
    .filter((entry) => entry.grammar.includes('v') && expressionMatches(entry.key, lemma)).map((entry) => ({ result, entry })));
  return <div className="dictionary-link">
    {candidates.length === 0 && <p>{text.dictionaryUnavailable}</p>}
    {candidates.length > 1 && <p>{text.chooseDictionary}</p>}
    {candidates.map(({ result, entry }) => <button key={entry.ordinal} type="button" onClick={() => onOpen({ result, ordinal: entry.ordinal })}>
      {text.openDictionary}: <span lang="fr">{entry.key}</span>
      {candidates.length > 1 && ` · ${entry.senses.flatMap((sense) => sense.translations).join(' / ')}`}
    </button>)}
  </div>;
}
