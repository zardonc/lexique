import { useEffect, useMemo, useRef } from 'react';
import type { ReactNode } from 'react';
import { FrequencyBadge, FrequencyDetails, FrequencyState, useFrequency } from './frequency';
import type { NotebookText } from './notebook-content';
import { characterCount } from './platform/vocabulary';
import type { VocabularySnapshot } from './platform/vocabulary';
import type { Notebook } from './use-notebook';
import type { RuntimeAdapter } from './platform/runtime';
import { ReviewCards } from './review-cards';
import { ExampleSpeech } from './example-speech';
import { SpeechButton } from './speech-button';

export function VocabularySummary({ snapshot, text, frequency }: { readonly snapshot: VocabularySnapshot; readonly text: NotebookText; readonly frequency?: ReactNode }) {
  return <section className="vocabulary-summary" aria-label={text.summary}>
    <h3><span lang="fr">{snapshot.headword}</span> {frequency}</h3>
    {snapshot.digest.map((entry, index) => <dl key={index}>
      <dt>{text.pronunciation}</dt><dd lang="fr">{entry.pronunciations.join(' · ') || text.missing}</dd>
      <dt>{text.grammar}</dt><dd>{entry.grammar.join(' · ') || text.missing}</dd>
      <dt>{text.translation}</dt><dd lang="zh">{entry.translation || text.missing}</dd>
    </dl>)}
  </section>;
}

function SavedVocabularySummary({ snapshot, text, runtime }: {
  readonly snapshot: VocabularySnapshot; readonly text: NotebookText; readonly runtime: RuntimeAdapter;
}) {
  const keys = useMemo(() => [snapshot.headword], [snapshot.headword]);
  const frequencies = useFrequency(runtime, keys);
  const frequency = frequencies.lookup?.entries.find((entry) => entry.key === snapshot.headword);
  return <><VocabularySummary snapshot={snapshot} text={text} frequency={<FrequencyBadge entry={frequency} text={text.frequency} />} />
    <FrequencyState state={frequencies} text={text.frequency} />
    {frequencies.lookup && frequency && <FrequencyDetails lookup={frequencies.lookup} entry={frequency} text={text.frequency} />}
  </>;
}

function SavedSource({ snapshot, text }: { readonly snapshot: VocabularySnapshot; readonly text: NotebookText }) {
  const source = snapshot.reference;
  return <footer className="dictionary-source"><h4>{text.source}</h4>
    <p>{source.dictionary} · {source.version}</p><p>{source.attribution ?? text.missing} · {source.license ?? text.missing}</p>
    <p>{source.source_url ?? text.missing}</p><p>{source.license_url ?? text.missing}</p>
  </footer>;
}

function Conflict({ notebook, text }: { readonly notebook: Notebook; readonly text: NotebookText }) {
  if (!notebook.conflict) return null;
  return <section role="alert" className="runtime-error" aria-label={text.conflict}>
    <h4>{text.conflict}</h4><p>{text.current}</p><pre>{notebook.conflict.notes || '∅'}</pre>
    <p>{text.exampleFr}</p><pre>{notebook.conflict.example_fr || '∅'}</pre>
    <p>{text.exampleZh}</p><pre>{notebook.conflict.example_zh || '∅'}</pre>
    <p>{text.exampleSense}</p><pre>{notebook.conflict.example_sense === null ? text.exampleSenseUnspecified : notebook.conflict.digest[notebook.conflict.example_sense]?.translation || text.missing}</pre>
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
  return <div className="actions"><button disabled={overLimit || notebook.meaningRequired || notebook.busy || !notebook.dirty}
    onClick={() => { void notebook.save(); }}>{notebook.busy ? text.saving : text.save}</button>
    <span role="status">{notebook.saved ? text.saved : notebook.dirty ? text.unsaved : ''}</span>
    {notebook.meaningRequired && <p role="alert" className="runtime-error">{text.exampleSenseRequired}</p>}</div>;
}

function exampleTooLong(editor: NonNullable<Notebook['editor']>) {
  return [editor.example_fr, editor.example_zh, ...editor.more_examples.flatMap((e) => [e.fr, e.zh])].some((value) => characterCount(value) > 20_000);
}

