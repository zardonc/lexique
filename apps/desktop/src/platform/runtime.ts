import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';

const runtimeStatusSchema = z.object({
  version: z.string().min(1),
  schemaVersion: z.number().int().positive(),
  storage: z.literal('sqlite'),
}).strict().readonly();

export type RuntimeStatus = z.infer<typeof runtimeStatusSchema>;

export interface RuntimeAdapter {
  readonly getStatus: () => Promise<RuntimeStatus | null>;
}

export const runtime: RuntimeAdapter = {
  async getStatus() {
    if (!isTauri()) return null;
    return runtimeStatusSchema.parse(await invoke('runtime_status'));
  },
};
