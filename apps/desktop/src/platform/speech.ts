import { z } from 'zod';

export const speechPreferencesSchema = z.object({
  voice: z.enum(['female', 'male']),
  speed: z.enum(['normal', 'slow']),
}).strict().readonly();

export type SpeechPreferences = z.infer<typeof speechPreferencesSchema>;
