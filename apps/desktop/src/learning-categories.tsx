import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { singleLineNameSchema } from './platform/notes';
import type { CategoryBulkRequest, CategoryBulkResult, LearningCategory } from './platform/notes';
import type { NotesText } from './notes-content';
import type { NotebookText } from './notebook-content';

export type NotebookMutation = (action: () => Promise<boolean>) => void;

export function useLearningCategories(runtime: RuntimeAdapter, version: number) {
  const [categories, setCategories] = useState<readonly LearningCategory[] | null>();
  const [error, setError] = useState(''), [retry, setRetry] = useState(0);
  useEffect(() => {
    let live = true;
    setError('');
    void runtime.learningCategories().then((value) => { if (live) setCategories(value); })
      .catch((cause: unknown) => { if (live) setError(errorMessage(cause)); });
    return () => { live = false; };
  }, [runtime, version, retry]);
  return { categories, error, retry: () => setRetry((value) => value + 1) };
}
export type LearningCategories = ReturnType<typeof useLearningCategories>;

export function CategoryPicker({ state, selected, onChange, disabled, text }: {
  readonly state: LearningCategories; readonly selected: readonly number[]; readonly disabled: boolean;
  readonly onChange: (ids: readonly number[]) => void; readonly text: NotesText;
}) {
  return <>{state.error && <p role="alert">{state.error} <button type="button" onClick={state.retry}>{text.retry}</button></p>}
    <fieldset disabled={disabled || !state.categories} className="category-picker"><legend>{text.picker}</legend>
    {state.categories?.length === 0 && <p>{text.none}</p>}
    {state.categories?.map((category) => <label key={category.id}><input type="checkbox" checked={selected.includes(category.id)}
      onChange={(event) => onChange(event.target.checked ? [...selected, category.id].sort((a, b) => a - b) : selected.filter((id) => id !== category.id))} />{category.name}</label>)}
  </fieldset></>;
}

function useCategoryAction(onMutation: NotebookMutation) {
  const [busy, setBusy] = useState(false), [error, setError] = useState('');
  function run(action: () => Promise<void>) {
    onMutation(async () => {
      setBusy(true); setError('');
      try { await action(); return true; }
      catch (cause) { setError(errorMessage(cause)); return false; }
      finally { setBusy(false); }
    });
  }
  return { busy, error, run };
}

function CategoryRow({ category, runtime, onMutation, text }: {
  readonly category: LearningCategory; readonly runtime: RuntimeAdapter; readonly onMutation: NotebookMutation; readonly text: NotesText;
}) {
  const [name, setName] = useState(category.name);
  const action = useCategoryAction(onMutation), dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { setName(category.name); }, [category.name]);
  return <li><form onSubmit={(event) => { event.preventDefault(); action.run(async () => {
    if (!await runtime.renameCategory(category.id, name)) throw new Error('Native storage unavailable');
  }); }}><label>{text.renameLabel}: {category.name}<input value={name} disabled={action.busy} onChange={(event) => setName(event.target.value)} /></label>
    <button type="submit" disabled={action.busy || name === category.name || !singleLineNameSchema.safeParse(name).success}>{text.rename}</button>
    <button type="button" disabled={action.busy} onClick={() => dialog.current?.showModal()}>{text.deleteCategory}</button>
  </form>{action.error && <p role="alert">{text.categoryError}: {action.error}</p>}
    <dialog ref={dialog} aria-label={text.deleteCategoryHeading}><h3>{text.deleteCategoryHeading}</h3><p>{category.name}</p><p>{text.deleteCategoryDetail}</p>
      <div className="actions"><button disabled={action.busy} onClick={() => action.run(async () => {
        if (await runtime.deleteCategory(category.id) === null) throw new Error('Native storage unavailable');
        dialog.current?.close();
      })}>{text.confirmDelete}</button><button disabled={action.busy} onClick={() => dialog.current?.close()}>{text.cancel}</button></div>
      {action.error && <p role="alert">{text.categoryError}: {action.error}</p>}
    </dialog></li>;
}

