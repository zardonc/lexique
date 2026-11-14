import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { nativeFixture } from './native-fixture';
import { cleanup, completed, nativeCalls, ready, recovery, retained } from './backup-restore-fixture';

declare global { interface Window { repairPendingStorage: () => void } }

async function failPendingStorage(page: Page, fault: 'enumeration' | 'remove' | 'readback' | 'readback error') {
  await page.addInitScript((fault) => {
    localStorage.setItem('lexique-pending-rating-99999', 'old timeline request');
    localStorage.setItem('reader-preference', 'keep');
    let active = true;
    const original = { key: Storage.prototype.key, getItem: Storage.prototype.getItem, removeItem: Storage.prototype.removeItem };
    const fail = (key: string) => active && key === 'lexique-pending-rating-99999';
    Storage.prototype.key = function (index) {
      if (active && fault === 'enumeration') throw new Error('Storage enumeration denied');
      return original.key.call(this, index);
    };
    Storage.prototype.removeItem = function (key) {
      if (fail(key) && fault === 'remove') throw new Error('Storage removal denied');
      if (fail(key) && fault === 'readback') return;
      original.removeItem.call(this, key);
    };
    Storage.prototype.getItem = function (key) {
      if (fail(key) && fault === 'readback error') throw new Error('Storage readback denied');
      return original.getItem.call(this, key);
    };
    window.repairPendingStorage = () => { active = false; };
  }, fault);
}

test('startup retries a failed status subscription before reading or mounting learning', async ({ page }) => {
  await nativeFixture(page, { 'plugin:event|listen:lexique-startup-status': [
    { error: 'Startup subscription unavailable' }, { error: 'Startup subscription unavailable' }, { value: null },
  ] });
  await page.addInitScript(() => localStorage.setItem('lexique-pending-rating-99999', 'original request'));
  await page.goto('/');
  await expect(page.getByRole('alert')).toContainText('Startup subscription unavailable');
  expect(await nativeCalls(page, 'startup_status')).toEqual([]);
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
  await page.getByRole('button', { name: '重试恢复', exact: true }).click();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  expect(await nativeCalls(page, 'startup_status')).toHaveLength(1);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-99999'))).toBe('original request');
});

test('a cleanup event before the startup retry reply still waits for both cleanup acknowledgements', async ({ page }) => {
  await nativeFixture(page, { startup_status: [{ value: { ...recovery, operation: 'preview-1' } }],
    startup_retry: [{ value: cleanup, gate: 'retry-reply' }], backup_restore_cleanup_begin: [{ value: cleanup }],
    backup_restore_cleanup_complete: [{ value: completed, gate: 'cleanup-complete' }] });
  await page.addInitScript(() => localStorage.setItem('lexique-pending-rating-99999', 'original request'));
  await page.goto('/');
  await page.getByRole('button', { name: '重试恢复', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'startup_retry')).toHaveLength(1);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), cleanup);
  await expect.poll(() => nativeCalls(page, 'backup_restore_cleanup_complete')).toHaveLength(1);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), completed);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('retry-reply'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('cleanup-complete'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await expect(page.getByText('恢复已完成', { exact: true })).toBeVisible();
  expect(await nativeCalls(page, 'backup_restore_cleanup_begin')).toHaveLength(1);
  expect(await nativeCalls(page, 'backup_restore_cleanup_complete')).toHaveLength(1);
});

test('recovery keeps retained copy identities and close failures visible while retrying startup', async ({ page }) => {
  const copies = [...retained, { id: 'database-copy', kind: 'original_database', path: 'C:\\Backups\\original-database', byte_length: 16384 },
    { id: 'audio-copy', kind: 'original_audio', path: 'C:\\Backups\\original-audio', byte_length: 32768 }];
  await nativeFixture(page, { startup_status: [{ value: { ...recovery, retained_copies: copies } }],
    startup_retry: [{ error: 'Recovery still unavailable' }, { value: { ...ready, operation: recovery.operation, retained_copies: copies, error: 'Previous learning state was restored' } }],
    backup_retained_directory_open: [{ error: 'Explorer unavailable' }], finish_close: [{ error: 'Close request failed' }, { value: null }] });
  await page.addInitScript(() => localStorage.setItem('lexique-pending-rating-99999', 'original request'));
  await page.goto('/');
  const copiesRegion = page.getByRole('region', { name: '保留的副本', exact: true });
  await expect(copiesRegion).toContainText('原学习数据库');
  await expect(copiesRegion).toContainText('原单词音频');
  await expect(copiesRegion).toContainText('16,384 字节');
  await copiesRegion.getByRole('button', { name: '打开保留目录', exact: true }).nth(1).click();
  await expect(copiesRegion.getByRole('alert')).toContainText('Explorer unavailable');
  expect((await nativeCalls(page, 'backup_retained_directory_open'))[0].args).toEqual({ copy: 'database-copy' });
  await page.getByRole('button', { name: '关闭应用', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'Close request failed' })).toBeVisible();
  await page.getByRole('button', { name: '关闭应用', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'finish_close')).toHaveLength(2);
  await page.getByRole('button', { name: '重试恢复', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'Recovery still unavailable' })).toBeVisible();
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
  await page.getByRole('button', { name: '重试恢复', exact: true }).click();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await expect(page.getByRole('alert')).toContainText('Previous learning state was restored');
  await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-99999'))).toBe('original request');
});

