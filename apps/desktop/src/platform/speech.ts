import { z } from 'zod';

export const speechVoiceSchema = z.enum(['female', 'male']);

export const speechPreferencesSchema = z.object({
  voice: speechVoiceSchema,
  wpm: z.number().int().min(80).max(240).multipleOf(20),
}).strict().readonly();

export type SpeechPreferences = z.infer<typeof speechPreferencesSchema>;

export const speechAudioSchema = z.object({
  bytes: z.array(z.number().int().min(0).max(255)).min(1).max(10_000_000).readonly(),
  voice: speechVoiceSchema.nullable(),
}).strict().readonly();

export type SpeechAudio = z.infer<typeof speechAudioSchema>;

// Measured with complete French sentences; see docs/evidence/issue-56/calibration.json.
export const speechBaselineWpm = { female: 149.256686, male: 149.148901 } as const;

export function speechPlaybackRate(wpm: number, voice: SpeechAudio['voice']) {
  return wpm / speechBaselineWpm[voice ?? 'female'];
}
