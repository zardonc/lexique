import type { NotebookText } from './notebook-content';
import type { BatchReview } from './use-batch-review';

function FailedItems({ batch, text }: { readonly batch: BatchReview; readonly text: NotebookText }) {
  const items = batch.status?.items ?? [];
  return <ul>{items.filter((item) => item.error || item.audio_state === 'failed').slice(0, 100).map((item) => <li key={item.vocabulary_id}>
    #{item.vocabulary_id}: {item.error ?? item.audio_error}
    {!item.error && <button onClick={() => { void batch.retryAudio(item.vocabulary_id); }}>{text.retry} #{item.vocabulary_id}</button>}
  </li>)}</ul>;
}

function CommittedProgress({ batch, text }: { readonly batch: BatchReview; readonly text: NotebookText }) {
  const items = batch.status?.items ?? [];
  const created = items.reduce((total, item) => total + item.created_cards, 0);
  const existing = items.filter((item) => !item.error && item.created_cards === 0).length;
  const failed = items.filter((item) => item.error).length;
  const ready = items.filter((item) => item.audio_state === 'ready').length;
  const active = items.some((item) => ['queued', 'running', 'retry_wait'].includes(item.audio_state ?? ''));
  return items.length > 0 && <><p role="status">{text.cardsCreated}: {created} · {text.existing}: {existing} · {text.failed}: {failed} · {text.audioProgress}: {ready}/{items.length - failed}</p>
      {active && !batch.status?.cancelled && <button onClick={() => { void batch.cancel(); }}>{text.stopAudio}</button>}
      {batch.status?.cancelled && <p role="status">{text.audioCancelled}</p>}
      <FailedItems batch={batch} text={text} /></>;
}

export function BatchProgress({ batch, text }: { readonly batch: BatchReview; readonly text: NotebookText }) {
  return <section aria-label={text.batchProgress}>
    {batch.working && <p role="status">{text.saving}</p>}
    {batch.error && <p role="alert">{text.reviewError}: {batch.error}</p>}
    {batch.pending && <button disabled={batch.working} onClick={() => { void batch.retry(); }}>{text.retry}</button>}
    {batch.result && <p role="status">{text.success}: {batch.result}</p>}
    <CommittedProgress batch={batch} text={text} />
  </section>;
}
