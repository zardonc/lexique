import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { noteEnrichmentSchema } from './note-enrichment';
import { importSourceSchema, recordSchema } from './vocabulary';

const operationSchema = z.string().min(1).max(110).regex(/^[a-zA-Z0-9-]+$/);
const column = z.number().int().nonnegative().safe().nullable();
export const noteImportMappingSchema = z.object({ headword: column, notes: column, example_fr: column, example_zh: column }).strict().readonly();
export type NoteImportMapping = z.infer<typeof noteImportMappingSchema>;
const optionsSchema = z.object({ delimiter: z.enum(['comma', 'semicolon']), mapping: noteImportMappingSchema.nullable() }).strict().readonly();
export type NoteImportOptions = z.infer<typeof optionsSchema>;
export const noteImportGroupSchema = z.object({
  content: z.object({ headword: z.string(), notes: z.string().nullable(),
    examples: z.array(z.object({ fr: z.string(), zh: z.string() }).strict().readonly()).readonly().nullable(),
  }).strict().readonly(),
  rows: z.array(z.number().int().nonnegative().safe()).readonly(),
  existing: recordSchema.nullable(), status: z.enum(['create', 'identical', 'different', 'invalid']), error: z.string().nullable(),
  enrichment: noteEnrichmentSchema.nullable().optional(),
}).strict().readonly();
export type NoteImportGroup = z.infer<typeof noteImportGroupSchema>;
export const noteImportPreviewSchema = z.object({
  operation: operationSchema, source: importSourceSchema, columns: z.array(z.string()).readonly(),
  mapping: noteImportMappingSchema, groups: z.array(noteImportGroupSchema).readonly(), error: z.string().nullable(),
}).strict().readonly();
export type NoteImportPreview = z.infer<typeof noteImportPreviewSchema>;
const selectionSchema = z.object({ index: z.number().int().nonnegative().safe(), action: z.enum(['create', 'update', 'skip']),
  allow_unlinked: z.boolean().default(false) }).strict().readonly();
export type NoteImportSelection = z.infer<typeof selectionSchema>;
const resultSchema = z.object({ created: z.number().int().nonnegative().safe(), updated: z.number().int().nonnegative().safe(),
  skipped: z.number().int().nonnegative().safe(), records: z.array(recordSchema).readonly() }).strict().readonly();
export type NoteImportResult = z.infer<typeof resultSchema>;

export const noteImportRuntime = {
  async previewNoteImport(operation: string, options: NoteImportOptions): Promise<NoteImportPreview | null> {
    operationSchema.parse(operation); optionsSchema.parse(options);
    if (!isTauri()) throw new Error('CSV import requires the desktop application');
    return noteImportPreviewSchema.nullable().parse(await invoke('note_import_preview', { operation, options }));
  },
  async remapNoteImport(operation: string, options: NoteImportOptions): Promise<NoteImportPreview> {
    operationSchema.parse(operation); optionsSchema.parse(options);
    if (!isTauri()) throw new Error('CSV import requires the desktop application');
    return noteImportPreviewSchema.parse(await invoke('note_import_remap', { operation, options }));
  },
  async commitNoteImport(operation: string, selections: readonly NoteImportSelection[]): Promise<NoteImportResult> {
    operationSchema.parse(operation); z.array(selectionSchema).parse(selections);
    if (!isTauri()) throw new Error('CSV import requires the desktop application');
    return resultSchema.parse(await invoke('note_import_commit', { operation, selections }));
  },
  async cancelNoteImport(operation: string) {
    operationSchema.parse(operation);
    if (isTauri()) await invoke('note_import_cancel', { operation });
  },
};
