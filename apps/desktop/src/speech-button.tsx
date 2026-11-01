import { useEffect, useRef, useState } from 'react';
import { Volume2 } from 'lucide-react';
import type { RuntimeAdapter } from './platform/runtime';
import type { NotebookText } from './notebook-content';
import { errorMessage } from './platform/vocabulary';

export function SpeechButton({ word, runtime, text }: { readonly word: string; readonly runtime: RuntimeAdapter; readonly text: NotebookText }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const player = useRef<HTMLAudioElement | null>(null);
  const objectUrl = useRef<string | null>(null);
  const request = useRef(0);
  function release() { player.current?.pause(); if (objectUrl.current) URL.revokeObjectURL(objectUrl.current); objectUrl.current = null; }
  useEffect(() => () => { request.current += 1; release(); }, [word]);
  async function play() {
    const current = ++request.current;
    setBusy(true); setError('');
    try {
      const [audio, preferences] = await Promise.all([runtime.previewSpeech(word), runtime.speechPreferences()]);
      if (current !== request.current) return;
      if (!audio || !preferences) throw new Error(text.preview);
      const url = URL.createObjectURL(new Blob([audio.slice().buffer as ArrayBuffer], { type: 'audio/ogg; codecs=opus' }));
      objectUrl.current = url;
      const element = new Audio(url);
      player.current = element;
      element.playbackRate = preferences.speed === 'slow' ? 0.6 : 1;
      element.onended = () => { if (current === request.current) release(); };
      element.onerror = () => { if (current === request.current) { release(); setError(text.playbackError); } };
      try { await element.play(); } catch (cause) { if (current === request.current) release(); throw cause; }
    } catch (cause) { if (current === request.current) setError(errorMessage(cause)); }
    finally { if (current === request.current) setBusy(false); }
  }
  return <span><button type="button" title={`${text.playWord} (${text.machineVoice})`} aria-label={`${text.playWord}: ${word} (${text.machineVoice})`} disabled={busy}
    onClick={() => { release(); void play(); }}><Volume2 size={18} aria-hidden="true" /></button>
    {error && <span role="alert">{text.playbackError}: {error}</span>}</span>;
}
