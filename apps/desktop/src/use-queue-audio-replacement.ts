import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';

export function useQueueAudioReplacement(id: number, runtime: RuntimeAdapter) {
  const [audio, setAudio] = useState<Uint8Array | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const request = useRef(0);
  const running = useRef(false);
  useEffect(() => () => { request.current += 1; }, [id, runtime]);
  async function regenerate() {
    if (running.current) return;
    running.current = true;
    const version = ++request.current;
    setBusy(true); setError('');
    try {
      const bytes = await runtime.generateSpeech(id, true);
      if (!bytes) throw new Error('Native speech unavailable');
      if (version === request.current) setAudio(bytes);
    } catch (cause) { if (version === request.current) setError(errorMessage(cause)); }
    finally { running.current = false; if (version === request.current) setBusy(false); }
  }
  return { audio, busy, error, regenerate };
}
