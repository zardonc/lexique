import { z } from 'zod';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Locale } from '../content';

export const translationPreferencesSchema = z.object({
  active_provider: z.enum(['disabled', 'deepl', 'openai_compatible']),
  source_language: z.enum(['auto', 'fr', 'zh-CN', 'en']),
  target_language: z.enum(['fr', 'zh-CN', 'en']),
  deepl: z.object({ tier: z.enum(['free', 'pro']) }).strict().readonly(),
  openai_compatible: z.object({ base_url: z.string().max(512), model: z.string().max(256) }).strict().readonly(),
}).strict().readonly();

export const translationSettingsSchema = z.object({
  preferences: translationPreferencesSchema,
  keys: z.object({ deepl: z.enum(['missing', 'configured', 'unreadable']), openai_compatible: z.enum(['missing', 'configured', 'unreadable']) }).strict().readonly(),
}).strict().readonly();

export const translationResultSchema = z.object({ text: z.string(), source: z.enum(['fr', 'zh-CN', 'en']), target: z.enum(['fr', 'zh-CN', 'en']) }).strict().readonly();
export type TranslationPreferences = z.infer<typeof translationPreferencesSchema>;
export type TranslationSettings = z.infer<typeof translationSettingsSchema>;
export type TranslationResult = z.infer<typeof translationResultSchema>;
const translationDeltaSchema = z.object({ id: z.string().max(128), kind: z.literal('delta'), text: z.string(), source: z.enum(['fr', 'zh-CN', 'en']), target: z.enum(['fr', 'zh-CN', 'en']) }).strict();
const translationProgressSchema = z.union([translationDeltaSchema, z.object({ id: z.string().max(128), kind: z.enum(['started', 'completed', 'failed']) }).strict()]);
type Replacement = { readonly provider: 'deepl' | 'openai_compatible'; readonly key: string };

export const translationRuntime = {
  async translationSettings(locale: Locale, preferences?: TranslationPreferences, replacement?: Replacement): Promise<TranslationSettings | null> {
    if (!isTauri()) return null;
    return translationSettingsSchema.parse(await invoke('translation_settings', { locale, preferences: preferences ? translationPreferencesSchema.parse(preferences) : null, replacement: replacement ?? null }));
  },
  async translationClearKey(provider: Replacement['provider']) {
    if (!isTauri()) throw new Error('Translation services are available in the desktop app only');
    await invoke('translation_clear_key', { provider });
  },
  async translate(id: string, text: string, locale: Locale): Promise<TranslationResult> {
    if (!isTauri()) throw new Error('Translation is available in the desktop app only');
    return translationResultSchema.parse(await invoke('translation_execute', { id, text, locale }));
  },
  async cancelTranslation(id: string) { if (isTauri()) await invoke('translation_cancel', { id }); },
  async listenTranslation(callback: (value: z.infer<typeof translationDeltaSchema>) => void) {
    if (!isTauri()) return () => {};
    return listen('translation-progress', (event) => { const value = translationProgressSchema.parse(event.payload); if (value.kind === 'delta') callback(value); });
  },
  async testTranslationConnection(preferences: TranslationPreferences, replacement?: Replacement) {
    if (!isTauri()) throw new Error('Translation services are available in the desktop app only');
    return z.object({ host: z.string(), elapsed_ms: z.number().nonnegative() }).strict().parse(await invoke('translation_test_connection', { preferences: translationPreferencesSchema.parse(preferences), replacement: replacement ?? null }));
  },
};
