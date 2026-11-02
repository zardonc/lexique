import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';
import type { QueueView, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { nextQueueWake, QueueRefresh, readQueue } from './queue-refresh';
import { subscribeReviewEvents } from './review-events';

type Status = 'loading' | 'ready' | 'preview' | 'error';
interface Subscription {
  readonly runtime: RuntimeAdapter; readonly skipped: RefObject<number[]>; readonly advance: RefObject<number>;
  readonly current: RefObject<QueueView | null>; readonly controller: RefObject<QueueRefresh | null>;
  readonly pendingMessage: RefObject<string>; readonly setWakeAt: (value: number | undefined) => void;
  readonly setError: (value: string) => void; readonly setStatus: (value: Status) => void;
  readonly setView: (value: QueueView | null) => void; readonly setAudio: (value: Uint8Array | null) => void;
}

async function queueAudio(runtime: RuntimeAdapter, result: QueueView | null, pending: string) {
  const card = result?.queue.cards[0];
  if (card?.kind !== 'listening') return null;
  const bytes = await runtime.cachedSpeech(card.vocabulary_id);
  if (!bytes) throw new Error(pending);
  return bytes;
}

function useQueueEvents(options: Subscription) {
  const { runtime, skipped, advance, current, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio } = options;
  useEffect(() => {
    let active = true;
    const refresh = new QueueRefresh({ visible: () => document.visibilityState !== 'hidden',
      failed: (cause) => { setError(errorMessage(cause)); setStatus('error'); },
      read: async () => {
        const version = advance.current;
        const valid = () => active && version === advance.current;
        const result = await readQueue(runtime, skipped.current);
        if (!valid()) return;
        setWakeAt(nextQueueWake(result)); setError('');
        if (!current.current?.queue.cards.length) {
          const bytes = await queueAudio(runtime, result, pendingMessage.current);
          if (!valid()) return;
          current.current = result; setView(result); setAudio(bytes);
        }
        setStatus(result ? 'ready' : 'preview');
      } });
    controller.current = refresh;
    const resume = () => { if (document.visibilityState === 'hidden') { refresh.suspend(); setWakeAt(undefined); } else refresh.invalidate(); };
    const stop = subscribeReviewEvents(runtime, { changed: () => refresh.invalidate(), resumed: resume,
      failed: (cause) => setError(errorMessage(cause)) });
    refresh.invalidate(true);
    return () => { active = false; refresh.stop(); stop(); controller.current = null; };
  }, [runtime, skipped, advance, current, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio]);
}

export function useReviewQueue(runtime: RuntimeAdapter, pending: string) {
  const [view, setView] = useState<QueueView | null>(null);
  const [audio, setAudio] = useState<Uint8Array | null>(null);
  const [status, setStatus] = useState<Status>('loading');
  const [error, setError] = useState('');
  const [wakeAt, setWakeAt] = useState<number>();
  const skipped = useRef<number[]>([]);
  const current = useRef<QueueView | null>(null);
  const advance = useRef(0);
  const controller = useRef<QueueRefresh | null>(null);
  const pendingMessage = useRef(pending); pendingMessage.current = pending;
  useQueueEvents({ runtime, skipped, current, advance, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio });
  useEffect(() => {
    if (wakeAt === undefined || document.visibilityState === 'hidden') return;
    const timer = window.setTimeout(() => controller.current?.invalidate(true), Math.min(2_147_483_647, Math.max(1, wakeAt - Date.now())));
    return () => window.clearTimeout(timer);
  }, [wakeAt]);
  async function refresh() {
    advance.current += 1; current.current = null; setView(null); setAudio(null); setStatus('loading');
    controller.current?.invalidate(true);
  }
  function skip(id: number) { skipped.current = [...skipped.current, id]; void refresh(); }
  function retry() { setError(''); controller.current?.invalidate(true); }
  return { view, audio, status, error, refresh, retry, skip };
}
