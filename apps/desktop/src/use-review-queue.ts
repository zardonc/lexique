import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';
import type { QueueView, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { nextQueueWake, QueueRefresh, readQueue } from './queue-refresh';
import { subscribeReviewEvents } from './review-events';
import type { SpeechAudio } from './platform/speech';
import type { ReviewScope } from './platform/review-scope';

type Status = 'loading' | 'ready' | 'preview' | 'error';
interface Subscription {
  readonly scope: ReviewScope | null;
  readonly preserveScopePresentation: RefObject<boolean>;
  readonly runtime: RuntimeAdapter; readonly skipped: RefObject<number[]>; readonly advance: RefObject<number>;
  readonly current: RefObject<QueueView | null>; readonly controller: RefObject<QueueRefresh | null>;
  readonly latest: RefObject<QueueView | null>;
  readonly pendingMessage: RefObject<string>; readonly setWakeAt: (value: number | undefined) => void;
  readonly setError: (value: string) => void; readonly setStatus: (value: Status) => void;
  readonly setView: (value: QueueView | null) => void; readonly setAudio: (value: SpeechAudio | null) => void;
}

function preservePresentation(current: QueueView | null, result: QueueView | null) {
  const entry = current?.window[0];
  if (!entry || !result) return result;
  const window = [entry, ...result.window.filter((item) => item.card.id !== entry.card.id)].slice(0, 4);
  return { ...result, record: entry.record, window, queue: { ...result.queue, cards: window.map((item) => item.card) } };
}

function nextPresentation(result: QueueView | null, id: number | undefined) {
  if (!result) return null;
  const window = result.window.filter((entry) => entry.card.id !== id);
  const consumed = result.queue.cards.some((card) => card.id === id) ? 1 : 0;
  return { ...result, record: window[0]?.record ?? null, window, queue: { ...result.queue, cards: window.map((entry) => entry.card) },
    summary: { ...result.summary, admitted_count: Math.max(0, result.summary.admitted_count - consumed) } };
}

function queueAudio(result: QueueView | null, pending: string) {
  const card = result?.queue.cards[0];
  if (card?.kind !== 'listening') return null;
  const entry = result?.window.find((entry) => entry.card.id === card.id);
  const bytes = entry?.audio ?? null;
  if (!bytes) throw new Error(pending);
  return bytes;
}

function useQueueEvents(options: Subscription) {
  const { runtime, scope, preserveScopePresentation, skipped, advance, current, latest, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio } = options;
  useEffect(() => {
    if (!scope) return;
    advance.current += 1; latest.current = null;
    if (!preserveScopePresentation.current) { current.current = null; setView(null); setAudio(null); }
    setWakeAt(undefined); setStatus('loading'); setError('');
    let active = true;
    const refresh = new QueueRefresh({ visible: () => document.visibilityState !== 'hidden',
      failed: (cause) => { setError(errorMessage(cause)); setStatus('error'); },
      read: async () => {
        const version = advance.current;
        const valid = () => active && version === advance.current;
        const result = await readQueue(runtime, skipped.current);
        if (!valid()) return;
        if (result && JSON.stringify(result.summary.scope) !== JSON.stringify(scope)) throw new Error('Queue response does not match the applied review scope');
        latest.current = result;
        setWakeAt(nextQueueWake(result)); setError('');
        if (!current.current?.queue.cards.length) {
          const bytes = queueAudio(result, pendingMessage.current);
          if (!valid()) return;
          current.current = result; setView(result); setAudio(bytes);
        } else { current.current = preservePresentation(current.current, result); setView(current.current); }
        setStatus(result ? 'ready' : 'preview');
      } });
    controller.current = refresh;
    const resume = () => { if (document.visibilityState === 'hidden') { refresh.suspend(); setWakeAt(undefined); } else refresh.invalidate(); };
    const stop = subscribeReviewEvents(runtime, { changed: () => { advance.current += 1; latest.current = null; refresh.invalidate(); }, resumed: resume,
      failed: (cause) => setError(errorMessage(cause)) });
    refresh.invalidate(true);
    return () => { active = false; refresh.stop(); stop(); controller.current = null; };
  }, [runtime, scope, preserveScopePresentation, skipped, advance, current, latest, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio]);
}

export function useReviewQueue(runtime: RuntimeAdapter, pending: string, { applied: scope, preservePresentation: preserveScopePresentation }: {
  readonly applied: ReviewScope | null; readonly preservePresentation: RefObject<boolean>;
}) {
  const [view, setView] = useState<QueueView | null>(null);
  const [audio, setAudio] = useState<SpeechAudio | null>(null);
  const [status, setStatus] = useState<Status>('loading');
  const [error, setError] = useState('');
  const [wakeAt, setWakeAt] = useState<number>();
  const skipped = useRef<number[]>([]);
  const current = useRef<QueueView | null>(null);
  const latest = useRef<QueueView | null>(null);
  const advance = useRef(0);
  const controller = useRef<QueueRefresh | null>(null);
  const pendingMessage = useRef(pending); pendingMessage.current = pending;
  useQueueEvents({ runtime, scope, preserveScopePresentation, skipped, current, latest, advance, controller, pendingMessage, setWakeAt, setError, setStatus, setView, setAudio });
  useEffect(() => {
    if (wakeAt === undefined || document.visibilityState === 'hidden') return;
    const timer = window.setTimeout(() => controller.current?.invalidate(true), Math.min(2_147_483_647, Math.max(1, wakeAt - Date.now())));
    return () => window.clearTimeout(timer);
  }, [wakeAt]);
  async function refresh() {
    advance.current += 1; current.current = null; setView(null); setAudio(null); setStatus('loading');
    controller.current?.invalidate(true);
  }
  async function next() {
    advance.current += 1;
    const result = nextPresentation(latest.current, current.current?.queue.cards[0]?.id);
    latest.current = result; current.current = result; setView(result); setAudio(queueAudio(result, pendingMessage.current));
    setStatus(result ? 'ready' : 'loading'); controller.current?.invalidate(true);
  }
  function skip(id: number) { skipped.current = [...skipped.current, id]; void next(); }
  function retry() { setError(''); controller.current?.invalidate(true); }
  return { view, audio, status, error, refresh, next, retry, skip };
}
