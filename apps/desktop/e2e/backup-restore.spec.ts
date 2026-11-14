import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';
import { cancelledBatch, cleanup, completed, pendingBatch, pendingRating, pendingUndo, preparing, preserving, previewReady, ratingResult, recovery, restarting } from './backup-restore-fixture';

test('unresolved startup recovery gates every learning mount and supports close', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: recovery }], runtime_status: [{ error: 'Database unavailable' }] });
  await page.addInitScript(() => localStorage.setItem('lexique-pending-rating-88', 'original request'));
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '需要恢复学习数据', exact: true })).toBeVisible();
  await expect(page.getByRole('alert')).toContainText('Restore journal cannot be read');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => !call.command.startsWith('plugin:event|')).map((call) => call.command))).toEqual(['startup_status']);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe('original request');
  await page.evaluate(() => window.testNative.close());
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'finish_close'))).toBe(true);
});

test('committed restore clears all known pending requests before native acknowledgement and learning', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: cleanup }], backup_restore_cleanup_begin: [{ value: cleanup, gate: 'cleanup-begin' }],
    backup_restore_cleanup_complete: [{ value: completed, gate: 'cleanup-ack' }] });
  await page.addInitScript(() => {
    for (const key of ['lexique-pending-rating-88', 'lexique-pending-rating-99999', 'lexique-pending-undo', 'lexique-pending-review-batch']) localStorage.setItem(key, 'original request');
    localStorage.setItem('reader-preference', 'keep');
  });
  await page.goto('/');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_restore_cleanup_begin'))).toBe(true);
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('lexique-pending-')).length)).toBe(4);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => !call.command.startsWith('plugin:event|')).map((call) => call.command))).toEqual(['startup_status', 'backup_restore_cleanup_begin']);
  await page.evaluate(() => window.testNative.release('cleanup-begin'));
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_restore_cleanup_complete'))).toBe(true);
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('lexique-pending-')))).toEqual([]);
  expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), completed);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('cleanup-ack'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await expect(page.getByText('恢复已完成', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => ['review_rate', 'review_undo', 'review_add_batch'].includes(call.command)))).toEqual([]);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command.startsWith('backup_restore_cleanup_')).map((call) => call.args))).toEqual([{ operation: 'preview-1' }, { operation: 'preview-1' }]);
});

test('whole-backup restore preserves confirmed requests and gates learning through controlled restart', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], review_rate: [{ value: ratingResult }], review_batch_status: [{ value: cancelledBatch }],
    backup_restore_start: [{ value: preserving, gate: 'restore-start' }], backup_restore_status: [{ value: preparing }] });
  await page.addInitScript(({ rating, undo, batch }) => {
    localStorage.setItem('lexique-pending-rating-88', JSON.stringify(rating));
    localStorage.setItem('lexique-pending-rating-99999', JSON.stringify({ ...rating, operation: 'pending-rating-99999', card_id: 99999 }));
    localStorage.setItem('lexique-pending-undo', JSON.stringify(undo));
    localStorage.setItem('lexique-pending-review-batch', JSON.stringify(batch));
    localStorage.setItem('reader-preference', 'keep');
  }, { rating: pendingRating, undo: pendingUndo, batch: pendingBatch });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await page.getByRole('button', { name: '恢复整份备份', exact: true }).click();
  const confirmation = page.getByRole('dialog', { name: '确认恢复整份备份', exact: true });
  await expect(confirmation).toContainText('checkpoint.lexique-backup');
  await expect(confirmation.getByRole('button', { name: '确认并恢复', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_restore_start'))).toBe(false);
  await confirmation.getByRole('checkbox').check();
  await confirmation.getByRole('button', { name: '确认并恢复', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_restore_start'))).toBe(true);
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('lexique-pending-')).length)).toBe(4);
  expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate').map((call) => call.args?.request))).toEqual([
    pendingRating, { ...pendingRating, operation: 'pending-rating-99999', card_id: 99999 }]);
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_add_batch'))).toBe(false);
  await expect(page.getByRole('heading', { name: '设置', exact: true })).toHaveCount(0);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), { ...restarting, operation: 'stale-preview' });
  await expect(page.getByText('已请求重启，正在等待应用重新启动…', { exact: true })).toHaveCount(0);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), preparing);
  await expect(page.getByRole('status')).toContainText('正在准备恢复数据和音频');
  await page.evaluate(() => window.testNative.release('restore-start'));
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_restore_status'))).toBe(true);
  await expect(page.getByRole('status')).toContainText('正在准备恢复数据和音频');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), restarting);
  await expect(page.getByRole('status')).toContainText('已请求重启');
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '取消恢复', exact: true })).toHaveCount(0);
  await expect(page.getByRole('region', { name: '保留的副本', exact: true })).toContainText('48,000 字节');
  await page.getByRole('button', { name: '打开保留目录', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_retained_directory_open'))).toBe(true);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'backup_restore_start').map((call) => call.args))).toEqual([{ operation: 'preview-1' }]);
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_retained_directory_open')?.args)).toEqual({ copy: 'retained-backup-1' });
});
