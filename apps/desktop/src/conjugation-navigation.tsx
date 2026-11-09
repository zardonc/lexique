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
  readonly dictionaryId?: string;
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
  function open(next: number, update?: () => void) { beforeLeave(() => { setDictionary(undefined); update?.(); setPage(next); }); }
  return { page, conjugation, dictionary, setConjugation,
    changePage: (next: number) => open(next),
    beforeSearch: (action: () => void) => open(0, action),
    openConjugation: (view: ConjugationView) => open(1, () => setConjugation(view)),
    openDictionary: (view: DictionaryView) => open(0, () => {
      const context = { ...conjugation, dictionaryId: view.result.dictionary_id ?? 'builtin', dictionaryKey: view.result.entries.find((entry) => entry.ordinal === view.ordinal)?.key };
      setConjugation(context); setDictionary({ ...view, conjugation: context });
    }),
    openDictionaryResult: (view: DictionaryView) => open(0, () => setDictionary(view)),
    openInput: (action: () => void) => open(5, action),
  };
}

function correspondingConjugation(previous: ConjugationView | undefined, dictionaryKey: string, dictionaryId?: string) {
  return previous?.result && expressionMatches(previous.selected, dictionaryKey)
    && (previous.dictionaryId ?? 'builtin') === (dictionaryId ?? 'builtin') ? previous : undefined;
}

export function DictionaryConjugation({ runtime, locale, dictionaryKey, dictionaryId, previous, posUnknown, onOpen }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly dictionaryKey: string;
  readonly previous?: ConjugationView;
  readonly dictionaryId?: string;
  readonly posUnknown?: boolean;
  readonly onOpen: (view: ConjugationView) => void;
}) {
  const text = conjugationMessages[locale];
  const context = correspondingConjugation(previous, dictionaryKey, dictionaryId);
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
    {posUnknown && <p>{text.dictionaryPosUnknown}</p>}
    {candidates.length === 0 && <p>{text.conjugationUnavailable}</p>}
    {candidates.length > 1 && <p>{text.chooseConjugation}</p>}
    {candidates.map((table) => <button key={table.lemma} type="button" onClick={() => onOpen({
      key: context?.key ?? result.query, result, selected: table.lemma, dictionaryKey, dictionaryId,
    })}>{text.openConjugation}: <span lang="fr">{table.lemma}</span></button>)}
  </div>;
}

function dictionaryKeys(lemma: string, preferred?: string) {
  return preferred && expressionMatches(preferred, lemma) ? [preferred]
    : [...new Set([lemma, lemma.replaceAll('’', "'"), lemma.replaceAll("'", '’')])];
}

function dictionaryEntries(result: DictionaryLookup, lemma: string) {
  return result.entries.filter((entry) => expressionMatches(entry.key, lemma)
    && (entry.grammar.includes('v') || (entry.html !== undefined && entry.grammar.length === 0)));
}

export function ConjugationDictionary({ runtime, locale, lemma, dictionaryKey, dictionaryId, onOpen }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly lemma: string; readonly dictionaryKey?: string;
  readonly onOpen: (view: DictionaryView) => void;
  readonly dictionaryId?: string;
}) {
  const text = conjugationMessages[locale], dictionaryText = messages[locale].dictionary;
  const [results, setResults] = useState<readonly DictionaryLookup[] | null>();
  const [error, setError] = useState<DictionaryFailure>();
  useEffect(() => {
    let cancelled = false;
    setResults(undefined); setError(undefined);
    const keys = dictionaryKeys(lemma, dictionaryKey);
    void Promise.all(keys.map((key) => runtime.lookupDictionary(key, dictionaryId)))
      .then((values) => { if (!cancelled) setResults(values.includes(null) ? null : values.filter((value) => value !== null)); })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause)); });
    return () => { cancelled = true; };
  }, [runtime, lemma, dictionaryKey, dictionaryId]);
  if (error) return <p role="alert">{dictionaryText[error.code]} {error.message}</p>;
  if (results === undefined) return <p>{text.loading}</p>;
  if (results === null) return <p>{text.linksPreview}</p>;
  const candidates = [...new Map(results.flatMap((result) => dictionaryEntries(result, lemma)
    .map((entry) => [JSON.stringify([result.dictionary_id, entry.key, entry.ordinal]), { result, entry }] as const))).values()];
  return <div className="dictionary-link">
    {candidates.length === 0 && <p>{text.dictionaryUnavailable}</p>}
    {candidates.length > 1 && <p>{text.chooseDictionary}</p>}
    {candidates.some(({ entry }) => !entry.grammar.includes('v')) && <p>{text.dictionaryPosUnknown}</p>}
    {candidates.map(({ result, entry }) => <button key={JSON.stringify([result.dictionary_id, entry.key, entry.ordinal])} type="button" onClick={() => onOpen({ result, ordinal: entry.ordinal })}>
      {text.openDictionary}: <span lang="fr">{entry.key}</span>
      {candidates.length > 1 && ` · ${entry.senses.flatMap((sense) => sense.translations).join(' / ')}`}
    </button>)}
  </div>;
}

