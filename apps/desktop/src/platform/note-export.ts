import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { characterCount, sortSchema } from './vocabulary';

const id = z.number().int().positive().safe();
export const noteExportScopeSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('all'), sort: sortSchema }).strict(),
  z.object({ kind: z.literal('search'), query: z.string().refine((value) => characterCount(value) <= 20_000, 'Search is too long'), sort: sortSchema,
    category: z.union([z.literal('all'), z.literal('uncategorized'), id]) }).strict(),
  z.object({ kind: z.literal('single'), id }).strict(),
]);
export type NoteExportScope = z.infer<typeof noteExportScopeSchema>;
const count = z.number().int().nonnegative().safe();
const snapshotSchema = z.object({ token: z.string().min(1), records: count, rows: count }).strict().readonly();
export type NoteExportSnapshot = z.infer<typeof snapshotSchema>;
const resultSchema = z.object({ state: z.enum(['saved', 'cancelled']), records: count, rows: count }).strict().readonly();

export const noteExportRuntime = {
  async noteExportSnapshot(scope: NoteExportScope) {
    if (!isTauri()) throw new Error('Note export is available in the desktop app only');
    return snapshotSchema.parse(await invoke('note_export_snapshot', { scope: noteExportScopeSchema.parse(scope) }));
  },
  async exportNoteCsv(token: string) {
    if (!isTauri()) throw new Error('Note export is available in the desktop app only');
    return resultSchema.parse(await invoke('note_export_csv', { token: z.string().min(1).parse(token) }));
  },
};
