import { z } from 'zod';
import { dictionaryId } from './dictionary';

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

export const backupPreviewOperation = z.string().min(1).max(128).refine((value) => value.trim().length > 0);
const nonblank = z.string().min(1).refine((value) => value.trim().length > 0);
const previewSchema = z.object({
  filename: nonblank, byte_length: z.number().int().positive().safe(), created_at: count,
  application_version: nonblank, counts: countsSchema, requires_restart: z.literal(true),
  pending_dictionaries: z.array(z.object({ id: dictionaryId.refine((id) => id.startsWith('mdx-')), title: nonblank }).strict().readonly())
    .max(15).readonly().refine((items) => new Set(items.map((item) => item.id)).size === items.length),
}).strict().readonly();
const previewFields = { operation: backupPreviewOperation, cancel_requested: z.boolean() };
const emptyPreview = { phase: z.null(), preview: z.null(), error: z.null() };

export const backupPreviewStatusSchema = z.discriminatedUnion('state', [
  z.object({ ...previewFields, ...emptyPreview, state: z.literal('choosing') }).strict().readonly(),
  z.object({ ...previewFields, ...emptyPreview, state: z.literal('running'), phase: z.enum(['validating', 'staging']) }).strict().readonly(),
  z.object({ ...previewFields, ...emptyPreview, state: z.literal('preview_ready'), preview: previewSchema }).strict().readonly(),
  z.object({ ...previewFields, ...emptyPreview, state: z.literal('cancelled') }).strict().readonly(),
  z.object({ ...previewFields, ...emptyPreview, state: z.literal('failed'), error: nonblank }).strict().readonly(),
]);

export type BackupPreview = z.infer<typeof previewSchema>;
export type BackupPreviewStatus = z.infer<typeof backupPreviewStatusSchema>;
