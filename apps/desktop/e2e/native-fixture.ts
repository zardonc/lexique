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
  id: 1, headword: 'école', notes: '', example_fr: '', example_zh: '', more_examples: [], revision: 1, created_at: 1000, updated_at: 1000,
  reference: { dictionary: dictionary.source.title, lookup_key: 'école', version: dictionary.source.version,
    attribution: dictionary.source.attribution, license: dictionary.source.license,
    source_url: dictionary.source.source_url, license_url: dictionary.source.license_url, sha256: dictionary.sha256 },
  digest: [{ pronunciations: ['e.kɔl'], grammar: ['n', 'fem'], translation: '学校' },
    { pronunciations: [], grammar: [], translation: '流派' }],
};

interface Reply { readonly value?: unknown; readonly error?: unknown; readonly delay?: number }
interface Call { readonly command: string; readonly args?: Record<string, unknown> }
declare global {
  interface Window { testNative: { calls: Call[]; close: () => void } }
}

export async function nativeFixture(page: Page, responses: Readonly<Record<string, readonly Reply[]>> = {}) {
  await page.addInitScript(({ responses, dictionary }) => {
    const callbacks = new Map<number, (event: unknown) => void>();
    const indexes: Record<string, number> = {};
    window.testNative = { calls: [], close: () => { for (const callback of callbacks.values()) callback({ payload: null }); } };
    const defaults: Record<string, unknown> = { runtime_status: { version: '0.1.0', schemaVersion: 6, storage: 'sqlite' },
      dictionary_lookup: dictionary, vocabulary_find: null, vocabulary_list: [], vocabulary_sort: 'updated-desc', finish_close: null,
      speech_preferences: { voice: 'female', speed: 'normal' }, review_settings: { retention_percent: 90, new_limit: 20, review_limit: 50 }, review_queue: { queue: { cards: [], reason: null, available_at: null }, record: null, audio_pending: false, audio_error: null }, review_cards: [], speech_cached: null,
      review_undo_target: null, review_undo: { status: 'committed', queue_rebuilt: false },
      review_auto_enroll: false, review_pause: false,
      review_progress: { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } } };
    function defaultAvailability() {
        const progress = (responses.review_progress?.[0]?.value ?? defaults.review_progress) as { card: { due_at: number | null } };
        const due = progress.card.due_at;
        return { progress, reason: defaults.review_pause ? 'paused' : due && due * 1000 > Date.now() ? 'due' : null, available_at: due && due * 1000 > Date.now() ? due : null };
    }
    function updateBoolean(command: string, args?: Record<string, unknown>) {
      const booleanKey = { review_auto_enroll: 'enabled', review_pause: 'paused' }[command];
      if (booleanKey && typeof args?.[booleanKey] === 'boolean') defaults[command] = args[booleanKey];
    }
    function defaultReply(command: string, args?: Record<string, unknown>) {
      const settingKey = command === 'speech_preferences' ? 'preferences' : 'settings';
      if ((command === 'speech_preferences' || command === 'review_settings') && args?.[settingKey]) defaults[command] = args[settingKey];
      updateBoolean(command, args);
      if (command === 'review_availability') return defaultAvailability();
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
        if (response.delay) await new Promise((resolve) => setTimeout(resolve, response.delay));
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
