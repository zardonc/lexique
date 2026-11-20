import { useState } from 'react';
import type { GrammarRecord } from './platform/notes';
import type { RuntimeAdapter } from './platform/runtime';
import type { NotebookText } from './notebook-content';
import { ReviewRating } from './review-rating';

export function GrammarReviewCard({ record, text, runtime, onRated, onSkip, enabled, progressRevision }: {
  readonly record: GrammarRecord; readonly text: NotebookText; readonly runtime: RuntimeAdapter;
  readonly onRated: () => void; readonly onSkip: () => void; readonly enabled: boolean; readonly progressRevision: number;
}) {
  const [revealed, setRevealed] = useState(false), [busy, setBusy] = useState(false);
  if (!record.card) return null;
  return <section className="queue-review-card grammar-review-card" aria-label={text.queueCard}>
    <p className="eyebrow">{text.notesText.card}</p><h3 className="grammar-text">{record.card.question}</h3>
    {revealed && <><div className="grammar-text">{record.card.answer}</div>
      <ReviewRating id={record.card.id} runtime={runtime} text={text} enabled={enabled} presentationRevision={progressRevision} onBusyChange={setBusy} onSuccess={onRated} /></>}
    <div className="actions">{!revealed && <button onClick={() => setRevealed(true)}>{text.reveal}</button>}
      <button disabled={busy} onClick={onSkip}>{text.skip}</button></div>
  </section>;
}
