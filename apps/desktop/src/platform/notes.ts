import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { categoryIdsSchema, characterCount, recordSchema, sortSchema } from './vocabulary';
import type { VocabularySort } from './vocabulary';

const id = z.number().int().positive().safe();
const operation = z.string().min(1).max(128).regex(/^[a-zA-Z0-9-]+$/);
export const singleLineNameSchema = z.string().refine((value) => value.trim().length > 0 && characterCount(value.trim()) <= 200
  && !Array.from(value).some((char) => { const code = char.codePointAt(0)!; return code < 32 || code >= 127 && code <= 159 || code === 0x2028 || code === 0x2029; }), 'Use 1–200 characters on one line');
const storedName = singleLineNameSchema.refine((value) => value.trim() === value, 'Stored names must be trimmed');
const body = z.string().refine((value) => characterCount(value) <= 20_000, 'Use at most 20,000 characters');
const answerText = body.refine((value) => value.trim().length > 0, 'Question and answer are required');
export const grammarCardSchema = z.object({ id, question: answerText, answer: answerText }).strict().readonly();
export const grammarRecordSchema = z.object({ id, title: storedName, body, category_ids: categoryIdsSchema,
  card: grammarCardSchema.nullable(), revision: id, created_at: z.number().int().nonnegative().safe(), updated_at: z.number().int().nonnegative().safe() }).strict().readonly();
export const grammarSaveRequestSchema = z.object({ operation, id: id.nullable(), revision: id.nullable(), title: singleLineNameSchema,
  body, category_ids: categoryIdsSchema }).strict().readonly().refine((value) => (value.id === null) === (value.revision === null));
export const grammarCardRequestSchema = z.object({ operation, id, revision: id, question: answerText, answer: answerText }).strict().readonly();
const grammarSaveResultSchema = z.object({ outcome: z.enum(['created', 'updated', 'replayed', 'conflict']), record: grammarRecordSchema }).strict().readonly();
const categorySchema = z.object({ id, name: storedName }).strict().readonly();
const categoryListSchema = z.array(categorySchema).readonly().refine((values) => new Set(values.map((value) => value.id)).size === values.length, 'Duplicate category identity');
const bulkRequestSchema = z.object({ record_kind: z.enum(['vocabulary', 'grammar']), ids: z.array(id).max(10_000).readonly()
  .refine((ids) => new Set(ids).size === ids.length), category_id: id, action: z.enum(['add', 'remove']) }).strict().readonly();
const bulkResultSchema = z.object({ id, status: z.enum(['saved', 'failed']), error: z.string().min(1).nullable() }).strict().readonly()
  .refine((value) => (value.status === 'saved') === (value.error === null));
export const reviewRecordSchema = z.union([recordSchema, grammarRecordSchema]);
export type GrammarRecord = z.infer<typeof grammarRecordSchema>;
export type GrammarSaveRequest = z.infer<typeof grammarSaveRequestSchema>;
export type GrammarCardRequest = z.infer<typeof grammarCardRequestSchema>;
export type GrammarSaveResult = z.infer<typeof grammarSaveResultSchema>;
export type LearningCategory = z.infer<typeof categorySchema>;
export type CategoryBulkRequest = z.infer<typeof bulkRequestSchema>;
export type CategoryBulkResult = z.infer<typeof bulkResultSchema>;
export type ReviewRecord = z.infer<typeof reviewRecordSchema>;
export function matchesReviewRecord(record: ReviewRecord, card: { readonly id: number; readonly vocabulary_id: number; readonly kind: 'recognition' | 'listening' | 'grammar' }) {
  return record.id === card.vocabulary_id && ('title' in record ? card.kind === 'grammar' && record.card?.id === card.id : card.kind !== 'grammar');
}
export type CategoryFilter = 'all' | 'uncategorized' | number;
export const categoryNameKey = (name: string) => name.trim().normalize('NFC').toLowerCase().normalize('NFC');
export const matchesCategory = (record: { readonly category_ids?: readonly number[] }, filter: CategoryFilter) => filter === 'all'
  || (filter === 'uncategorized' ? !record.category_ids?.length : record.category_ids?.includes(filter) === true);

export const notesRuntime = {
  async listGrammar(query: string, sort: VocabularySort) {
    if (!isTauri()) return null;
    return z.array(grammarRecordSchema).readonly().parse(await invoke('grammar_list', { query: body.parse(query), sort: sortSchema.parse(sort) }));
  },
  async getGrammar(value: number) {
    if (!isTauri()) return null;
    const requested = id.parse(value), record = grammarRecordSchema.nullable().parse(await invoke('grammar_get', { id: requested }));
    if (record && record.id !== requested) throw new Error('Grammar record identity differs from the request');
    return record;
  },
  async saveGrammar(request: GrammarSaveRequest) {
    if (!isTauri()) return null;
    const parsed = grammarSaveRequestSchema.parse(request);
    const result = grammarSaveResultSchema.parse(await invoke('grammar_save', { request: { ...parsed, title: parsed.title.trim() } }));
    if (parsed.id !== null && result.record.id !== parsed.id) throw new Error('Saved grammar identity differs from the request');
    return result;
  },
  async saveGrammarCard(request: GrammarCardRequest) {
    if (!isTauri()) return null;
    const parsed = grammarCardRequestSchema.parse(request), result = grammarSaveResultSchema.parse(await invoke('grammar_card_save', { request: parsed }));
    if (result.record.id !== parsed.id) throw new Error('Saved grammar identity differs from the request');
    return result;
  },
  async deleteGrammar(value: number, revision: number) {
    if (!isTauri()) return null;
    z.null().parse(await invoke('grammar_delete', { id: id.parse(value), revision: id.parse(revision) }));
  },
  async grammarSort(sort?: VocabularySort) {
    if (!isTauri()) return null;
    const result = sortSchema.parse(await invoke('grammar_sort', { sort: sort === undefined ? null : sortSchema.parse(sort) }));
    if (sort !== undefined && sort !== result) throw new Error('Saved grammar sort differs from the request');
    return result;
  },
  async learningCategories() {
    if (!isTauri()) return null;
    return categoryListSchema.parse(await invoke('learning_categories'));
  },
  async createCategory(name: string) {
    if (!isTauri()) return null;
    const requested = singleLineNameSchema.parse(name).trim(), result = categorySchema.parse(await invoke('category_create', { name: requested }));
    if (result.name !== requested) throw new Error('Saved category name differs from the request');
    return result;
  },
  async renameCategory(value: number, name: string) {
    if (!isTauri()) return null;
    const requested = singleLineNameSchema.parse(name).trim(), result = categorySchema.parse(await invoke('category_rename', { id: id.parse(value), name: requested }));
    if (result.id !== value || result.name !== requested) throw new Error('Saved category differs from the request');
    return result;
  },
  async deleteCategory(value: number) {
    if (!isTauri()) return null;
    z.null().parse(await invoke('category_delete', { id: id.parse(value) }));
  },
  async bulkClassify(request: CategoryBulkRequest) {
    if (!isTauri()) return null;
    const parsed = bulkRequestSchema.parse(request), results = z.array(bulkResultSchema).readonly().parse(await invoke('category_bulk', { request: parsed }));
    if (results.length !== parsed.ids.length || results.some((result, index) => result.id !== parsed.ids[index])) throw new Error('Classification results differ from the requested records');
    return results;
  },
};
