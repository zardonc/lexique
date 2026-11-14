import { z } from 'zod';
import { batchRequestSchema, rateRequestSchema } from './platform/runtime';
import type { BatchRequest, BatchStatus, RuntimeAdapter } from './platform/runtime';
import type { RawPendingRequest } from './platform/backup';

const ratingPrefix = 'lexique-pending-rating-';
export const pendingUndoKey = 'lexique-pending-undo';
export const pendingBatchKey = 'lexique-pending-review-batch';
export function pendingRatingKey(id: number) { return `${ratingPrefix}${id}`; }

let revision = 0, preserving = false;
export function pendingReviewRevision() { return revision; }
export function releasePendingReviews() { preserving = false; revision += 1; }
export function removePendingReview(key: string, requestRevision: number) {
  if (preserving || requestRevision !== revision) return false;
  localStorage.removeItem(key);
  return true;
}

export function pendingReviewKeys() {
  const keys: string[] = [];
  for (let index = 0; index < localStorage.length; index += 1) {
    const key = localStorage.key(index);
    if (key === null) throw new Error('Could not enumerate pending review requests');
    if (key.startsWith(ratingPrefix) || key === pendingUndoKey || key === pendingBatchKey) keys.push(key);
  }
  return keys;
}

export function capturePendingReviews(): readonly RawPendingRequest[] {
  preserving = true; revision += 1;
  const requests = pendingReviewKeys().map((key) => {
    const value = localStorage.getItem(key);
    if (value === null) throw new Error('Pending review request disappeared before raw-file preservation');
    return { key, value };
  });
  confirmedKeys(new Map(requests.map(({ key, value }) => [key, value])), true);
  return requests;
}

export function clearPendingReviews() {
  for (const key of pendingReviewKeys()) {
    localStorage.removeItem(key);
    if (localStorage.getItem(key) !== null) throw new Error('Pending review request removal was not confirmed');
  }
  if (pendingReviewKeys().length) throw new Error('Pending review requests remain after restore cleanup');
}

const undoSchema = z.object({ operation: z.string().min(1).max(128), ratingOperation: z.string().min(1).max(128) }).strict().readonly();

function parseRating(saved: string, id: number) {
  const request = rateRequestSchema.parse(JSON.parse(saved));
  if (request.card_id !== id) throw new Error('Invalid pending rating');
  return request;
}

export function readPendingRating(id: number) {
  const saved = localStorage.getItem(pendingRatingKey(id));
  return saved === null ? null : parseRating(saved, id);
}

export function readPendingUndo(target: string | null) {
  const saved = localStorage.getItem(pendingUndoKey);
  return undoSchema.parse(saved === null ? { operation: crypto.randomUUID(), ratingOperation: target } : JSON.parse(saved));
}

export function readPendingBatch(): BatchRequest | null {
  const saved = localStorage.getItem(pendingBatchKey);
  if (saved === null) return null;
  const request = batchRequestSchema.parse(JSON.parse(saved));
  if (request.vocabulary_ids.length > 100) throw new Error('Invalid pending review batch');
  return request;
}

interface Confirmation { readonly preserve: boolean; readonly revision: number }
function confirm(key: string, saved: string, mode: Confirmation) {
  const current = localStorage.getItem(key);
  if (current !== saved && (mode.preserve || current !== null)) throw new Error('Pending review request changed. Confirm pending requests again.');
  if (!mode.preserve) removePendingReview(key, mode.revision);
}

function validateBatch(request: BatchRequest, result: BatchStatus) {
  const ids = new Set(request.vocabulary_ids);
  if (result.operation !== request.operation || !result.items.every((item) => ids.delete(item.vocabulary_id))) throw new Error('Invalid pending batch response identity');
}

async function resolveBatch(runtime: RuntimeAdapter, mode: Confirmation) {
  const saved = localStorage.getItem(pendingBatchKey), request = readPendingBatch();
  if (saved === null || request === null) return null;
  let result = await runtime.reviewBatchStatus(request.operation);
  if (result) validateBatch(request, result);
  if (!result || (!result.cancelled && result.items.length !== new Set(request.vocabulary_ids).size)) result = await runtime.addReviewBatch(request);
  if (!result) throw new Error('Native review storage unavailable');
  validateBatch(request, result);
  if (!result.cancelled && result.items.length !== new Set(request.vocabulary_ids).size) throw new Error('Pending review batch is not fully confirmed');
  confirm(pendingBatchKey, saved, mode);
  return result;
}

function confirmedKeys(snapshot: ReadonlyMap<string, string | null>, preserve: boolean) {
  const remaining = pendingReviewKeys();
  const unchanged = remaining.length === snapshot.size && remaining.every((key) => snapshot.get(key) === localStorage.getItem(key));
  if (preserve ? !unchanged : remaining.length > 0) throw new Error('Pending review requests changed. Confirm pending requests again.');
}

export async function resolvePendingReviews(runtime: RuntimeAdapter, preserve = false) {
  if (preserve) { preserving = true; revision += 1; }
  const mode = { preserve, revision }, keys = pendingReviewKeys();
  const snapshot = new Map<string, string | null>(keys.map((key) => [key, localStorage.getItem(key)]));
  for (const key of keys.filter((key) => key.startsWith(ratingPrefix))) {
    const saved = snapshot.get(key);
    if (saved == null) throw new Error('Pending review request changed. Confirm pending requests again.');
    const request = parseRating(saved, Number(key.slice(ratingPrefix.length)));
    if (!await runtime.rateReview(request)) throw new Error('Native review storage unavailable');
    confirm(key, saved, mode);
  }
  const undo = localStorage.getItem(pendingUndoKey);
  if (undo !== null) {
    const request = readPendingUndo(null);
    if (!await runtime.undoReview(request.operation, request.ratingOperation)) throw new Error('Native undo unavailable');
    confirm(pendingUndoKey, undo, mode);
  }
  const batch = await resolveBatch(runtime, mode);
  confirmedKeys(snapshot, preserve);
  return batch;
}
