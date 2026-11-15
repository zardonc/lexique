import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';
import { confirmRestore, nativeCalls, pendingBatch, preparing, previewReady, ratingResult, selectBackup } from './backup-restore-fixture';

async function learningReads(page: Page) {
  return page.evaluate(() => window.testNative.calls.filter((call) => ['review_queue', 'review_availability', 'review_undo_target'].includes(call.command)));
}

async function pendingRating(page: Page) {
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await page.getByRole('button', { name: '显示答案', exact: true }).click();
  await page.getByRole('button', { name: '记得', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'review_rate')).toHaveLength(1);
  return page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'));
}

for (const action of ['cancel', 'retry audio'] as const) {
  test(`a delayed batch ${action} reply cannot read learning state after formal restore starts`, async ({ page }) => {
    const status = { operation: pendingBatch.operation, cancelled: false,
      items: [{ vocabulary_id: 10, created_cards: 2, error: null, audio_state: action === 'cancel' ? 'queued' : 'failed', audio_error: 'Audio missing' }] };
    const command = action === 'cancel' ? 'review_cancel_batch' : 'speech_generate';
    await nativeFixture(page, { review_batch_status: [{ value: status }], backup_preview_start: [{ value: previewReady }],
      backup_restore_start: [{ value: preparing }], backup_restore_status: [{ value: preparing }],
      [command]: [{ gate: 'old-batch-reply', value: action === 'cancel' ? null : { bytes: [79, 103, 103, 83], voice: 'female' } }] });
    await page.goto('/');
    await page.getByRole('button', { name: action === 'cancel' ? '停止剩余音频生成' : '重试 #10', exact: true }).click();
    await expect.poll(() => nativeCalls(page, command)).toHaveLength(1);
    await selectBackup(page);
    await confirmRestore(page);
    await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
    const previous = await nativeCalls(page, 'review_batch_status');
    await page.evaluate(() => window.testNative.release('old-batch-reply'));
    expect(await nativeCalls(page, 'review_batch_status')).toHaveLength(previous.length);
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
    await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  });
}

for (const result of ['success', 'rejection'] as const) {
  for (const outcome of ['running', 'returned'] as const) {
    test(`a delayed rating ${result} preserves its key with restore ${outcome}`, async ({ page }) => {
      const queue = { queue: { cards: [{ id: 88, vocabulary_id: 1, kind: 'recognition' }], reason: null, available_at: null }, record, audio_pending: false, audio_error: null };
      const restore = outcome === 'running' ? preparing : { ...preparing, state: 'failed', phase: null, error: 'Preparation failed' };
      const original = result === 'success' ? ratingResult : { status: 'rejected', code: 'conflict', detail: 'Original rating changed' };
      await nativeFixture(page, { review_queue: [{ value: queue }], review_rate: [{ value: original, gate: 'original-rating' }, { value: ratingResult }],
        backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: restore }], backup_restore_status: [{ value: restore }] });
      await page.goto('/');
      const saved = await pendingRating(page);
      expect(saved).not.toBeNull();
      await selectBackup(page);
      await confirmRestore(page);
      await expect.poll(() => nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
      if (outcome === 'returned') {
        await page.getByRole('button', { name: '返回学习', exact: true }).click();
        await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
      } else await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
      const reads = await learningReads(page);
      await page.evaluate(() => window.testNative.release('original-rating'));
      expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe(saved);
      expect(await learningReads(page)).toEqual(reads);
      expect(await nativeCalls(page, 'review_rate')).toHaveLength(2);
      await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
    });
  }
}

for (const result of ['success', 'rejection'] as const) {
  test(`a delayed undo ${result} cannot clear its confirmed original request or reread learning`, async ({ page }) => {
    const original = result === 'success' ? { status: 'committed', queue_rebuilt: true } : { status: 'rejected', detail: 'Original undo changed' };
    await nativeFixture(page, { review_undo_target: [{ value: 'last-rating' }],
      review_undo: [{ value: original, gate: 'original-undo' }, { value: { status: 'committed', queue_rebuilt: true } }],
      backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: preparing }], backup_restore_status: [{ value: preparing }] });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    await page.getByRole('button', { name: '撤销最近评分', exact: true }).click();
    await expect.poll(() => nativeCalls(page, 'review_undo')).toHaveLength(1);
    const saved = await page.evaluate(() => localStorage.getItem('lexique-pending-undo'));
    expect(saved).not.toBeNull();
    await selectBackup(page);
    await confirmRestore(page);
    await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
    const reads = await learningReads(page);
    await page.evaluate(() => window.testNative.release('original-undo'));
    expect(await page.evaluate(() => localStorage.getItem('lexique-pending-undo'))).toBe(saved);
    expect(await learningReads(page)).toEqual(reads);
    expect(await nativeCalls(page, 'review_undo')).toHaveLength(2);
  });
}