interface LemmaLink { readonly lemma: string; readonly key: string; readonly unknownPos: boolean }
interface LemmaTargets { readonly links: readonly LemmaLink[]; readonly errors: readonly DictionaryFailure[] }
interface LemmaState extends LemmaTargets { readonly conjugation: ConjugationLookup | null; readonly hasMatches: boolean }

async function lemmaTargets(runtime: RuntimeAdapter, lemma: string, id: string): Promise<LemmaTargets> {
  const targets = await Promise.all(dictionaryKeys(lemma).map(async (key): Promise<LemmaTargets> => {
    try {
      const result = await runtime.lookupDictionary(key, id);
      return { links: result ? dictionaryEntries(result, lemma).map((entry) => ({ lemma, key: entry.key, unknownPos: !entry.grammar.includes('v') })) : [], errors: [] };
    } catch (cause) { const error = dictionaryFailure(cause); return { links: [], errors: [{ ...error, message: `${key}: ${error.message}` }] }; }
  }));
  return { links: [...new Map(targets.flatMap((target) => target.links).map((link) => [link.key, link])).values()], errors: targets.flatMap((target) => target.errors) };
}

async function loadLemmas(runtime: RuntimeAdapter, conjugation: ConjugationLookup | null, id: string): Promise<LemmaState> {
  const matches = conjugation?.tables.filter((table) => !expressionMatches(table.lemma, conjugation.query)) ?? [];
  const targets = await Promise.all(matches.map((table) => lemmaTargets(runtime, table.lemma, id)));
  return { conjugation, hasMatches: matches.length > 0, links: targets.flatMap((target) => target.links).sort((a, b) => a.key.localeCompare(b.key, 'fr')),
    errors: targets.flatMap((target) => target.errors) };
}

export function DictionaryLemmas({ runtime, locale, query, dictionaryId, sourceVersion, onChoose }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly query: string; readonly dictionaryId: string; readonly sourceVersion: string;
  readonly onChoose: (key: string, view: ConjugationView) => void;
}) {
  const text = conjugationMessages[locale], dictionaryText = messages[locale].dictionary;
  const [state, setState] = useState<LemmaState>(), [retry, setRetry] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setState(undefined);
    void runtime.lookupConjugation(query).then(async (conjugation) => {
      if (cancelled) return;
      const value = await loadLemmas(runtime, conjugation, dictionaryId);
      if (!cancelled) setState(value);
    })
      .catch((cause: unknown) => { if (!cancelled) setState({ conjugation: null, hasMatches: false, links: [], errors: [dictionaryFailure(cause)] }); });
    return () => { cancelled = true; };
  }, [runtime, query, dictionaryId, sourceVersion, retry]);
  if (!state) return <p role="status">{text.lemmasLoading}</p>;
  if (!state.hasMatches && !state.errors.length) return null;
  return <section className="dictionary-lemmas" aria-label={text.lemmas}><h3>{text.lemmas}</h3>
    {state.errors.length > 0 && <div role="alert">{state.errors.map((error, index) => <p key={index}>{dictionaryText[error.code]} {error.message}</p>)}
      <button onClick={() => setRetry((value) => value + 1)}>{dictionaryText.retry}</button></div>}
    {state.hasMatches && !state.links.length && <p>{text.lemmaUnavailable}</p>}
    {state.links.length > 0 && <p>{text.chooseLemma}</p>}
    {state.links.some((link) => link.unknownPos) && <p>{text.dictionaryPosUnknown}</p>}
    {state.links.map((link) => <button type="button" key={`${link.lemma}:${link.key}`} onClick={() => onChoose(link.key, {
      key: query, result: state.conjugation, selected: link.lemma, dictionaryKey: link.key, dictionaryId,
    })}>{text.lookupLemma}<span lang="fr">{link.key}</span></button>)}
  </section>;
}
