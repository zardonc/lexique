import { useEffect, useState } from 'react';
import type { NotebookText } from './notebook-content';
import type { Frequency, FrequencyLookup } from './platform/frequency';
import type { RuntimeAdapter } from './platform/runtime';
import { dictionaryFailure } from './platform/dictionary';

type FrequencyText = NotebookText['frequency'];

export function useFrequency(runtime: RuntimeAdapter, keys: readonly string[]) {
  const [lookup, setLookup] = useState<FrequencyLookup | null>();
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setLookup(undefined); setError('');
    if (keys.length) void Promise.resolve().then(async () => {
      if (cancelled) return;
      const value = await runtime.lookupFrequency(keys);
      if (!cancelled) setLookup(value);
    }).catch((cause: unknown) => { if (!cancelled) setError(dictionaryFailure(cause).message); });
    return () => { cancelled = true; };
  }, [runtime, keys, attempt]);
  return { lookup, error, retry: () => setAttempt((value) => value + 1) };
}

export function FrequencyBadge({ entry, text }: { readonly entry?: Frequency; readonly text: FrequencyText }) {
  const value = entry?.per_million;
  const band = value === null || value === undefined ? 'unknown' : value >= 100 ? 'high' : value >= 10 ? 'medium' : 'low';
  return <span className="frequency-badge" data-band={band}>{text[band]}</span>;
}

export function FrequencyState({ state, text }: { readonly state: ReturnType<typeof useFrequency>; readonly text: FrequencyText }) {
  if (state.error) return <p role="alert">{text.readError}: {state.error} <button onClick={state.retry}>{text.retry}</button></p>;
  if (state.lookup === undefined) return <p role="status">{text.loading}</p>;
  return state.lookup === null && <p role="status">{text.preview}</p>;
}

function number(value: number | null, text: FrequencyText) {
  return value === null ? '—' : value.toLocaleString(text.locale, { maximumFractionDigits: 2 });
}

export function FrequencyDetails({ lookup, entry, text }: {
  readonly lookup: FrequencyLookup; readonly entry: Frequency; readonly text: FrequencyText;
}) {
  return <details className="frequency-details"><summary>{text.details}</summary>
    <p>{lookup.source.title} · {text.corpus} · <code>{lookup.metric}</code></p>
    <p>{text.lemma}: <span lang="fr">{entry.lemma ?? '—'}</span> · {text.total}: {number(entry.per_million, text)} {text.units}</p>
    {entry.reason && <p>{text.reasons[entry.reason]}</p>}
    {entry.reason === 'ambiguous' && <p>{text.candidates}: <span lang="fr">{entry.candidates.join(' · ')}</span></p>}
    {!!entry.parts.length && <table><caption>{text.parts}</caption><thead><tr><th scope="col">{text.pos}</th>
      <th scope="col">{text.units}</th><th scope="col">{text.reason}</th></tr></thead><tbody>
      {entry.parts.map((part, index) => <tr key={index}><td>{part.part_of_speech ?? text.missingPos}</td>
        <td>{number(part.per_million, text)}</td><td>{part.reason ? text.reasons[part.reason] : '—'}</td></tr>)}
    </tbody></table>}
    <p>{text.standard}</p><p>{lookup.source.attribution} · {lookup.source.license}</p>
    <p>{lookup.source.source_url}</p><p>{lookup.source.license_url}</p>
  </details>;
}
