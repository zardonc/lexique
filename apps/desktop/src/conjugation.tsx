import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure } from './platform/dictionary';
import { matchingCells, defaultConjugationOrder } from './platform/conjugation';
import type { ConjugationLookup, ConjugationTable, ConjugationMode, ConjugationOrder } from './platform/conjugation';
import { ConjugationBlocks } from './conjugation-table';
import { ColoringControls, ColoringNotice } from './conjugation-coloring';
import { OrderControls } from './conjugation-order';
import { conjugationMessages, conjugationUse } from './conjugation-content';
import type { ConjugationText } from './conjugation-content';
import { ConjugationDictionary } from './conjugation-navigation';
import type { ConjugationView, DictionaryView } from './conjugation-navigation';

function Meanings({ table, text }: { readonly table: ConjugationTable; readonly text: ConjugationText }) {
  return <span className="conjugation-meaning">{table.meanings.status === 'unavailable' ? text.meaning
    : table.meanings.senses.map((sense, index) => <span key={`${sense.source_entry}/${sense.source_sense}`}>
      {index > 0 && '; '}{sense.use === 'pronominal' && <small lang="fr">{sense.headword} · </small>}
      {sense.labels.length > 0 && <small>{sense.labels.join(', ')} · </small>}
      <span lang="en">{sense.glosses.join(', ')}</span>
    </span>)}</span>;
}

function VerbMetadata({ table, text }: { readonly table: ConjugationTable; readonly text: ConjugationText }) {
  const verb = table.verb;
  const group = verb.group === 3 ? `3 · ${text.irregular}` : `${verb.group} · -${verb.group === 1 ? 'er' : 'ir'}`;
  return <><p>{text.group}: {verb.group_status === 'supported' ? group : text.unconfirmed}</p>
    <p>{text.usage}: {verb.usage_labels.map((use) => conjugationUse(text, use)).join(' · ') || text.unconfirmed}</p>
    <p>{text.auxiliary}: {verb.auxiliaries.map((auxiliary) => `${auxiliary.lemma} (${conjugationUse(text, auxiliary.use)})`).join(' · ')}
      {verb.auxiliary_status !== 'supported' && ` · ${verb.auxiliary_status === 'not_applicable' ? text.notApplicable : text.unconfirmed}`}</p>
    <p><Meanings table={table} text={text} /></p>
    {verb.pronominal && <p>{text.pronominalAgreement[verb.pronominal.agreement]}</p>}
    {verb.ordinary_status === 'not_applicable' && !verb.pronominal && <p>{text.rawOnly}</p>}</>;
}

function Results({ result, selected, onSelect, text, mode, order, runtime, locale, dictionaryKey, onDictionary }: {
  readonly result: ConjugationLookup; readonly selected: string; readonly onSelect: (lemma: string) => void; readonly text: ConjugationText;
  readonly mode: ConjugationMode; readonly order: ConjugationOrder;
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly dictionaryKey?: string;
  readonly onDictionary: (view: DictionaryView) => void;
}) {
  const tables = [...result.tables].sort((a, b) => a.lemma.localeCompare(b.lemma, 'fr') || (a.lemma < b.lemma ? -1 : 1));
  const table = tables.find((candidate) => candidate.lemma === selected);
  const count = tables.reduce((sum, candidate) => sum + matchingCells(candidate, result.query), 0);
  return <><p role="status">{tables.length ? `${tables.length} ${text.verbs} · ${count} ${text.matches}` : text.noMatch}</p>
    {tables.length > 1 && <div className="conjugation-choices" role="group" aria-label={text.choices}>{tables.map((candidate) =>
      <button key={candidate.lemma} type="button" aria-pressed={candidate.lemma === selected} onClick={() => onSelect(candidate.lemma)}>
        <strong lang="fr">{candidate.lemma}</strong> · {candidate.verb.usage_labels.map((use) => conjugationUse(text, use)).join(' · ') || text.unconfirmed}
        {` · ${text.matches}: ${matchingCells(candidate, result.query)}`}
        <br /><Meanings table={candidate} text={text} />
      </button>)}</div>}
    {tables.length > 1 && !table && <p>{text.choose}</p>}
    {table && <article className="conjugation-table"><h3 lang="fr" tabIndex={-1}>{table.lemma}</h3><VerbMetadata table={table} text={text} />
      <ConjugationDictionary runtime={runtime} locale={locale} lemma={table.lemma} dictionaryKey={dictionaryKey} onOpen={onDictionary} />
      <ColoringNotice table={table} mode={mode} text={text} />
      <ConjugationBlocks key={table.lemma} table={table} query={result.query} text={text} mode={mode} order={order} runtime={runtime} locale={locale} />
      <details><summary>{text.source}</summary><pre>{JSON.stringify(table, null, 2)}</pre></details></article>}
    <footer className="dictionary-source"><p>{text.sourceOnly}</p>
      <p>Lefff © Benoît Sagot · LGPL-LR · <a href="https://pauillac.inria.fr/~sagot/index.html#lefff">Lefff 3.4</a></p>
      <p>Grammalecte © Olivier R. · GPL-3.0 · <a href="https://grammalecte.net/">Grammalecte 2.3.0</a></p>
      <p>FreeDict © Horst Eyermann, Michael Bunk, John Darrington, FreeDict · GPL-2.0-or-later · <a href="https://freedict.org/">fra-eng 0.4.1</a></p></footer></>;
}

