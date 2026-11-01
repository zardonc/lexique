import { useCallback, useEffect, useRef, useState } from 'react';
import type { NotebookText } from './notebook-content';
import type { ReviewCard, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';
import type { VocabularyRecord } from './platform/vocabulary';
import { ReviewRating } from './review-rating';

function ExampleSense({ record, text }: { readonly record: VocabularyRecord; readonly text: NotebookText }) {
  const meaning = record.example_sense === null ? text.exampleSenseUnspecified : record.digest[record.example_sense]?.translation || text.missing;
  return <p>{text.exampleSense}：{meaning}</p>;
}

function Examples({ record, text }: { readonly record: VocabularyRecord; readonly text: NotebookText }) {
  const examples = [{ fr: record.example_fr, zh: record.example_zh }, ...record.more_examples]
    .filter((example) => example.fr.trim() || example.zh.trim());
  const [first, ...rest] = examples;
  return <>
    {first && <section aria-label={text.exampleUser}><p>{text.exampleUser}</p>
      {first.fr.trim() && <p lang="fr">{first.fr}</p>}{first.zh.trim() && <p lang="zh">{first.zh}</p>}</section>}
    {(!first?.fr.trim() || !first?.zh.trim()) && <p>{text.exampleMissing}</p>}
    {rest.length > 0 && <details><summary>{text.moreExamples}</summary>{rest.map((example, index) => <section key={index}>
      <p lang="fr">{example.fr}</p><p lang="zh">{example.zh}</p>{(!example.fr.trim() || !example.zh.trim()) && <p>{text.exampleMissing}</p>}</section>)}</details>}
  </>;
}

function Answer({ record, text }: { readonly record: VocabularyRecord; readonly text: NotebookText }) {
  return <>
    <p>{record.digest.map((entry) => entry.translation || text.missing).join(' · ')}</p>
    <ExampleSense record={record} text={text} />
    <Examples record={record} text={text} />
    <details><summary>{text.more}</summary><p>{record.notes || text.missing}</p>
      <p>{record.reference.dictionary} · {record.reference.version} · {record.reference.attribution} · {record.reference.license}</p>
      <p>{record.reference.source_url}</p><p>{record.reference.license_url}</p></details>
  </>;
}

export function QueueReviewCard({ record, text, runtime, card, audio, onRated, onSkip }: {
  readonly record: VocabularyRecord; readonly text: NotebookText; readonly runtime: RuntimeAdapter;
  readonly card: ReviewCard; readonly audio: Uint8Array | null; readonly onRated: () => void; readonly onSkip: () => void;
}) {
  const [revealed, setRevealed] = useState(false);
  const [busy, setBusy] = useState(false);
  const { player, played, error, play, mediaError } = useListeningAudio({ audio, runtime, autoPlay: true });
  return <section className="queue-review-card" aria-label={text.queueCard}>
    {audio ? <><p>{text.machineVoice}</p><audio ref={player} onError={() => mediaError(text.playbackError)} />
      <button onClick={() => { void play(); }}>{text.replay}</button></> : <h3 lang="fr">{record.headword}</h3>}
    {error && <p role="alert">{text.playbackError}: {error}</p>}
    {revealed && <>{audio && <h3 lang="fr">{record.headword}</h3>}<Answer record={record} text={text} />
      <ReviewRating id={card.id} runtime={runtime} text={text} enabled={!audio || played} onBusyChange={setBusy} onSuccess={onRated} /></>}
    <div className="actions">{!revealed && <button onClick={() => setRevealed(true)}>{text.reveal}</button>}
      <button disabled={busy} onClick={onSkip}>{text.skip}</button></div>
  </section>;
}

export function RecognitionCard({ record, text, runtime, card, onRated, onSkip }: { readonly record: VocabularyRecord; readonly text: NotebookText; readonly runtime: RuntimeAdapter; readonly card: ReviewCard; readonly onRated?: () => void; readonly onSkip?: () => void }) {
  const [revealed, setRevealed] = useState(false);
  const [ratingBusy, setRatingBusy] = useState(false);
  const dialog = useRef<HTMLDialogElement>(null);
  return <>
    <p>{text.recognition}</p>
    <div className="actions"><button onClick={() => { setRevealed(false); dialog.current?.showModal(); }}>{text.openRecognition}</button></div>
    <dialog ref={dialog} className="review-card" aria-label={text.recognition} onCancel={(event) => { if (ratingBusy) event.preventDefault(); }}>
      <section aria-label={text.recognition}>
        <h3 lang="fr">{record.headword}</h3>
        {revealed && <Answer record={record} text={text} />}
        {revealed && <ReviewRating id={card.id} runtime={runtime} text={text} enabled onBusyChange={setRatingBusy} onSuccess={() => { dialog.current?.close(); onRated?.(); }} />}
        <div className="actions">
          {!revealed && <button onClick={() => setRevealed(true)}>{text.reveal}</button>}
          <button disabled={ratingBusy} onClick={() => dialog.current?.close()}>{text.closeCard}</button>
          {onSkip && <button disabled={ratingBusy} onClick={() => { dialog.current?.close(); onSkip(); }}>{text.skip}</button>}
        </div>
      </section>
    </dialog>
  </>;
}

function useListeningAudio({ audio, runtime, autoPlay, onPlayback }: {
  readonly audio: Uint8Array | null; readonly runtime: RuntimeAdapter; readonly autoPlay: boolean;
  readonly onPlayback?: (failed: boolean) => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const player = useRef<HTMLAudioElement>(null);
  const playback = useRef(0);
  const failedPlayback = useRef(-1);
  const onPlaybackRef = useRef<(failed: boolean) => void>(() => {});
  const [url, setUrl] = useState('');
  const [error, setError] = useState('');
  const [played, setPlayed] = useState(false);
  useEffect(() => { onPlaybackRef.current = onPlayback ?? (() => {}); }, [onPlayback]);
  const cancel = useCallback(() => { playback.current += 1; player.current?.pause(); }, []);
  const current = useCallback((request: number) => request === playback.current && (autoPlay || dialog.current?.open), [autoPlay]);
  const play = useCallback(async () => {
    const request = ++playback.current;
    setPlayed(false); setError('');
    try {
      const preferences = await runtime.speechPreferences();
      if (!current(request)) return;
      if (!preferences || !player.current) throw new Error('Speech preferences unavailable');
      player.current.playbackRate = preferences.speed === 'slow' ? 0.6 : 1;
      player.current.currentTime = 0;
      await player.current.play();
      if (current(request) && failedPlayback.current !== request) { setPlayed(true); onPlaybackRef.current(false); }
    } catch (cause) {
      if (current(request)) {
        setError(errorMessage(cause)); onPlaybackRef.current(true);
      }
    }
  }, [runtime, current]);
  useEffect(() => {
    if (!audio) return;
    const objectUrl = URL.createObjectURL(new Blob([audio.slice().buffer as ArrayBuffer], { type: 'audio/ogg; codecs=opus' }));
    setUrl(objectUrl);
    if (autoPlay && player.current) { player.current.src = objectUrl; void play(); }
    return () => { cancel(); URL.revokeObjectURL(objectUrl); };
  }, [audio, autoPlay, cancel, play]);
  function mediaError(message: string) {
    if (!autoPlay && !dialog.current?.open) return;
    failedPlayback.current = playback.current;
    setPlayed(false); setError((previous) => previous || message); onPlaybackRef.current(true);
  }
  function close() { cancel(); dialog.current?.close(); }
  return { dialog, player, url, error, played, play, close, cancel, mediaError };
}

export function ListeningCard({ record, text, runtime, audio, onPlayback, card, onRated, onSkip }: {
  readonly record: VocabularyRecord; readonly text: NotebookText; readonly runtime: RuntimeAdapter; readonly audio: Uint8Array;
  readonly card: ReviewCard;
  readonly onPlayback: (failed: boolean) => void;
  readonly onRated?: () => void;
  readonly onSkip?: () => void;
}) {
  const [ratingBusy, setRatingBusy] = useState(false);
  const { dialog, player, url, error, played, play, close, cancel, mediaError } = useListeningAudio({ audio, runtime, autoPlay: false, onPlayback });
  const [revealed, setRevealed] = useState(false);
  return <>
    <div className="actions"><button disabled={!url} onClick={() => {
      setRevealed(false); dialog.current?.showModal(); void play();
    }}>{text.openListening}</button></div>
    <dialog ref={dialog} className="review-card" aria-label={text.listening} onCancel={(event) => { if (ratingBusy) event.preventDefault(); else cancel(); }}>
      <section aria-label={text.listening}>
        <p>{text.machineVoice}</p><audio ref={player} src={url} onError={() => mediaError(text.playbackError)} />
        {error && <p role="alert">{text.playbackError}: {error}</p>}
        {revealed && <><h3 lang="fr">{record.headword}</h3><Answer record={record} text={text} /></>}
        {revealed && <ReviewRating id={card.id} runtime={runtime} text={text} enabled={played} onBusyChange={setRatingBusy} onSuccess={() => { close(); onRated?.(); }} />}
        <div className="actions"><button onClick={() => { void play(); }}>{text.replay}</button>
          {!revealed && <button onClick={() => setRevealed(true)}>{text.reveal}</button>}
          <button disabled={ratingBusy} onClick={() => { close(); onSkip?.(); }}>{text.skip}</button></div>
      </section>
    </dialog>
  </>;
}

function usePause({ id, runtime, text, setError, setBusy }: { readonly id: number; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly setError: (value: string) => void; readonly setBusy: (value: boolean) => void }) {
  const [paused, setPaused] = useState<boolean | null>(null);
  useEffect(() => {
    let active = true;
    setPaused(null);
    void runtime.reviewPause(id).then((value) => { if (active) setPaused(value); })
      .catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [id, runtime, setError]);
  async function changePause(value: boolean) {
    setBusy(true); setError('');
    try {
      const result = await runtime.reviewPause(id, value);
      if (result === null) throw new Error(text.preview);
      setPaused(result);
    } catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return { paused, changePause };
}

function useReviewCards({ id, runtime, text, onChanged }: {
  readonly id: number; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onChanged: () => void;
}) {
  const [cards, setCards] = useState<readonly ReviewCard[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [audio, setAudio] = useState<Uint8Array | null>(null);
  const [playbackFailed, setPlaybackFailed] = useState(false);
  const { paused, changePause } = usePause({ id, runtime, text, setError, setBusy });
  useEffect(() => {
    let active = true;
    setAudio(null);
    setPlaybackFailed(false);
    setCards([]);
    setError('');
    setLoading(true);
    void runtime.reviewCards(id).then(async (result) => {
      if (active && result) setCards(result);
      if (result?.some((card) => card.kind === 'listening')) {
        const cached = await runtime.cachedSpeech(id);
        if (active) setAudio(cached);
      }
    }).catch((cause: unknown) => { if (active) setError(errorMessage(cause)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [id, runtime]);
  async function add() {
    setBusy(true); setError('');
    try {
      const result = await runtime.addReview(id);
      if (!result) throw new Error('Native review storage unavailable');
      setCards(result);
      onChanged();
      if (!audio || playbackFailed) {
        const generated = await runtime.generateSpeech(id, playbackFailed);
        if (!generated) throw new Error('Native speech unavailable');
        setAudio(generated);
        setPlaybackFailed(false);
      }
    } catch (cause) { setError(errorMessage(cause)); }
    finally { setBusy(false); }
  }
  return { cards, error, busy, loading, audio, playbackFailed, setPlaybackFailed, paused, add, changePause };
}

export function ReviewCards({ record, runtime, text, onChanged }: {
  readonly record: VocabularyRecord; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onChanged: () => void;
}) {
  const { cards, error, busy, loading, audio, setPlaybackFailed, paused, add, changePause } = useReviewCards({ id: record.id, runtime, text, onChanged });
  return <section>
    <div className="actions"><button disabled={busy || loading} onClick={() => { void add(); }}>{text.addReview}</button></div>
    {cards.length > 0 && paused !== null && <div className="actions"><button disabled={busy} onClick={() => void changePause(!paused)}>{paused ? text.resume : text.pause}</button></div>}
    {paused && <p role="status">{text.paused}</p>}
    {error && <p role="alert">{text.reviewError}: {error}</p>}
    {paused !== null && cards.filter((card) => card.kind === 'recognition').map((card) => <RecognitionCard key={`${card.id}-${paused}`} card={card} record={record} text={text} runtime={runtime} onRated={onChanged} />)}
    {paused !== null && cards.filter((card) => card.kind === 'listening').map((card) => <div key={`${card.id}-${paused}`}><p>{audio ? text.listeningReady : text.listeningPending}</p>
      {audio && <ListeningCard card={card} record={record} text={text} runtime={runtime} audio={audio} onPlayback={setPlaybackFailed} onRated={onChanged} />}</div>)}
  </section>;
}
