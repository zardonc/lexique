import type { Page } from '@playwright/test';

export const dictionary = {
  source: { title: 'FreeDict+WikDict français-中文', version: '2025.11.23',
    attribution: 'Karl Bartel; Wiktionary contributors', license: 'CC-BY-SA-3.0',
    source_url: 'https://download.freedict.org/dictionaries/fra-zho/2025.11.23/',
    license_url: 'https://creativecommons.org/licenses/by-sa/3.0/' },
  sha256: 'a'.repeat(64),
  entries: [{ ordinal: 1, key: 'école', pronunciations: ['e.kɔl'], grammar: ['n', 'fem'],
    senses: [{ translations: ['学校', '学派'], definitions: [], senses: [] }] },
    { ordinal: 2, key: 'école', pronunciations: [], grammar: [],
      senses: [{ translations: ['流派'], definitions: [], senses: [] }] }],
};

export const record = {
  id: 1, headword: 'école', notes: '', revision: 1, created_at: 1000, updated_at: 1000,
  reference: { dictionary: dictionary.source.title, lookup_key: 'école', version: dictionary.source.version,
    attribution: dictionary.source.attribution, license: dictionary.source.license,
    source_url: dictionary.source.source_url, license_url: dictionary.source.license_url, sha256: dictionary.sha256 },
  digest: [{ pronunciations: ['e.kɔl'], grammar: ['n', 'fem'], translation: '学校' },
    { pronunciations: [], grammar: [], translation: '流派' }],
};

interface Reply { readonly value?: unknown; readonly error?: unknown }
interface Call { readonly command: string; readonly args?: Record<string, unknown> }
declare global {
  interface Window { testNative: { calls: Call[]; close: () => void } }
}

export async function nativeFixture(page: Page, responses: Readonly<Record<string, readonly Reply[]>> = {}) {
  await page.addInitScript(({ responses, dictionary }) => {
    const callbacks = new Map<number, (event: unknown) => void>();
    const indexes: Record<string, number> = {};
    window.testNative = { calls: [], close: () => { for (const callback of callbacks.values()) callback({ payload: null }); } };
    const defaults: Record<string, unknown> = { runtime_status: { version: '0.1.0', schemaVersion: 2, storage: 'sqlite' },
      dictionary_lookup: dictionary, vocabulary_find: null, vocabulary_list: [], vocabulary_sort: 'updated-desc', finish_close: null,
      speech_preferences: { voice: 'female', speed: 'normal' } };
    function defaultReply(command: string, args?: Record<string, unknown>) {
      if (command === 'speech_preferences' && args?.preferences) defaults[command] = args.preferences;
      return command === 'vocabulary_sort' ? args?.sort ?? defaults[command] : defaults[command];
    }
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {
      transformCallback: (callback: (event: unknown) => void) => { const id = callbacks.size + 1; callbacks.set(id, callback); return id; },
      unregisterCallback: (id: number) => callbacks.delete(id),
      invoke: async (command: string, args?: Record<string, unknown>) => {
        window.testNative.calls.push({ command, args });
        if (command === 'plugin:event|listen') return 1;
        if (command === 'plugin:event|unlisten') return null;
        const queue = responses[command];
        if (!queue) return defaultReply(command, args);
        const index = indexes[command] ?? 0;
        indexes[command] = index + 1;
        const response = queue[Math.min(index, queue.length - 1)];
        if (response.error !== undefined) throw response.error;
        if (command === 'vocabulary_save') defaults.vocabulary_find = (response.value as { record: unknown }).record;
        return response.value;
      },
    } });
  }, { responses, dictionary });
}

export async function collect(page: Page) {
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
}
