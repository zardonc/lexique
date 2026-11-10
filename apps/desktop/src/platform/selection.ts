import { z } from 'zod';

const request = z.number().int().positive().safe();
export const selectionText = z.string().transform((value) => Array.from(value).slice(0, 5_000).join(''));
export const ocrImageSchema = z.object({ width: z.number().int().min(1).max(4096), height: z.number().int().min(1).max(4096),
  data_url: z.string().max(22_369_650).regex(/^data:image\/png;base64,[A-Za-z0-9+/]+={0,2}$/) }).strict().readonly();
export type OcrImage = z.infer<typeof ocrImageSchema>;
export const selectionEventSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('pending'), request }).strict(),
  z.object({ kind: z.literal('cancelled'), request }).strict(),
  z.object({ kind: z.literal('captured'), request, text: selectionText }).strict(),
  z.object({ kind: z.literal('image'), request, image: ocrImageSchema, recognize: z.boolean().default(false) }).strict(),
  z.object({ kind: z.literal('failed'), request,
    code: z.enum(['no_selection', 'unsupported', 'permission', 'read_failed', 'busy', 'invalid_response', 'image_invalid', 'image_limit', 'capture_failed', 'screen_permission']),
    message: z.string().max(2000) }).strict(),
]);
export type SelectionEvent = z.infer<typeof selectionEventSchema>;

export const shortcutText = z.string().min(1).max(128).refine((value) => !/\p{Cc}/u.test(value));
export const shortcutStatusSchema = z.object({ shortcut: shortcutText, registered: z.boolean(),
  error: z.string().max(2000).nullable() }).strict().readonly();
export type ShortcutStatus = z.infer<typeof shortcutStatusSchema>;

export function readSelectionEvent(payload: unknown): SelectionEvent {
  const result = selectionEventSchema.safeParse(payload);
  if (result.success) return result.data;
  const identity = z.object({ request }).safeParse(payload);
  return { kind: 'failed', request: identity.success ? identity.data.request : 1,
    code: 'invalid_response', message: 'Invalid selection response' };
}
