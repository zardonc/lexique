import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { importSourceSchema, recordSchema, saveResultSchema } from './vocabulary';
import { noteEnrichmentSchema } from './note-enrichment';

const operationSchema = z.string().min(1).max(128).regex(/^[a-zA-Z0-9-]+$/);
export const noteImportPreviewSchema = z.object({
  operation: operationSchema, source: importSourceSchema,
  content: z.object({ headword: z.string(), notes: z.string().nullable(),
    examples: z.array(z.object({ fr: z.string(), zh: z.string() }).strict().readonly()).readonly().nullable(),
  }).strict().readonly().nullable(),
  rows: z.array(z.number().int().nonnegative().safe()).readonly(),
  existing: recordSchema.nullable(), status: z.enum(['create', 'identical', 'different', 'invalid']), error: z.string().nullable(),
  enrichment: noteEnrichmentSchema.nullable(),
}).strict().readonly();
export type NoteImportPreview = z.infer<typeof noteImportPreviewSchema>;

export const noteImportRuntime = {
  async previewNoteImport(operation: string): Promise<NoteImportPreview | null> {
    operationSchema.parse(operation);
    if (!isTauri()) throw new Error('CSV import requires the desktop application');
    return noteImportPreviewSchema.nullable().parse(await invoke('note_import_preview', { operation }));
  },
  async commitNoteImport(operation: string, allowUnlinked = false) {
    operationSchema.parse(operation);
    if (!isTauri()) throw new Error('CSV import requires the desktop application');
    return saveResultSchema.parse(await invoke('note_import_commit', { operation, allowUnlinked: z.boolean().parse(allowUnlinked) }));
  },
  async cancelNoteImport(operation: string) {
    operationSchema.parse(operation);
    if (isTauri()) await invoke('note_import_cancel', { operation });
  },
};
