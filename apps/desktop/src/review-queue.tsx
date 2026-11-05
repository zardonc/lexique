import { useEffect, useState } from 'react';
import { useReviewQueue } from './use-review-queue';
import type { Locale } from './content';
import type { NotebookText } from './notebook-content';
import type { QueueView, RuntimeAdapter } from './platform/runtime';
import { UndoRejected } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { QueueReviewCard } from './review-cards';

const messages = {
  zh: { loading: '正在读取复习队列', preview: '浏览器预览无法读取复习队列。', done: '本轮完成', waiting: '尚未到期', quota: '今日额度已用完', verifying: '听音待校验', error: '队列读取失败', retry: '刷新队列', undo: '撤销最近评分', undoError: '撤销失败', rebuilt: '旧版评分已撤销，队列顺序可能变化。', counts: (count: number) => `待复习：${count} 张` },
  en: { loading: 'Loading review queue', preview: 'Review queue is unavailable in browser preview.', done: 'Review complete', waiting: 'Not due yet', quota: 'Daily limit reached', verifying: 'Audio awaiting verification', error: 'Could not load review queue', retry: 'Refresh queue', undo: 'Undo last rating', undoError: 'Could not undo rating', rebuilt: 'Earlier rating undone. The queue order may change.', counts: (count: number) => `Cards remaining: ${count}` },
  fr: { loading: 'Chargement de la file', preview: 'La file est indisponible dans l’aperçu web.', done: 'Révision terminée', waiting: 'Pas encore à réviser', quota: 'Limite quotidienne atteinte', verifying: 'Audio en attente de vérification', error: 'Lecture de la file impossible', retry: 'Actualiser la file', undo: 'Annuler la dernière évaluation', undoError: 'Annulation impossible', rebuilt: 'Ancienne évaluation annulée. L’ordre de la file peut changer.', counts: (count: number) => `Cartes restantes : ${count}` },
} as const;


function QueueCard({ view, audio, runtime, text, refresh, skip }: { readonly view: QueueView; readonly audio: Uint8Array | null; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly refresh: () => Promise<void>; readonly skip: (id: number) => void }) {
  const card = view.queue.cards[0];
  const record = view.record;
  if (!card || !record) return null;
  if (card.kind === 'listening' && !audio) return null;
  const enabled = view.summary.admitted_count > 0 || view.queue.reason === null;
  return <QueueReviewCard key={card.id} card={card} record={record} runtime={runtime} text={text} enabled={enabled} progressRevision={view.window[0].progress.revision} audio={card.kind === 'listening' ? audio : null} onRated={() => { void refresh(); }} onSkip={() => skip(card.id)} />;
}

function emptyMessage(view: QueueView, labels: (typeof messages)[Locale], text: NotebookText) {
  if (view.queue.reason === 'quota') return labels.quota;
  if (view.queue.reason === 'clock') return text.availability.clock;
  if (view.queue.reason === 'deferred') return text.availability.deferred;
  if (view.queue.reason) return labels.waiting;
  if (view.audio_error) return labels.error;
  if (view.summary.audio_pending_count > 0) return text.listeningPending;
  if (view.summary.verification_pending) return labels.verifying;
  return labels.done;
}

function showVerificationNotice(view: QueueView) {
  return view.summary.verification_pending && Boolean(view.queue.cards.length > 0 || view.summary.audio_pending_count > 0 || view.queue.reason || view.audio_error);
}

function ReadyQueue({ view, audio, runtime, text, refresh, labels, skip }: { readonly view: QueueView; readonly audio: Uint8Array | null; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly refresh: () => Promise<void>; readonly labels: (typeof messages)[Locale]; readonly skip: (id: number) => void }) {
  return <>{view.audio_error && <p role="alert">{text.playbackError}: {view.audio_error}</p>}
    {view.summary.unknown_legacy && <p role="status">{text.unknownLegacyRepair}</p>}
    {showVerificationNotice(view) && <p role="status">{labels.verifying}</p>}
    {view.queue.cards.length > 0 && <p role="status">{labels.counts(view.summary.admitted_count)}</p>}
    {view.queue.cards.length > 0 && view.summary.admitted_count === 0 && view.queue.reason && <p role="status">{emptyMessage(view, labels, text)}</p>}
    {view.queue.cards.length ? <QueueCard view={view} audio={audio} runtime={runtime} text={text} refresh={refresh} skip={skip} /> :
      <p role="status">{emptyMessage(view, labels, text)}
        {view.queue.available_at != null && `: ${new Date(view.queue.available_at * 1000).toLocaleString()}`}</p>}</>;
}

