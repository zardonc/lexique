import { useEffect, useRef, useState } from 'react';
import type { BatchRequest, BatchStatus, RuntimeAdapter } from './platform/runtime';
import { batchRequestSchema } from './platform/runtime';
import type { VocabularyRecord } from './platform/vocabulary';
import { errorMessage } from './platform/vocabulary';
import { QueueRefresh } from './queue-refresh';

const pendingKey = 'lexique-pending-review-batch';

function pendingRequest(): BatchRequest | null {
  const saved = localStorage.getItem(pendingKey);
  if (!saved) return null;
  const value = batchRequestSchema.parse(JSON.parse(saved));
  if (value.vocabulary_ids.length > 100) throw new Error('Invalid pending review batch');
  return value;
}

function useBatchSubscription({ runtime, setPending, setError, setStatus }: {
  readonly runtime: RuntimeAdapter; readonly setPending: (value: BatchRequest | null) => void;
  readonly setError: (value: string) => void; readonly setStatus: (value: BatchStatus | null) => void;
}) {
  useEffect(() => {
    let active = true;
    try { setPending(pendingRequest()); } catch (cause) { setError(errorMessage(cause)); }
    const controller = new QueueRefresh({ visible: () => document.visibilityState !== 'hidden',
      failed: (cause) => { if (active) setError(errorMessage(cause)); },
      read: async () => { const value = await runtime.reviewBatchStatus(); if (active) setStatus(value); } });
    let unlisten: (() => void) | undefined;
    void runtime.onReviewChanged(() => controller.invalidate()).then((stop) => { if (active) unlisten = stop; else stop(); })
      .catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    const resume = () => { if (document.visibilityState === 'hidden') controller.suspend(); else controller.invalidate(); };
    document.addEventListener('visibilitychange', resume); window.addEventListener('focus', resume);
    controller.invalidate(true);
    return () => { active = false; controller.stop(); unlisten?.(); document.removeEventListener('visibilitychange', resume); window.removeEventListener('focus', resume); };
  }, [runtime, setPending, setError, setStatus]);
}

export function useBatchReview(runtime: RuntimeAdapter) {
  const [status, setStatus] = useState<BatchStatus | null>(null);
  const [pending, setPending] = useState<BatchRequest | null>(null);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState('');
  const [result, setResult] = useState('');
  const submitting = useRef(false);
  useBatchSubscription({ runtime, setPending, setError, setStatus });
  async function submit(request: BatchRequest) {
    if (submitting.current) return;
    submitting.current = true; setWorking(true); setError('');
    try {
      localStorage.setItem(pendingKey, JSON.stringify(request)); setPending(request);
      const value = await runtime.addReviewBatch(request);
      if (!value) throw new Error('Native review storage unavailable');
      setStatus(value); localStorage.removeItem(pendingKey); setPending(null);
    } catch (cause) { setError(errorMessage(cause)); }
    finally { submitting.current = false; setWorking(false); }
  }
  async function run(records: readonly VocabularyRecord[], action: 'add' | 'pause' | 'resume') {
    if (!records.length || submitting.current || pending) return;
    if (action === 'add') { await submit({ operation: crypto.randomUUID(), vocabulary_ids: records.map((record) => record.id) }); return; }
    submitting.current = true; setWorking(true); setError(''); setResult('');
    try {
      for (const record of records) {
        if (await runtime.reviewPause(record.id, action === 'pause') === null) throw new Error('Native review storage unavailable');
      }
      setResult(String(records.length));
    } catch (cause) { setError(errorMessage(cause)); }
    finally { submitting.current = false; setWorking(false); }
  }
  async function cancel() {
    if (!status) return;
    try { await runtime.cancelReviewBatch(status.operation); setStatus(await runtime.reviewBatchStatus(status.operation)); }
    catch (cause) { setError(errorMessage(cause)); }
  }
  async function retryAudio(id: number) {
    try { await runtime.generateSpeech(id, true); setStatus(await runtime.reviewBatchStatus()); }
    catch (cause) { setError(errorMessage(cause)); }
  }
  return { status, pending, working, error, result, run, cancel, retryAudio, retry: () => pending && submit(pending) };
}

export type BatchReview = ReturnType<typeof useBatchReview>;
