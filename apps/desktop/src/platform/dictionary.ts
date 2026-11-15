import { z } from 'zod';

export const dictionaryQuery = z.string().min(1).refine((value) =>
  value.trim().length > 0 && new TextEncoder().encode(value).length <= 512 && !/\p{Cc}/u.test(value));

export const dictionaryId = z.string().regex(/^(builtin|builtin-fra-eng|mdx-[1-9]\d*)$/)
  .refine((id) => !id.startsWith('mdx-') || Number(id.slice(4)) < 0xffff_ffff);
export const dictionaryContentHash = z.string().regex(/^[a-f0-9]{64}$/);
export const MAX_DICTIONARIES = 17;
export const dictionaryOrder = z.array(dictionaryId).min(2).max(MAX_DICTIONARIES).readonly()
  .refine((ids) => new Set(ids).size === ids.length && ids.includes('builtin') && ids.includes('builtin-fra-eng'));
export const dictionaryCatalog = z.array(z.object({ id: dictionaryId, title: z.string().min(1),
  content_hash: dictionaryContentHash.nullable(), availability: z.enum(['available', 'missing']),
}).strict().readonly()).min(2).max(MAX_DICTIONARIES).readonly()
  .refine((items) => dictionaryOrder.safeParse(items.map((item) => item.id)).success);
export type DictionaryCatalog = z.infer<typeof dictionaryCatalog>;
export const dictionaryRelink = z.discriminatedUnion('status', [
  z.object({ status: z.literal('linked'), catalog: dictionaryCatalog }).strict().readonly(),
  z.object({ status: z.literal('confirmation_required'), dictionary_id: dictionaryId,
    expected_hash: dictionaryContentHash.nullable(), content_hash: dictionaryContentHash }).strict().readonly(),
]);
export type DictionaryRelink = z.infer<typeof dictionaryRelink>;

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
