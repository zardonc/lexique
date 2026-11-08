import { useEffect, useRef, useState } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import { AudioGenerationFailed } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import type { SpeechAudio } from './platform/speech';

export function useQueueAudioReplacement(id: number, runtime: RuntimeAdapter) {
  const [audio, setAudio] = useState<SpeechAudio | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const request = useRef(0);
  const running = useRef(false);
  const operation = useRef<string | null>(null);
  useEffect(() => () => { request.current += 1; }, [id, runtime]);
  async function regenerate() {
    if (running.current) return;
    running.current = true;
    const version = ++request.current;
    setBusy(true); setError('');
    try {
      operation.current ??= crypto.randomUUID();
      const bytes = await runtime.generateSpeech(id, true, operation.current);
      if (!bytes) throw new Error('Native speech unavailable');
      operation.current = null;
      if (version === request.current) setAudio(bytes);
    } catch (cause) {
      if (cause instanceof AudioGenerationFailed) operation.current = null;
      if (version === request.current) setError(errorMessage(cause));
    }
    finally { running.current = false; if (version === request.current) setBusy(false); }
  }
  return { audio, busy, error, regenerate };
}
