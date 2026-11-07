import { useEffect, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { defaultConjugationOrder, conjugationTenseSchema } from './platform/conjugation';
import type { ConjugationOrder, ConjugationTense } from './platform/conjugation';
import { conjugationTitles } from './conjugation-table';
import type { ConjugationText } from './conjugation-content';

type OrderState = 'loading' | 'ready' | 'preview' | 'saving' | 'saved' | 'failed';
function orderFeedback(state: OrderState, text: ConjugationText): string {
  return { loading: text.orderLoading, saving: text.orderLoading, preview: text.orderPreview,
    saved: text.orderSaved, ready: '', failed: '' }[state];
}

export function OrderControls({ runtime, text, order, onChange }: {
  readonly runtime: RuntimeAdapter; readonly text: ConjugationText; readonly order: ConjugationOrder;
  readonly onChange: (order: ConjugationOrder) => void;
}) {
  const [selected, setSelected] = useState<ConjugationTense>('P');
  const [state, setState] = useState<OrderState>('loading');
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  const index = order.indexOf(selected);
  const saving = state === 'saving', disabled = !loaded || saving;
  const earlierDisabled = !loaded || index === 0, laterDisabled = !loaded || index === order.length - 1;
  useEffect(() => {
    let active = true;
    runtime.conjugationBlockOrder().then((value) => {
      if (!active) return;
      if (value) onChange(value);
      setLoaded(true); setState(value ? 'ready' : 'preview');
    }).catch((reason: unknown) => { if (active) { setError(String(reason)); setState('failed'); } });
    return () => { active = false; };
  }, [runtime, attempt, onChange]);
  async function save(next: ConjugationOrder) {
    if (!loaded || state === 'saving') return;
    setState('saving'); setError('');
    try {
      const value = await runtime.conjugationBlockOrder(next);
      onChange(value ?? next); setState(value ? 'saved' : 'preview');
    } catch (reason) { setError(String(reason)); setState('failed'); }
  }
  function move(direction: -1 | 1) {
    if (index + direction < 0 || index + direction >= order.length) return;
    const next = [...order]; next.splice(index, 1); next.splice(index + direction, 0, selected);
    void save(next);
  }
  return <section className="conjugation-order" aria-label={text.blockOrder} aria-busy={saving}>
    <div className="actions"><label htmlFor="conjugation-block-to-move">{text.blockToMove}</label>
      <select id="conjugation-block-to-move" value={selected} disabled={disabled}
        onChange={(event) => setSelected(conjugationTenseSchema.parse(event.target.value))}>
        {order.map((code, position) => <option key={code} value={code} lang="fr">{position + 1} · {conjugationTitles[code]}</option>)}
      </select>
      <button type="button" disabled={earlierDisabled} aria-disabled={earlierDisabled || saving} onClick={() => move(-1)}>{text.moveEarlier}</button>
      <button type="button" disabled={laterDisabled} aria-disabled={laterDisabled || saving} onClick={() => move(1)}>{text.moveLater}</button>
      <button type="button" disabled={!loaded} aria-disabled={disabled} onClick={() => void save(defaultConjugationOrder)}>{text.resetOrder}</button>
    </div>
    <p aria-live="polite">{orderFeedback(state, text)}
      {loaded && <> {conjugationTitles[selected]} · {index + 1}/23</>}</p>
    {error && <div role="alert">{text.orderFailed}: {error}{!loaded && <button type="button"
      onClick={() => { setError(''); setState('loading'); setAttempt((value) => value + 1); }}>{text.orderRetry}</button>}</div>}
  </section>;
}