function ExampleEditor({ notebook, text, runtime }: { readonly notebook: Notebook; readonly text: NotebookText; readonly runtime: RuntimeAdapter }) {
  const editor = notebook.editor!;
  const overLimit = exampleTooLong(editor);
  return <><h4>{text.exampleUser}</h4>
    <label htmlFor="example-sense">{text.exampleSense}</label><select id="example-sense" disabled={notebook.busy} aria-invalid={notebook.meaningRequired} aria-describedby={notebook.meaningRequired ? 'example-sense-required' : undefined} value={editor.example_sense === null ? '' : String(editor.example_sense)} onChange={(event) => notebook.editSense(event.target.value === '' ? null : Number(event.target.value))}>
      <option value="">{text.exampleSenseUnspecified}</option>{editor.snapshot.digest.map((entry, index) => <option key={index} value={index}>{entry.translation || text.missing}</option>)}
    </select>
    {notebook.meaningRequired && <p id="example-sense-required" className="runtime-error">{text.exampleSenseRequired}</p>}
    <label htmlFor="example-fr">{text.exampleFr}</label><textarea id="example-fr" lang="fr" value={editor.example_fr} disabled={notebook.busy} rows={3} aria-invalid={characterCount(editor.example_fr) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editExample('example_fr', event.target.value)} />
    <ExampleSpeech value={editor.example_fr} runtime={runtime} text={text} />
    <label htmlFor="example-zh">{text.exampleZh}</label><textarea id="example-zh" lang="zh" value={editor.example_zh} disabled={notebook.busy} rows={3} aria-invalid={characterCount(editor.example_zh) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editExample('example_zh', event.target.value)} />
    {editor.more_examples.map((example, index) => <fieldset key={index}><legend>{text.exampleUser} {index + 2}</legend>
      <label htmlFor={`example-fr-${index}`}>{text.exampleFr}</label><textarea id={`example-fr-${index}`} lang="fr" value={example.fr} disabled={notebook.busy} aria-invalid={characterCount(example.fr) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editMore(editor.more_examples.map((item, i) => i === index ? { ...item, fr: event.target.value } : item))} />
      <ExampleSpeech value={example.fr} runtime={runtime} text={text} />
      <label htmlFor={`example-zh-${index}`}>{text.exampleZh}</label><textarea id={`example-zh-${index}`} lang="zh" value={example.zh} disabled={notebook.busy} aria-invalid={characterCount(example.zh) > 20_000} aria-describedby={overLimit ? 'example-limit' : undefined} onChange={(event) => notebook.editMore(editor.more_examples.map((item, i) => i === index ? { ...item, zh: event.target.value } : item))} />
      <button disabled={notebook.busy} onClick={() => notebook.editMore(editor.more_examples.filter((_, i) => i !== index))}>{text.removeExample}</button></fieldset>)}
    <button disabled={notebook.busy || editor.more_examples.length >= 100} onClick={() => notebook.editMore([...editor.more_examples, { fr: '', zh: '' }])}>{text.addExample}</button>
    {overLimit && <p id="example-limit" role="alert">{text.exampleLimit}</p>}</>;
}

export function VocabularyEditor({ notebook, text, viewExisting, runtime, onReviewChanged }: {
  readonly notebook: Notebook; readonly text: NotebookText; readonly viewExisting: () => void; readonly runtime: RuntimeAdapter; readonly onReviewChanged: () => void;
}) {
  const textarea = useRef<HTMLTextAreaElement>(null);
  const headword = notebook.editor?.snapshot.headword;
  useEffect(() => { textarea.current?.focus(); }, [headword]);
  if (!notebook.editor) return null;
  const { snapshot, notes } = notebook.editor;
  const exampleOverLimit = exampleTooLong(notebook.editor);
  const overLimit = characterCount(notes) > 20_000 || exampleOverLimit;
  return <section className="vocabulary-editor" aria-label={text.edit}>
    <p className="eyebrow">{text.fallback}</p>{notebook.editor.record
      ? <SavedVocabularySummary snapshot={snapshot} text={text} runtime={runtime} /> : <VocabularySummary snapshot={snapshot} text={text} />}
    <SpeechButton key={snapshot.headword} word={snapshot.headword} runtime={runtime} text={text} />
    <label htmlFor="personal-notes">{text.notes}</label>
    <textarea ref={textarea} id="personal-notes" value={notes} disabled={notebook.busy} rows={8} spellCheck={false}
      aria-invalid={characterCount(notes) > 20_000} aria-describedby="note-count" onChange={(event) => notebook.edit(event.target.value)} />
    <p id="note-count">{characterCount(notes).toLocaleString()} / 20,000 {text.count}</p>
    {characterCount(notes) > 20_000 && <p role="alert">{text.limit}</p>}
    <ExampleEditor key={snapshot.headword} notebook={notebook} text={text} runtime={runtime} />
    {notebook.error && <p role="alert">{text.saveError}: {notebook.error}</p>}
    {notebook.audioError && <p role="alert">{text.listeningPending}: {notebook.audioError}</p>}
    <SaveActions notebook={notebook} text={text} overLimit={overLimit} />
    {notebook.editor.record && <ReviewCards key={`${notebook.editor.record.id}-${notebook.version}`} record={notebook.editor.record} runtime={runtime} text={text} onChanged={onReviewChanged} />}
    {notebook.existing && <p role="status">{text.existing} <button onClick={viewExisting}>{text.view}</button></p>}
    <Conflict notebook={notebook} text={text} /><SavedSource snapshot={snapshot} text={text} />
  </section>;
}
