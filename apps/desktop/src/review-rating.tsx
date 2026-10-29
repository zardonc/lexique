import { useEffect, useState } from 'react';
import type { NotebookText } from './notebook-content';
import { RatingRejected, rateRequestSchema } from './platform/runtime';
import type { RateRequest, Rating, ReviewAvailability, RuntimeAdapter } from './platform/runtime';
import { errorMessage } from './platform/vocabulary';

const ratings: readonly Rating[] = ['again', 'hard', 'good', 'easy'];

function RatingChoices({ text, disabled, onRate }: {
  readonly text: NotebookText; readonly disabled: boolean; readonly onRate: (rating: Rating) => void;
}) {
  return <><p>{text.ratingHint}</p><div className="actions">{ratings.map((rating, index) =>
    <button key={rating} disabled={disabled} onClick={() => onRate(rating)}>{text.ratings[index]}</button>)}</div></>;
}

function useAvailabilityTimer({ id, runtime, availability, setAvailability, setError }: {
  readonly id: number; readonly runtime: RuntimeAdapter; readonly availability: ReviewAvailability | null;
  readonly setAvailability: (value: ReviewAvailability | null) => void; readonly setError: (value: string) => void;
}) {
  useEffect(() => {
    if (!availability?.available_at) return;
    const delay = Math.max(1000, Math.min(2_147_483_647, availability.available_at * 1000 - Date.now() + 1000));
    const timer = window.setTimeout(() => { void runtime.reviewAvailability(id).then((value) => { setAvailability(value); setError(''); }).catch((cause: unknown) => setError(errorMessage(cause))); }, delay);
    return () => window.clearTimeout(timer);
  }, [availability, id, runtime, setAvailability, setError]);
}

function useReviewRating({ id, runtime, text, onSuccess, onBusyChange }: {
  readonly id: number; readonly runtime: RuntimeAdapter; readonly text: NotebookText; readonly onSuccess: () => void; readonly onBusyChange: (busy: boolean) => void;
}) {
  const [availability, setAvailability] = useState<ReviewAvailability | null>(null);
  const [pending, setPending] = useState<RateRequest | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [blocked, setBlocked] = useState(false);
  const key = `lexique-pending-rating-${id}`;
  useEffect(() => {
    let active = true;
    try {
      const saved = localStorage.getItem(key);
      if (saved) {
        const request = rateRequestSchema.parse(JSON.parse(saved));
        if (request.card_id !== id) throw new Error('Invalid pending rating');
        setPending(request);
      }
    } catch (cause) { setBlocked(true); setError(errorMessage(cause)); }
    void runtime.reviewAvailability(id).then((value) => { if (active) setAvailability(value); })
      .catch((cause: unknown) => { if (active) setError(errorMessage(cause)); });
    return () => { active = false; };
  }, [id, key, runtime]);
  useAvailabilityTimer({ id, runtime, availability, setAvailability, setError });
  async function submit(request: RateRequest) {
    setBusy(true); onBusyChange(true); setError('');
    try {
      localStorage.setItem(key, JSON.stringify(request));
      setPending(request);
      const result = await runtime.rateReview(request);
      if (!result) throw new Error(text.ratingUnavailable);
      localStorage.removeItem(key);
      setPending(null); onSuccess();
    } catch (cause) {
      const message = errorMessage(cause);
      if (cause instanceof RatingRejected) {
        localStorage.removeItem(key); setPending(null); setAvailability(null);
        try { setAvailability(await runtime.reviewAvailability(id)); }
        catch (readError) { setError(`${text.availability[cause.code]}: ${errorMessage(readError)}`); return; }
        setError(`${text.availability[cause.code]}: ${message}`);
      } else setError(message);
    }
    finally { setBusy(false); onBusyChange(false); }
  }
  async function refresh() { setError(''); try { setAvailability(await runtime.reviewAvailability(id)); } catch (cause) { setError(errorMessage(cause)); } }
  return { availability, pending, error, busy: busy || blocked, submit, refresh };
}

export function ReviewRating({ id, runtime, text, enabled, onSuccess, onBusyChange }: {
  readonly id: number; readonly runtime: RuntimeAdapter; readonly text: NotebookText;
  readonly enabled: boolean; readonly onSuccess: () => void; readonly onBusyChange: (busy: boolean) => void;
}) {
  const { availability, pending, error, busy, submit, refresh } = useReviewRating({ id, runtime, text, onSuccess, onBusyChange });
  return <RatingBody id={id} text={text} enabled={enabled} availability={availability} pending={pending} error={error} busy={busy} submit={submit} refresh={refresh} />;
}

function AvailabilityError({ error, text, pending, busy, availability, refresh }: {
  readonly error: string; readonly text: NotebookText; readonly pending: RateRequest | null; readonly busy: boolean;
  readonly availability: ReviewAvailability | null; readonly refresh: () => Promise<void>;
}) {
  if (!error) return null;
  const retry = !pending && !busy && (!availability || availability.reason);
  return <><p role="alert">{text.reviewError}: {error}</p>{retry && <button onClick={() => { void refresh(); }}>{text.retry}</button>}</>;
}

function RatingBody({ id, text, enabled, availability, pending, error, busy, submit, refresh }: {
  readonly id: number; readonly text: NotebookText; readonly enabled: boolean; readonly availability: ReviewAvailability | null;
  readonly pending: RateRequest | null; readonly error: string; readonly busy: boolean; readonly submit: (request: RateRequest) => Promise<void>;
  readonly refresh: () => Promise<void>;
}) {
  let controls;
  if (pending) controls = <button disabled={busy || !enabled} onClick={() => { void submit(pending); }}>{text.retry}</button>;
  else if (availability?.reason) controls = <p>{text.availability[availability.reason]}{availability.available_at != null && `: ${new Date(availability.available_at * 1000).toLocaleString()}`}</p>;
  else controls = <RatingChoices text={text} disabled={!enabled || busy || !availability} onRate={(rating) => {
    if (!availability) return;
    void submit({ operation: crypto.randomUUID(), card_id: id, expected_revision: availability.progress.revision,
      rating, timezone: Intl.DateTimeFormat().resolvedOptions().timeZone });
  }} />;
  return <>
    <AvailabilityError error={error} text={text} pending={pending} busy={busy} availability={availability} refresh={refresh} />
    {controls}
  </>;
}
