import { useEffect, useRef, useState } from 'react';
import { Volume2 } from 'lucide-react';
import type { RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import { speechPlaybackRate } from './platform/speech';
import type { SpeechPreferences } from './platform/speech';

export function SpeechButton({ word, runtime, text, onStart, preferences: previewPreferences, disabled = false }: {
  readonly word: string; readonly runtime: RuntimeAdapter;
  readonly text: { readonly playWord: string; readonly machineVoice: string; readonly preview: string; readonly playbackError: string; readonly approximateSpeed: string };
  readonly onStart?: (cancel: () => void) => void;
  readonly preferences?: SpeechPreferences; readonly disabled?: boolean;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [approximate, setApproximate] = useState(false);
  const player = useRef<HTMLAudioElement | null>(null);
  const objectUrl = useRef<string | null>(null);
  const request = useRef(0);
  function release() { player.current?.pause(); if (objectUrl.current) URL.revokeObjectURL(objectUrl.current); objectUrl.current = null; }
  useEffect(() => () => { request.current += 1; release(); }, [word]);
  async function play() {
    const current = ++request.current;
    setBusy(true); setError(''); setApproximate(false);
    try {
      const [audio, preferences] = await Promise.all([runtime.previewSpeech(word, previewPreferences?.voice),
        previewPreferences ? Promise.resolve(previewPreferences) : runtime.speechPreferences()]);
      if (current !== request.current) return;
      if (!audio || !preferences) throw new Error(text.preview);
      setApproximate(audio.voice === null);
      const url = URL.createObjectURL(new Blob([new Uint8Array(audio.bytes).buffer], { type: 'audio/ogg; codecs=opus' }));
      objectUrl.current = url;
      const element = new Audio(url);
      player.current = element;
      element.preservesPitch = true;
      element.playbackRate = speechPlaybackRate(preferences.wpm, audio.voice);
      element.onended = () => { if (current === request.current) release(); };
      element.onerror = () => { if (current === request.current) { release(); setError(text.playbackError); } };
      await element.play();
    } catch (cause) { if (current === request.current) { release(); setError(errorMessage(cause)); } }
    finally { if (current === request.current) setBusy(false); }
  }
  return <span><button type="button" title={`${text.playWord} (${text.machineVoice})`} aria-label={`${text.playWord}: ${word} (${text.machineVoice})`} disabled={busy || disabled}
    onClick={() => {
      onStart?.(() => { request.current += 1; release(); setBusy(false); }); release(); void play();
    }}><Volume2 size={18} aria-hidden="true" /></button>
    {approximate && <span role="status">{text.approximateSpeed}</span>}
    {error && <span role="alert">{text.playbackError}: {error}</span>}</span>;
}
