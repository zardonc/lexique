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
