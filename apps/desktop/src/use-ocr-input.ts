import { useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import type { RuntimeAdapter } from './platform/runtime';
import { selectionText, type SelectionEvent, type OcrImage } from './platform/selection';
import { errorMessage } from './platform/vocabulary';

const response = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('progress'), progress: z.number().min(0).max(1) }),
  z.object({ kind: z.literal('complete'), text: selectionText, confidence: z.number().min(0).max(100) }),
  z.object({ kind: z.literal('failed'), message: z.string().max(2000) }),
]);

function recognitionWorker(image: OcrImage, receive: (value: unknown) => void, failed: (message: string) => void) {
  const worker = new Worker(new URL('./ocr.worker.ts', import.meta.url), { type: 'module' });
  worker.onerror = (event) => { event.preventDefault(); failed(event.message); };
  worker.onmessage = (event: MessageEvent<unknown>) => receive(event.data);
  worker.postMessage({ image: image.data_url, base: new URL('ocr/', document.baseURI).href });
  return worker;
}

export function useOcrInput(runtime: RuntimeAdapter, selection: SelectionEvent | undefined, edit: (value: string) => void) {
  const [image, setImage] = useState<OcrImage>(), [busy, setBusy] = useState(false), [progress, setProgress] = useState(0);
  const [confidence, setConfidence] = useState<number>(), [error, setError] = useState(''), [empty, setEmpty] = useState(false);
  const request = useRef(0), worker = useRef<Worker | undefined>(undefined), timeout = useRef<number | undefined>(undefined);
  function stop() {
    request.current++; worker.current?.terminate(); worker.current = undefined;
    window.clearTimeout(timeout.current); timeout.current = undefined; setBusy(false);
  }
  function cancel() { stop(); setImage(undefined); setError(''); setEmpty(false); setConfidence(undefined); }
  useEffect(() => {
    stop();
    if (selection) { setImage(selection.kind === 'image' ? selection.image : undefined); setError(''); setEmpty(false); setConfidence(undefined); }
  }, [selection]);
  useEffect(() => { if (image && selection?.kind === 'image' && selection.recognize) recognize(); }, [image]);
  useEffect(() => () => { request.current++; worker.current?.terminate(); window.clearTimeout(timeout.current); }, []);
  async function importImage(file: File) {
    cancel(); const identity = request.current;
    setBusy(true);
    try {
      if (!file.size || file.size > 16 * 1024 * 1024) throw new Error('PNG/JPEG: 1–16 MiB');
      const bytes = new Uint8Array(await file.arrayBuffer());
      if (identity === request.current) await runtime.importOcrImage(bytes);
    } catch (cause) { if (identity === request.current) setError(errorMessage(cause)); }
    finally { if (identity === request.current) setBusy(false); }
  }
  function screen() { cancel(); void runtime.captureScreen().catch((cause: unknown) => setError(errorMessage(cause))); }
  function recognize() {
    if (!image) return;
    stop(); const identity = request.current;
    setBusy(true); setProgress(0); setError(''); setEmpty(false); setConfidence(undefined);
    function fail(message: string) { if (identity === request.current) { stop(); setError(message); } }
    function receive(value: unknown) {
      if (identity !== request.current) return;
      const parsed = response.safeParse(value);
      if (!parsed.success) { fail('Invalid OCR response'); return; }
      const result = parsed.data;
      if (result.kind === 'progress') setProgress(result.progress);
      else if (result.kind === 'failed') fail(result.message);
      else {
        stop(); setConfidence(result.confidence);
        if (result.text.trim()) edit(result.text); else setEmpty(true);
      }
    }
    try {
      worker.current = recognitionWorker(image, receive, fail);
      timeout.current = window.setTimeout(() => fail('OCR timed out after 30 seconds'), 30_000);
    } catch (cause) { fail(errorMessage(cause)); }
  }
  return { image, busy, progress, confidence, error, empty, importImage, recognize, screen, stop, cancel };
}
