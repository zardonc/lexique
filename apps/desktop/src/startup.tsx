import { useEffect, useRef, useState } from 'react';
import type { Locale } from './content';
import { messages } from './content';
import type { StartupStatus } from './platform/backup';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { clearPendingReviews, releasePendingReviews } from './pending-review-requests';
import { RetainedCopies } from './retained-copies';
import { restoreMessages } from './restore-content';
import { BackupSettings } from './backup-settings';
import type { FormalRestore } from './backup-restore';

const text = {
  zh: { recovery: '需要恢复学习数据', starting: '正在启动应用…', cleanup: '正在完成恢复清理…', close: '关闭应用', restored: '恢复已完成' },
  en: { recovery: 'Learning data recovery required', starting: 'Starting the app…', cleanup: 'Finishing restore cleanup…', close: 'Close app', restored: 'Restore completed' },
  fr: { recovery: 'Récupération des données requise', starting: 'Démarrage de l’application…', cleanup: 'Finalisation de la restauration…', close: 'Fermer l’application', restored: 'Restauration terminée' },
} as const;

function restoredSuccessfully(status: StartupStatus) { return status.state === 'ready' && status.restored !== null && status.error === null; }

function acceptsStartup(previous: StartupStatus | null, value: StartupStatus, expected: string | null = null) {
  if (expected !== null && value.operation !== expected) return false;
  if (!previous || previous.operation === null) return true;
  if (value.operation !== previous.operation) return expected !== null ? previous.state !== 'cleanup_required' : previous.state === 'ready';
  if (previous.state === 'cleanup_required') return value.state !== 'ready';
  return previous.state !== 'ready' || value.state !== 'cleanup_required';
}

function requireCleanupBegin(status: StartupStatus) {
  if (status.state !== 'cleanup_required' || status.error) throw new Error(status.error || 'Restore cleanup did not begin');
}
function requireCleanupComplete(status: StartupStatus) {
  if (!restoredSuccessfully(status)) throw new Error(status.error || 'Restore cleanup was not confirmed');
}

function useCommittedCleanup(runtime: RuntimeAdapter, status: StartupStatus | null, controller: {
  readonly current: { readonly current: StartupStatus | null }; readonly update: (value: StartupStatus) => void;
}) {
  const [pending, setPending] = useState(false), [error, setError] = useState('');
  const attempted = useRef<string | null>(null), live = useRef(true), running = useRef(false);
  async function run(operation: string) {
    if (running.current) return;
    running.current = true;
    setPending(true); setError('');
    const current = () => live.current && controller.current.current?.operation === operation && controller.current.current.state === 'cleanup_required';
    try {
      const begun = await runtime.beginRestoreCleanup(operation);
      if (!current()) return;
      requireCleanupBegin(begun);
      clearPendingReviews();
      const completed = await runtime.completeRestoreCleanup(operation);
      if (!current()) return;
      requireCleanupComplete(completed);
      releasePendingReviews();
      controller.update(completed);
    } catch (cause) { if (current()) setError(errorMessage(cause)); }
    finally { running.current = false; if (live.current) setPending(false); }
  }
  useEffect(() => { live.current = true; return () => { live.current = false; }; }, []);
  useEffect(() => {
    if (status?.state !== 'cleanup_required' || !status.operation || attempted.current === status.operation) return;
    attempted.current = status.operation; void run(status.operation);
  }, [runtime, status?.operation, status?.state]);
  return { pending, error, retry: () => {
    const operation = controller.current.current?.operation;
    if (operation && controller.current.current?.state === 'cleanup_required') return run(operation);
  } };
}

function acceptsStartupRead(previous: StartupStatus | null, value: StartupStatus, expected: string | null) {
  return acceptsStartup(previous, value) || acceptsStartup(previous, value, expected)
    || (value.state === 'ready' && value.operation === null && previous?.state !== 'cleanup_required');
}

function useStartupRetry(runtime: RuntimeAdapter, controller: {
  readonly current: { readonly current: StartupStatus | null }; readonly updates: { readonly current: number };
  readonly expected: { readonly current: string | null };
  readonly connected: { readonly current: boolean }; readonly live: { readonly current: boolean };
  readonly update: (value: StartupStatus) => void; readonly failed: (cause: unknown) => void;
  readonly reconnect: () => void; readonly cleanup: ReturnType<typeof useCommittedCleanup>;
}) {
  const [pending, setPending] = useState(false), running = useRef(false);
  async function continueCleanup(value: StartupStatus) {
    const current = controller.current.current;
    if (current?.state === 'cleanup_required' && current.operation === value.operation) await controller.cleanup.retry();
  }
  async function receive(value: StartupStatus, version: number) {
    if (!controller.live.current) return;
    if (version !== controller.updates.current) { await continueCleanup(value); return; }
    if (!acceptsStartupRead(controller.current.current, value, controller.expected.current)) throw new Error('Startup recovery response identity differs from the current restore');
    controller.update(value);
    if (value.state === 'cleanup_required') await controller.cleanup.retry();
  }
  async function retry() {
    if (running.current || controller.cleanup.pending) return;
    if (!controller.connected.current) { controller.reconnect(); return; }
    if (controller.current.current?.state === 'cleanup_required') { await controller.cleanup.retry(); return; }
    running.current = true; setPending(true);
    const version = controller.updates.current;
    try {
      await receive(await runtime.retryStartup(), version);
    } catch (cause) { if (version === controller.updates.current) controller.failed(cause); }
    finally { running.current = false; if (controller.live.current) setPending(false); }
  }
  return { pending, retry };
}

