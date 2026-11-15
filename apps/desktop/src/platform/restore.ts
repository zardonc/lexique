import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { z } from 'zod';
import { backupPreviewOperation, restoreStatusSchema, startupStatusSchema } from './backup';
import type { RawPendingRequest, RestoreStatus, StartupStatus } from './backup';

async function restoreRequest(command: 'backup_restore_start' | 'backup_restore_status', operation: string, pendingRequests?: readonly RawPendingRequest[]) {
  if (!isTauri()) throw new Error('Restoring backups is available in the desktop app only');
  const id = backupPreviewOperation.parse(operation);
  const value = restoreStatusSchema.parse(await invoke(command, { operation: id, ...(pendingRequests === undefined ? {} : { pendingRequests }) }));
  if (value.operation !== id) throw new Error('Restore response identity differs from the request');
  return value;
}

export const restoreRuntime = {
  startRestore: (operation: string, pendingRequests?: readonly RawPendingRequest[]) => restoreRequest('backup_restore_start', operation, pendingRequests),
  restoreStatus: (operation: string) => restoreRequest('backup_restore_status', operation),
  async onRestoreProgress(callback: (status: RestoreStatus) => void, failed: (cause: unknown) => void) {
    if (!isTauri()) return () => {};
    return listen('lexique-backup-restore-progress', (event) => {
      const value = restoreStatusSchema.safeParse(event.payload);
      if (value.success) callback(value.data); else failed(value.error);
    });
  },
  async startupStatus(): Promise<StartupStatus> {
    if (!isTauri()) return { state: 'ready', operation: null, restored: null, retainedCopies: [], error: null };
    return startupStatusSchema.parse(await invoke('startup_status'));
  },
  async onStartupStatus(callback: (status: StartupStatus) => void, failed: (cause: unknown) => void) {
    if (!isTauri()) return () => {};
    return listen('lexique-startup-status', (event) => {
      const value = startupStatusSchema.safeParse(event.payload);
      if (value.success) callback(value.data); else failed(value.error);
    });
  },
  async retryStartup(): Promise<StartupStatus> {
    if (!isTauri()) throw new Error('Startup recovery is available in the desktop app only');
    return startupStatusSchema.parse(await invoke('startup_retry'));
  },
  async beginRestoreCleanup(operation: string) {
    if (!isTauri()) throw new Error('Restore cleanup is available in the desktop app only');
    const id = backupPreviewOperation.parse(operation);
    const value = startupStatusSchema.parse(await invoke('backup_restore_cleanup_begin', { operation: id }));
    if (value.operation !== id) throw new Error('Restore cleanup response identity differs from the request');
    return value;
  },
  async completeRestoreCleanup(operation: string) {
    if (!isTauri()) throw new Error('Restore cleanup is available in the desktop app only');
    const id = backupPreviewOperation.parse(operation);
    const value = startupStatusSchema.parse(await invoke('backup_restore_cleanup_complete', { operation: id }));
    if (value.operation !== id) throw new Error('Restore cleanup response identity differs from the request');
    return value;
  },
  async openRetainedDirectory(copy: string) {
    if (!isTauri()) throw new Error('Retained copies are available in the desktop app only');
    const id = z.string().min(1).refine((value) => value.trim().length > 0).parse(copy);
    await invoke('backup_retained_directory_open', { copy: id });
  },
};
