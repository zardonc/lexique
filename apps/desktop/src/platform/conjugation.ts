import { z } from 'zod';

const status = z.enum(['supported', 'unconfirmed', 'not_applicable']);
const slotSchema = z.object({
  source_line: z.number().int().positive(), form: z.string().min(1), lemma: z.string().min(1),
  raw_tag: z.string(), tense: z.enum(['P', 'I', 'J', 'F', 'C', 'S', 'T', 'Y', 'W', 'G', 'K', 'PC', 'PQP', 'PA', 'FA', 'CP', 'SP', 'SQP', 'YP', 'WP', 'KP', 'GP', 'GPP']).nullable(),
  person: z.enum(['1', '2', '3']).nullable(), number: z.enum(['s', 'p']).nullable(),
  gender: z.enum(['m', 'f']).nullable(), expressions: z.array(z.string().min(1)).readonly().optional(),
}).strict().readonly();
const verbSchema = z.object({
  blocks: z.record(z.string(), z.enum(['supported', 'unconfirmed', 'not_applicable', 'missing'])).readonly().optional(),
  group: z.union([z.literal(1), z.literal(2), z.literal(3)]).nullable(),
  group_status: z.enum(['supported', 'unconfirmed']), auxiliary_status: status, ordinary_status: status,
  auxiliaries: z.array(z.object({ lemma: z.enum(['avoir', 'être']), use: z.string().min(1) }).strict().readonly()).readonly(),
  usage_labels: z.array(z.string()).readonly(), source: z.json().nullable(), correction: z.json().nullable(),
}).strict().readonly();
export const conjugationSchema = z.object({
  query: z.string(), tables: z.array(z.object({
    lemma: z.string().min(1), verb: verbSchema,
    cells: z.array(slotSchema).readonly(),
  }).strict().readonly()).readonly(),
}).strict().readonly().superRefine((result, context) => {
  const lemmas = new Set<string>();
  for (const table of result.tables) {
    if (lemmas.has(table.lemma) || table.cells.length === 0 || !table.cells.some((cell) => cellMatches(cell, result.query))
      || table.cells.some((cell) => cell.lemma !== table.lemma)) context.addIssue({ code: 'custom', message: 'Invalid conjugation table identity' });
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
  return cell.form === query || (cell.expressions?.includes(query) ?? false);
}
