import { useEffect, useRef, useState } from 'react';
import type { GrammarNotebook } from './use-grammar-notebook';
import type { RuntimeAdapter } from './platform/runtime';
import { singleLineNameSchema } from './platform/notes';
import { characterCount, errorMessage } from './platform/vocabulary';
import type { NotebookText } from './notebook-content';
import { CategoryPicker } from './learning-categories';
import type { LearningCategories } from './learning-categories';
import { NoteExport } from './note-export';

function useGrammarPause(runtime: RuntimeAdapter, id: number) {
  const [paused, setPaused] = useState<boolean>(), [busy, setBusy] = useState(false), [error, setError] = useState('');
  useEffect(() => {
    let live = true;
    void runtime.reviewPause(id).then((value) => {
      if (value === null) throw new Error('Native review storage unavailable');
      if (live) setPaused(value);
    }).catch((cause: unknown) => { if (live) setError(errorMessage(cause)); });
    return () => { live = false; };
  }, [runtime, id]);
  async function toggle() {
    if (paused === undefined) return false;
    setBusy(true); setError('');
    try {
      const value = await runtime.reviewPause(id, !paused);
      if (value === null) throw new Error('Native review storage unavailable');
      setPaused(value); return true;
    } catch (cause) { setError(errorMessage(cause)); return false; }
    finally { setBusy(false); }
  }
  return { paused, busy, error, toggle };
}

function GrammarPause({ runtime, id, text, onChanged }: {
  readonly runtime: RuntimeAdapter; readonly id: number; readonly text: NotebookText; readonly onChanged: () => void;
}) {
  const pause = useGrammarPause(runtime, id);
  return <><button disabled={pause.busy || pause.paused === undefined} onClick={() => { void pause.toggle().then((saved) => { if (saved) onChanged(); }); }}>
    {pause.paused ? text.resume : text.pause}</button>{pause.paused && <p role="status">{text.paused}</p>}
    {pause.error && <p role="alert">{text.reviewError}: {pause.error}</p>}</>;
}

function GrammarCardEditor({ notebook, runtime, text, onChanged }: {
  readonly notebook: GrammarNotebook; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onChanged: () => void;
}) {
  const editor = notebook.editor!, labels = text.notesText;
  if (!editor.card) return <MakeGrammarCard notebook={notebook} text={text} />;
  const card = editor.card;
  const valid = [card.question, card.answer].every((value) => value.trim().length > 0 && characterCount(value) <= 20_000);
  return <fieldset disabled={notebook.busy} className="grammar-card-editor"><legend>{labels.card}</legend><p>{labels.independent}</p>
    {(['question', 'answer'] as const).map((field) => <div key={field}><label htmlFor={`grammar-${field}`}>{labels[field]}</label><textarea id={`grammar-${field}`} rows={5} value={card[field]} spellCheck={false}
      aria-describedby={`grammar-${field}-count`} aria-invalid={characterCount(card[field]) > 20_000} onChange={(event) => notebook.change({ card: { ...card, [field]: event.target.value } })} />
      <p id={`grammar-${field}-count`}>{characterCount(card[field]).toLocaleString()} / 20,000 {text.count}</p></div>)}
    {!valid && <p role="alert">{labels.answerRequired}</p>}
    <div className="actions"><button disabled={!valid || !notebook.cardDirty} onClick={() => { void notebook.saveCard().then((saved) => { if (saved) onChanged(); }); }}>
      {editor.record?.card ? labels.saveCard : labels.enroll}</button>
      {editor.record?.card && <GrammarPause runtime={runtime} id={editor.record.id} text={text} onChanged={onChanged} />}</div>
  </fieldset>;
}

function MakeGrammarCard({ notebook, text }: { readonly notebook: GrammarNotebook; readonly text: NotebookText }) {
  const disabled = !notebook.editor?.record || notebook.noteDirty;
  return <><button disabled={notebook.busy || disabled} onClick={notebook.makeCard}>{text.notesText.makeCard}</button>
    {disabled && <p>{text.notesText.saveFirst}</p>}</>;
}

function grammarSaveStatus(notebook: GrammarNotebook, text: NotebookText) {
  return notebook.dirty ? text.unsaved : notebook.saved ? text.saved : '';
}

