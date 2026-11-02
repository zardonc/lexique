import type { QueueView, RuntimeAdapter } from './platform/runtime';

const flights = new WeakMap<RuntimeAdapter, Promise<QueueView | null>>();

// A pending native read survives page unmount. The next context waits for it.
export async function readQueue(runtime: RuntimeAdapter, skipped: readonly number[]) {
  while (flights.has(runtime)) { try { await flights.get(runtime); } catch { /* The original reader owns its error. */ } }
  const request = runtime.reviewQueue(skipped);
  flights.set(runtime, request);
  try { return await request; } finally { if (flights.get(runtime) === request) flights.delete(runtime); }
}

interface Options {
  readonly read: () => Promise<void>;
  readonly visible: () => boolean;
  readonly failed: (cause: unknown) => void;
}

export class QueueRefresh {
  private timer: ReturnType<typeof setTimeout> | undefined;
  private pending = false;
  private running = false;
  private active = true;
  private first = 0;

  constructor(private readonly options: Options) {}

  invalidate(immediate = false) {
    if (!this.active) return;
    this.pending = true;
    this.first ||= Date.now();
    if (!this.options.visible() || this.running) return;
    clearTimeout(this.timer);
    const delay = immediate ? 0 : Math.max(0, Math.min(50, this.first + 200 - Date.now()));
    this.timer = setTimeout(() => { void this.read(); }, delay);
  }

  suspend() { clearTimeout(this.timer); this.timer = undefined; }
  stop() { this.active = false; this.suspend(); }

  private async read() {
    if (!this.active || !this.options.visible() || this.running) return;
    this.running = true; this.pending = false; this.first = 0;
    try { await this.options.read(); }
    catch (cause) { if (this.active) this.options.failed(cause); }
    finally {
      this.running = false;
      if (this.pending && this.active) this.invalidate();
    }
  }
}

// One absolute wake, including the next 04:00 local learning day. Focus/resume
// rechecks wall time and timezone; no idle interval polls the database.
export function nextQueueWake(view: QueueView | null) {
  const day = new Date(); day.setHours(4, 0, 0, 0);
  if (day.getTime() <= Date.now()) day.setDate(day.getDate() + 1);
  const due = view?.queue.available_at == null ? Infinity : view.queue.available_at * 1000;
  return Math.min(day.getTime(), due > Date.now() ? due : Infinity);
}