export function useStartup(runtime: RuntimeAdapter) {
  const [status, setStatus] = useState<StartupStatus | null>(null), [error, setError] = useState('');
  const current = useRef<StartupStatus | null>(null), updates = useRef(0), live = useRef(true), connected = useRef(false);
  const expected = useRef<string | null>(null);
  const [registration, setRegistration] = useState(0);
  function update(value: StartupStatus) { if (live.current) { current.current = value; updates.current += 1; setStatus(value); setError(''); } }
  function failed(cause: unknown) { if (live.current) setError(errorMessage(cause)); }
  function expectRestore(operation: string) { expected.current = operation; updates.current += 1; }
  function receive(value: StartupStatus) {
    if (!acceptsStartup(current.current, value, expected.current)) return;
    update(value);
  }
  async function refresh() {
    const version = updates.current;
    try {
      const value = await runtime.startupStatus();
      if (!live.current) return null;
      if (version === updates.current && acceptsStartupRead(current.current, value, expected.current)) update(value);
      return current.current;
    } catch (cause) { if (version === updates.current) failed(cause); return null; }
  }
  useEffect(() => {
    live.current = true;
    let disposed = false;
    let stop: (() => void) | undefined;
    void runtime.onStartupStatus((value) => { if (!disposed) receive(value); }, (cause) => { if (!disposed) failed(cause); }).then((unlisten) => {
      if (disposed) { unlisten(); return; }
      stop = unlisten; connected.current = true; void refresh();
    }).catch((cause: unknown) => { if (!disposed) failed(cause); });
    return () => { disposed = true; live.current = false; connected.current = false; updates.current += 1; stop?.(); };
  }, [runtime, registration]);
  const cleanup = useCommittedCleanup(runtime, status, { current, update });
  const retry = useStartupRetry(runtime, { current, updates, live, connected, expected, update, failed, cleanup,
    reconnect: () => { setStatus(null); setError(''); setRegistration((value) => value + 1); } });
  return { status, error: error || cleanup.error, pending: cleanup.pending || retry.pending, current, update, failed, refresh, expectRestore, retry: retry.retry };
}

export function StartupFeedback({ runtime, status, locale }: { readonly runtime: RuntimeAdapter; readonly status: StartupStatus; readonly locale: Locale }) {
  return <>{status.error && <p role="alert">{status.error}</p>}
    {restoredSuccessfully(status) && <p role="status">{text[locale].restored}</p>}
    {status.retainedCopies.length > 0 && <details><summary>{restoreMessages[locale].copies}</summary>
      <RetainedCopies runtime={runtime} copies={status.retainedCopies} locale={locale} /></details>}</>;
}

function useRecoveryClose(runtime: RuntimeAdapter) {
  const [error, setError] = useState('');
  const live = useRef(true);
  async function close() { try { await runtime.finishClose(); } catch (cause) { if (live.current) setError(errorMessage(cause)); } }
  useEffect(() => {
    live.current = true;
    let disposed = false;
    let stop: (() => void) | undefined;
    void runtime.onCloseRequested(() => { if (!disposed) void close(); }).then((unlisten) => {
      if (disposed) unlisten(); else stop = unlisten;
    }).catch((cause: unknown) => { if (!disposed) setError(errorMessage(cause)); });
    return () => { disposed = true; live.current = false; stop?.(); };
  }, [runtime]);
  return { error, close };
}

export function RecoveryClose({ runtime, locale }: { readonly runtime: RuntimeAdapter; readonly locale: Locale }) {
  const closing = useRecoveryClose(runtime);
  return <>{closing.error && <p role="alert">{closing.error}</p>}
    <button onClick={() => { void closing.close(); }}>{text[locale].close}</button></>;
}

function startupLabel(status: StartupStatus | null, failure: string | null | undefined, locale: Locale) {
  if (failure || status?.state === 'recovery_required') return text[locale].recovery;
  return status?.state === 'cleanup_required' ? text[locale].cleanup : text[locale].starting;
}

const immediately = (action: () => void) => action();
const noLearningResult = () => {};
function recoveryAvailable(startup: ReturnType<typeof useStartup>) { return startup.status?.state === 'recovery_required' && !startup.pending; }

export function StartupScreen({ runtime, startup, restore }: {
  readonly runtime: RuntimeAdapter; readonly startup: ReturnType<typeof useStartup>; readonly restore: FormalRestore;
}) {
  const [locale, setLocale] = useState<Locale>('zh');
  const failure = startup.error || startup.status?.error;
  const label = startupLabel(startup.status, failure, locale);
  const heading = useRef<HTMLHeadingElement>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => { heading.current?.focus(); document.documentElement.lang = locale === 'zh' ? 'zh-CN' : locale; }, [label, locale]);
  return <div className="app startup-shell" data-theme="white"><main aria-labelledby="startup-heading">
    <select aria-label={messages[locale].language} value={locale} onChange={(event) => setLocale(event.target.value as Locale)}>
      <option value="zh">中文</option><option value="en">English</option><option value="fr">Français</option>
    </select>
    <h1 id="startup-heading" ref={heading} tabIndex={-1}>{label}</h1>
    {failure ? <p role="alert">{failure}</p> : <p role="status">{label}</p>}
    <RetainedCopies runtime={runtime} copies={startup.status?.retainedCopies ?? []} locale={locale} />
    {recoveryAvailable(startup) && <BackupSettings recovery runtime={runtime} locale={locale} restore={restore}
      visible beforeStart={immediately} onPause={setBusy} onResolved={noLearningResult} />}
    <div className="actions"><button disabled={busy || startup.pending || !runtime.backupAvailable()} onClick={() => { void startup.retry(); }}>{restoreMessages[locale].retry}</button>
      <RecoveryClose runtime={runtime} locale={locale} /></div>
  </main></div>;
}