export function ReviewQueuePanel({ runtime, locale, text }: { readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly text: NotebookText }) {
  const labels = messages[locale];
  const [rebuilt, setRebuilt] = useState(false);
  const [ratingVersion, setRatingVersion] = useState(0);
  const { view, audio, status, error, refresh, next, retry, skip } = useReviewQueue(runtime, text.listeningPending);
  async function rated() { setRatingVersion((version) => version + 1); await next(); }
  return <section aria-label={labels.loading}>
    {status === 'loading' && <p role="status">{labels.loading}</p>}
    {status === 'preview' && <p>{labels.preview}</p>}
    {error && <p role="alert">{labels.error}: {error}</p>}
    {view && <ReadyQueue view={view} audio={audio} runtime={runtime} text={text} refresh={rated} labels={labels} skip={skip} />}
    {rebuilt && <p role="status">{labels.rebuilt}</p>}
    <QueueActions status={status} view={view} runtime={runtime} text={text} labels={labels} refresh={refresh} retry={retry} setRebuilt={setRebuilt} ratingVersion={ratingVersion} />
  </section>;
}

function QueueActions({ status, view, runtime, text, labels, refresh, retry, setRebuilt, ratingVersion }: {
  readonly status: string; readonly view: QueueView | null; readonly runtime: RuntimeAdapter; readonly text: NotebookText;
  readonly labels: (typeof messages)[Locale]; readonly refresh: () => Promise<void>; readonly setRebuilt: (value: boolean) => void;
  readonly retry: () => void;
  readonly ratingVersion: number;
}) {
  return <div className="actions">{status !== 'preview' && status !== 'loading' && <UndoRating runtime={runtime} labels={labels} refresh={refresh} onRebuilt={setRebuilt} ratingVersion={ratingVersion} />}
    {(status === 'error' || view?.audio_error) && <button onClick={retry}>{text.retry}</button>}
    {view?.audio_pending && view.audio_vocabulary_id !== null && <MissingAudio runtime={runtime} text={text} id={view.audio_vocabulary_id} />}</div>;
}

function MissingAudio({ runtime, text, id }: { readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly id: number }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  async function retry() {
    setBusy(true); setError('');
    try { if (await runtime.retryMissingSpeech(id) === null) throw new Error(text.preview); }
    catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return <>{error && <p role="alert">{error}</p>}<button disabled={busy} onClick={() => { void retry(); }}>{text.retry} · {text.listeningPending}</button></>;
}

function undoRequest(target: string | null) {
  const saved = localStorage.getItem('lexique-pending-undo');
  const pending: unknown = saved ? JSON.parse(saved) : { operation: crypto.randomUUID(), ratingOperation: target };
  if (!pending || typeof pending !== 'object' || !('operation' in pending) || typeof pending.operation !== 'string' || !('ratingOperation' in pending) || typeof pending.ratingOperation !== 'string') throw new Error('Invalid pending undo');
  return { operation: pending.operation, ratingOperation: pending.ratingOperation };
}

function UndoRating({ runtime, labels, refresh, onRebuilt, ratingVersion }: { readonly runtime: RuntimeAdapter; readonly labels: (typeof messages)[Locale]; readonly refresh: () => Promise<void>; readonly onRebuilt: (value: boolean) => void; readonly ratingVersion: number }) {
  const [target, setTarget] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [pending, setPending] = useState(false);
  useEffect(() => {
    let active = true;
    try { setPending(localStorage.getItem('lexique-pending-undo') !== null); }
    catch (cause) { setError(errorMessage(cause)); }
    void runtime.undoTarget().then((value) => { if (active) setTarget(value); }).catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [runtime, ratingVersion]);
  async function undo() {
    setBusy(true); setError('');
    try {
      const key = 'lexique-pending-undo';
      const pending = undoRequest(target);
      localStorage.setItem(key, JSON.stringify(pending));
      setPending(true);
      const result = await runtime.undoReview(pending.operation, pending.ratingOperation);
      if (!result) throw new Error('Native undo unavailable');
      onRebuilt(result.queue_rebuilt);
      localStorage.removeItem(key);
      setPending(false);
      setTarget(null);
      await refresh();
    } catch (cause) {
      setError(errorMessage(cause));
      if (cause instanceof UndoRejected) {
        try { localStorage.removeItem('lexique-pending-undo'); setPending(false); setTarget(await runtime.undoTarget()); }
        catch (readError) { setError(errorMessage(readError)); }
      }
    }
    finally { setBusy(false); }
  }
  return <>{error && <p role="alert">{labels.undoError}: {error}</p>}<button disabled={busy || (!target && !pending)} onClick={() => { void undo(); }}>{labels.undo}</button></>;
}
