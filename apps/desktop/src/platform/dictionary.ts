import { z } from 'zod';

export interface DictionarySense {
  readonly translations: readonly string[];
  readonly definitions: readonly string[];
  readonly senses: readonly DictionarySense[];
}

const senseSchema: z.ZodType<DictionarySense> = z.lazy(() => z.object({
  translations: z.array(z.string()).readonly(),
  definitions: z.array(z.string()).readonly(),
  senses: z.array(senseSchema).readonly(),
}).strict().readonly());

export const dictionarySchema = z.object({
  source: z.object({
    title: z.string().min(1), version: z.string().min(1), attribution: z.string().min(1),
    license: z.string().min(1), source_url: z.url(), license_url: z.url(),
  }).strict().readonly(),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
  entries: z.array(z.object({
    ordinal: z.number().int().positive(), key: z.string().min(1),
    pronunciations: z.array(z.string()).readonly(), grammar: z.array(z.string()).readonly(),
    senses: z.array(senseSchema).readonly(),
  }).strict().readonly()).readonly(),
}).strict().readonly();

const errorSchema = z.object({
  code: z.enum(['missing', 'corrupt', 'query_failed', 'invalid_query']), message: z.string(),
}).strict().readonly();

export type DictionaryLookup = z.infer<typeof dictionarySchema>;
export type DictionaryFailure = z.infer<typeof errorSchema>;

export function dictionaryFailure(error: unknown): DictionaryFailure {
  const parsed = errorSchema.safeParse(error);
  if (parsed.success) return parsed.data;
  return { code: 'query_failed', message: error instanceof Error ? error.message : String(error) };
}
