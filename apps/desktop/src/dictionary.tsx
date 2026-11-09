import { useEffect, useRef, useState } from 'react';
import type { KeyboardEvent, ReactNode } from 'react';
import type { Locale, Messages } from './content';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure, DictionaryLookup, DictionarySense, DictionaryCatalog } from './platform/dictionary';
import type { RuntimeAdapter } from './platform/runtime';
import type { NotebookText } from './notebook-content';
import { errorMessage, snapshotFromLookup } from './platform/vocabulary';
import type { VocabularyRecord, VocabularySnapshot } from './platform/vocabulary';
import { VocabularySummary } from './vocabulary-view';
import { SpeechButton } from './speech-button';
import { DictionaryConjugation, DictionaryLemmas } from './conjugation-navigation';
import type { ConjugationView, DictionaryView } from './conjugation-navigation';
import { DictionaryHtml } from './dictionary-html';
import { useDictionaryTabs } from './use-dictionary-tabs';
import { expressionMatches } from './platform/conjugation';

function Senses({ senses }: { readonly senses: readonly DictionarySense[] }) {
  return <ol className="dictionary-senses">{senses.map((sense, index) => <li key={index}>
    {sense.translations.map((translation, item) => <p key={`t${item}`} lang="zh">{translation}</p>)}
    {sense.definitions.map((definition, item) => <p key={`d${item}`} lang="fr">{definition}</p>)}
    {sense.senses.length > 0 && <Senses senses={sense.senses} />}
  </li>)}</ol>;
}

function linkedVerb(entry: DictionaryLookup['entries'][number], result: DictionaryLookup, previous?: ConjugationView) {
  if (entry.grammar.includes('v')) return true;
  return entry.html !== undefined && (previous?.dictionaryId ?? 'builtin') === (result.dictionary_id ?? 'builtin')
    && (previous?.result?.tables.some((table) => table.lemma === previous.selected && expressionMatches(table.lemma, entry.key)) ?? false);
}

export function DictionaryResults({ result, text, navigation }: {
  readonly result: DictionaryLookup; readonly text: Messages['dictionary'];
  readonly navigation?: { readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly previous?: ConjugationView; readonly onOpen: (view: ConjugationView) => void };
}) {
  return <>
    <p role="status">{result.entries.length === 0 ? text.noMatch : `${text.records}: ${result.entries.length}`}</p>
    {result.entries.map((entry) => <article className="dictionary-entry" key={entry.ordinal} data-ordinal={entry.ordinal}>
      <h3 lang="fr" tabIndex={-1}>{entry.key}</h3>
      <p className="dictionary-grammar">{entry.pronunciations.join(' · ')} {entry.grammar.join(' · ')}</p>
      {entry.html === undefined ? <Senses senses={entry.senses} /> : <DictionaryHtml html={entry.html} title={result.source.title} />}
      {navigation && linkedVerb(entry, result, navigation.previous) && <DictionaryConjugation runtime={navigation.runtime} locale={navigation.locale}
        dictionaryKey={entry.key} dictionaryId={result.dictionary_id} previous={navigation.previous} posUnknown={!entry.grammar.includes('v')} onOpen={navigation.onOpen} />}
    </article>)}
    <footer className="dictionary-source">
      <p>{text.source}: {result.source.title} · {result.source.version}</p>
      <p>{result.source.attribution ?? text.unknown} · {result.source.license ?? text.unknown}</p>
      <p>{result.source.source_url ?? text.unknown}</p><p>{result.source.license_url ?? text.unknown}</p>
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
    <SpeechButton key={headword} word={headword} runtime={runtime} text={text} />
    {error ? <p role="alert">{text.readError}: {error} <button onClick={() => setRetry((value) => value + 1)}>{text.retry}</button></p>
      : record ? <p role="status">{text.existing} <button onClick={() => onOpen(record)}>{text.view}</button></p>
        : <button disabled={record === undefined} onClick={() => onCollect(result)}>{text.collect}</button>}</>;
}

export function RecordOriginal({ reference, runtime, text, title }: {
  readonly reference: VocabularySnapshot['reference']; readonly runtime: RuntimeAdapter; readonly text: Messages['dictionary']; readonly title: string;
}) {
  const [result, setResult] = useState<DictionaryLookup | null>();
  const [error, setError] = useState<DictionaryFailure>();
  useEffect(() => {
    let cancelled = false;
    setResult(undefined); setError(undefined);
    void runtime.lookupDictionary(reference.lookup_key, reference.dictionary_id ?? 'builtin').then((value) => { if (!cancelled) setResult(value); })
      .catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause)); });
    return () => { cancelled = true; };
  }, [runtime, reference.lookup_key, reference.dictionary_id]);
  return <section className="dictionary-original" aria-label={title}><h3>{title}</h3>
    {error && <p role="alert">{text[error.code]} {error.message}</p>}
    {result === undefined && !error && <p role="status">{text.loading}</p>}
    {result === null && <p role="status">{text.preview}</p>}
    {result && <>{result.sha256 !== reference.sha256 && <p role="status">{text.changedSource}</p>}<DictionaryResults result={result} text={text} /></>}
  </section>;
}

