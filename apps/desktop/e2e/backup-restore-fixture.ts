import type { Page } from '@playwright/test';

export const ready = { state: 'ready', operation: null, restored: null, retained_copies: [], error: null };
export const recovery = { ...ready, state: 'recovery_required', operation: 'restore-1', error: 'Restore journal cannot be read' };
export const restored = { filename: 'checkpoint.lexique-backup', byte_length: 4096, created_at: 1730000000, application_version: '0.1.0', requires_restart: true,
  counts: { vocabulary: 1, cards: 2, ratings: 3, batches: 1, custom_dictionaries: 0, legacy_audio: 0, included_audio_versions: 1,
    included_legacy_audio: 0, audio_files: 1, source_missing_audio: 0, source_invalid_audio: 0 }, pending_dictionaries: [] };
export const cleanup = { ...ready, state: 'cleanup_required', operation: 'preview-1', restored };
export const completed = { ...cleanup, state: 'ready' };
export const previewReady = { operation: 'preview-1', state: 'preview_ready', phase: null, cancel_requested: false, error: null, preview: restored };
export const preserving = { operation: 'preview-1', state: 'running', phase: 'preserving', preview: restored, retained_copies: [], error: null };
export const retained = [{ id: 'retained-backup-1', kind: 'logical_backup', path: 'C:\\Backups\\before-restore.lexique-backup', byte_length: 48000 }];
export const preparing = { ...preserving, phase: 'preparing', retained_copies: retained };
export const restarting = { ...preparing, state: 'restart_requested', phase: null };
export const ratingResult = { status: 'committed', result: {
  card: { phase: 'review', memory: { stability: 7, difficulty: 5 }, last_review_at: 1000, due_at: 87400 }, elapsed_days: 0, step_seconds: null,
  interval_days: 1, model: { version: 'test-model', parameters: [] }, retention_percent: 90, timezone_database: '2026b',
} };
export const pendingRating = { operation: 'pending-rating-88', card_id: 88, expected_revision: 2, rating: 'good', timezone: 'UTC' };
export const pendingUndo = { operation: 'pending-undo', ratingOperation: 'earlier-rating' };
export const pendingBatch = { operation: 'pending-batch', vocabulary_ids: [10, 20, 30] };
export const cancelledBatch = { operation: 'pending-batch', cancelled: true,
  items: [{ vocabulary_id: 10, created_cards: 2, error: null, audio_state: 'cancelled', audio_error: null }] };

export async function nativeCalls(page: Page, command: string) {
  return page.evaluate((name) => window.testNative.calls.filter((call) => call.command === name), command);
}

export async function selectBackup(page: Page) {
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
}

export async function confirmRestore(page: Page) {
  await page.getByRole('button', { name: '恢复整份备份', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '确认恢复整份备份', exact: true });
  await dialog.getByRole('checkbox').check();
  await dialog.getByRole('button', { name: '确认并恢复', exact: true }).click();
}
