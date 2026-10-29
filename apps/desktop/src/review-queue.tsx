import { useCallback, useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { NotebookText } from './notebook-content';
import type { QueueView, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { ListeningCard, RecognitionCard } from './review-cards';

const messages = {
  zh: { loading: '正在读取复习队列', preview: '浏览器预览无法读取复习队列。', done: '本轮完成', waiting: '尚未到期', quota: '今日额度已用完', error: '队列读取失败', retry: '刷新队列' },
  en: { loading: 'Loading review queue', preview: 'Review queue is unavailable in browser preview.', done: 'Review complete', waiting: 'Not due yet', quota: 'Daily limit reached', error: 'Could not load review queue', retry: 'Refresh queue' },
  fr: { loading: 'Chargement de la file', preview: 'La file est indisponible dans l’aperçu web.', done: 'Révision terminée', waiting: 'Pas encore à réviser', quota: 'Limite quotidienne atteinte', error: 'Lecture de la file impossible', retry: 'Actualiser la file' },
} as const;

async function queueAudio(runtime: RuntimeAdapter, view: QueueView | null, pending: string) {
  const first = view?.queue.cards[0];
  if (first?.kind !== 'listening') return null;
  const audio = await runtime.cachedSpeech(first.vocabulary_id);
  if (!audio) throw new Error(pending);
  return audio;
}

function useQueue(runtime: RuntimeAdapter, pending: string) {
  const [view, setView] = useState<QueueView | null>(null);
  const [audio, setAudio] = useState<Uint8Array | null>(null);
  const [status, setStatus] = useState<'loading' | 'ready' | 'preview' | 'error'>('loading');
  const [error, setError] = useState('');
  const skipped = useRef<number[]>([]);
  const generation = useRef(0);
  const refresh = useCallback(async () => {
    const request = ++generation.current;
    setStatus('loading'); setError('');
    try {
      const result = await runtime.reviewQueue(skipped.current);
      const bytes = await queueAudio(runtime, result, pending);
      if (request !== generation.current) return;
      setAudio(bytes); setView(result); setStatus(result ? 'ready' : 'preview');
    } catch (cause) { if (request === generation.current) { setError(errorMessage(cause)); setStatus('error'); } }
  }, [runtime, pending]);
  useEffect(() => { void refresh(); return () => { generation.current += 1; }; }, [refresh]);
  function skip(id: number) { skipped.current = [...skipped.current, id]; void refresh(); }
  return { view, audio, status, error, refresh, skip };
}

function QueueCard({ view, audio, runtime, text, refresh, skip }: { readonly view: QueueView; readonly audio: Uint8Array | null; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly refresh: () => Promise<void>; readonly skip: (id: number) => void }) {
  const card = view.queue.cards[0];
  const record = view.record;
  if (!card || !record) return null;
  if (card.kind === 'recognition') return <RecognitionCard key={card.id} card={card} record={record} runtime={runtime} text={text} onRated={() => { void refresh(); }} />;
  return audio && <ListeningCard key={card.id} card={card} record={record} runtime={runtime} text={text} audio={audio} onPlayback={() => {}} onRated={() => { void refresh(); }} onSkip={() => skip(card.id)} />;
}

function emptyMessage(view: QueueView, labels: (typeof messages)[Locale], text: NotebookText) {
  if (view.queue.reason === 'quota') return labels.quota;
  if (view.queue.reason === 'clock') return text.availability.clock;
  if (view.queue.reason) return labels.waiting;
  if (view.audio_error) return labels.error;
  if (view.audio_pending) return text.listeningPending;
  return labels.done;
}

function ReadyQueue({ view, audio, runtime, text, refresh, labels, skip }: { readonly view: QueueView; readonly audio: Uint8Array | null; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly refresh: () => Promise<void>; readonly labels: (typeof messages)[Locale]; readonly skip: (id: number) => void }) {
  return <>{view.audio_error && <p role="alert">{text.playbackError}: {view.audio_error}</p>}
    {view.queue.cards.length ? <QueueCard view={view} audio={audio} runtime={runtime} text={text} refresh={refresh} skip={skip} /> :
      <p role="status">{emptyMessage(view, labels, text)}
        {view.queue.available_at != null && `: ${new Date(view.queue.available_at * 1000).toLocaleString()}`}</p>}</>;
}

export function ReviewQueuePanel({ runtime, locale, text }: { readonly runtime: RuntimeAdapter; readonly locale: Locale; readonly text: NotebookText }) {
  const labels = messages[locale];
  const { view, audio, status, error, refresh, skip } = useQueue(runtime, text.listeningPending);
  return <section aria-label={labels.loading}>
    {status === 'loading' && <p role="status">{labels.loading}</p>}
    {status === 'preview' && <p>{labels.preview}</p>}
    {error && <p role="alert">{labels.error}: {error}</p>}
    {status === 'ready' && view && <ReadyQueue view={view} audio={audio} runtime={runtime} text={text} refresh={refresh} labels={labels} skip={skip} />}
    <button disabled={status === 'loading'} onClick={() => { void refresh(); }}>{labels.retry}</button>
  </section>;
}
