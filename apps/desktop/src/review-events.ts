import type { RuntimeAdapter } from './platform/runtime';

export function subscribeReviewEvents(runtime: RuntimeAdapter, callbacks: {
  readonly changed: () => void; readonly resumed: () => void; readonly failed: (cause: unknown) => void;
}) {
  let active = true;
  let unlisten: (() => void) | undefined;
  const changed = () => { if (active) callbacks.changed(); };
  const resumed = () => { if (active) callbacks.resumed(); };
  void runtime.onReviewChanged(changed).then((stop) => { if (active) unlisten = stop; else stop(); })
    .catch((cause: unknown) => { if (active) callbacks.failed(cause); });
  window.addEventListener('focus', resumed); window.addEventListener('pageshow', resumed);
  document.addEventListener('visibilitychange', resumed);
  return () => {
    active = false; unlisten?.();
    window.removeEventListener('focus', resumed); window.removeEventListener('pageshow', resumed);
    document.removeEventListener('visibilitychange', resumed);
  };
}
