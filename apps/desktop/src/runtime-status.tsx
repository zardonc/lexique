import { useEffect, useState } from 'react';
import type { Messages } from './content';
import type { RuntimeAdapter, RuntimeStatus } from './platform/runtime';

interface RuntimeStatusProps {
  readonly runtime: RuntimeAdapter;
  readonly text: Messages;
}

export function RuntimeStatusPanel({ runtime, text }: RuntimeStatusProps) {
  const [status, setStatus] = useState<RuntimeStatus | null | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    runtime.getStatus().then((result) => {
      if (active) setStatus(result);
    }).catch((cause: unknown) => {
      if (active) setError(cause instanceof Error ? cause.message : String(cause));
    });
    return () => { active = false; };
  }, [runtime, attempt]);
  useEffect(() => {
    if (!status?.audioMigration.pending) return;
    const timer = window.setTimeout(() => setAttempt((value) => value + 1), 1000);
    return () => window.clearTimeout(timer);
  }, [status]);
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    runtime.onReviewChanged(() => { if (active) setAttempt((value) => value + 1); })
      .then((stop) => { if (active) unlisten = stop; else stop(); })
      .catch((cause: unknown) => { if (active) setError(cause instanceof Error ? cause.message : String(cause)); });
    return () => { active = false; unlisten?.(); };
  }, [runtime]);
  if (error) return <div className="runtime-error" role="alert">{error}<button type="button" onClick={() => {
    setError(null); setStatus(undefined); setAttempt((value) => value + 1);
  }}>{text.retry}</button></div>;
  if (status === undefined) return <p className="runtime-status" role="status">{text.statusLoading}</p>;
  return <>
    <p className="runtime-status" role="status">{status === null ? text.statusPreview : status.audioMigration.pending ? text.audioMigrating : status.audioMigration.failed ? text.audioMigrationFailed : text.statusReady}</p>
    {status?.audioCache && <AudioCacheDetails cache={status.audioCache} text={text} />}
  </>;
}

function AudioCacheDetails({ cache, text }: { readonly cache: NonNullable<RuntimeStatus['audioCache']>; readonly text: Messages }) {
  const bytes = (value: number | null) => value === null ? text.audioCache.unavailable : `${(value / 1_048_576).toFixed(2)} MiB`;
  const sizes = [[text.audioCache.logical, cache.logical_bytes], [text.audioCache.files, cache.file_bytes],
    [text.audioCache.allocated, cache.allocated_bytes], [text.audioCache.permanent, cache.permanent_bytes],
    [text.audioCache.protected, cache.protected_bytes], [text.audioCache.reclaimable, cache.reclaimable_bytes]] as const;
  return <details className="runtime-status runtime-cache">
    <summary>{text.audioCache.title}</summary>
    {cache.in_progress && <p>{text.audioCache.checking}</p>}
    <dl>{sizes.map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{bytes(value)}</dd></div>)}
      <div><dt>{text.audioCache.unknown}</dt><dd>{cache.unknown_files} ({bytes(cache.unknown_bytes)})</dd></div>
      <div><dt>{text.audioCache.failures}</dt><dd>{cache.gc_failures + cache.scan_failures + cache.unsafe_files}</dd></div>
    </dl>
    {cache.over_capacity && <p>{text.audioCache.over}</p>}
    {cache.error && <p role="alert">{cache.error}</p>}
  </details>;
}
