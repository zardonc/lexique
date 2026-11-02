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
  id: 1, headword: 'école', notes: '', example_fr: '', example_zh: '', example_sense: null, more_examples: [], revision: 1, created_at: 1000, updated_at: 1000,
  reference: { dictionary: dictionary.source.title, lookup_key: 'école', version: dictionary.source.version,
    attribution: dictionary.source.attribution, license: dictionary.source.license,
    source_url: dictionary.source.source_url, license_url: dictionary.source.license_url, sha256: dictionary.sha256 },
  digest: [{ pronunciations: ['e.kɔl'], grammar: ['n', 'fem'], translation: '学校' },
    { pronunciations: [], grammar: [], translation: '流派' }],
};

interface Reply { readonly value?: unknown; readonly error?: unknown; readonly delay?: number; readonly gate?: string }
interface Call { readonly command: string; readonly args?: Record<string, unknown> }
declare global {
  interface Window { testNative: { calls: Call[]; close: () => void; emit: (event: string) => void; release: (gate: string) => void } }
}

class NativeFixture {
  private readonly callbacks = new Map<number, (event: unknown) => void>();
  private readonly indexes: Record<string, number> = {};
  private readonly gates = new Map<string, () => void>();
  private readonly listeners = new Map<number, { event: string; handler: number }>();
  private readonly calls: Call[] = [];
  private readonly defaults: Record<string, unknown>;

  constructor(private readonly responses: Readonly<Record<string, readonly Reply[]>>, lookup: typeof dictionary) {
    this.defaults = { runtime_status: { version: '0.1.0', schemaVersion: 11, storage: 'sqlite' },
      dictionary_lookup: lookup, vocabulary_find: null, vocabulary_list: [], vocabulary_sort: 'updated-desc', finish_close: null,
      speech_preferences: { voice: 'female', speed: 'normal' }, review_settings: { retention_percent: 90, new_limit: 20, review_limit: 50, new_cards_ignore_review_limit: false },
      review_queue: { queue: { cards: [], reason: null, available_at: null }, record: null, audio_pending: false, audio_error: null }, review_cards: [], speech_cached: null,
      review_undo_target: null, review_undo: { status: 'committed', queue_rebuilt: false }, review_auto_enroll: false, review_pause: false,
      review_batch_status: null, speech_retry_missing: 1, review_cancel_batch: null,
      review_progress: { revision: 0, card: { phase: 'new', memory: null, last_review_at: null, due_at: null } } };
  }
  private emit(event: string) {
    for (const [id, listener] of this.listeners) if (listener.event === event) this.callbacks.get(listener.handler)?.({ event, id, payload: null });
  }
  private availability() {
    const progress = (this.responses.review_progress?.[0]?.value ?? this.defaults.review_progress) as { card: { due_at: number | null } };
    const due = progress.card.due_at;
    const future = due !== null && due * 1000 > Date.now();
    return { progress, reason: this.defaults.review_pause ? 'paused' : future ? 'due' : null, available_at: future ? due : null };
  }
  private batch(args?: Record<string, unknown>) {
    const request = args?.request as { operation: string; vocabulary_ids: number[] };
    return this.defaults.review_batch_status = { operation: request.operation, cancelled: false,
      items: request.vocabulary_ids.map((vocabulary_id) => ({ vocabulary_id, created_cards: 2, error: null, audio_state: 'queued', audio_error: null })) };
  }
  private settings(command: string, args?: Record<string, unknown>) {
    const settingKey = command === 'speech_preferences' ? 'preferences' : 'settings';
    if ((command === 'speech_preferences' || command === 'review_settings') && args?.[settingKey]) this.defaults[command] = args[settingKey];
    const booleanKey = { review_auto_enroll: 'enabled', review_pause: 'paused' }[command];
    if (booleanKey && typeof args?.[booleanKey] === 'boolean') this.defaults[command] = args[booleanKey];
  }
  private defaultReply(command: string, args?: Record<string, unknown>) {
    if (command === 'review_add_batch') return this.batch(args);
    this.settings(command, args);
    if (command === 'review_availability') return this.availability();
    return command === 'vocabulary_sort' ? args?.sort ?? this.defaults[command] : this.defaults[command];
  }
  private event(command: string, args?: Record<string, unknown>) {
    if (command === 'plugin:event|listen') {
      const id = Math.max(0, ...this.listeners.keys()) + 1;
      this.listeners.set(id, { event: String(args?.event), handler: Number(args?.handler) }); return id;
    }
    this.listeners.delete(Number(args?.eventId)); return null;
  }
  private async invoke(command: string, args?: Record<string, unknown>) {
    this.calls.push({ command, args });
    if (command.startsWith('plugin:event|')) return this.event(command, args);
    const queue = this.responses[command];
    if (!queue) return this.defaultReply(command, args);
    const index = this.indexes[command] ?? 0; this.indexes[command] = index + 1;
    const response = queue[Math.min(index, queue.length - 1)];
    if (response.gate) await new Promise<void>((resolve) => { this.gates.set(response.gate!, resolve); });
    else if (response.delay) await new Promise((resolve) => setTimeout(resolve, response.delay));
    if (response.error !== undefined) throw response.error;
    if (command === 'vocabulary_save') this.defaults.vocabulary_find = (response.value as { record: unknown }).record;
    if (command === 'review_add_batch') return this.batchResponse(response.value, args);
    return response.value;
  }
  private batchResponse(value: unknown, args?: Record<string, unknown>) {
    const batch = value as { operation: string };
    const request = args?.request as { operation: string };
    return this.defaults.review_batch_status = batch.operation === 'fixture' ? { ...batch, operation: request.operation } : batch;
  }
  install() {
    window.testNative = { calls: this.calls, close: () => this.emit('lexique-close-requested'), emit: (event) => this.emit(event),
      release: (gate) => { this.gates.get(gate)?.(); this.gates.delete(gate); } };
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {
      transformCallback: (callback: (event: unknown) => void) => { const id = Math.max(0, ...this.callbacks.keys()) + 1; this.callbacks.set(id, callback); return id; },
      unregisterCallback: (id: number) => this.callbacks.delete(id),
      invoke: (command: string, args?: Record<string, unknown>) => this.invoke(command, args),
    } });
  }
}

export async function nativeFixture(page: Page, responses: Readonly<Record<string, readonly Reply[]>> = {}) {
  // Serialize the complete self-contained fixture, so browser callbacks share one registry.
  await page.addInitScript(`new (${NativeFixture.toString()})(${JSON.stringify(responses)}, ${JSON.stringify(dictionary)}).install();`);
}

export async function collect(page: Page) {
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
}