for (const acknowledgement of ['begin', 'complete'] as const) {
  test(`cleanup rejects the wrong ${acknowledgement} identity and retries before enabling learning`, async ({ page }) => {
    const wrong = { ...(acknowledgement === 'begin' ? cleanup : completed), operation: 'other-restore' };
    await nativeFixture(page, { startup_status: [{ value: cleanup }],
      backup_restore_cleanup_begin: acknowledgement === 'begin' ? [{ value: wrong }, { value: cleanup }] : [{ value: cleanup }],
      backup_restore_cleanup_complete: acknowledgement === 'complete' ? [{ value: wrong }, { value: completed }] : [{ value: completed }] });
    await page.addInitScript(() => {
      localStorage.setItem('lexique-pending-rating-99999', 'old timeline request');
      localStorage.setItem('reader-preference', 'keep');
    });
    await page.goto('/');
    await expect(page.getByRole('alert')).toContainText('Restore cleanup response identity');
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
    await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
    expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
    expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
    if (acknowledgement === 'begin') {
      expect(await nativeCalls(page, 'backup_restore_cleanup_complete')).toEqual([]);
      expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-99999'))).toBe('old timeline request');
    }
    await page.getByRole('button', { name: '重试恢复', exact: true }).click();
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
    await expect(page.getByText('恢复已完成', { exact: true })).toBeVisible();
    expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-99999'))).toBeNull();
  });
}

for (const [fault, message] of [
  ['enumeration', 'Storage enumeration denied'], ['remove', 'Storage removal denied'],
  ['readback', 'Pending review request removal was not confirmed'], ['readback error', 'Storage readback denied'],
] as const) {
  test(`cleanup keeps learning gated on ${fault} failure until repaired and retried`, async ({ page }) => {
    await nativeFixture(page, { startup_status: [{ value: cleanup }], backup_restore_cleanup_begin: [{ value: cleanup }],
      backup_restore_cleanup_complete: [{ value: completed }] });
    await failPendingStorage(page, fault);
    await page.goto('/');
    await expect(page.getByRole('alert')).toContainText(message);
    expect(await nativeCalls(page, 'backup_restore_cleanup_complete')).toEqual([]);
    expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
    expect(await nativeCalls(page, 'review_rate')).toEqual([]);
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toHaveCount(0);
    await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
    await page.evaluate(() => window.repairPendingStorage());
    await page.getByRole('button', { name: '重试恢复', exact: true }).click();
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
    await expect(page.getByText('恢复已完成', { exact: true })).toBeVisible();
    expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-99999'))).toBeNull();
    expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
  });
}

for (const acknowledgement of ['begin', 'complete'] as const) {
  test(`cleanup keeps learning gated on native ${acknowledgement} rejection and supports retry`, async ({ page }) => {
    await nativeFixture(page, { startup_status: [{ value: cleanup }],
      backup_restore_cleanup_begin: acknowledgement === 'begin' ? [{ error: 'Native cleanup begin failed' }, { value: cleanup }] : [{ value: cleanup }],
      backup_restore_cleanup_complete: acknowledgement === 'complete' ? [{ error: 'Native cleanup complete failed' }, { value: completed }] : [{ value: completed }] });
    await page.addInitScript(() => localStorage.setItem('lexique-pending-undo', 'old undo'));
    await page.goto('/');
    await expect(page.getByRole('alert')).toContainText(`Native cleanup ${acknowledgement} failed`);
    expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
    await expect(page.getByText('恢复已完成', { exact: true })).toHaveCount(0);
    if (acknowledgement === 'begin') {
      expect(await nativeCalls(page, 'backup_restore_cleanup_complete')).toEqual([]);
      expect(await page.evaluate(() => localStorage.getItem('lexique-pending-undo'))).toBe('old undo');
    }
    await page.getByRole('button', { name: '重试恢复', exact: true }).click();
    await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
    await expect(page.getByText('恢复已完成', { exact: true })).toBeVisible();
  });
}
