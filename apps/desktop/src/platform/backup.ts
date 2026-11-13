import { z } from 'zod';

const count = z.number().int().nonnegative().safe();
const countsSchema = z.object({
  vocabulary: count, cards: count, ratings: count, batches: count, custom_dictionaries: count,
  legacy_audio: count, included_audio_versions: count, included_legacy_audio: count,
  audio_files: count, source_missing_audio: count, source_invalid_audio: count,
}).strict().readonly();

const resultSchema = z.object({
  destination: z.string().min(1), byte_length: z.number().int().positive().safe(), created_at: count,
  counts: countsSchema,
}).strict().readonly();

const fields = { operation: z.string().min(1).max(128), cancel_requested: z.boolean() };
const phaseSchema = z.enum(['draining', 'collecting', 'packaging', 'validating', 'publishing']);
const empty = { phase: z.null(), result: z.null(), error: z.null() };

export const backupStatusSchema = z.discriminatedUnion('state', [
  z.object({ ...fields, ...empty, state: z.literal('choosing') }).strict().readonly(),
  z.object({ ...fields, ...empty, state: z.literal('running'), phase: phaseSchema }).strict().readonly(),
  z.object({ ...fields, ...empty, state: z.literal('completed'), result: resultSchema }).strict().readonly(),
  z.object({ ...fields, ...empty, state: z.literal('cancelled') }).strict().readonly(),
  z.object({ ...fields, ...empty, state: z.literal('failed'), error: z.string().min(1).refine((value) => value.trim().length > 0) }).strict().readonly(),
]);

export type BackupStatus = z.infer<typeof backupStatusSchema>;
export type BackupResult = z.infer<typeof resultSchema>;
export type BackupCounts = z.infer<typeof countsSchema>;
