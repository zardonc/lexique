import { z } from 'zod';
import { categoryIdsSchema } from './vocabulary';

export const reviewScopeSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('all') }).strict().readonly(),
  z.object({ kind: z.literal('vocabulary'), category_ids: categoryIdsSchema,
    include_uncategorized: z.boolean() }).strict().readonly()
    .refine((scope) => scope.category_ids.length > 0 || scope.include_uncategorized, 'Review scope has no selected condition'),
  z.object({ kind: z.literal('needs_selection') }).strict().readonly(),
]);
export type ReviewScope = z.infer<typeof reviewScopeSchema>;
