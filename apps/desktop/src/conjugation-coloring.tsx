import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import type { ConjugationMode, ConjugationSpans, ConjugationTable } from './platform/conjugation';
import type { ConjugationText } from './conjugation-content';

export function ColoringNotice({ table, mode, text }: {
  readonly table: ConjugationTable; readonly mode: ConjugationMode; readonly text: ConjugationText;
}) {
  if (mode !== 'ending') return null;
  if (table.verb.group === 3) return <p>{text.endingExcluded}</p>;
  if ([1, 2].includes(table.verb.group ?? 0) && table.cells.some((cell) => Object.values(cell.coloring).every((span) => span.ending === null))) return <p>{text.endingUnconfirmed}</p>;
  return null;
}

export function ColoringControls({ runtime, text, mode, onChange }: {
  readonly runtime: RuntimeAdapter; readonly text: ConjugationText; readonly mode: ConjugationMode;
  readonly onChange: (mode: ConjugationMode) => void;
}) {
  const [state, setState] = useState<'loading' | 'ready' | 'preview' | 'saving' | 'saved' | 'failed'>('loading');
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    runtime.conjugationPreferences().then((value) => {
      if (!active) return;
      if (value) onChange(value.mode);
      setLoaded(true); setState(value ? 'ready' : 'preview');
    }).catch((reason: unknown) => { if (active) { setError(String(reason)); setState('failed'); } });
    return () => { active = false; };
  }, [runtime, attempt, onChange]);
  async function save(choice: ConjugationMode) {
    setState('saving'); setError('');
    try {
      const value = await runtime.conjugationPreferences({ mode: choice });
      onChange(value?.mode ?? choice); setState(value ? 'saved' : 'preview');
    } catch (reason) { setError(String(reason)); setState('failed'); }
  }
  return <section className="conjugation-coloring" aria-label={text.coloring}>
    <div className="actions" role="group" aria-label={text.coloring}>{(['verbal', 'main', 'ending'] as const).map((choice, index) =>
      <button key={choice} type="button" aria-pressed={mode === choice} disabled={!loaded || state === 'saving'}
        onClick={() => void save(choice)}>{index + 1} · {text.modes[choice]}</button>)}</div>
    <p aria-live="polite">{state === 'loading' || state === 'saving' ? text.preferencesLoading : state === 'preview' ? text.preferencesPreview : state === 'saved' ? text.preferencesSaved : ''}</p>
    {error && <div role="alert">{text.preferencesFailed}: {error}{!loaded && <button type="button"
      onClick={() => { setError(''); setState('loading'); setAttempt((value) => value + 1); }}>{text.preferencesRetry}</button>}</div>}
  </section>;
}

export function ColoredExpression({ expression, spans, mode }: {
  readonly expression: string; readonly spans: ConjugationSpans | undefined; readonly mode: ConjugationMode;
}) {
  if (!spans) return expression;
  const ranges = mode === 'verbal' ? spans.verbal : mode === 'main' ? [spans.main] : spans.ending ? [spans.ending] : [];
  const characters = [...expression], parts: ReactNode[] = [];
  let cursor = 0;
  for (const [start, end] of ranges) {
    parts.push(characters.slice(cursor, start).join(''));
    parts.push(<em className="conjugation-colored" key={`${start}/${end}`}>{characters.slice(start, end).join('')}</em>);
    cursor = end;
  }
  parts.push(characters.slice(cursor).join(''));
  return parts;
}
