import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import type { Notebook } from './use-notebook';
import type { NotebookText } from './notebook-content';

export function useLeaveGuard(runtime: RuntimeAdapter, notebook: Notebook) {
  const [pending, setPending] = useState<(() => void) | null>(null);
  const [error, setError] = useState('');
  function request(action: () => void) {
    if (notebook.busy) return;
    if (notebook.dirty) setPending(() => action);
    else action();
  }
  const latest = useRef({ request, notebook });
  useEffect(() => { latest.current = { request, notebook }; });
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const close = () => { void runtime.finishClose().catch((cause: unknown) => setError(errorMessage(cause))); };
    void runtime.onCloseRequested(() => latest.current.request(close)).then((cleanup) => {
      if (disposed) cleanup(); else unlisten = cleanup;
    }).catch((cause: unknown) => setError(errorMessage(cause)));
    function beforeUnload(event: BeforeUnloadEvent) {
      if (latest.current.notebook.dirty) { event.preventDefault(); event.returnValue = ''; }
    }
    window.addEventListener('beforeunload', beforeUnload);
    return () => { disposed = true; unlisten?.(); window.removeEventListener('beforeunload', beforeUnload); };
  }, [runtime]);
  function leave() { setPending(null); notebook.clear(); pending?.(); }
  async function saveLeave() { if (await notebook.save()) leave(); }
  return { request, pending: pending !== null, error, leave, saveLeave, stay: () => setPending(null) };
}

export function LeaveDialog({ guard, notebook, text }: {
  readonly guard: ReturnType<typeof useLeaveGuard>; readonly notebook: Notebook; readonly text: NotebookText;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { if (guard.pending) dialog.current?.showModal(); else dialog.current?.close(); }, [guard.pending]);
  return <dialog ref={dialog} aria-labelledby="leave-heading" onCancel={(event) => { event.preventDefault(); guard.stay(); }}>
    <h3 id="leave-heading">{text.leave}</h3>
    {notebook.error && <p role="alert">{text.saveError}: {notebook.error}</p>}
    {notebook.conflict && <p role="alert">{text.conflict}</p>}
    {notebook.existing && <p role="alert">{text.existing}</p>}
    <div className="actions"><button disabled={notebook.busy} onClick={() => { void guard.saveLeave(); }}>{text.saveLeave}</button>
      <button disabled={notebook.busy} onClick={guard.leave}>{text.discard}</button>
      <button disabled={notebook.busy} onClick={guard.stay}>{text.stay}</button></div>
  </dialog>;
}
