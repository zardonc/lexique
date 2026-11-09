import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import type { SelectionEvent } from './platform/selection';
import { errorMessage } from './platform/vocabulary';

export function useSelectionInput(runtime: RuntimeAdapter, page: number, open: (action: () => void) => void) {
  const [input, setInput] = useState<SelectionEvent>();
  const [error, setError] = useState('');
  const latest = useRef<SelectionEvent | undefined>(undefined);
  const accepted = useRef(0), ignored = useRef(0), beforeOpen = useRef(open);
  useEffect(() => { beforeOpen.current = open; });
  function cancel() {
    ignored.current = latest.current?.request ?? ignored.current;
    accepted.current = 0; setInput(undefined);
    void runtime.cancelSelection().catch((cause: unknown) => setError(errorMessage(cause)));
  }
  function local() {
    const field = document.activeElement;
    let text = window.getSelection()?.toString() ?? '';
    if (field instanceof HTMLTextAreaElement || (field instanceof HTMLInputElement && ['text', 'search'].includes(field.type)))
      text = field.value.slice(field.selectionStart ?? 0, field.selectionEnd ?? 0);
    void runtime.captureSelection(text).catch((cause: unknown) => setError(errorMessage(cause)));
  }
  useEffect(() => {
    if (page !== 5 && accepted.current) cancel();
  }, [page]); // Only leaving the input page invalidates an accepted capture.
  useEffect(() => {
    let disposed = false, unlisten: (() => void) | undefined;
    function receive(event: SelectionEvent) {
      if (disposed || event.request <= ignored.current || event.request < (latest.current?.request ?? 0)) return;
      const first = event.request !== latest.current?.request;
      latest.current = event;
      if (first) beforeOpen.current(() => {
        if (latest.current?.request !== event.request || event.request <= ignored.current) return;
        accepted.current = event.request; setInput(latest.current);
      });
      else if (accepted.current === event.request) setInput(event);
    }
    void runtime.onSelectionInput(receive).then((cleanup) => {
      if (disposed) cleanup(); else unlisten = cleanup;
    }).catch((cause: unknown) => setError(errorMessage(cause)));
    return () => { disposed = true; unlisten?.(); };
  }, [runtime]);
  return { input, error, cancel, local };
}