function GrammarConflict({ notebook, text }: { readonly notebook: GrammarNotebook; readonly text: NotebookText }) {
  if (!notebook.conflict) return null;
  return <section role="alert" aria-label={text.conflict}><h4>{text.conflict}</h4><p>{text.current}</p>
    <pre>{notebook.conflict.title}{'\n'}{notebook.conflict.body}</pre>
    {notebook.conflict.card && <pre>{notebook.conflict.card.question}{'\n'}{notebook.conflict.card.answer}</pre>}
    <p>{text.discardWarning}</p><div className="actions"><button disabled={notebook.busy} onClick={notebook.adopt}>{text.adopt}</button>
      <button disabled={notebook.busy} onClick={notebook.back}>{text.back}</button></div></section>;
}

function GrammarDelete({ notebook, text, onChanged }: { readonly notebook: GrammarNotebook; readonly text: NotebookText; readonly onChanged: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null), labels = text.notesText;
  if (!notebook.editor?.record) return null;
  return <><button disabled={notebook.busy} className="delete-note" onClick={() => dialog.current?.showModal()}>{labels.delete}</button>
    <dialog ref={dialog} aria-label={labels.deleteHeading}><h3>{labels.deleteHeading}</h3><p>{notebook.editor.record.title}</p><p>{labels.deleteDetail}</p>
      {notebook.error && <p role="alert">{notebook.error}</p>}
      <div className="actions"><button disabled={notebook.busy} onClick={() => { void notebook.remove().then((saved) => { if (saved) { dialog.current?.close(); onChanged(); } }); }}>{labels.confirmDelete}</button>
        <button disabled={notebook.busy} onClick={() => dialog.current?.close()}>{labels.cancel}</button></div>
    </dialog></>;
}

export function GrammarEditor({ notebook, categories, runtime, text, onChanged }: {
  readonly notebook: GrammarNotebook; readonly categories: LearningCategories; readonly runtime: RuntimeAdapter;
  readonly text: NotebookText; readonly onChanged: () => void;
}) {
  const title = useRef<HTMLInputElement>(null), identity = notebook.editor?.record?.id;
  useEffect(() => { title.current?.focus(); }, [identity, notebook.editor !== null]);
  if (!notebook.editor) return null;
  const editor = notebook.editor, labels = text.notesText;
  const validTitle = singleLineNameSchema.safeParse(editor.title).success, validBody = characterCount(editor.body) <= 20_000;
  return <section className="vocabulary-editor grammar-editor" aria-label={labels.grammar}><h3>{labels.grammar}</h3>
    <fieldset disabled={notebook.busy}><label htmlFor="grammar-title">{labels.title}</label><input id="grammar-title" ref={title} value={editor.title} required aria-invalid={!validTitle} spellCheck={false}
      onChange={(event) => notebook.change({ title: event.target.value })} /><p>{characterCount(editor.title.trim()).toLocaleString()} / 200 {text.count}</p>
      {!validTitle && <p>{labels.nameLimit}</p>}
      <label htmlFor="grammar-body">{labels.body}</label><textarea id="grammar-body" rows={8} spellCheck={false} value={editor.body} aria-invalid={!validBody}
        onChange={(event) => notebook.change({ body: event.target.value })} /><p>{characterCount(editor.body).toLocaleString()} / 20,000 {text.count}</p>
      {!validBody && <p role="alert">{text.limit}</p>}
      <CategoryPicker state={categories} selected={editor.category_ids} onChange={(category_ids) => notebook.change({ category_ids })} disabled={notebook.busy} text={labels} />
      <div className="actions"><button disabled={!validTitle || !validBody || !notebook.noteDirty} onClick={() => { void notebook.saveNote(); }}>{labels.save}</button>
        <span role="status">{grammarSaveStatus(notebook, text)}</span></div>
    </fieldset>
    <GrammarCardEditor notebook={notebook} runtime={runtime} text={text} onChanged={onChanged} />
    {editor.record && <NoteExport key={editor.record.id} runtime={runtime} text={text.grammarExportText}
      scope={{ kind: 'single', id: editor.record.id }} recordKind="grammar" disabled={notebook.busy} />}
    {notebook.error && <p role="alert">{text.saveError}: {notebook.error}</p>}
    <GrammarConflict notebook={notebook} text={text} /><GrammarDelete notebook={notebook} text={text} onChanged={onChanged} />
  </section>;
}
