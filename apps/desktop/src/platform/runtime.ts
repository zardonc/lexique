import { invoke, isTauri } from '@tauri-apps/api/core';
import dictionaryResources from '../dictionary-resources.json';
import { conjugationQuery, conjugationSchema, conjugationPreferencesSchema, conjugationOrderSchema } from './conjugation';
import type { ConjugationLookup, ConjugationPreferences, ConjugationOrder } from './conjugation';
import { z } from 'zod';
import { listen } from '@tauri-apps/api/event';
import { recordSchema, saveRequestSchema, saveResultSchema, sortSchema } from './vocabulary';
import type { SaveRequest, SaveResult, VocabularyRecord, VocabularySort } from './vocabulary';
import { dictionarySchema, dictionaryQuery, dictionaryId, dictionaryCatalog, dictionaryOrder } from './dictionary';
import type { DictionaryLookup, DictionaryCatalog } from './dictionary';
import { frequencyQuery, frequencySchema } from './frequency';
import type { FrequencyLookup } from './frequency';
import { speechAudioSchema, speechPreferencesSchema, speechVoiceSchema } from './speech';
import type { SpeechAudio, SpeechPreferences } from './speech';
import { readSelectionEvent, selectionText, shortcutStatusSchema, shortcutText } from './selection';
import type { SelectionEvent, ShortcutStatus } from './selection';
import type { Locale } from '../content';
import { interfacePreferencesSchema } from './interface';
import type { InterfacePreferences } from './interface';

const runtimeStatusSchema = z.object({
  version: z.string().min(1),
  schemaVersion: z.number().int().positive(),
  storage: z.literal('sqlite'),
  audioMigration: z.object({ pending: z.number().int().nonnegative().safe(), failed: z.number().int().nonnegative().safe() }).strict().readonly(),
  audioCache: z.object({
    logical_bytes: z.number().int().nonnegative().safe(), permanent_bytes: z.number().int().nonnegative().safe(),
    protected_bytes: z.number().int().nonnegative().safe(), reclaimable_bytes: z.number().int().nonnegative().safe(),
    publication_bytes: z.number().int().nonnegative().safe(), unowned_bytes: z.number().int().nonnegative().safe(),
    marked_bytes: z.number().int().nonnegative().safe(), over_capacity: z.boolean(), gc_failures: z.number().int().nonnegative().safe(),
    file_bytes: z.number().int().nonnegative().safe(), allocated_bytes: z.number().int().nonnegative().safe().nullable(),
    unknown_files: z.number().int().nonnegative().safe(), unknown_bytes: z.number().int().nonnegative().safe(),
    unsafe_files: z.number().int().nonnegative().safe(), scan_failures: z.number().int().nonnegative().safe(),
    scanned_entries: z.number().int().nonnegative().safe(), in_progress: z.boolean(),
    last_reconciled_at: z.number().int().nonnegative().safe().nullable(), error: z.string().nullable(),
  }).strict().readonly().nullable().optional(),
}).strict().readonly();

export type RuntimeStatus = z.infer<typeof runtimeStatusSchema>;

const cardsSchema = z.array(z.object({ id: z.number().int().positive().safe(),
  kind: z.enum(['recognition', 'listening']) }).strict().readonly()).max(2).readonly();
export type ReviewCard = z.infer<typeof cardsSchema>[number];
const ratingSchema = z.enum(['again', 'hard', 'good', 'easy']);
const scheduleCardSchema = z.object({ phase: z.enum(['new', 'learning_first', 'learning_last', 'review', 'relearning']),
  memory: z.object({ stability: z.number().finite(), difficulty: z.number().finite() }).strict().nullable(),
  last_review_at: z.number().int().safe().nullable(), due_at: z.number().int().safe().nullable() }).strict();
const progressSchema = z.object({ revision: z.number().int().nonnegative().safe(), card: scheduleCardSchema }).strict();
const availabilitySchema = z.object({ progress: progressSchema, reason: z.enum(['clock', 'due', 'quota', 'deferred', 'paused']).nullable(),
  available_at: z.number().int().safe().nullable() }).strict();
