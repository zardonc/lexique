import { useEffect, useRef } from 'react';
import type { NotebookText } from './notebook-content';
import { characterCount } from './platform/vocabulary';
import type { VocabularySnapshot } from './platform/vocabulary';
import type { Notebook } from './use-notebook';

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

export function VocabularyEditor({ notebook, text, viewExisting }: {
  readonly notebook: Notebook; readonly text: NotebookText; readonly viewExisting: () => void;
}) {
  const textarea = useRef<HTMLTextAreaElement>(null);
  const headword = notebook.editor?.snapshot.headword;
  useEffect(() => { textarea.current?.focus(); }, [headword]);
  if (!notebook.editor) return null;
  const { snapshot, notes } = notebook.editor;
  const overLimit = characterCount(notes) > 20_000;
  return <section className="vocabulary-editor" aria-label={text.edit}>
    <p className="eyebrow">{text.fallback}</p><VocabularySummary snapshot={snapshot} text={text} />
    <label htmlFor="personal-notes">{text.notes}</label>
    <textarea ref={textarea} id="personal-notes" value={notes} disabled={notebook.busy} rows={8} spellCheck={false}
      aria-invalid={overLimit} aria-describedby="note-count" onChange={(event) => notebook.edit(event.target.value)} />
    <p id="note-count">{characterCount(notes).toLocaleString()} / 20,000 {text.count}</p>
    {overLimit && <p role="alert">{text.limit}</p>}
    {notebook.error && <p role="alert">{text.saveError}: {notebook.error}</p>}
    <SaveActions notebook={notebook} text={text} overLimit={overLimit} />
    {notebook.existing && <p role="status">{text.existing} <button onClick={viewExisting}>{text.view}</button></p>}
    <Conflict notebook={notebook} text={text} /><SavedSource snapshot={snapshot} text={text} />
  </section>;
}
