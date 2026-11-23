import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { grammarRecordSchema } from './notes';
import type { NoteImportSelection } from './note-import';
import { importSourceSchema } from './vocabulary';

const operation = z.string().min(1).max(110).regex(/^[a-zA-Z0-9-]+$/);
const column = z.number().int().nonnegative().safe().nullable();
const mappingSchema = z.object({ exchange_id: column, title: column, body: column, question: column, answer: column }).strict().readonly();
export type GrammarImportMapping = z.infer<typeof mappingSchema>;
const optionsSchema = z.object({ delimiter: z.enum(['comma', 'semicolon']), mapping: mappingSchema.nullable(), format: z.literal('markdown').optional() }).strict();
export type GrammarImportOptions = z.infer<typeof optionsSchema>;
const content = z.object({ exchange_id: z.string().nullable(), title: z.string(), body: z.string().nullable(),
  question: z.string().nullable(), answer: z.string().nullable() }).strict().readonly();
const previewSchema = z.object({ operation, source: importSourceSchema, error: z.string().nullable(),
  columns: z.array(z.string()).readonly(), mapping: mappingSchema,
  groups: z.array(z.object({ content, rows: z.array(z.number().int().positive().safe()).readonly(), existing: grammarRecordSchema.nullable(),
    status: z.enum(['create', 'identical', 'different', 'invalid']), error: z.string().nullable() }).strict().readonly()).readonly(),
}).strict().readonly();
const count = z.number().int().nonnegative().safe();
const resultSchema = z.object({ created: count, updated: count, skipped: count,
  records: z.array(grammarRecordSchema).readonly(), new_card_ids: z.array(z.number().int().positive().safe()).readonly() }).strict().readonly();
export type GrammarImportPreview = z.infer<typeof previewSchema>;
export type GrammarImportResult = z.infer<typeof resultSchema>;

function options(value: GrammarImportOptions) {
  optionsSchema.parse(value);
  return { record_kind: 'grammar', delimiter: value.delimiter, mapping: null, grammar_mapping: value.mapping, ...(value.format ? { format: value.format } : {}) };
}
function desktop(id: string, format?: 'markdown') {
  operation.parse(id);
  if (!isTauri()) throw new Error(`Grammar ${format === 'markdown' ? 'Markdown' : 'CSV'} import requires the desktop application`);
}
export const grammarImportRuntime = {
  async previewGrammarImport(id: string, value: GrammarImportOptions = { delimiter: 'comma', mapping: null }): Promise<GrammarImportPreview | null> {
    desktop(id, value.format);
    return previewSchema.nullable().parse(await invoke('note_import_preview', { operation: id, options: options(value) }));
  },
  async remapGrammarImport(id: string, value: GrammarImportOptions): Promise<GrammarImportPreview> {
    desktop(id, value.format);
    return previewSchema.parse(await invoke('note_import_remap', { operation: id, options: options(value) }));
  },
  async commitGrammarImport(id: string, selections: readonly NoteImportSelection[]): Promise<GrammarImportResult> {
    desktop(id);
    z.array(z.object({ index: count, action: z.enum(['create', 'update', 'skip']), allow_unlinked: z.boolean().optional() }).strict()).parse(selections);
    return resultSchema.parse(await invoke('note_import_commit', { operation: id, selections }));
  },
};