export const rateRequestSchema = z.object({ operation: z.string().min(1).max(128), card_id: z.number().int().positive().safe(),
  expected_revision: z.number().int().nonnegative().safe(), rating: ratingSchema, timezone: z.string().min(1).max(128) }).strict();
const scheduledSchema = z.object({ card: scheduleCardSchema, elapsed_days: z.number().int().nonnegative(),
  step_seconds: z.number().int().positive().nullable(), interval_days: z.number().int().positive().nullable(),
  model: z.object({ version: z.string(), parameters: z.array(z.number().finite()) }).strict(),
  retention_percent: z.number().int().min(70).max(99), timezone_database: z.string() }).strict();
const rateOutcomeSchema = z.discriminatedUnion('status', [
  z.object({ status: z.literal('committed'), result: scheduledSchema }).strict(),
  z.object({ status: z.literal('rejected'), code: z.enum(['clock', 'due', 'quota', 'deferred', 'paused', 'conflict', 'invalid']), detail: z.string() }).strict(),
]);
export class RatingRejected extends Error {
  constructor(readonly code: 'clock' | 'due' | 'quota' | 'deferred' | 'paused' | 'conflict' | 'invalid', detail: string) { super(detail); }
}
export class UndoRejected extends Error {}
export class AudioGenerationFailed extends Error {}
const undoOutcomeSchema = z.discriminatedUnion('status', [
  z.object({ status: z.literal('committed'), queue_rebuilt: z.boolean() }).strict(),
  z.object({ status: z.literal('rejected'), detail: z.string() }).strict(),
]);
export type Rating = z.infer<typeof ratingSchema>;
export type RateRequest = z.infer<typeof rateRequestSchema>;
export type ReviewProgress = z.infer<typeof progressSchema>;
export type ReviewAvailability = z.infer<typeof availabilitySchema>;
const reviewSettingsSchema = z.object({ retention_percent: z.number().int().min(70).max(99),
  new_limit: z.number().int().min(0).max(1_000_000), review_limit: z.number().int().min(0).max(1_000_000),
  new_cards_ignore_review_limit: z.boolean() }).strict().readonly();
export type ReviewSettings = z.infer<typeof reviewSettingsSchema>;
const queueCardSchema = z.object({ id: z.number().int().positive().safe(), vocabulary_id: z.number().int().positive().safe(), kind: z.enum(['recognition', 'listening']) }).strict();
const windowEntrySchema = z.object({ card: queueCardSchema, record: recordSchema, progress: progressSchema,
  version: z.string().regex(/^[a-f0-9]{64}$/).nullable(), content_hash: z.string().regex(/^[a-f0-9]{64}$/).nullable(),
  audio: speechAudioSchema.nullable() }).strict()
  .refine((value) => value.record.id === value.card.vocabulary_id && (value.card.kind !== 'listening' || value.version !== null && value.content_hash !== null), 'Invalid verified queue entry');
const queuePayloadSchema = z.object({ queue: z.object({ cards: z.array(queueCardSchema).max(4), reason: z.enum(['clock', 'due', 'quota', 'deferred']).nullable(), available_at: z.number().int().safe().nullable() }).strict(), record: recordSchema.nullable(), audio_pending: z.boolean(), audio_error: z.string().nullable(), audio_vocabulary_id: z.number().int().positive().safe().nullable().default(null),
  summary: z.object({ admitted_count: z.number().int().nonnegative().safe(), audio_pending_count: z.number().int().nonnegative().safe(), missing_vocabulary_ids: z.array(z.number().int().positive().safe()).max(100), verification_pending: z.boolean(), learning_day: z.string(), timezone: z.string().min(1).max(128), revision: z.number().int().nonnegative().safe(), unknown_legacy: z.boolean().default(false) }).strict(),
  delta: z.object({ before_revision: z.number().int().nonnegative().safe().nullable(), after_revision: z.number().int().nonnegative().safe(), removed: z.array(z.number().int().positive().safe()).max(4), order: z.array(z.number().int().positive().safe()).max(4) }).strict(),
  window: z.array(windowEntrySchema).max(4), metrics: z.object({ metadata_candidates: z.number().int().min(0).max(50), strict_probes: z.number().int().min(0).max(8), compressed_bytes: z.number().int().min(0).max(33_554_432), payload_bytes: z.number().int().min(0).max(33_554_432), validation_wait_ms: z.number().nonnegative().finite(), metadata_loaded_rows: z.number().int().nonnegative().safe(), parsed_progress: z.number().int().nonnegative().safe(), metadata_rebuilt: z.boolean(), rebuild_reason: z.string().nullable(), database_ms: z.number().nonnegative().finite() }).strict() }).strict()
  .refine((value) => value.delta.after_revision === value.summary.revision && value.delta.order.length === value.queue.cards.length && value.delta.order.every((id, index) => id === value.queue.cards[index]?.id), 'Invalid queue delta order')
  .refine((value) => new Set(value.delta.order).size === value.delta.order.length && new Set(value.window.map((entry) => entry.card.id)).size === value.window.length && value.window.every((entry) => value.delta.order.includes(entry.card.id)), 'Invalid queue delta entries');
