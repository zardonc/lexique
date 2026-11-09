import { z } from 'zod';
import { conjugationQuery } from './conjugation';

const valueSchema = z.number().finite().nonnegative().nullable();
const partSchema = z.object({ part_of_speech: z.string().min(1).nullable(), per_million: valueSchema,
  reason: z.enum(['missing', 'invalid', 'conflict', 'missing_pos']).nullable() }).strict().readonly()
  .refine((part) => part.reason === null ? part.part_of_speech !== null && part.per_million !== null
    : part.reason === 'missing_pos' ? part.part_of_speech === null && part.per_million !== null : part.per_million === null,
  'Inconsistent frequency part');

const entrySchema = z.object({ key: conjugationQuery, lemma: z.string().min(1).nullable(), per_million: valueSchema,
  reason: z.enum(['missing', 'invalid', 'conflict', 'missing_pos', 'ambiguous', 'not_found']).nullable(),
  parts: z.array(partSchema).readonly(), candidates: z.array(z.string().min(1)).readonly() }).strict().readonly()
  .refine((entry) => (entry.per_million === null) === (entry.reason !== null), 'Frequency requires a value or a reason')
  .refine((entry) => entry.lemma === null ? entry.parts.length === 0 &&
    ((entry.reason === 'not_found' && entry.candidates.length === 0) || (entry.reason === 'ambiguous' && entry.candidates.length > 1))
    : entry.parts.length > 0 && entry.candidates.length === 1 && entry.candidates[0] === entry.lemma &&
      entry.reason !== 'not_found' && entry.reason !== 'ambiguous', 'Inconsistent frequency lemma')
  .refine((entry) => entry.per_million === null ? entry.lemma === null || entry.parts.some((part) => part.reason === entry.reason)
    : entry.parts.every((part) => part.reason === null) &&
      Math.abs(entry.parts.reduce((sum, part) => sum + (part.per_million ?? 0), 0) - entry.per_million) < 0.000001,
  'Frequency differs from its POS groups');

export const frequencyQuery = z.array(conjugationQuery).min(1).max(100).readonly();
export const frequencySchema = z.object({
  source: z.object({ title: z.string().min(1), version: z.literal('3.83'), attribution: z.string().min(1),
    license: z.string().min(1), source_url: z.url(), license_url: z.url() }).strict().readonly(),
  metric: z.literal('freqlemfilms2'), entries: z.array(entrySchema).min(1).max(100).readonly(),
}).strict().readonly();

export type FrequencyLookup = z.infer<typeof frequencySchema>;
export type Frequency = FrequencyLookup['entries'][number];
