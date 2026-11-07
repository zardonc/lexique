import { useRef } from 'react';
import { cellKey, cellMatches, expressionMatches, defaultConjugationOrder } from './platform/conjugation';
import type { ConjugationCell, ConjugationTable, ConjugationMode, ConjugationOrder, ConjugationTense } from './platform/conjugation';
import { conjugationUse, type ConjugationText } from './conjugation-content';
import { ColoredExpression } from './conjugation-coloring';
import type { Locale } from './content';
import type { RuntimeAdapter } from './platform/runtime';
import { notebookMessages } from './notebook-content';
import { SpeechButton } from './speech-button';

export const conjugationTitles: Readonly<Record<ConjugationTense, string>> = {
  P: 'Présent de l’indicatif', PC: 'Passé composé', I: 'Imparfait', F: 'Futur simple',
  C: 'Conditionnel présent', S: 'Subjonctif présent', PQP: 'Plus-que-parfait',
  J: 'Passé simple', PA: 'Passé antérieur', FA: 'Futur antérieur', CP: 'Conditionnel passé',
  SP: 'Subjonctif passé', T: 'Subjonctif imparfait', SQP: 'Subjonctif plus-que-parfait',
  Y: 'Impératif présent', YP: 'Impératif passé', W: 'Infinitif présent', WP: 'Infinitif passé',
  G: 'Participe présent', K: 'Participe passé', KP: 'Participe composé',
  GP: 'Gérondif présent', GPP: 'Gérondif passé',
};
const priority = new Set<ConjugationTense>(defaultConjugationOrder.slice(0, 7));
const persons = ['1s', '2s', '3s', '1p', '2p', '3p'] as const;
const subjects: Readonly<Record<string, readonly string[]>> = {
  '1s': ['je'], '2s': ['tu'], '3s': ['il', 'elle', 'on'], '1p': ['nous'], '2p': ['vous'], '3p': ['ils', 'elles'],
};
function groupCells(cells: readonly ConjugationCell[]) {
  const groups = new Map<string, ConjugationCell[]>();
  for (const cell of cells) {
    const key = cellKey(cell);
    const values = groups.get(key) ?? []; values.push(cell); groups.set(key, values);
  }
  return [...groups];
}

function expressions(cell: ConjugationCell, impersonal: boolean): readonly string[] {
  if (cell.expressions) return cell.expressions.length ? cell.expressions : [cell.form];
  if (!cell.tense || 'YWGK'.includes(cell.tense)) return [cell.form];
  const pronouns = impersonal && cell.person === '3' && cell.number === 's' ? ['il'] : subjects[`${cell.person}${cell.number}`];
  if (!pronouns) return [cell.form];
  return pronouns.map((subject) => {
    const verbal = subject === 'je' && /^[aàâeéèêëiîïoôœuùûü]/iu.test(cell.form) ? `j’${cell.form}` : `${subject} ${cell.form}`;
    if (!'ST'.includes(cell.tense ?? '')) return verbal;
    return /^[ieo]/u.test(verbal) ? `qu’${verbal}` : `que ${verbal}`;
  });
}

function SourceCell({ cells, query, text, raw, mode, runtime, locale, onStart, impersonal = false }: {
  readonly cells: readonly ConjugationCell[]; readonly query: string; readonly text: ConjugationText; readonly raw: boolean; readonly impersonal?: boolean;
  readonly mode: ConjugationMode;
  readonly runtime: RuntimeAdapter; readonly locale: Locale;
  readonly onStart: (cancel: () => void) => void;
}) {
  const first = cells[0];
  if (!first) return <span>{text.unconfirmed} · {text.absent}</span>;
  const variants = [...new Map(cells.map((cell) => [cell.form, cell])).values()];
  const matched = variants.some((cell) => cellMatches(cell, query));
  const elision = first.person === '1' && first.number === 's' && variants.some((cell) => !cell.expressions?.length && /^[hy]/iu.test(cell.form));
  const speechText = { ...notebookMessages[locale], playWord: text.playExpression, preview: text.speechPreview };
  return <div className="conjugation-cell" data-matched={matched} data-person={`${first.person}${first.number}`}>
    {matched && <strong className="conjugation-match">✓ {text.exact}</strong>}
    {(raw || elision) && <small>{text.unconfirmed} · {raw ? text.expression : text.elision}</small>}
    {variants.map((cell) => <div className="conjugation-variant" key={cell.form}>
      {cell.raw_tag.startsWith('ordinary/v1/') && cell.raw_tag.split('/')[2] !== 'gerund' && <small>{conjugationUse(text, cell.raw_tag.split('/').slice(3).join('/'))}</small>}
      {cell.raw_tag.startsWith('reference/') && <small>{conjugationUse(text, cell.raw_tag.slice(10))}</small>}
      {(raw || elision ? [cell.form] : expressions(cell, impersonal)).map((expression) => <span className="conjugation-expression"
        key={expression} data-matched={expressionMatches(cell.form, query) || expressionMatches(expression, query)}>
        <span lang="fr"><ColoredExpression expression={expression} spans={cell.coloring[expression]} mode={mode} /></span>
        <SpeechButton word={expression} runtime={runtime} text={speechText} onStart={onStart} /></span>)}
    </div>)}
  </div>;
}

