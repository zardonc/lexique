import { z } from 'zod';
import { dictionarySnapshotSchema } from './vocabulary';

export const noteEnrichmentSchema = z.object({
  status: z.enum(['linked', 'not_found', 'error', 'retained']),
  snapshot: dictionarySnapshotSchema.nullable(),
  warnings: z.array(z.object({ dictionary_id: z.string(), code: z.string(), message: z.string() }).strict().readonly()).readonly(),
}).strict().readonly().refine((value) => (value.status === 'linked') === (value.snapshot !== null)
  && (value.snapshot === null || value.snapshot.digest.length > 0)
  && (value.status !== 'error' || value.warnings.length > 0), 'Inconsistent dictionary enrichment');
export type NoteEnrichment = z.infer<typeof noteEnrichmentSchema>;
