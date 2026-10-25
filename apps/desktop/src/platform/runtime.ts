import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { dictionarySchema } from './dictionary';
import type { DictionaryLookup } from './dictionary';

const runtimeStatusSchema = z.object({
  version: z.string().min(1),
  schemaVersion: z.number().int().positive(),
  storage: z.literal('sqlite'),
}).strict().readonly();

export type RuntimeStatus = z.infer<typeof runtimeStatusSchema>;

export interface RuntimeAdapter {
  readonly getStatus: () => Promise<RuntimeStatus | null>;
  readonly lookupDictionary: (key: string) => Promise<DictionaryLookup | null>;
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
};
