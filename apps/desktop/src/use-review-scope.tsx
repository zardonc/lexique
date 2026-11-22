import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { LearningCategory } from './platform/notes';
import type { RuntimeAdapter } from './platform/runtime';
import type { ReviewScope } from './platform/review-scope';
import { errorMessage } from './platform/vocabulary';
import { subscribeReviewEvents } from './review-events';

const messages = {
  zh: { label: '复习范围', all: '全部复习', uncategorized: '未分类', choose: '请选择复习范围', current: (name: string) => `当前范围：${name}`, pending: (name: string) => `将在当前卡片评分或跳过后应用：${name}`, readError: '复习范围读取失败', saveError: '复习范围保存失败', retryRead: '重读复习范围', retrySave: '重试保存复习范围' },
  en: { label: 'Review scope', all: 'All reviews', uncategorized: 'Uncategorized', choose: 'Choose a review scope', current: (name: string) => `Applied scope: ${name}`, pending: (name: string) => `Applies after rating or skipping the current card: ${name}`, readError: 'Could not read review scope', saveError: 'Could not save review scope', retryRead: 'Reload review scope', retrySave: 'Retry saving review scope' },
  fr: { label: 'Périmètre de révision', all: 'Toutes les révisions', uncategorized: 'Sans catégorie', choose: 'Choisissez un périmètre', current: (name: string) => `Périmètre actuel : ${name}`, pending: (name: string) => `S’appliquera après l’évaluation ou le passage de la carte actuelle : ${name}`, readError: 'Lecture du périmètre impossible', saveError: 'Enregistrement du périmètre impossible', retryRead: 'Relire le périmètre', retrySave: 'Réessayer l’enregistrement du périmètre' },
} as const;

function remainingChoice(scope: ReviewScope | null, categories: readonly LearningCategory[]): ReviewScope | null {
  if (scope?.kind !== 'vocabulary') return scope;
  const category_ids = scope.category_ids.filter((id) => categories.some((category) => category.id === id));
  return category_ids.length || scope.include_uncategorized ? { ...scope, category_ids } : { kind: 'needs_selection' };
}

export function useReviewScope(runtime: RuntimeAdapter) {
  const [applied, setApplied] = useState<ReviewScope | null>(null);
  const [categories, setCategories] = useState<readonly LearningCategory[]>([]);
  const [pending, setPending] = useState<ReviewScope | null>(null);
  const pendingRef = useRef<ReviewScope | null>(null);
  const [busy, setBusy] = useState(false);
  const saving = useRef(false), preservePresentation = useRef(false);
  const requestRef = useRef(0);
  const [error, setError] = useState('');
  const [saveError, setSaveError] = useState('');
  const [version, setVersion] = useState(0);
  useEffect(() => {
    let active = true;
    async function read(live = false) {
      if (saving.current) return;
      const current = ++requestRef.current;
      try {
        const [scope, categories] = await Promise.all([runtime.reviewScope(), runtime.learningCategories()]);
        if (!active || current !== requestRef.current) return;
        preservePresentation.current = live;
        setApplied((previous) => JSON.stringify(previous) === JSON.stringify(scope ?? { kind: 'all' }) ? previous : scope ?? { kind: 'all' });
        setCategories(categories ?? []); setError('');
        pendingRef.current = remainingChoice(pendingRef.current, categories ?? []); setPending(pendingRef.current);
      } catch (cause) { if (active && current === requestRef.current) setError(errorMessage(cause)); }
    }
    const stop = subscribeReviewEvents(runtime, { changed: () => { void read(true); }, resumed: () => { void read(true); }, failed: (cause) => setError(errorMessage(cause)) }); void read();
    return () => { active = false; stop(); };
  }, [runtime, version]);
  async function applyPending() {
    const choice = pendingRef.current;
    if (!choice) return true;
    if (saving.current) return false;
    requestRef.current += 1; saving.current = true; setBusy(true); setError(''); setSaveError('');
    try {
      const saved = await runtime.reviewScope(choice);
      if (!saved) throw new Error('Native review scope unavailable');
      preservePresentation.current = false; setApplied(saved); setPending(null); pendingRef.current = null;
      return true;
    } catch (cause) { setSaveError(errorMessage(cause)); return false; }
    finally { saving.current = false; setBusy(false); }
  }
  function select(selection: ReviewScope, hasCard: boolean) {
    const choice: ReviewScope = selection.kind === 'vocabulary' && selection.category_ids.length === 0 && !selection.include_uncategorized
      ? { kind: 'needs_selection' } : selection;
    const pending = JSON.stringify(choice) === JSON.stringify(applied) ? null : choice;
    setPending(pending); pendingRef.current = pending; setError(''); setSaveError('');
    if (!hasCard) void applyPending();
  }
  return { applied, pending, categories, busy, error: saveError || error, saveError, select, applyPending, preservePresentation, retryRead: () => { setError(''); setVersion((value) => value + 1); } };
}

export function ReviewScopeSelector({ state, locale, hasCard }: {
  readonly state: ReturnType<typeof useReviewScope>; readonly locale: Locale; readonly hasCard: boolean;
}) {
  const labels = messages[locale];
  function name(scope: ReviewScope | null) {
    return scope?.kind === 'all' ? labels.all : scope?.kind === 'vocabulary'
      ? [...scope.category_ids.map((id) => state.categories.find((category) => category.id === id)?.name ?? labels.choose), ...(scope.include_uncategorized ? [labels.uncategorized] : [])].join(', ')
      : labels.choose;
  }
  const selected = state.pending ?? state.applied ?? { kind: 'needs_selection' };
  const conditions = selected.kind === 'vocabulary' ? selected : { category_ids: [], include_uncategorized: false };
  const ids: readonly number[] = conditions.category_ids;
  const uncategorized = conditions.include_uncategorized;
  return <div>
    <fieldset className="category-picker" disabled={!state.applied || state.busy}><legend>{labels.label}</legend>
      <label><input type="checkbox" checked={selected.kind === 'all'}
        onChange={(event) => state.select({ kind: event.target.checked ? 'all' : 'needs_selection' }, hasCard)} />{labels.all}</label>
      {state.categories.map((category) => <label key={category.id}><input type="checkbox" checked={ids.includes(category.id)}
        onChange={(event) => state.select({ kind: 'vocabulary', category_ids: event.target.checked ? [...ids, category.id].sort((a, b) => a - b) : ids.filter((id) => id !== category.id), include_uncategorized: uncategorized }, hasCard)} />{category.name}</label>)}
      <label><input type="checkbox" checked={uncategorized}
        onChange={(event) => state.select({ kind: 'vocabulary', category_ids: ids, include_uncategorized: event.target.checked }, hasCard)} />{labels.uncategorized}</label>
    </fieldset>
    {state.applied && <p role="status">{labels.current(name(state.applied))}</p>}
    {state.pending && <p role="status">{labels.pending(name(state.pending))}</p>}
    {state.error && <><p role="alert">{state.saveError ? labels.saveError : labels.readError}: {state.error}</p>
      <button disabled={state.busy} onClick={() => { if (state.saveError) void state.applyPending(); else state.retryRead(); }}>{state.saveError ? labels.retrySave : labels.retryRead}</button></>}
  </div>;
}
