import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';
import { confirmRestore, nativeCalls, pendingRating, preparing, preserving, previewReady, ratingResult, ready, recovery, restarting, selectBackup } from './backup-restore-fixture';

const failed = { ...preparing, state: 'failed', phase: null, error: 'Preparation failed' };

test('formal failure exposes the fresh recovery diagnostic while learning remains gated', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: ready }, { value: { ...recovery, operation: 'preview-1', error: 'Recovery journal unavailable' } }],
    backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: failed }] });
  await page.goto('/');
  await selectBackup(page);
  await confirmRestore(page);
  await expect(page.getByRole('alert')).toContainText('Preparation failed');
  const initialReads = await nativeCalls(page, 'runtime_status');
  await page.getByRole('button', { name: '返回学习', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Recovery journal unavailable');
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '重试恢复', exact: true })).toBeVisible();
  expect(await nativeCalls(page, 'runtime_status')).toEqual(initialReads);
});

test('a failed formal attempt needs a fresh startup read and a newly verified preview', async ({ page }) => {
  const nextPreview = { ...previewReady, operation: 'preview-2' }, nextRestore = { ...preparing, operation: 'preview-2' };
  await nativeFixture(page, { startup_status: [{ value: ready }, { value: ready, gate: 'fresh-startup' }],
    backup_preview_start: [{ value: previewReady }, { value: nextPreview }], backup_restore_start: [{ value: failed }, { value: nextRestore }],
    backup_restore_status: [{ value: nextRestore }], review_rate: [{ value: ratingResult }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)), pendingRating);
  await page.goto('/');
  await selectBackup(page);
  await confirmRestore(page);
  await expect(page.getByRole('alert')).toContainText('Preparation failed');
  await page.getByRole('button', { name: '返回学习', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'startup_status')).toHaveLength(2);
  await expect(page.getByRole('button', { name: '返回学习', exact: true })).toBeDisabled();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('fresh-startup'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('button', { name: '恢复整份备份', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await confirmRestore(page);
  await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), failed);
  await expect(page.getByRole('status')).toContainText('正在准备恢复数据和音频');
  expect((await nativeCalls(page, 'backup_restore_start')).map((call) => call.args)).toEqual([{ operation: 'preview-1' }, { operation: 'preview-2' }]);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe(JSON.stringify(pendingRating));
});

test('an explicit native rejection allows returning only after rereading ready startup', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: ready }, { value: ready, gate: 'ready-recheck' }],
    backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ error: 'Restore identity expired' }] });
  await page.goto('/');
  await selectBackup(page);
  await confirmRestore(page);
  await expect(page.getByRole('alert')).toContainText('Restore identity expired');
  await page.getByRole('button', { name: '返回学习', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'startup_status')).toHaveLength(2);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('ready-recheck'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
});

test('a malformed start reply keeps learning gated until a native failed status is known', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }],
    backup_restore_start: [{ value: { ...preparing, phase: 'unknown' } }], backup_restore_status: [{ value: failed }] });
  await page.goto('/');
  await selectBackup(page);
  await confirmRestore(page);
  await expect(page.getByRole('alert')).toBeVisible();
  await expect(page.getByRole('button', { name: '返回学习', exact: true })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '检查恢复状态', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Preparation failed');
  await page.getByRole('button', { name: '返回学习', exact: true }).click();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
});

test('restore progress ignores phase and terminal regression and rejects malformed events', async ({ page }) => {
  const closing = { ...preparing, phase: 'closing' };
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: preparing }],
    backup_restore_status: [{ value: preparing }, { value: closing }] });
  await page.goto('/');
  await selectBackup(page);
  await confirmRestore(page);
  await expect.poll(() => nativeCalls(page, 'backup_restore_status')).toHaveLength(1);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), closing);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), preserving);
  await expect(page.getByRole('status')).toContainText('正在关闭旧学习状态');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), { ...closing, phase: 'unknown' });
  await expect(page.getByRole('alert')).toBeVisible();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '检查恢复状态', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('正在关闭旧学习状态');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), restarting);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), failed);
  await expect(page.getByRole('status')).toContainText('已请求重启');
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  expect(await nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
});

test('double confirmation starts one formal restore and preserves its pending request', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], review_rate: [{ value: ratingResult, gate: 'confirm-pending' }],
    backup_restore_start: [{ value: preparing, gate: 'formal-start' }], backup_restore_status: [{ value: preparing }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)), pendingRating);
  await page.goto('/');
  await selectBackup(page);
  await page.getByRole('button', { name: '恢复整份备份', exact: true }).click();
  const confirmation = page.getByRole('dialog', { name: '确认恢复整份备份', exact: true });
  await confirmation.getByRole('checkbox').check();
  await confirmation.getByRole('button', { name: '确认并恢复', exact: true }).dblclick();
  await expect.poll(() => nativeCalls(page, 'review_rate')).toHaveLength(1);
  expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
  await page.evaluate(() => window.testNative.release('confirm-pending'));
  await expect.poll(() => nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
  await expect(page.getByRole('button', { name: '恢复整份备份', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('formal-start'));
  await expect(page.getByRole('status')).toContainText('正在准备恢复数据和音频');
  expect(await nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe(JSON.stringify(pendingRating));
});

test('a new formal restore reports current progress after a completed rollback', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: { ...ready, operation: 'previous-restore', error: 'Previous learning state was restored' } }],
    backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: preparing }], backup_restore_status: [{ value: preparing }] });
  await page.goto('/');
  await expect(page.getByRole('alert')).toContainText('Previous learning state was restored');
  await selectBackup(page);
  await confirmRestore(page);
  await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
  await expect(page.getByRole('status')).toContainText('正在准备恢复数据和音频');
  await expect(page.getByRole('alert')).toHaveCount(0);
});
