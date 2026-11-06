import { cellKey } from './platform/conjugation';
import type { ConjugationCell, ConjugationTable } from './platform/conjugation';
import type { ConjugationText } from './conjugation-content';

// Priority order, then the remaining mood families in the confirmed M3 matrix.
const blocks = [
  ['P', 'Présent de l’indicatif'], ['PC', 'Passé composé'], ['I', 'Imparfait'], ['F', 'Futur simple'],
  ['C', 'Conditionnel présent'], ['S', 'Subjonctif présent'], ['PQP', 'Plus-que-parfait'],
  ['J', 'Passé simple'], ['PA', 'Passé antérieur'], ['FA', 'Futur antérieur'], ['CP', 'Conditionnel passé'],
  ['SP', 'Subjonctif passé'], ['T', 'Subjonctif imparfait'], ['SQP', 'Subjonctif plus-que-parfait'],
  ['Y', 'Impératif présent'], ['YP', 'Impératif passé'], ['W', 'Infinitif présent'], ['WP', 'Infinitif passé'],
  ['G', 'Participe présent'], ['K', 'Participe passé'], ['KP', 'Participe composé'],
  ['GP', 'Gérondif présent'], ['GPP', 'Gérondif passé'],
] as const;
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
  if (!cell.tense || 'YWGK'.includes(cell.tense)) return [cell.form];
  const pronouns = impersonal && cell.person === '3' && cell.number === 's' ? ['il'] : subjects[`${cell.person}${cell.number}`];
  if (!pronouns) return [cell.form];
  return pronouns.map((subject) => {
    const verbal = subject === 'je' && /^[aàâeéèêëiîïoôœuùûü]/iu.test(cell.form) ? `j’${cell.form}` : `${subject} ${cell.form}`;
    if (!'ST'.includes(cell.tense ?? '')) return verbal;
    return /^[ieo]/u.test(verbal) ? `qu’${verbal}` : `que ${verbal}`;
  });
}

function SourceCell({ cells, query, text, raw, impersonal = false }: {
  readonly cells: readonly ConjugationCell[]; readonly query: string; readonly text: ConjugationText; readonly raw: boolean; readonly impersonal?: boolean;
}) {
  const first = cells[0];
  if (!first) return <span>{text.unconfirmed} · {text.absent}</span>;
  const variants = [...new Map(cells.map((cell) => [cell.form, cell])).values()];
  const matched = variants.some((cell) => cell.form === query);
  const elision = first.person === '1' && first.number === 's' && variants.some((cell) => /^[hy]/iu.test(cell.form));
  return <div className="conjugation-cell" data-matched={matched} data-person={`${first.person}${first.number}`}>
    {matched && <strong className="conjugation-match">✓ {text.exact}</strong>}
    {(raw || elision) && <small>{text.unconfirmed} · {raw ? text.expression : text.elision}</small>}
    {variants.map((cell) => <div className="conjugation-variant" key={cell.form}>
      {(raw || elision ? [cell.form] : expressions(cell, impersonal)).map((expression) => <span className="conjugation-expression" lang="fr"
        key={expression} data-matched={cell.form === query}>{expression}</span>)}
    </div>)}
  </div>;
}

function Block({ code, table, query, text }: {
  readonly code: string; readonly table: ConjugationTable; readonly query: string; readonly text: ConjugationText;
}) {
  const cells = table.cells.filter((cell) => cell.tense === code);
  const raw = table.verb.ordinary_status === 'not_applicable';
  const impersonal = table.verb.usage_labels.includes('impersonal');
  if (['P', 'I', 'J', 'F', 'C', 'S', 'T', 'Y'].includes(code)) {
    const rows: readonly string[] = code === 'Y' ? ['2s', '1p', '2p'] : persons;
    const unassigned = cells.filter((cell) => !rows.includes(`${cell.person}${cell.number}`));
    return <><dl>{rows.map((person) => <div key={person}><dt lang="fr">{impersonal && person === '3s' ? 'il' : subjects[person]?.join(' / ')}</dt>
      <dd><SourceCell cells={cells.filter((cell) => `${cell.person}${cell.number}` === person)} query={query} text={text} raw={raw} impersonal={impersonal} /></dd></div>)}</dl>
      {groupCells(unassigned).map(([key, values]) => <SourceCell key={key} cells={values} query={query} text={text} raw />)}</>;
  }
  const groups = groupCells(cells);
  return groups.length ? <dl>{groups.map(([key, values]) => <div key={key}><dt>{code === 'K'
    ? [values[0]?.gender, values[0]?.number].filter(Boolean).join(' · ') || text.gender : '—'}</dt>
    <dd><SourceCell cells={values} query={query} text={text} raw={raw} /></dd></div>)}</dl>
    : <p>{text.unconfirmed} · {text.absent}</p>;
}

export function ConjugationBlocks({ table, query, text }: {
  readonly table: ConjugationTable; readonly query: string; readonly text: ConjugationText;
}) {
  const other = table.cells.filter((cell) => cell.tense === null);
  return <><div className="conjugation-blocks">{blocks.map(([code, title], index) => <section className="conjugation-block"
    key={code} data-tense={code} data-priority={index < 7} aria-labelledby={`tense-${code}`}>
    <header><h4 lang="fr" id={`tense-${code}`}>{title}</h4>{index < 7 && <small>★ {text.priority}</small>}</header>
    <Block code={code} table={table} query={query} text={text} />
  </section>)}</div>{other.length > 0 && <section><h4>{text.other}</h4>{groupCells(other).map(([key, values]) =>
    <SourceCell key={key} cells={values} query={query} text={text} raw />)}</section>}
  <p className="dictionary-hint">{text.historical}</p></>;
}
