import { useState } from 'react';
import type { FormEvent } from 'react';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import { dictionaryFailure } from './platform/dictionary';
import type { DictionaryFailure } from './platform/dictionary';
import { matchingCells } from './platform/conjugation';
import type { ConjugationLookup, ConjugationTable } from './platform/conjugation';
import { ConjugationBlocks } from './conjugation-table';
import { conjugationMessages, conjugationUse } from './conjugation-content';
import type { ConjugationText } from './conjugation-content';

function VerbMetadata({ table, text }: { readonly table: ConjugationTable; readonly text: ConjugationText }) {
  const verb = table.verb;
  const group = verb.group === 3 ? `3 · ${text.irregular}` : `${verb.group} · -${verb.group === 1 ? 'er' : 'ir'}`;
  return <><p>{text.group}: {verb.group_status === 'supported' ? group : text.unconfirmed}</p>
    <p>{text.usage}: {verb.usage_labels.map((use) => conjugationUse(text, use)).join(' · ') || text.unconfirmed}</p>
    <p>{text.auxiliary}: {verb.auxiliaries.map((auxiliary) => `${auxiliary.lemma} (${conjugationUse(text, auxiliary.use)})`).join(' · ')}
      {verb.auxiliary_status !== 'supported' && ` · ${verb.auxiliary_status === 'not_applicable' ? text.notApplicable : text.unconfirmed}`}</p>
    <p>{text.meaning}</p>{verb.ordinary_status === 'not_applicable' && <p>{text.rawOnly}</p>}</>;
}

function Results({ result, selected, onSelect, text }: {
  readonly result: ConjugationLookup; readonly selected: string; readonly onSelect: (lemma: string) => void; readonly text: ConjugationText;
}) {
  const tables = [...result.tables].sort((a, b) => a.lemma.localeCompare(b.lemma, 'fr') || (a.lemma < b.lemma ? -1 : 1));
  const table = tables.find((candidate) => candidate.lemma === selected);
  const count = tables.reduce((sum, candidate) => sum + matchingCells(candidate, result.query), 0);
  return <><p role="status">{tables.length ? `${tables.length} ${text.verbs} · ${count} ${text.matches}` : text.noMatch}</p>
    {tables.length > 1 && <div className="conjugation-choices" role="group" aria-label={text.choices}>{tables.map((candidate) =>
      <button key={candidate.lemma} type="button" aria-pressed={candidate.lemma === selected} onClick={() => onSelect(candidate.lemma)}>
        <strong lang="fr">{candidate.lemma}</strong> · {candidate.verb.usage_labels.map((use) => conjugationUse(text, use)).join(' · ') || text.unconfirmed}
        {` · ${text.matches}: ${matchingCells(candidate, result.query)}`}
      </button>)}</div>}
    {tables.length > 1 && !table && <p>{text.choose}</p>}
    {tables.length > 1 && !table && <p>{text.meaning}</p>}
    {table && <article className="conjugation-table"><h3 lang="fr">{table.lemma}</h3><VerbMetadata table={table} text={text} />
      <ConjugationBlocks table={table} query={result.query} text={text} />
      <details><summary>{text.source}</summary><pre>{JSON.stringify(table, null, 2)}</pre></details></article>}
    <footer className="dictionary-source"><p>{text.sourceOnly}</p>
      <p>Lefff © Benoît Sagot · LGPL-LR · <a href="https://pauillac.inria.fr/~sagot/index.html#lefff">Lefff 3.4</a></p>
      <p>Grammalecte © Olivier R. · GPL-3.0 · <a href="https://grammalecte.net/">Grammalecte 2.3.0</a></p></footer></>;
}

export function ConjugationPanel({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const text = conjugationMessages[locale];
  const [key, setKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ConjugationLookup | null>();
  const [selected, setSelected] = useState('');
  const [error, setError] = useState<DictionaryFailure>();
  async function search(event: FormEvent) {
    event.preventDefault(); setLoading(true); setResult(undefined); setError(undefined); setSelected('');
    try {
      const value = await runtime.lookupConjugation(key); setResult(value);
      if (value?.tables.length === 1) setSelected(value.tables[0]?.lemma ?? '');
    } catch (cause) { setError(dictionaryFailure(cause)); }
    finally { setLoading(false); }
  }
  return <div className="conjugation"><form onSubmit={(event) => { void search(event); }}>
    <label htmlFor="conjugation-key">{text.label}</label><div className="dictionary-search">
      <input id="conjugation-key" lang="fr" value={key} maxLength={512} autoCapitalize="none" autoCorrect="off" spellCheck={false}
        disabled={loading} onChange={(event) => setKey(event.target.value)} />
      <button type="submit" disabled={loading}>{loading ? text.loading : text.search}</button></div></form>
    <p className="dictionary-hint">{text.hint}</p>
    {loading && <p role="status">{text.loading}</p>}
    {result === null && <p role="status">{text.preview}</p>}
    {error && <div role="alert"><p>{text[error.code]}</p><details><summary>{text.source}</summary><p>{error.message}</p></details></div>}
    {result && <Results result={result} selected={selected} onSelect={setSelected} text={text} />}
  </div>;
}