const queueViewSchema = queuePayloadSchema
  .refine((value) => value.queue.cards.length ? value.record?.id === value.queue.cards[0]?.vocabulary_id : value.record === null, 'Invalid queue record')
  .refine((value) => value.window.length === value.queue.cards.length && value.window.every((entry, index) => entry.card.id === value.queue.cards[index]?.id && (entry.card.kind !== 'listening' || entry.audio !== null)) && value.window.reduce((sum, entry) => sum + (entry.audio?.bytes.length ?? 0), 0) <= 33_554_432, 'Invalid queue window');
export type QueueView = z.infer<typeof queueViewSchema>;

let receivedQueue: QueueView | null = null;
function queueCursor(view: QueueView | null) {
  return view && { revision: view.summary.revision, window: view.window.map((entry) => ({ card: entry.card,
    record_revision: entry.record.revision, progress_revision: entry.progress.revision,
    version: entry.version, content_hash: entry.content_hash })) };
}
function reuseQueueAudio(entry: QueueView['window'][number], known: QueueView['window'][number] | undefined) {
  if (entry.audio !== null) return entry.audio;
  if (!known || known.version !== entry.version || known.content_hash !== entry.content_hash || known.card.vocabulary_id !== entry.card.vocabulary_id) return null;
  return known.audio;
}
function applyQueueDelta(payload: z.infer<typeof queuePayloadSchema>, previous: QueueView | null) {
  const { delta } = payload;
  if (delta.before_revision !== null && delta.before_revision !== previous?.summary.revision) throw new Error('Queue delta revision mismatch');
  const entries = new Map(delta.before_revision === null ? [] : previous?.window.map((entry) => [entry.card.id, entry]));
  for (const id of delta.removed) entries.delete(id);
  for (const entry of payload.window) {
    entries.set(entry.card.id, { ...entry, audio: reuseQueueAudio(entry, entries.get(entry.card.id)) });
  }
  const window = delta.order.map((id) => { const entry = entries.get(id); if (!entry) throw new Error('Queue delta references an unknown card'); return entry; });
  return queueViewSchema.parse({ ...payload, window });
}

export const batchRequestSchema = z.object({ operation: z.string().min(1).max(128), vocabulary_ids: z.array(z.number().int().positive().safe()).min(1).max(10_000) }).strict().readonly();
const batchSchema = z.object({ operation: z.string().min(1).max(128), cancelled: z.boolean(), items: z.array(z.object({
  vocabulary_id: z.number().int().safe(), created_cards: z.number().int().min(0).max(2), error: z.string().nullable(),
  audio_state: z.enum(['queued', 'running', 'retry_wait', 'ready', 'failed', 'cancelled']).nullable(), audio_error: z.string().nullable(),
}).strict().readonly()).max(10_000).readonly() }).strict().readonly();
export type BatchRequest = z.infer<typeof batchRequestSchema>;
export type BatchStatus = z.infer<typeof batchSchema>;

