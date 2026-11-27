import { z } from 'zod';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { dictionaryQuery } from './dictionary';

export const CEFR_QUERY_LIMIT = 200;
export const cefrQueries = z.array(z.object({ key: dictionaryQuery, sense: z.object({
  sense_id: z.string().min(1).max(128).refine((value) => !/\p{Cc}/u.test(value)),
  pos: z.string().min(1).max(16), source_headword: dictionaryQuery,
}).strict().readonly().nullable() }).strict().readonly()).min(1).max(CEFR_QUERY_LIMIT).readonly();
export type CefrQuery = z.infer<typeof cefrQueries>[number];
export const cefrSettingsSchema = z.object({ row_count: z.number().int().positive().nullable() }).strict().readonly();
export type CefrSettings = z.infer<typeof cefrSettingsSchema>;
export const cefrLookupSchema = z.object({ imported: z.boolean(), entries: z.array(z.object({
  key: dictionaryQuery, sense_id: z.string().nullable(), level: z.enum(['A1', 'A2', 'B1', 'B2', 'C1', 'C2']).nullable(),
}).strict().readonly()).max(CEFR_QUERY_LIMIT).readonly() }).strict().readonly();
export type CefrLookup = z.infer<typeof cefrLookupSchema>;

export const cefrRuntime = {
  async cefrSettings(): Promise<CefrSettings | null> {
    return isTauri() ? cefrSettingsSchema.parse(await invoke('cefr_settings')) : null;
  },
  async importCefr(file: File): Promise<CefrSettings | null> {
    if (!isTauri()) return null;
    if (!/\.tsv$/i.test(file.name) || !file.size || file.size > 5 * 1024 * 1024)
      throw new Error('FLELex: select a non-empty .tsv file of at most 5 MiB');
    return cefrSettingsSchema.parse(await invoke('cefr_import', new Uint8Array(await file.arrayBuffer())));
  },
  async removeCefr(): Promise<CefrSettings | null> {
    return isTauri() ? cefrSettingsSchema.parse(await invoke('cefr_remove')) : null;
  },
  async lookupCefr(queries: readonly CefrQuery[]): Promise<CefrLookup | null> {
    if (!isTauri()) return null;
    const valid = cefrQueries.parse(queries);
    const result = cefrLookupSchema.parse(await invoke('cefr_lookup', { queries: valid }));
    if (result.entries.length !== valid.length || result.entries.some((entry, index) =>
      entry.key !== valid[index].key || entry.sense_id !== (valid[index].sense?.sense_id ?? null))
      || (!result.imported && result.entries.some((entry) => entry.level !== null)))
      throw new Error('CEFR response differs from the requested resource or senses');
    return result;
  },
};
