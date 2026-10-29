import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { listen } from '@tauri-apps/api/event';
import { recordSchema, saveRequestSchema, saveResultSchema, sortSchema } from './vocabulary';
import type { SaveRequest, SaveResult, VocabularyRecord, VocabularySort } from './vocabulary';
import { dictionarySchema } from './dictionary';
import type { DictionaryLookup } from './dictionary';
import { speechPreferencesSchema } from './speech';
import type { SpeechPreferences } from './speech';

const runtimeStatusSchema = z.object({
  version: z.string().min(1),
  schemaVersion: z.number().int().positive(),
  storage: z.literal('sqlite'),
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
const availabilitySchema = z.object({ progress: progressSchema, reason: z.enum(['clock', 'due', 'quota', 'deferred']).nullable(),
  available_at: z.number().int().safe().nullable() }).strict();
export const rateRequestSchema = z.object({ operation: z.string().min(1).max(128), card_id: z.number().int().positive().safe(),
  expected_revision: z.number().int().nonnegative().safe(), rating: ratingSchema, timezone: z.string().min(1).max(128) }).strict();
const scheduledSchema = z.object({ card: scheduleCardSchema, elapsed_days: z.number().int().nonnegative(),
  step_seconds: z.number().int().positive().nullable(), interval_days: z.number().int().positive().nullable(),
  model: z.object({ version: z.string(), parameters: z.array(z.number().finite()) }).strict(),
  retention_percent: z.number().int().min(70).max(99), timezone_database: z.string() }).strict();
const rateOutcomeSchema = z.discriminatedUnion('status', [
  z.object({ status: z.literal('committed'), result: scheduledSchema }).strict(),
  z.object({ status: z.literal('rejected'), code: z.enum(['clock', 'due', 'quota', 'deferred', 'conflict', 'invalid']), detail: z.string() }).strict(),
]);
export class RatingRejected extends Error {
  constructor(readonly code: 'clock' | 'due' | 'quota' | 'deferred' | 'conflict' | 'invalid', detail: string) { super(detail); }
}
export class UndoRejected extends Error {}
const undoOutcomeSchema = z.discriminatedUnion('status', [
  z.object({ status: z.literal('committed'), queue_rebuilt: z.boolean() }).strict(),
  z.object({ status: z.literal('rejected'), detail: z.string() }).strict(),
]);
export type Rating = z.infer<typeof ratingSchema>;
export type RateRequest = z.infer<typeof rateRequestSchema>;
export type ReviewProgress = z.infer<typeof progressSchema>;
export type ReviewAvailability = z.infer<typeof availabilitySchema>;
const reviewSettingsSchema = z.object({ retention_percent: z.number().int().min(70).max(99),
  new_limit: z.number().int().min(0).max(1_000_000), review_limit: z.number().int().min(0).max(1_000_000) }).strict().readonly();
export type ReviewSettings = z.infer<typeof reviewSettingsSchema>;
const queueCardSchema = z.object({ id: z.number().int().positive().safe(), vocabulary_id: z.number().int().positive().safe(), kind: z.enum(['recognition', 'listening']) }).strict();
const queueViewSchema = z.object({ queue: z.object({ cards: z.array(queueCardSchema), reason: z.enum(['clock', 'due', 'quota', 'deferred']).nullable(), available_at: z.number().int().safe().nullable() }).strict(), record: recordSchema.nullable(), audio_pending: z.boolean(), audio_error: z.string().nullable() }).strict()
  .refine((value) => value.queue.cards.length ? value.record?.id === value.queue.cards[0]?.vocabulary_id : value.record === null, 'Invalid queue record');
export type QueueView = z.infer<typeof queueViewSchema>;

export interface RuntimeAdapter {
  readonly getStatus: () => Promise<RuntimeStatus | null>;
  readonly lookupDictionary: (key: string) => Promise<DictionaryLookup | null>;
  readonly findVocabulary: (headword: string) => Promise<VocabularyRecord | null>;
  readonly listVocabulary: (query: string, sort: VocabularySort) => Promise<readonly VocabularyRecord[] | null>;
  readonly saveVocabulary: (request: SaveRequest) => Promise<SaveResult | null>;
  readonly vocabularySort: (sort?: VocabularySort) => Promise<VocabularySort | null>;
  readonly speechPreferences: (preferences?: SpeechPreferences) => Promise<SpeechPreferences | null>;
  readonly addReview: (id: number) => Promise<readonly ReviewCard[] | null>;
  readonly reviewCards: (id: number) => Promise<readonly ReviewCard[] | null>;
  readonly reviewProgress: (id: number) => Promise<ReviewProgress | null>;
  readonly reviewAvailability: (id: number) => Promise<ReviewAvailability | null>;
  readonly rateReview: (request: RateRequest) => Promise<ReviewProgress | null>;
  readonly undoTarget: () => Promise<string | null>;
  readonly undoReview: (operation: string, ratingOperation: string) => Promise<{ readonly queue_rebuilt: boolean } | null>;
  readonly reviewSettings: (settings?: ReviewSettings) => Promise<ReviewSettings | null>;
  readonly reviewQueue: (skipped?: readonly number[]) => Promise<QueueView | null>;
  readonly generateSpeech: (id: number, regenerate?: boolean) => Promise<Uint8Array | null>;
  readonly cachedSpeech: (id: number) => Promise<Uint8Array | null>;
  readonly onCloseRequested: (callback: () => void) => Promise<() => void>;
  readonly finishClose: () => Promise<void>;
}

export const runtime: RuntimeAdapter = {
  async getStatus() {
    if (!isTauri()) return null;
    return runtimeStatusSchema.parse(await invoke('runtime_status'));
  },
  async lookupDictionary(key) {
    if (!isTauri()) return null;
    return dictionarySchema.parse(await invoke('dictionary_lookup', { key }));
  },
  async findVocabulary(headword) {
    if (!isTauri()) return null;
    return recordSchema.nullable().parse(await invoke('vocabulary_find', { headword }));
  },
  async listVocabulary(query, sort) {
    if (!isTauri()) return null;
    return recordSchema.array().readonly().parse(await invoke('vocabulary_list', { query, sort: sortSchema.parse(sort) }));
  },
  async saveVocabulary(request) {
    if (!isTauri()) return null;
    return saveResultSchema.parse(await invoke('vocabulary_save', { request: saveRequestSchema.parse(request) }));
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
    return queueViewSchema.parse(await invoke('review_queue', { timezone: Intl.DateTimeFormat().resolvedOptions().timeZone, skipped: z.array(z.number().int().positive().safe()).max(10_000).parse(skipped) }));
  },
  async generateSpeech(id, regenerate) {
    if (!isTauri()) return null;
    const bytes = z.array(z.number().int().min(0).max(255)).min(1).max(10_000_000)
      .parse(await invoke('speech_generate', { vocabularyId: z.number().int().positive().safe().parse(id),
        ...(regenerate ? { regenerate: true } : {}) }));
    return new Uint8Array(bytes);
  },
  async cachedSpeech(id) {
    if (!isTauri()) return null;
    const bytes = z.array(z.number().int().min(0).max(255)).min(1).max(10_000_000).nullable()
      .parse(await invoke('speech_cached', { vocabularyId: z.number().int().positive().safe().parse(id) }));
    return bytes === null ? null : new Uint8Array(bytes);
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
