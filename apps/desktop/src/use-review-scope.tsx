import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { LearningCategory } from './platform/notes';
import type { RuntimeAdapter } from './platform/runtime';
import type { ReviewScope } from './platform/review-scope';
import { errorMessage } from './platform/vocabulary';

const messages = {
  zh: { label: '复习范围', all: '全部复习', choose: '请选择复习范围', current: (name: string) => `当前范围：${name}`, pending: (name: string) => `将在当前卡片评分或跳过后应用：${name}`, readError: '复习范围读取失败', saveError: '复习范围保存失败', retryRead: '重读复习范围', retrySave: '重试保存复习范围' },
  en: { label: 'Review scope', all: 'All reviews', choose: 'Choose a review scope', current: (name: string) => `Applied scope: ${name}`, pending: (name: string) => `Applies after rating or skipping the current card: ${name}`, readError: 'Could not read review scope', saveError: 'Could not save review scope', retryRead: 'Reload review scope', retrySave: 'Retry saving review scope' },
  fr: { label: 'Périmètre de révision', all: 'Toutes les révisions', choose: 'Choisissez un périmètre', current: (name: string) => `Périmètre actuel : ${name}`, pending: (name: string) => `S’appliquera après l’évaluation ou le passage de la carte actuelle : ${name}`, readError: 'Lecture du périmètre impossible', saveError: 'Enregistrement du périmètre impossible', retryRead: 'Relire le périmètre', retrySave: 'Réessayer l’enregistrement du périmètre' },
} as const;

export function useReviewScope(runtime: RuntimeAdapter) {
  const [applied, setApplied] = useState<ReviewScope | null>(null);
  const [categories, setCategories] = useState<readonly LearningCategory[]>([]);
  const [pending, setPending] = useState<ReviewScope | null>(null);
  const pendingRef = useRef<ReviewScope | null>(null);
  const [busy, setBusy] = useState(false);
  const saving = useRef(false);
  const [error, setError] = useState('');
  const [version, setVersion] = useState(0);
  useEffect(() => {
    let active = true;
    void Promise.all([runtime.reviewScope(), runtime.learningCategories()]).then(([scope, categories]) => {
      if (!active) return;
      setApplied(scope ?? { kind: 'all' }); setCategories(categories ?? []); setError('');
    }).catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [runtime, version]);
  async function applyPending() {
    const choice = pendingRef.current;
    if (!choice || saving.current) return;
    saving.current = true; setBusy(true); setError('');
    try {
      const saved = await runtime.reviewScope(choice);
      if (!saved) throw new Error('Native review scope unavailable');
      setApplied(saved); setPending(null); pendingRef.current = null;
    } catch (cause) { setError(errorMessage(cause)); }
    finally { saving.current = false; setBusy(false); }
  }
  function select(value: string, hasCard: boolean) {
    const choice: ReviewScope = value === 'all' ? { kind: 'all' } : { kind: 'vocabulary', category_ids: [Number(value)], include_uncategorized: false };
    const pending = JSON.stringify(choice) === JSON.stringify(applied) ? null : choice;
    setPending(pending); pendingRef.current = pending; setError('');
    if (!hasCard) void applyPending();
  }
  return { applied, pending, categories, busy, error, select, applyPending, retryRead: () => { setError(''); setVersion((value) => value + 1); } };
}

function scopeValue(scope: ReviewScope | null) {
  if (scope?.kind === 'all') return 'all';
  return scope?.kind === 'vocabulary' ? String(scope.category_ids[0] ?? '') : '';
}

export function ReviewScopeSelector({ state, locale, hasCard }: {
  readonly state: ReturnType<typeof useReviewScope>; readonly locale: Locale; readonly hasCard: boolean;
}) {
  const labels = messages[locale];
  function name(scope: ReviewScope | null) {
    return scope?.kind === 'all' ? labels.all : scope?.kind === 'vocabulary'
      ? scope.category_ids.map((id) => state.categories.find((category) => category.id === id)?.name ?? labels.choose).join(', ')
      : labels.choose;
  }
  const selected = state.pending ?? state.applied;
  const value = scopeValue(selected);
  return <div>
    <label>{labels.label} <select aria-label={labels.label} value={value} disabled={!state.applied || state.busy}
      onChange={(event) => state.select(event.target.value, hasCard)}>
      <option value="" disabled>{labels.choose}</option><option value="all">{labels.all}</option>
      {state.categories.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
    </select></label>
    {state.applied && <p role="status">{labels.current(name(state.applied))}</p>}
    {state.pending && <p role="status">{labels.pending(name(state.pending))}</p>}
    {state.error && <><p role="alert">{state.applied ? labels.saveError : labels.readError}: {state.error}</p>
      <button disabled={state.busy} onClick={() => { if (state.applied) void state.applyPending(); else state.retryRead(); }}>{state.applied ? labels.retrySave : labels.retryRead}</button></>}
  </div>;
}
