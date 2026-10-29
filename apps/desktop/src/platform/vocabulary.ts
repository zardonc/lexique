import { z } from 'zod';
import type { DictionaryLookup, DictionarySense } from './dictionary';

export const characterCount = (value: string) => Array.from(value).length;
const headword = z.string().refine((value) => value.trim().length > 0 && characterCount(value) <= 200
  && !Array.from(value).some((char) => {
    const code = char.codePointAt(0)!;
    return code < 32 || (code >= 127 && code <= 159);
  }), 'Headword must be 1–200 characters on one line');
const notes = z.string().refine((value) => characterCount(value) <= 20_000, 'Notes exceed 20,000 Unicode characters');
const moreExamples = z.array(z.object({ fr: notes, zh: notes }).strict().readonly()).max(100).readonly();
const positiveId = z.number().int().positive().max(Number.MAX_SAFE_INTEGER);
const digestEntry = z.object({
  pronunciations: z.array(z.string()).readonly(), grammar: z.array(z.string()).readonly(),
  translation: z.string().nullable(),
}).strict().readonly();
const reference = z.object({
  dictionary: z.string().min(1), version: z.string().min(1), attribution: z.string().min(1),
  lookup_key: z.string().min(1),
  license: z.string().min(1), source_url: z.url(), license_url: z.url(),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
}).strict().readonly();
export const recordSchema = z.object({
  id: positiveId, headword, reference, digest: z.array(digestEntry).min(1).readonly(), notes, example_fr: notes, example_zh: notes, more_examples: moreExamples,
  revision: positiveId, created_at: positiveId, updated_at: positiveId,
}).strict().readonly();
export const saveRequestSchema = z.object({
  operation: z.string().min(1).max(128).regex(/^[a-zA-Z0-9-]+$/), headword, notes, example_fr: notes, example_zh: notes, more_examples: moreExamples,
  id: positiveId.nullable(), revision: positiveId.nullable(),
}).strict().readonly().refine((value) => (value.id === null) === (value.revision === null));
export const saveResultSchema = z.object({
  outcome: z.enum(['created', 'existing', 'updated', 'replayed', 'conflict']), record: recordSchema,
}).strict().readonly();
export const sortSchema = z.enum(['headword-asc', 'headword-desc', 'created-asc', 'created-desc', 'updated-asc', 'updated-desc']);
export type VocabularyRecord = z.infer<typeof recordSchema>;
export type SaveRequest = z.infer<typeof saveRequestSchema>;
export type SaveResult = z.infer<typeof saveResultSchema>;
export type VocabularySort = z.infer<typeof sortSchema>;
export type VocabularySnapshot = Pick<VocabularyRecord, 'headword' | 'reference' | 'digest'>;

function firstTranslation(senses: readonly DictionarySense[]): string | null {
  for (const sense of senses) {
    const translation = sense.translations[0] ?? firstTranslation(sense.senses);
    if (translation !== null) return translation;
  }
  return null;
}

export function snapshotFromLookup(lookup: DictionaryLookup): VocabularySnapshot {
  const first = lookup.entries[0];
  if (!first) throw new Error('Dictionary has no matching headword');
  return {
    headword: first.key,
    reference: { dictionary: lookup.source.title, lookup_key: first.key, version: lookup.source.version,
      attribution: lookup.source.attribution, license: lookup.source.license,
      source_url: lookup.source.source_url, license_url: lookup.source.license_url, sha256: lookup.sha256 },
    digest: lookup.entries.map((entry) => ({ pronunciations: entry.pronunciations,
      grammar: entry.grammar, translation: firstTranslation(entry.senses) })),
  };
}

export const errorMessage = (error: unknown) => error instanceof Error ? error.message : String(error);
