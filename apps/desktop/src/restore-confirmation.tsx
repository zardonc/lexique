import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { FormalRestore } from './backup-restore';
import type { useBackupPreview } from './backup-preview';
import type { BackupPreviewStatus } from './platform/backup';
import type { BatchStatus, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { capturePendingReviews, releasePendingReviews, resolvePendingReviews } from './pending-review-requests';
import { restoreMessages } from './restore-content';

interface RestorePreparation {
  readonly runtime: RuntimeAdapter; readonly preview: ReturnType<typeof useBackupPreview>; readonly restore: FormalRestore;
  readonly locale: Locale; readonly onResolved: (batch: BatchStatus | null) => void; readonly onPreparing: (value: boolean) => void;
  readonly blocked: boolean;
  readonly recovery?: boolean;
}

function verifiedPreview(options: RestorePreparation, operation: string) {
  const selected = options.preview.current.current;
  return selected?.state === 'preview_ready' && selected.operation === operation ? selected : null;
}
function previewFilename(status: BackupPreviewStatus | null) { return status?.preview?.filename ?? ''; }
async function preparePendingRequests(options: RestorePreparation) {
  if (options.recovery) return capturePendingReviews();
  options.onResolved(await resolvePendingReviews(options.runtime, true));
}

function useRestorePreparation(options: RestorePreparation) {
  const [error, setError] = useState(''), pending = useRef(false), live = useRef(true);
  const latest = useRef(options);
  useEffect(() => { latest.current = options; });
  useEffect(() => { live.current = true; return () => { live.current = false; }; }, []);
  async function start(operation: string) {
    const value = latest.current;
    if (pending.current || value.blocked) return;
    const selected = verifiedPreview(value, operation);
    if (!selected) { setError('The verified backup is no longer available. Choose and verify it again.'); return; }
    pending.current = true; value.onPreparing(true); setError('');
    try {
      const requests = await preparePendingRequests(value);
      if (!live.current) { releasePendingReviews(); return; }
      if (!verifiedPreview(value, operation)) throw new Error('The verified backup is no longer available. Choose and verify it again.');
      value.preview.reset();
      await value.restore.start(selected, value.locale, requests);
    } catch (cause) { releasePendingReviews(); if (live.current) setError(errorMessage(cause)); }
    finally { pending.current = false; if (live.current) value.onPreparing(false); }
  }
  return { start, error };
}

function WholeBackupDialog({ locale, filename, open, disabled, close, confirm, recovery }: {
  readonly locale: Locale; readonly filename: string; readonly open: boolean; readonly disabled: boolean;
  readonly close: () => void; readonly confirm: () => void;
  readonly recovery?: boolean;
}) {
  const text = restoreMessages[locale], [checked, setChecked] = useState(false), dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    setChecked(false);
    if (open) dialog.current?.showModal(); else dialog.current?.close();
  }, [open]);
  return <dialog ref={dialog} aria-labelledby="restore-confirm-heading" onCancel={(event) => { event.preventDefault(); close(); }}>
    <h3 id="restore-confirm-heading">{text.confirmation}</h3><p>{filename}</p>
    <p>{text.warning}</p><p>{recovery ? text.rawPending : text.pending}</p>
    {recovery && <><p>{text.rawCopy}</p><p>{text.deviceSettings}</p></>}
    <label><input type="checkbox" checked={checked} onChange={(event) => setChecked(event.target.checked)} /> {text.acknowledge}</label>
    <div className="actions"><button disabled={!checked || disabled} onClick={() => { if (checked && !disabled) confirm(); }}>{text.confirm}</button>
      <button onClick={close}>{text.cancel}</button></div>
  </dialog>;
}

export function RestoreConfirmation({ beforeStart, preparing, ...options }: RestorePreparation & {
  readonly beforeStart: (action: () => void) => void; readonly preparing: boolean;
}) {
  const confirmation = useRestorePreparation(options), text = restoreMessages[options.locale], { status } = options.preview;
  const blocked = preparing || options.blocked || options.preview.pending;
  const [open, setOpen] = useState(false);
  useEffect(() => { setOpen(false); }, [status?.operation, status?.state]);
  function confirm() {
    if (status?.state !== 'preview_ready' || preparing || options.blocked) return;
    const operation = status.operation;
    setOpen(false); beforeStart(() => { void confirmation.start(operation); });
  }
  return <>
    {status?.state === 'preview_ready' && <button disabled={blocked} onClick={() => setOpen(true)}>{text.start}</button>}
    {preparing && <p role="status">{options.recovery ? text.rawConfirming : text.confirming}</p>}
    {confirmation.error && <p role="alert">{text.guardFailed}: {confirmation.error}</p>}
    <WholeBackupDialog locale={options.locale} filename={previewFilename(status)} open={open} disabled={blocked} recovery={options.recovery}
      close={() => setOpen(false)} confirm={confirm} />
  </>;
}
