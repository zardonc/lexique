import { z } from 'zod';

export const conjugationModeSchema = z.enum(['verbal', 'main', 'ending']);
export const conjugationPreferencesSchema = z.object({ mode: conjugationModeSchema }).strict().readonly();
export type ConjugationMode = z.infer<typeof conjugationModeSchema>;
export type ConjugationPreferences = z.infer<typeof conjugationPreferencesSchema>;
export const defaultConjugationOrder = [
  'P', 'PC', 'I', 'F', 'C', 'S', 'PQP', 'J', 'PA', 'FA', 'CP', 'SP', 'T', 'SQP',
  'Y', 'YP', 'W', 'WP', 'G', 'K', 'KP', 'GP', 'GPP',
] as const;
export const conjugationTenseSchema = z.enum(defaultConjugationOrder);
export const conjugationOrderSchema = z.array(conjugationTenseSchema).length(defaultConjugationOrder.length)
  .refine((order) => new Set(order).size === order.length, 'Block order must contain all 23 tense blocks exactly once').readonly();
export type ConjugationTense = z.infer<typeof conjugationTenseSchema>;
export type ConjugationOrder = z.infer<typeof conjugationOrderSchema>;
const rangeSchema = z.tuple([z.number().int().nonnegative().safe(), z.number().int().positive().safe()]).readonly();
const spansSchema = z.object({ verbal: z.array(rangeSchema).min(1).max(2).readonly(), main: rangeSchema, ending: rangeSchema.nullable() }).strict().readonly();
export type ConjugationSpans = z.infer<typeof spansSchema>;
function validSpans(text: string, spans: ConjugationSpans): boolean {
  const length = [...text].length;
  const ranges = [...spans.verbal, spans.main, ...(spans.ending ? [spans.ending] : [])];
  if (ranges.some(([start, end]) => start >= end || end > length)) return false;
  if (!spans.verbal.some(([start, end]) => start <= spans.main[0] && end >= spans.main[1])) return false;
  if (spans.ending && (spans.ending[0] < spans.main[0] || spans.ending[1] !== spans.main[1])) return false;
  return spans.verbal.every(([start], index) => index === 0 || start >= (spans.verbal[index - 1]?.[1] ?? length));
}
const coloringSchema = z.record(z.string().min(1), spansSchema).readonly()
  .refine((value) => Object.entries(value).every(([text, spans]) => validSpans(text, spans)), 'Invalid conjugation coloring spans');
const status = z.enum(['supported', 'unconfirmed', 'not_applicable']);
const slotSchema = z.object({
  source_line: z.number().int().positive(), form: z.string().min(1), lemma: z.string().min(1),
  raw_tag: z.string(), tense: conjugationTenseSchema.nullable(),
  person: z.enum(['1', '2', '3']).nullable(), number: z.enum(['s', 'p']).nullable(),
  gender: z.enum(['m', 'f']).nullable(), expressions: z.array(z.string().min(1)).readonly().optional(),
  coloring: coloringSchema,
}).strict().readonly().refine((cell) => {
  const texts = new Set([cell.form, ...(cell.expressions ?? [])]);
  return Object.keys(cell.coloring).length === texts.size && Object.keys(cell.coloring).every((text) => texts.has(text));
}, 'Conjugation coloring differs from original expressions');
const verbSchema = z.object({
  blocks: z.record(z.string(), z.enum(['supported', 'unconfirmed', 'not_applicable', 'missing'])).readonly().optional(),
  group: z.union([z.literal(1), z.literal(2), z.literal(3)]).nullable(),
  group_status: z.enum(['supported', 'unconfirmed']), auxiliary_status: status, ordinary_status: status,
  auxiliaries: z.array(z.object({ lemma: z.enum(['avoir', 'être']), use: z.string().min(1) }).strict().readonly()).readonly(),
  usage_labels: z.array(z.string()).readonly(), source: z.json().nullable(), correction: z.json().nullable(),
  pronominal: z.object({
    base: z.string().min(1), agreement: z.enum(['subject', 'contextual', 'invariable']),
    persons: z.array(z.enum(['1s', '2s', '3s', '1p', '2p', '3p'])).min(1).readonly(),
    with_en: z.boolean(), reciprocal: z.boolean(), source: z.json(),
  }).strict().readonly().optional(),
}).strict().readonly();
const meaningsSchema = z.object({
  status: z.enum(['available', 'unavailable']), senses: z.array(z.object({
    source_entry: z.number().int().positive(), source_sense: z.number().int().positive(),
    headword: z.string().min(1), use: z.enum(['ordinary', 'pronominal']),
    labels: z.array(z.string().min(1)).readonly(), glosses: z.array(z.string().min(1)).min(1).readonly(),
  }).strict().readonly()).readonly(),
}).strict().readonly().refine((value) => (value.status === 'available') === (value.senses.length > 0));
export const conjugationSchema = z.object({
  query: z.string(), tables: z.array(z.object({
    lemma: z.string().min(1), verb: verbSchema, meanings: meaningsSchema,
    cells: z.array(slotSchema).readonly(),
  }).strict().readonly()).readonly(),
}).strict().readonly().superRefine((result, context) => {
  const lemmas = new Set<string>();
  for (const table of result.tables) {
    if (lemmas.has(table.lemma) || table.cells.length === 0 || !table.cells.some((cell) => cellMatches(cell, result.query))
      || table.cells.some((cell) => cell.lemma !== table.lemma)) context.addIssue({ code: 'custom', message: 'Invalid conjugation table identity' });
    if (table.verb.group_status !== 'supported' || ![1, 2].includes(table.verb.group ?? 0)) {
      if (table.cells.some((cell) => Object.values(cell.coloring).some((spans) => spans.ending !== null))) context.addIssue({ code: 'custom', message: 'Irregular conjugation cannot have a regular ending' });
    }
    lemmas.add(table.lemma);
  }
});
export const conjugationQuery = z.string().refine((key) => key.trim().length > 0
  && new TextEncoder().encode(key).length <= 512 && !/\p{Cc}/u.test(key), 'Enter 1–512 UTF-8 bytes without control characters');
export type ConjugationLookup = z.infer<typeof conjugationSchema>;
export type ConjugationTable = ConjugationLookup['tables'][number];
export type ConjugationCell = ConjugationTable['cells'][number];

export function matchingCells(table: ConjugationTable, query: string): number {
  return new Set(table.cells.filter((cell) => cellMatches(cell, query)).map(cellKey)).size;
}
export function cellKey(cell: ConjugationCell): string {
  return [cell.tense, cell.person, cell.number, cell.tense === 'K' ? cell.gender : null].join(':');
}

export function cellMatches(cell: ConjugationCell, query: string): boolean {
  return expressionMatches(cell.form, query) || (cell.expressions?.some((expression) => expressionMatches(expression, query)) ?? false);
}

export function expressionMatches(expression: string, query: string): boolean {
  return normalizeQuery(expression) === normalizeQuery(query);
}

function normalizeQuery(query: string): string {
  return query.normalize('NFC').toLowerCase().replaceAll("'", '’').split(/\p{White_Space}+/u).filter(Boolean).join(' ');
}