export interface RuntimeAdapter {
  readonly getStatus: () => Promise<RuntimeStatus | null>;
  readonly interfacePreferences: (preferences?: InterfacePreferences) => Promise<InterfacePreferences | null>;
  readonly lookupDictionary: (key: string, dictionaryId?: string) => Promise<DictionaryLookup | null>;
  readonly dictionarySettings: (order?: readonly string[]) => Promise<DictionaryCatalog | null>;
  readonly importDictionary: (file: File) => Promise<DictionaryCatalog | null>;
  readonly removeDictionary: (id: string) => Promise<DictionaryCatalog | null>;
  readonly openDictionaryResource: (id: string) => Promise<void>;
  readonly lookupFrequency: (keys: readonly string[]) => Promise<FrequencyLookup | null>;
  readonly onSelectionInput: (callback: (event: SelectionEvent) => void) => Promise<() => void>;
  readonly captureSelection: (text?: string) => Promise<void>;
  readonly cancelSelection: () => Promise<void>;
  readonly selectionShortcut: (shortcut?: string) => Promise<ShortcutStatus | null>;
  readonly importOcrImage: (bytes: Uint8Array) => Promise<void>;
  readonly captureScreen: () => Promise<void>;
  readonly setCaptureLocale: (locale: Locale) => Promise<void>;
  readonly ocrShortcut: (shortcut?: string) => Promise<ShortcutStatus | null>;
  readonly lookupConjugation: (key: string) => Promise<ConjugationLookup | null>;
  readonly conjugationPreferences: (preferences?: ConjugationPreferences) => Promise<ConjugationPreferences | null>;
  readonly conjugationBlockOrder: (order?: ConjugationOrder) => Promise<ConjugationOrder | null>;
  readonly findVocabulary: (headword: string) => Promise<VocabularyRecord | null>;
  readonly listVocabulary: (query: string, sort: VocabularySort) => Promise<readonly VocabularyRecord[] | null>;
  readonly saveVocabulary: (request: SaveRequest, dictionaryId?: string) => Promise<SaveResult | null>;
  readonly vocabularySort: (sort?: VocabularySort) => Promise<VocabularySort | null>;
  readonly speechPreferences: (preferences?: SpeechPreferences) => Promise<SpeechPreferences | null>;
  readonly addReview: (id: number) => Promise<readonly ReviewCard[] | null>;
  readonly addReviewBatch: (request: BatchRequest) => Promise<BatchStatus | null>;
  readonly reviewBatchStatus: (operation?: string) => Promise<BatchStatus | null>;
  readonly cancelReviewBatch: (operation: string) => Promise<void>;
  readonly retryMissingSpeech: (id: number) => Promise<number | null>;
  readonly onReviewChanged: (callback: () => void) => Promise<() => void>;
  readonly reviewCards: (id: number) => Promise<readonly ReviewCard[] | null>;
  readonly reviewPause: (id: number, paused?: boolean) => Promise<boolean | null>;
  readonly reviewAutoEnroll: (enabled?: boolean) => Promise<boolean | null>;
  readonly reviewProgress: (id: number) => Promise<ReviewProgress | null>;
  readonly reviewAvailability: (id: number) => Promise<ReviewAvailability | null>;
  readonly rateReview: (request: RateRequest) => Promise<ReviewProgress | null>;
  readonly undoTarget: () => Promise<string | null>;
  readonly undoReview: (operation: string, ratingOperation: string) => Promise<{ readonly queue_rebuilt: boolean } | null>;
  readonly reviewSettings: (settings?: ReviewSettings) => Promise<ReviewSettings | null>;
  readonly reviewQueue: (skipped?: readonly number[]) => Promise<QueueView | null>;
  readonly generateSpeech: (id: number, regenerate?: boolean, operation?: string) => Promise<SpeechAudio | null>;
  readonly previewSpeech: (text: string, voice?: SpeechPreferences['voice']) => Promise<SpeechAudio | null>;
  readonly cachedSpeech: (id: number) => Promise<SpeechAudio | null>;
  readonly onCloseRequested: (callback: () => void) => Promise<() => void>;
  readonly finishClose: () => Promise<void>;
}

async function readShortcut(command: string, shortcut?: string) {
  if (!isTauri()) return null;
  const valid = shortcut === undefined ? null : shortcutText.parse(shortcut);
  const result = shortcutStatusSchema.parse(await invoke(command, { shortcut: valid }));
  if (valid !== null && result.shortcut !== valid) throw new Error('Saved shortcut differs from the request');
  return result;
}