export function ConjugationPanel({ runtime, locale, initial, onChange, onDictionary }: {
  readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly initial: ConjugationView;
  readonly onChange: (view: ConjugationView) => void; readonly onDictionary: (view: DictionaryView) => void;
}) {
  const text = conjugationMessages[locale];
  const [key, setKey] = useState(initial.key);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ConjugationLookup | null | undefined>(initial.result);
  const [selected, setSelected] = useState(initial.selected);
  const [error, setError] = useState<DictionaryFailure>();
  const [mode, setMode] = useState<ConjugationMode>('ending');
  const [order, setOrder] = useState<ConjugationOrder>(defaultConjugationOrder);
  const panel = useRef<HTMLDivElement>(null);
  useEffect(() => { onChange({ key, result, selected, dictionaryKey: initial.dictionaryKey }); }, [key, result, selected, initial.dictionaryKey, onChange]);
  // Restore focus on entry while leaving preference controls in charge afterward.
  useEffect(() => { if (initial.selected) panel.current?.querySelector<HTMLHeadingElement>('.conjugation-table h3')?.focus(); }, []);
  async function search(event: FormEvent) {
    event.preventDefault(); setLoading(true); setResult(undefined); setError(undefined); setSelected('');
    try {
      const value = await runtime.lookupConjugation(key); setResult(value);
      if (value?.tables.length === 1) setSelected(value.tables[0]?.lemma ?? '');
    } catch (cause) { setError(dictionaryFailure(cause)); }
    finally { setLoading(false); }
  }
  return <div className="conjugation" ref={panel}><form onSubmit={(event) => { void search(event); }}>
    <label htmlFor="conjugation-key">{text.label}</label><div className="dictionary-search">
      <input id="conjugation-key" lang="fr" value={key} maxLength={512} autoCapitalize="none" autoCorrect="off" spellCheck={false}
        disabled={loading} onChange={(event) => setKey(event.target.value)} />
      <button type="submit" disabled={loading}>{loading ? text.loading : text.search}</button></div></form>
    <p className="dictionary-hint">{text.hint}</p>
    <ColoringControls runtime={runtime} text={text} mode={mode} onChange={setMode} />
    <OrderControls runtime={runtime} text={text} order={order} onChange={setOrder} />
    {loading && <p role="status">{text.loading}</p>}
    {result === null && <p role="status">{text.preview}</p>}
    {error && <div role="alert"><p>{text[error.code]}</p><details><summary>{text.source}</summary><p>{error.message}</p></details></div>}
    {result && <Results result={result} selected={selected} onSelect={setSelected} text={text} mode={mode} order={order}
      runtime={runtime} locale={locale} dictionaryKey={initial.dictionaryKey} onDictionary={onDictionary} />}
  </div>;
}