export function CategoryManager({ state, runtime, onMutation, text }: {
  readonly state: LearningCategories; readonly runtime: RuntimeAdapter; readonly onMutation: NotebookMutation; readonly text: NotesText;
}) {
  const [name, setName] = useState(''), action = useCategoryAction(onMutation);
  return <details className="category-manager"><summary>{text.manage}</summary>
    {state.error && <p role="alert">{text.categoryError}: {state.error} <button onClick={state.retry}>{text.retry}</button></p>}
    <form onSubmit={(event) => { event.preventDefault(); action.run(async () => {
      if (!await runtime.createCategory(name)) throw new Error('Native storage unavailable');
      setName('');
    }); }}><label>{text.name}<input value={name} disabled={action.busy || !state.categories} onChange={(event) => setName(event.target.value)} /></label>
      <button type="submit" disabled={action.busy || !state.categories || !singleLineNameSchema.safeParse(name).success}>{text.createCategory}</button></form>
    {action.error && <p role="alert">{text.categoryError}: {action.error}</p>}
    <ul>{state.categories?.map((category) => <CategoryRow key={category.id} category={category} runtime={runtime} onMutation={onMutation} text={text} />)}</ul>
  </details>;
}

type NamedBulkResult = CategoryBulkResult & { readonly title: string };
function useBulkClassification(runtime: RuntimeAdapter, onMutation: NotebookMutation) {
  const [results, setResults] = useState<readonly NamedBulkResult[]>([]), [busy, setBusy] = useState(false);
  const previous = useRef<CategoryBulkRequest | null>(null);
  function run(request: CategoryBulkRequest, names: ReadonlyMap<number, string>) {
    onMutation(async () => {
      previous.current = request; setBusy(true);
      const values: CategoryBulkResult[] = [];
      for (let start = 0; start < request.ids.length; start += 10_000) {
        const ids = request.ids.slice(start, start + 10_000);
        try {
          const saved = await runtime.bulkClassify({ ...request, ids });
          if (!saved) throw new Error('Native storage unavailable');
          values.push(...saved);
        } catch (cause) { values.push(...ids.map((id) => ({ id, status: 'failed' as const, error: errorMessage(cause) }))); }
      }
      setResults((old) => [...old.filter((item) => !request.ids.includes(item.id)), ...values.map((value) => ({ ...value, title: names.get(value.id) ?? String(value.id) }))]);
      setBusy(false); return values.some((value) => value.status === 'saved');
    });
  }
  function retry(result: NamedBulkResult) {
    if (previous.current) run({ ...previous.current, ids: [result.id] }, new Map([[result.id, result.title]]));
  }
  return { results, busy, run, retry, clear: () => setResults([]) };
}

export function CategoryBulkControls({ state, records, selected, kind, runtime, onMutation, text }: {
  readonly state: LearningCategories; readonly records: readonly { readonly id: number; readonly title: string }[];
  readonly selected: readonly number[]; readonly kind: 'vocabulary' | 'grammar'; readonly runtime: RuntimeAdapter;
  readonly onMutation: NotebookMutation; readonly text: NotebookText;
}) {
  const [categoryId, setCategoryId] = useState(0), bulk = useBulkClassification(runtime, onMutation), labels = text.notesText;
  useEffect(() => { bulk.clear(); }, [kind]);
  useEffect(() => { if (!state.categories?.some((category) => category.id === categoryId)) setCategoryId(0); }, [state.categories, categoryId]);
  return <section aria-label={labels.batchCategory}><div className="actions"><label>{labels.batchCategory}<select value={categoryId}
    disabled={bulk.busy || !state.categories?.length} onChange={(event) => setCategoryId(Number(event.target.value))}><option value={0}>—</option>
    {state.categories?.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}</select></label>
    {(['add', 'remove'] as const).map((action) => <button key={action} disabled={bulk.busy || !selected.length || !categoryId}
      onClick={() => { bulk.clear(); bulk.run({ record_kind: kind, ids: selected, category_id: categoryId, action }, new Map(records.map((record) => [record.id, record.title]))); }}>
      {action === 'add' ? labels.batchAdd : labels.batchRemove}</button>)}</div>
    {!!bulk.results.length && <div aria-live="polite"><h4>{labels.results}</h4><ul>{bulk.results.map((result) => <li key={result.id}>
      {result.title}: {result.status === 'saved' ? text.success : `${text.failed}: ${result.error}`}
      {result.status === 'failed' && <button disabled={bulk.busy} onClick={() => bulk.retry(result)}>{labels.retry}</button>}
    </li>)}</ul></div>}
  </section>;
}
