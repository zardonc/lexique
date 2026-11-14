import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import type { BackupPreviewStatus, RestoreStatus, StartupStatus } from './platform/backup';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { releasePendingReviews } from './pending-review-requests';
import { RetainedCopies } from './retained-copies';
import { restoreMessages } from './restore-content';
import { RecoveryClose } from './startup';
import type { useStartup } from './startup';

const phaseOrder = { preserving: 1, preparing: 2, closing: 3 } as const;
function acceptsRestore(previous: RestoreStatus | null, value: RestoreStatus) {
  if (!previous) return true;
  if (previous.state !== 'running') return value.state === previous.state;
  return value.state !== 'running' || phaseOrder[value.phase] >= phaseOrder[previous.phase];
}

function useRestoreStatus(runtime: RuntimeAdapter) {
  const [status, setStatus] = useState<RestoreStatus | null>(null), [error, setError] = useState(''), [active, setActive] = useState(false);
  const current = useRef<RestoreStatus | null>(null), identity = useRef<string | null>(null), updates = useRef(0), live = useRef(true);
  const unlisten = useRef<(() => void) | undefined>(undefined);
  function receive(value: RestoreStatus) {
    if (!live.current || value.operation !== identity.current || !acceptsRestore(current.current, value)) return;
    current.current = value; updates.current += 1; setStatus(value); setError('');
    if (value.state === 'failed') { unlisten.current?.(); unlisten.current = undefined; }
  }
  function failed(cause: unknown) { if (live.current) setError(errorMessage(cause)); }
  function begin(operation: string) {
    identity.current = operation; current.current = null; updates.current += 1;
    setStatus(null); setError(''); setActive(true);
  }
  function finish() {
    unlisten.current?.(); unlisten.current = undefined;
    identity.current = null; updates.current += 1; setActive(false); releasePendingReviews();
  }
  async function refresh() {
    const operation = identity.current, version = updates.current;
    if (!operation) return;
    try { const value = await runtime.restoreStatus(operation); if (version === updates.current) receive(value); }
    catch (cause) { if (version === updates.current) failed(cause); }
  }
  useEffect(() => {
    live.current = true;
    return () => { live.current = false; updates.current += 1; unlisten.current?.(); };
  }, [runtime]);
  return { active, status, error, current, identity, updates, live, unlisten, receive, failed, begin, finish, refresh };
}

function useRestoreStart(runtime: RuntimeAdapter, operation: ReturnType<typeof useRestoreStatus>) {
  const pending = useRef(false), [starting, setStarting] = useState(false), [rejected, setRejected] = useState(false);
  function owns(id: string) { return operation.live.current && operation.identity.current === id; }
  function fail(cause: unknown, version: number) {
    if (version === operation.updates.current) operation.failed(cause);
    if (operation.live.current && !operation.current.current) setRejected(typeof cause === 'string');
  }
  async function start(id: string) {
    if (pending.current || operation.identity.current || !runtime.backupAvailable()) return;
    pending.current = true; setStarting(true); setRejected(false); operation.begin(id);
    let version = operation.updates.current;
    try {
      const stop = await runtime.onRestoreProgress(operation.receive, operation.failed);
      if (!owns(id)) { stop(); return; }
      operation.unlisten.current = stop; version = operation.updates.current;
      const value = await runtime.startRestore(id);
      if (version === operation.updates.current) operation.receive(value);
      if (operation.current.current?.state === 'running') await operation.refresh();
    } catch (cause) { fail(cause, version); }
    finally { pending.current = false; if (operation.live.current) setStarting(false); }
  }
  return { start, starting, rejected };
}

export function useFormalRestore(runtime: RuntimeAdapter, startup: ReturnType<typeof useStartup>) {
  const operation = useRestoreStatus(runtime), request = useRestoreStart(runtime, operation);
  const [locale, setLocale] = useState<Locale>('zh'), [checking, setChecking] = useState(false);
  async function start(preview: Extract<BackupPreviewStatus, { state: 'preview_ready' }>, language: Locale) {
    if (operation.identity.current) return;
    setLocale(language); await request.start(preview.operation);
  }
  function canReturn() { return operation.current.current?.state === 'failed' || (request.rejected && !operation.current.current); }
  async function returnToLearning() {
    if (checking || request.starting || !canReturn()) return;
    setChecking(true);
    try {
      const value = await startup.refresh();
      if (value?.state === 'ready' && !startup.pending) operation.finish();
    } finally { if (operation.live.current) setChecking(false); }
  }
  useEffect(() => {
    const value = startup.status;
    if (operation.identity.current && value?.operation === operation.identity.current && value.state === 'ready' && value.restored && !value.error && !startup.error && !startup.pending) operation.finish();
  }, [startup.status, startup.error, startup.pending]);
  return { active: operation.active, status: operation.status, error: operation.error, locale, start, refresh: operation.refresh,
    checking, starting: request.starting, returnToLearning, canReturn: canReturn() };
}

export type FormalRestore = ReturnType<typeof useFormalRestore>;

function restoreFailure(restore: FormalRestore, startup: ReturnType<typeof useStartup>) {
  const recovery = startup.status?.state !== 'ready' ? startup.status?.error : null;
  return startup.error || recovery || restore.error || (restore.status?.state === 'failed' ? restore.status.error : '');
}
function restoreLabel(status: RestoreStatus | null, locale: Locale) {
  const text = restoreMessages[locale];
  if (status?.state === 'running') return text.phases[status.phase];
  return status?.state === 'restart_requested' ? text.restarting : text.beginning;
}
function restoreCopies(status: RestoreStatus | null, startup: StartupStatus | null) {
  return status?.retainedCopies.length ? status.retainedCopies : startup?.retainedCopies ?? [];
}
function RestoreActions({ restore, startup }: { readonly restore: FormalRestore; readonly startup: ReturnType<typeof useStartup> }) {
  const text = restoreMessages[restore.locale];
  return <div className="actions"><button disabled={restore.starting || restore.checking} onClick={() => { void restore.refresh(); }}>{text.check}</button>
    {startup.status?.state === 'recovery_required' && <button disabled={startup.pending} onClick={() => { void startup.retry(); }}>{text.retry}</button>}
    {restore.canReturn && <button disabled={restore.starting || restore.checking || startup.pending} onClick={() => { void restore.returnToLearning(); }}>{text.back}</button>}
  </div>;
}

export function RestoreScreen({ runtime, restore, startup }: {
  readonly runtime: RuntimeAdapter; readonly restore: FormalRestore; readonly startup: ReturnType<typeof useStartup>;
}) {
  const text = restoreMessages[restore.locale], { status } = restore;
  const failure = restoreFailure(restore, startup), label = restoreLabel(status, restore.locale);
  const copies = restoreCopies(status, startup.status);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => { heading.current?.focus(); }, []);
  return <div className="app startup-shell" data-theme="white"><main aria-labelledby="restore-heading">
    <h1 id="restore-heading" ref={heading} tabIndex={-1}>{failure ? text.failed : text.title}</h1>
    {failure ? <p role="alert">{failure}</p> : <><p role="status">{label}</p><progress aria-label={label} /></>}
    {status && <p>{status.preview.filename}</p>}
    <RetainedCopies runtime={runtime} copies={copies} locale={restore.locale} />
    <RestoreActions restore={restore} startup={startup} />{restore.canReturn && <p>{text.recheck}</p>}
    <RecoveryClose runtime={runtime} locale={restore.locale} />
  </main></div>;
}
