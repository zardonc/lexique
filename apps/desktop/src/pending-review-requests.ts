import { z } from 'zod';
import { batchRequestSchema, rateRequestSchema } from './platform/runtime';
import type { BatchRequest, BatchStatus, RuntimeAdapter } from './platform/runtime';

const ratingPrefix = 'lexique-pending-rating-';
export const pendingUndoKey = 'lexique-pending-undo';
export const pendingBatchKey = 'lexique-pending-review-batch';
export function pendingRatingKey(id: number) { return `${ratingPrefix}${id}`; }

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

function confirm(key: string, saved: string) {
  const current = localStorage.getItem(key);
  if (current !== null && current !== saved) throw new Error('Pending review request changed. Try creating the backup again.');
  localStorage.removeItem(key);
}

function validateBatch(request: BatchRequest, result: BatchStatus) {
  const ids = new Set(request.vocabulary_ids);
  if (result.operation !== request.operation || !result.items.every((item) => ids.delete(item.vocabulary_id))) throw new Error('Invalid pending batch response identity');
}

async function resolveBatch(runtime: RuntimeAdapter) {
  const saved = localStorage.getItem(pendingBatchKey), request = readPendingBatch();
  if (saved === null || request === null) return null;
  let result = await runtime.reviewBatchStatus(request.operation);
  if (result) validateBatch(request, result);
  if (!result || (!result.cancelled && result.items.length !== new Set(request.vocabulary_ids).size)) result = await runtime.addReviewBatch(request);
  if (!result) throw new Error('Native review storage unavailable');
  validateBatch(request, result);
  confirm(pendingBatchKey, saved);
  return result;
}

export async function resolvePendingReviews(runtime: RuntimeAdapter) {
  const keys = Object.keys(localStorage).filter((key) => key.startsWith(ratingPrefix));
  for (const key of keys) {
    const saved = localStorage.getItem(key);
    if (saved === null) continue;
    const request = parseRating(saved, Number(key.slice(ratingPrefix.length)));
    if (!await runtime.rateReview(request)) throw new Error('Native review storage unavailable');
    confirm(key, saved);
  }
  const undo = localStorage.getItem(pendingUndoKey);
  if (undo !== null) {
    const request = readPendingUndo(null);
    if (!await runtime.undoReview(request.operation, request.ratingOperation)) throw new Error('Native undo unavailable');
    confirm(pendingUndoKey, undo);
  }
  const batch = await resolveBatch(runtime);
  if (Object.keys(localStorage).some((key) => key.startsWith(ratingPrefix) || key === pendingUndoKey || key === pendingBatchKey)) throw new Error('Pending review requests changed. Try creating the backup again.');
  return batch;
}