function Block({ code, table, query, text, mode, runtime, locale, onStart }: {
  readonly code: string; readonly table: ConjugationTable; readonly query: string; readonly text: ConjugationText;
  readonly mode: ConjugationMode;
  readonly runtime: RuntimeAdapter; readonly locale: Locale;
  readonly onStart: (cancel: () => void) => void;
}) {
  const cells = table.cells.filter((cell) => cell.tense === code);
  const raw = table.verb.ordinary_status === 'not_applicable' && !table.verb.pronominal;
  const impersonal = table.verb.usage_labels.includes('impersonal');
  if (table.verb.blocks?.[code] === 'not_applicable') return <p>{text.notApplicable}</p>;
  if (['P', 'PC', 'I', 'PQP', 'J', 'PA', 'F', 'FA', 'C', 'CP', 'S', 'SP', 'T', 'SQP', 'Y', 'YP'].includes(code)) {
    const rows: readonly string[] = ['Y', 'YP'].includes(code) ? ['2s', '1p', '2p'] : persons;
    const unassigned = cells.filter((cell) => !rows.includes(`${cell.person}${cell.number}`));
    return <><dl>{rows.map((person) => <div key={person}><dt lang="fr">{impersonal && person === '3s' ? 'il' : subjects[person]?.join(' / ')}</dt>
      <dd>{table.verb.blocks?.[`${code}/${person}`] === 'not_applicable' ? text.notApplicable
        : <SourceCell cells={cells.filter((cell) => `${cell.person}${cell.number}` === person)} query={query} text={text} raw={raw} impersonal={impersonal} mode={mode} runtime={runtime} locale={locale} onStart={onStart} />}</dd></div>)}</dl>
      {groupCells(unassigned).map(([key, values]) => <SourceCell key={key} cells={values} query={query} text={text} raw mode={mode} runtime={runtime} locale={locale} onStart={onStart} />)}</>;
  }
  const groups = groupCells(cells);
  return groups.length ? <dl>{groups.map(([key, values]) => <div key={key}><dt>{code === 'K'
    ? [values[0]?.gender, values[0]?.number].filter(Boolean).join(' · ') || text.gender : '—'}</dt>
    <dd><SourceCell cells={values} query={query} text={text} raw={raw} mode={mode} runtime={runtime} locale={locale} onStart={onStart} /></dd></div>)}</dl>
    : <p>{table.verb.blocks?.[code] === 'missing' ? text.absent : text.unconfirmed}</p>;
}

export function ConjugationBlocks({ table, query, text, mode, order, runtime, locale }: {
  readonly table: ConjugationTable; readonly query: string; readonly text: ConjugationText;
  readonly mode: ConjugationMode; readonly order: ConjugationOrder;
  readonly runtime: RuntimeAdapter; readonly locale: Locale;
}) {
  const activeSpeech = useRef<(() => void) | null>(null);
  function startSpeech(cancel: () => void) { activeSpeech.current?.(); activeSpeech.current = cancel; }
  const other = table.cells.filter((cell) => cell.tense === null);
  return <><p className="dictionary-hint">{notebookMessages[locale].machineVoice} · Supertonic 3</p>
    <div className="conjugation-blocks">{order.map((code) => <section className="conjugation-block"
    key={code} data-tense={code} data-priority={priority.has(code)} aria-labelledby={`tense-${code}`}>
    <header><h4 lang="fr" id={`tense-${code}`}>{conjugationTitles[code]}</h4>{priority.has(code) && <small>★ {text.priority}</small>}</header>
    <Block code={code} table={table} query={query} text={text} mode={mode} runtime={runtime} locale={locale} onStart={startSpeech} />
  </section>)}</div>{other.length > 0 && <section><h4>{text.other}</h4>{groupCells(other).map(([key, values]) =>
    <SourceCell key={key} cells={values} query={query} text={text} raw mode={mode} runtime={runtime} locale={locale} onStart={startSpeech} />)}</section>}
  {!table.verb.pronominal && <p className="dictionary-hint">{text.agreement}</p>}<p className="dictionary-hint">{text.historical}</p></>;
}