function tabKey(event: KeyboardEvent<HTMLDivElement>, catalog: DictionaryCatalog, onSelect: (id: string) => void) {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  let index = catalog.findIndex((item) => `dictionary-tab-${item.id}` === (event.target as HTMLElement).id);
  if (index < 0) return;
  switch (event.key) {
    case 'ArrowRight': index = (index + 1) % catalog.length; break;
    case 'ArrowLeft': index = (index + catalog.length - 1) % catalog.length; break;
    case 'Home': index = 0; break;
    case 'End': index = catalog.length - 1; break;
    default: return;
  }
  event.preventDefault();
  const id = catalog[index].id;
  document.getElementById(`dictionary-tab-${id}`)?.focus(); onSelect(id);
}

function DictionaryTabStatus({ tabs, text, onSelect, children }: {
  readonly tabs: ReturnType<typeof useDictionaryTabs>; readonly text: Messages['dictionary']; readonly onSelect: (id: string) => void;
  readonly children: ReactNode;
}) {
  return <>
    {tabs.catalogError && <p role="alert">{text[tabs.catalogError.code]} {tabs.catalogError.message} <button onClick={tabs.reload}>{text.retry}</button></p>}
    {tabs.query && <div role="tablist" aria-label={text.tabs} onKeyDown={(event) => tabKey(event, tabs.catalog, onSelect)}>{tabs.catalog.map((item) => <button type="button" role="tab"
      id={`dictionary-tab-${item.id}`} aria-controls="dictionary-body" aria-selected={tabs.selected === item.id} key={item.id}
      tabIndex={tabs.selected === item.id ? 0 : -1} data-error={!!tabs.visits[item.id]?.error} onClick={() => onSelect(item.id)}>{item.title}</button>)}</div>}
    {tabs.query && <div id="dictionary-body" role="tabpanel" tabIndex={0} aria-labelledby={`dictionary-tab-${tabs.selected}`}>
      {Object.values(tabs.visits).some((visit) => visit.error) && <p role="status">{text.failedSources}</p>}
    {tabs.loading && <><p role="status">{text.loading}</p><button type="button" onClick={tabs.cancel}>{text.cancel}</button></>}
    {tabs.result === null && <p role="status">{text.preview}</p>}
    {tabs.error && <div className="runtime-error" role="alert"><p>{text[tabs.error.code]}</p><details>
      <summary>{text.details}</summary><p>{tabs.error.message}</p></details><button onClick={() => { void tabs.select(tabs.selected, true); }}>{text.retry}</button></div>}
      {children}</div>}
  </>;
}

export function DictionaryPanel({ runtime, text, notebookText, version, onCollect, onOpen, beforeSearch, locale, initial, onConjugation }: {
  readonly runtime: RuntimeAdapter; readonly text: Messages['dictionary'];
  readonly notebookText: NotebookText; readonly version: number;
  readonly onCollect: (result: DictionaryLookup) => void; readonly onOpen: (record: VocabularyRecord) => void;
  readonly beforeSearch: (action: () => void) => void;
  readonly locale: Locale; readonly initial?: DictionaryView; readonly onConjugation: (view: ConjugationView) => void;
}) {
  const tabs = useDictionaryTabs(runtime, initial);
  const [key, setKey] = useState(tabs.query);
  const { result, loading } = tabs;
  const [previous, setPrevious] = useState(() => initial?.conjugation);
  const panel = useRef<HTMLDivElement>(null);
  useEffect(() => { if (initial) panel.current?.querySelector<HTMLHeadingElement>(`[data-ordinal="${initial.ordinal}"] h3`)?.focus(); }, [initial]);
  return <div className="dictionary" ref={panel}>
    <form onSubmit={(event) => { event.preventDefault(); beforeSearch(() => { setPrevious(undefined); void tabs.search(key); }); }}>
      <label htmlFor="dictionary-key">{text.label}</label>
      <div className="dictionary-search"><input id="dictionary-key" lang="fr" value={key} required maxLength={512}
        autoCapitalize="none" autoCorrect="off" spellCheck={false} disabled={loading}
        onChange={(event) => setKey(event.target.value)} />
      <button type="submit" disabled={loading || tabs.catalogLoading || !!tabs.catalogError}>{loading ? text.loading : text.search}</button></div>
    </form>
    <p className="dictionary-hint">{text.hint}</p>
    <DictionaryTabStatus tabs={tabs} text={text} onSelect={(id) => beforeSearch(() => { void tabs.select(id); })}>
    {result && <>
      <DictionaryLemmas runtime={runtime} locale={locale} query={tabs.query} dictionaryId={tabs.selected} sourceVersion={result.sha256}
        onChoose={(headword, view) => beforeSearch(() => { setKey(headword); setPrevious(view); void tabs.search(headword, tabs.selected); })} />
      <Collection key={`${tabs.query}:${tabs.selected}`} result={result} runtime={runtime} text={notebookText} version={version}
      onCollect={onCollect} onOpen={onOpen} />
      <section className="dictionary-original" aria-label={notebookText.original}>
        <h3>{notebookText.original}</h3><DictionaryResults result={result} text={text}
          navigation={{ runtime, locale, previous, onOpen: onConjugation }} />
      </section></>}
    </DictionaryTabStatus>
  </div>;
}
