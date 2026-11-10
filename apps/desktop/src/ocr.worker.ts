import { createWorker, OEM } from 'tesseract.js';

function failed(cause: unknown) {
  self.postMessage({ kind: 'failed', message: String(cause ?? 'Local OCR failed; check the offline resources').slice(0, 2000) });
}

self.onmessage = async (event: MessageEvent<{ image: string; base: string }>) => {
  let engine: Awaited<ReturnType<typeof createWorker>> | undefined;
  let workerUrl: string | undefined;
  try {
    const { base, image } = event.data;
    const script = await fetch(`${base}worker.min.js`);
    if (!script.ok) throw new Error(`Local OCR worker could not load: ${script.status}`);
    // shortcut: WebView2 cannot load nested custom-protocol workers; remove after Tauri #15755 is fixed.
    workerUrl = URL.createObjectURL(await script.blob());
    engine = await createWorker('fra', OEM.LSTM_ONLY, {
      workerPath: workerUrl, corePath: base.slice(0, -1), langPath: base.slice(0, -1),
      gzip: false, workerBlobURL: false, cacheMethod: 'none',
      logger: ({ progress }) => self.postMessage({ kind: 'progress', progress }),
      errorHandler: failed,
    });
    const { data } = await engine.recognize(image);
    self.postMessage({ kind: 'complete', text: data.text, confidence: data.confidence });
  } catch (cause) {
    failed(cause);
  } finally {
    await engine?.terminate();
    if (workerUrl) URL.revokeObjectURL(workerUrl);
  }
};
