import { z } from 'zod';

export const dictionaryQuery = z.string().min(1).refine((value) =>
  value.trim().length > 0 && new TextEncoder().encode(value).length <= 512 && !/\p{Cc}/u.test(value));

export const dictionaryId = z.string().regex(/^(builtin|mdx-[1-9]\d*)$/);
export const dictionaryOrder = z.array(dictionaryId).min(1).max(16).readonly()
  .refine((ids) => new Set(ids).size === ids.length && ids.includes('builtin'));
export const dictionaryCatalog = z.array(z.object({ id: dictionaryId, title: z.string().min(1) }).strict().readonly())
  .min(1).max(16).readonly().refine((items) => dictionaryOrder.safeParse(items.map((item) => item.id)).success);
export type DictionaryCatalog = z.infer<typeof dictionaryCatalog>;

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
  dictionary_id: dictionaryId.optional(),
  source: z.object({
    title: z.string().min(1), version: z.string().min(1), attribution: z.string().min(1).nullable(),
    license: z.string().min(1).nullable(), source_url: z.url().nullable(), license_url: z.url().nullable(),
  }).strict().readonly(),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
  entries: z.array(z.object({
    ordinal: z.number().int().positive(), key: z.string().min(1),
    pronunciations: z.array(z.string()).readonly(), grammar: z.array(z.string()).readonly(),
    senses: z.array(senseSchema).readonly(),
    html: z.string().optional(),
  }).strict().readonly()).readonly(),
  warnings: z.array(z.object({ dictionary_id: dictionaryId, code: z.string(), message: z.string() }).strict().readonly()).readonly().optional(),
}).strict().readonly();

const errorSchema = z.object({
  code: z.enum(['missing', 'corrupt', 'query_failed', 'invalid_query', 'unsupported', 'limit']), message: z.string(),
}).strict().readonly();

export type DictionaryLookup = z.infer<typeof dictionarySchema>;
export type DictionaryFailure = z.infer<typeof errorSchema>;

export function dictionaryFailure(error: unknown): DictionaryFailure {
  const parsed = errorSchema.safeParse(error);
  if (parsed.success) return parsed.data;
  return { code: 'query_failed', message: error instanceof Error ? error.message : String(error) };
}
