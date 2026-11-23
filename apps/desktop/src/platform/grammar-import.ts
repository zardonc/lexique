import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { grammarRecordSchema } from './notes';
import { importSourceSchema } from './vocabulary';

const operation = z.string().min(1).max(110).regex(/^[a-zA-Z0-9-]+$/);
const content = z.object({ exchange_id: z.string().nullable(), title: z.string(), body: z.string().nullable(),
  question: z.string().nullable(), answer: z.string().nullable() }).strict().readonly();
const previewSchema = z.object({ operation, source: importSourceSchema, error: z.string().nullable(),
  group: z.object({ content, row: z.number().int().positive().safe(), existing: grammarRecordSchema.nullable(),
    status: z.enum(['create', 'identical', 'different', 'invalid']), error: z.string().nullable() }).strict().readonly().nullable(),
}).strict().readonly();
const resultSchema = z.object({ created: z.number().int().nonnegative().max(1), skipped: z.number().int().nonnegative().max(1),
  records: z.array(grammarRecordSchema).max(1).readonly() }).strict().readonly();
export type GrammarImportPreview = z.infer<typeof previewSchema>;
export type GrammarImportResult = z.infer<typeof resultSchema>;

export const grammarImportRuntime = {
  async previewGrammarImport(id: string): Promise<GrammarImportPreview | null> {
    operation.parse(id);
    if (!isTauri()) throw new Error('Grammar CSV import requires the desktop application');
    return previewSchema.nullable().parse(await invoke('note_import_preview', { operation: id,
      options: { record_kind: 'grammar', delimiter: 'comma', mapping: null } }));
  },
  async commitGrammarImport(id: string, action: 'create' | 'skip'): Promise<GrammarImportResult> {
    operation.parse(id); z.enum(['create', 'skip']).parse(action);
    if (!isTauri()) throw new Error('Grammar CSV import requires the desktop application');
    return resultSchema.parse(await invoke('note_import_commit', { operation: id, selections: [{ index: 0, action, allow_unlinked: false }] }));
  },
};
