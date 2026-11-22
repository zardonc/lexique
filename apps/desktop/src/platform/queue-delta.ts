import type { QueueView } from './runtime';

export function queueCursor(view: QueueView | null) {
  return view && { revision: view.summary.revision, scope: view.summary.scope, window: view.window.map((entry) => ({ card: entry.card,
    record_revision: entry.record.revision, progress_revision: entry.progress.revision,
    version: entry.version, content_hash: entry.content_hash })) };
}

function reuseQueueAudio(entry: QueueView['window'][number], known: QueueView['window'][number] | undefined) {
  if (entry.audio !== null) return entry.audio;
  if (!known || known.version !== entry.version || known.content_hash !== entry.content_hash || known.card.vocabulary_id !== entry.card.vocabulary_id) return null;
  return known.audio;
}

function validateContext(payload: QueueView, previous: QueueView | null) {
  if (payload.delta.before_revision === null) return;
  if (JSON.stringify(payload.summary.scope) !== JSON.stringify(previous?.summary.scope)) throw new Error('Queue delta scope mismatch');
  if (payload.delta.before_revision !== previous?.summary.revision) throw new Error('Queue delta revision mismatch');
}

export function applyQueueDelta(payload: QueueView, previous: QueueView | null) {
  validateContext(payload, previous);
  const { delta } = payload;
  const entries = new Map(delta.before_revision === null ? [] : previous?.window.map((entry) => [entry.card.id, entry]));
  for (const id of delta.removed) entries.delete(id);
  for (const entry of payload.window) {
    entries.set(entry.card.id, { ...entry, audio: reuseQueueAudio(entry, entries.get(entry.card.id)) });
  }
  const window = delta.order.map((id) => { const entry = entries.get(id); if (!entry) throw new Error('Queue delta references an unknown card'); return entry; });
  return { ...payload, window };
}
