import { z } from 'zod';

export const interfacePreferencesSchema = z.object({
  locale: z.enum(['zh', 'en', 'fr']),
  theme: z.enum(['white', 'sepia', 'dark']),
}).strict().readonly();

export type InterfacePreferences = z.infer<typeof interfacePreferencesSchema>;
