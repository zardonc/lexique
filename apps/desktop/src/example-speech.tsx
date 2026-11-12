import { createContext, useContext, useRef } from 'react';
import type { ReactNode } from 'react';
import type { RuntimeAdapter } from './platform/runtime';
import type { NotebookText } from './notebook-content';
import { SpeechButton } from './speech-button';

const Playback = createContext<(cancel: () => void) => void>(() => {});

export function ExampleSpeechGroup({ children }: { readonly children: ReactNode }) {
  const cancel = useRef<(() => void) | null>(null);
  return <Playback.Provider value={(next) => { cancel.current?.(); cancel.current = next; }}>{children}</Playback.Provider>;
}

export function ExampleSpeech({ value, runtime, text }: {
  readonly value: string; readonly runtime: RuntimeAdapter; readonly text: NotebookText;
}) {
  const onStart = useContext(Playback);
  const spoken = value.normalize('NFC').replace(/[\r\n\t]|[^\S\p{Cc}]/gu, ' ').replace(/ +/gu, ' ').replace(/^ | $/gu, '');
  const error = /\p{Cc}/u.test(spoken) ? text.exampleSpeechControl
    : [...spoken].length > 200 ? text.exampleSpeechLimit : '';
  if (!spoken) return null;
  return <><SpeechButton key={value} word={spoken} runtime={runtime} text={{ ...text, playWord: text.playExample }}
    disabled={!!error} onStart={onStart} />{error && <span role="status">{error}</span>}</>;
}
