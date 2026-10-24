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
  if (error) return <div className="runtime-error" role="alert">{error}<button type="button" onClick={() => {
    setError(null); setStatus(undefined); setAttempt((value) => value + 1);
  }}>{text.retry}</button></div>;
  if (status === undefined) return <p className="runtime-status" role="status">{text.statusLoading}</p>;
  return <p className="runtime-status" role="status">{status === null ? text.statusPreview : text.statusReady}</p>;
}
