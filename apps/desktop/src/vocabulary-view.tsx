import { useEffect, useRef } from 'react';
import type { NotebookText } from './notebook-content';
import { characterCount } from './platform/vocabulary';
import type { VocabularySnapshot } from './platform/vocabulary';
import type { Notebook } from './use-notebook';
import type { RuntimeAdapter } from './platform/runtime';
import { ReviewCards } from './review-cards';

export function VocabularySummary({ snapshot, text }: { readonly snapshot: VocabularySnapshot; readonly text: NotebookText }) {
  return <section className="vocabulary-summary" aria-label={text.summary}>
    <h3 lang="fr">{snapshot.headword}</h3>
    {snapshot.digest.map((entry, index) => <dl key={index}>
      <dt>{text.pronunciation}</dt><dd lang="fr">{entry.pronunciations.join(' · ') || text.missing}</dd>
      <dt>{text.grammar}</dt><dd>{entry.grammar.join(' · ') || text.missing}</dd>
      <dt>{text.translation}</dt><dd lang="zh">{entry.translation || text.missing}</dd>
    </dl>)}
  </section>;
}

function SavedSource({ snapshot, text }: { readonly snapshot: VocabularySnapshot; readonly text: NotebookText }) {
  const source = snapshot.reference;
  return <footer className="dictionary-source"><h4>{text.source}</h4>
    <p>{source.dictionary} · {source.version}</p><p>{source.attribution} · {source.license}</p>
    <p>{source.source_url}</p><p>{source.license_url}</p>
  </footer>;
}

function Conflict({ notebook, text }: { readonly notebook: Notebook; readonly text: NotebookText }) {
  if (!notebook.conflict) return null;
  return <section role="alert" className="runtime-error" aria-label={text.conflict}>
    <h4>{text.conflict}</h4><p>{text.current}</p><pre>{notebook.conflict.notes || '∅'}</pre>
    <p>{text.exampleFr}</p><pre>{notebook.conflict.example_fr || '∅'}</pre>
    <p>{text.exampleZh}</p><pre>{notebook.conflict.example_zh || '∅'}</pre>
    {notebook.conflict.more_examples.map((example, index) => <pre key={index}>{example.fr}{'\n'}{example.zh}</pre>)}
    <p>{text.discardWarning}</p><div className="actions">
      <button disabled={notebook.busy} onClick={notebook.adopt}>{text.adopt}</button>
      <button disabled={notebook.busy} onClick={notebook.back}>{text.back}</button>
    </div>
  </section>;
}

function SaveActions({ notebook, text, overLimit }: {
  readonly notebook: Notebook; readonly text: NotebookText; readonly overLimit: boolean;
}) {
  return <div className="actions"><button disabled={overLimit || notebook.busy || !notebook.dirty}
    onClick={() => { void notebook.save(); }}>{notebook.busy ? text.saving : text.save}</button>
    <span role="status">{notebook.saved ? text.saved : notebook.dirty ? text.unsaved : ''}</span></div>;
}

function exampleTooLong(editor: NonNullable<Notebook['editor']>) {
  return [editor.example_fr, editor.example_zh, ...editor.more_examples.flatMap((e) => [e.fr, e.zh])].some((value) => characterCount(value) > 20_000);
}

function ExampleEditor({ notebook, text }: { readonly notebook: Notebook; readonly text: NotebookText }) {
  const editor = notebook.editor!;
  const overLimit = exampleTooLong(editor);
  return <><h4>{text.exampleUser}</h4>
    <label htmlFor="example-fr">{text.exampleFr}</label><textarea id="example-fr" lang="fr" value={editor.example_fr} disabled={notebook.busy} rows={3} aria-invalid={characterCount(editor.example_fr) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editExample('example_fr', event.target.value)} />
    <label htmlFor="example-zh">{text.exampleZh}</label><textarea id="example-zh" lang="zh" value={editor.example_zh} disabled={notebook.busy} rows={3} aria-invalid={characterCount(editor.example_zh) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editExample('example_zh', event.target.value)} />
    {editor.more_examples.map((example, index) => <fieldset key={index}><legend>{text.exampleUser} {index + 2}</legend>
      <label htmlFor={`example-fr-${index}`}>{text.exampleFr}</label><textarea id={`example-fr-${index}`} lang="fr" value={example.fr} disabled={notebook.busy} aria-invalid={characterCount(example.fr) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editMore(editor.more_examples.map((item, i) => i === index ? { ...item, fr: event.target.value } : item))} />
      <label htmlFor={`example-zh-${index}`}>{text.exampleZh}</label><textarea id={`example-zh-${index}`} lang="zh" value={example.zh} disabled={notebook.busy} aria-invalid={characterCount(example.zh) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editMore(editor.more_examples.map((item, i) => i === index ? { ...item, zh: event.target.value } : item))} />
      <button disabled={notebook.busy} onClick={() => notebook.editMore(editor.more_examples.filter((_, i) => i !== index))}>{text.removeExample}</button></fieldset>)}
    <button disabled={notebook.busy || editor.more_examples.length >= 100} onClick={() => notebook.editMore([...editor.more_examples, { fr: '', zh: '' }])}>{text.addExample}</button>
    {overLimit && <p id="example-limit" role="alert">{text.exampleLimit}</p>}</>;
}

export function VocabularyEditor({ notebook, text, viewExisting, runtime }: {
  readonly notebook: Notebook; readonly text: NotebookText; readonly viewExisting: () => void; readonly runtime: RuntimeAdapter;
}) {
  const textarea = useRef<HTMLTextAreaElement>(null);
  const headword = notebook.editor?.snapshot.headword;
  useEffect(() => { textarea.current?.focus(); }, [headword]);
  if (!notebook.editor) return null;
  const { snapshot, notes } = notebook.editor;
  const exampleOverLimit = exampleTooLong(notebook.editor);
  const overLimit = characterCount(notes) > 20_000 || exampleOverLimit;
  return <section className="vocabulary-editor" aria-label={text.edit}>
    <p className="eyebrow">{text.fallback}</p><VocabularySummary snapshot={snapshot} text={text} />
    <label htmlFor="personal-notes">{text.notes}</label>
    <textarea ref={textarea} id="personal-notes" value={notes} disabled={notebook.busy} rows={8} spellCheck={false}
      aria-invalid={characterCount(notes) > 20_000} aria-describedby="note-count" onChange={(event) => notebook.edit(event.target.value)} />
    <p id="note-count">{characterCount(notes).toLocaleString()} / 20,000 {text.count}</p>
    {characterCount(notes) > 20_000 && <p role="alert">{text.limit}</p>}
    <ExampleEditor notebook={notebook} text={text} />
    {notebook.error && <p role="alert">{text.saveError}: {notebook.error}</p>}
    {notebook.audioError && <p role="alert">{text.listeningPending}: {notebook.audioError}</p>}
    <SaveActions notebook={notebook} text={text} overLimit={overLimit} />
    {notebook.editor.record && <ReviewCards key={`${notebook.editor.record.id}-${notebook.version}`} record={notebook.editor.record} runtime={runtime} text={text} />}
    {notebook.existing && <p role="status">{text.existing} <button onClick={viewExisting}>{text.view}</button></p>}
    <Conflict notebook={notebook} text={text} /><SavedSource snapshot={snapshot} text={text} />
  </section>;
}