export const runtime: RuntimeAdapter = {
  async interfacePreferences(preferences) {
    const valid = preferences === undefined ? null : interfacePreferencesSchema.parse(preferences);
    if (!isTauri()) return null;
    const result = interfacePreferencesSchema.parse(await invoke('interface_preferences', { preferences: valid }));
    if (valid && (result.locale !== valid.locale || result.theme !== valid.theme)) throw new Error('Saved interface preferences differ from the request');
    return result;
  },
  async setCaptureLocale(locale) { if (isTauri()) await invoke('ocr_locale', { locale }); },
  async captureScreen() {
    if (!isTauri()) throw new Error('Screen capture is available in the desktop app');
    await invoke('ocr_capture');
  },
  async importOcrImage(bytes) {
    if (!isTauri()) throw new Error('Image import is available in the desktop app');
    if (!bytes.length || bytes.length > 16 * 1024 * 1024) throw new Error('PNG/JPEG: 1–16 MiB');
    await invoke('ocr_image', bytes);
  },
  async onSelectionInput(callback) {
    if (!isTauri()) return () => {};
    return listen('selection-input', (event) => callback(readSelectionEvent(event.payload)));
  },
  async captureSelection(text) {
    if (!isTauri()) throw new Error('Selection capture is available in the desktop app');
    await invoke('selection_capture', { text: text === undefined ? null : selectionText.parse(text) });
  },
  async cancelSelection() { if (isTauri()) await invoke('selection_cancel'); },
  async selectionShortcut(shortcut) {
    return readShortcut('selection_shortcut', shortcut);
  },
  async ocrShortcut(shortcut) { return readShortcut('ocr_shortcut', shortcut); },
  async addReviewBatch(request) {
    if (!isTauri()) return null;
    const valid = batchRequestSchema.parse(request);
    const result = batchSchema.parse(await invoke('review_add_batch', { request: valid }));
    const ids = new Set(valid.vocabulary_ids);
    if (result.operation !== valid.operation || result.items.length !== ids.size || !result.items.every((item) => ids.delete(item.vocabulary_id))) throw new Error('Invalid batch response identity');
    return result;
  },
  async reviewBatchStatus(operation) {
    if (!isTauri()) return null;
    return batchSchema.nullable().parse(await invoke('review_batch_status', { operation: operation === undefined ? null : z.string().min(1).max(128).parse(operation) }));
  },
  async cancelReviewBatch(operation) {
    if (isTauri()) await invoke('review_cancel_batch', { operation: z.string().min(1).max(128).parse(operation) });
  },
  async retryMissingSpeech(id) {
    if (!isTauri()) return null;
    return z.number().int().min(0).max(1).parse(await invoke('speech_retry_missing', { vocabularyId: z.number().int().positive().safe().parse(id) }));
  },
  async onReviewChanged(callback) {
    if (!isTauri()) return () => {};
    return listen('lexique-review-changed', (event) => { z.null().parse(event.payload); callback(); });
  },
  async getStatus() {
    if (!isTauri()) return null;
    return runtimeStatusSchema.parse(await invoke('runtime_status'));
  },
  async dictionarySettings(order) {
    if (!isTauri()) return null;
    const valid = order === undefined ? null : dictionaryOrder.parse(order);
    const result = dictionaryCatalog.parse(await invoke('dictionary_settings', { order: valid }));
    if (valid && result.some((item, index) => item.id !== valid[index])) throw new Error('Saved dictionary order differs from the request');
    return result;
  },
  async openDictionaryResource(id) {
    const resource = dictionaryResources.find((item) => item.id === id);
    if (!resource) throw { code: 'invalid_query', message: 'Unknown dictionary resource' };
    if (isTauri()) await invoke('dictionary_resource_open', { id });
    else window.open(resource.url, '_blank', 'noopener,noreferrer');
  },
  async importDictionary(file) {
    if (!isTauri()) return null;
    if (!/\.mdx$/i.test(file.name) || !file.size || file.size > 128 * 1024 * 1024) throw { code: 'limit', message: 'MDX: non-empty, at most 128 MiB' };
    const name = btoa(Array.from(new TextEncoder().encode(file.name), (byte) => String.fromCharCode(byte)).join(''));
    return dictionaryCatalog.parse(await invoke('dictionary_import', new Uint8Array(await file.arrayBuffer()), { headers: { 'x-lexique-dictionary-name': name } }));
  },
  async removeDictionary(id) {
    if (!isTauri()) return null;
    return dictionaryCatalog.parse(await invoke('dictionary_remove', { id: dictionaryId.parse(id) }));
  },
  async lookupDictionary(key, id) {
    if (!isTauri()) return null;
    if (!dictionaryQuery.safeParse(key).success) throw { code: 'invalid_query', message: 'Enter 1–512 UTF-8 bytes without control characters' };
    const result = dictionarySchema.parse(await invoke('dictionary_lookup', { key, dictionary_id: id === undefined ? null : dictionaryId.parse(id) }));
    if (id !== undefined && (result.dictionary_id ?? 'builtin') !== id) throw new Error('Dictionary response source differs from the request');
    return result;
  },
  async lookupFrequency(keys) {
    if (!isTauri()) return null;
    const valid = frequencyQuery.parse(keys);
    const result = frequencySchema.parse(await invoke('frequency_lookup', { keys: valid }));
    if (result.entries.length !== valid.length || result.entries.some((entry, index) => entry.key !== valid[index]))
      throw new Error('Frequency response queries differ from the request');
    return result;
  },
  async lookupConjugation(key) {
    if (!isTauri()) return null;
    if (!conjugationQuery.safeParse(key).success) throw { code: 'invalid_query', message: 'Enter 1–512 UTF-8 bytes without control characters' };
    const result = conjugationSchema.parse(await invoke('conjugation_lookup', { key }));
    if (result.query !== key) throw new Error('Conjugation response query differs from the request');
    return result;
  },
  async conjugationPreferences(preferences) {
    if (!isTauri()) return null;
    const valid = preferences === undefined ? null : conjugationPreferencesSchema.parse(preferences);
    const result = conjugationPreferencesSchema.parse(await invoke('conjugation_preferences', { preferences: valid }));
    if (valid && result.mode !== valid.mode) throw new Error('Saved conjugation mode differs from the request');
    return result;
  },
  async conjugationBlockOrder(order) {
    if (!isTauri()) return null;
    const valid = order === undefined ? null : conjugationOrderSchema.parse(order);
    const result = conjugationOrderSchema.parse(await invoke('conjugation_block_order', { order: valid }));
    if (valid && result.some((code, index) => code !== valid[index])) throw new Error('Saved conjugation block order differs from the request');
    return result;
  },
  async findVocabulary(headword) {
    if (!isTauri()) return null;
    return recordSchema.nullable().parse(await invoke('vocabulary_find', { headword }));
  },
  async listVocabulary(query, sort) {
    if (!isTauri()) return null;
    return recordSchema.array().readonly().parse(await invoke('vocabulary_list', { query, sort: sortSchema.parse(sort) }));
  },
  async saveVocabulary(request, id) {
    if (!isTauri()) return null;
    return saveResultSchema.parse(await invoke('vocabulary_save', { request: saveRequestSchema.parse(request), dictionary_id: id === undefined ? null : dictionaryId.parse(id) }));
  },
  async vocabularySort(sort) {
    if (!isTauri()) return null;
    return sortSchema.parse(await invoke('vocabulary_sort', { sort: sort === undefined ? null : sortSchema.parse(sort) }));
  },
  async addReview(id) {
    if (!isTauri()) return null;
    return cardsSchema.parse(await invoke('review_add', { vocabularyId: z.number().int().positive().safe().parse(id) }));
  },
  async reviewCards(id) {
    if (!isTauri()) return null;
    return cardsSchema.parse(await invoke('review_cards', { vocabularyId: z.number().int().positive().safe().parse(id) }));
  },
  async reviewPause(id, paused) {
    if (!isTauri()) return null;
    return z.boolean().parse(await invoke('review_pause', { vocabularyId: z.number().int().positive().safe().parse(id), paused: paused === undefined ? null : z.boolean().parse(paused) }));
  },
  async reviewAutoEnroll(enabled) {
    if (!isTauri()) return null;
    return z.boolean().parse(await invoke('review_auto_enroll', { enabled: enabled === undefined ? null : z.boolean().parse(enabled) }));
  },
  async reviewProgress(id) {
    if (!isTauri()) return null;
    return progressSchema.parse(await invoke('review_progress', { cardId: z.number().int().positive().safe().parse(id) }));
  },
  async reviewAvailability(id) {
    if (!isTauri()) return null;
    return availabilitySchema.parse(await invoke('review_availability', { cardId: z.number().int().positive().safe().parse(id),
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone }));
  },
  async rateReview(request) {
    if (!isTauri()) return null;
    const valid = rateRequestSchema.parse(request);
    const outcome = rateOutcomeSchema.parse(await invoke('review_rate', { request: valid }));
    if (outcome.status === 'rejected') throw new RatingRejected(outcome.code, outcome.detail);
    return { revision: valid.expected_revision + 1, card: outcome.result.card };
  },
  async reviewSettings(settings) {
    if (!isTauri()) return null;
    return reviewSettingsSchema.parse(await invoke('review_settings', { settings: settings === undefined ? null : reviewSettingsSchema.parse(settings) }));
  },
  async undoTarget() {
    if (!isTauri()) return null;
    return z.string().min(1).max(128).nullable().parse(await invoke('review_undo_target'));
  },
  async undoReview(operation, ratingOperation) {
    if (!isTauri()) return null;
    const valid = z.string().min(1).max(128);
    const outcome = undoOutcomeSchema.parse(await invoke('review_undo', { operation: valid.parse(operation), ratingOperation: valid.parse(ratingOperation) }));
    if (outcome.status === 'rejected') throw new UndoRejected(outcome.detail);
    return { queue_rebuilt: outcome.queue_rebuilt };
  },
  async reviewQueue(skipped = []) {
    if (!isTauri()) return null;
    const previous = receivedQueue;
    const payload = queuePayloadSchema.parse(await invoke('review_queue', { timezone: Intl.DateTimeFormat().resolvedOptions().timeZone, skipped: z.array(z.number().int().positive().safe()).max(10_000).parse(skipped), cursor: queueCursor(previous) }));
    const result = applyQueueDelta(payload, previous);
    if (receivedQueue !== previous) throw new Error('Queue response arrived out of sequence');
    receivedQueue = result;
    return result;
  },
  async generateSpeech(id, regenerate, operation = crypto.randomUUID()) {
    if (!isTauri()) return null;
    const validOperation = z.string().min(1).max(128).refine((value) => [...value].every((character) => {
      const code = character.charCodeAt(0); return code >= 32 && (code < 127 || code > 159);
    })).parse(operation);
    try {
      return speechAudioSchema.parse(await invoke('speech_generate', { vocabularyId: z.number().int().positive().safe().parse(id), operation: validOperation,
          ...(regenerate ? { regenerate: true } : {}) }));
    } catch (cause) {
      if (typeof cause === 'string' && cause.startsWith('Audio generation failed: ')) throw new AudioGenerationFailed(cause);
      throw cause;
    }
  },
  async cachedSpeech(id) {
    if (!isTauri()) return null;
    return speechAudioSchema.nullable()
      .parse(await invoke('speech_cached', { vocabularyId: z.number().int().positive().safe().parse(id) }));
  },
  async previewSpeech(text, voice) {
    if (!isTauri()) return null;
    return speechAudioSchema.parse(await invoke('speech_preview', {
      text, ...(voice === undefined ? {} : { voice: speechVoiceSchema.parse(voice) }),
    }));
  },
  async onCloseRequested(callback) {
    if (!isTauri()) return () => {};
    return listen('lexique-close-requested', callback);
  },
  async speechPreferences(preferences) {
    if (!isTauri()) return null;
    return speechPreferencesSchema.parse(await invoke('speech_preferences', {
      preferences: preferences === undefined ? null : speechPreferencesSchema.parse(preferences),
    }));
  },
  async finishClose() {
    if (isTauri()) await invoke('finish_close');
  },
};
