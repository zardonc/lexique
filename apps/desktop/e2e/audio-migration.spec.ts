import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('audio preservation progress leaves the dictionary usable and exposes failure', async ({ page }) => {
  const status = { version: '0.1.0', schemaVersion: 17, storage: 'sqlite' };
  await nativeFixture(page, { runtime_status: [
    { value: { ...status, audioMigration: { pending: 32, failed: 0 } } },
    { value: { ...status, audioMigration: { pending: 32, failed: 0 } } },
    { value: { ...status, audioMigration: { pending: 0, failed: 1 } } },
    { value: { ...status, audioMigration: { pending: 0, failed: 0 } } },
  ] });
  await page.goto('/');
  await expect(page.getByText('正在保留已有单词音频，可以继续学习。')).toBeVisible();
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.getByText('部分已有音频未能保留，请打开生词重试。')).toBeVisible();
  await page.evaluate(() => window.testNative.emit('lexique-review-changed'));
  await expect(page.getByText('本地存储已就绪。')).toBeVisible();
});

test('audio storage reports protected and unknown content with incomplete measurements visible', async ({ page }) => {
  await nativeFixture(page, { runtime_status: [{ value: { version: '0.1.0', schemaVersion: 17, storage: 'sqlite',
    audioMigration: { pending: 0, failed: 0 }, audioCache: {
      logical_bytes: 1_048_576, permanent_bytes: 524_288, protected_bytes: 262_144, reclaimable_bytes: 262_144,
      publication_bytes: 0, unowned_bytes: 524_288, marked_bytes: 0, over_capacity: true, gc_failures: 1,
      file_bytes: 1_048_576, allocated_bytes: null, unknown_files: 2, unknown_bytes: 4096,
      unsafe_files: 0, scan_failures: 1, scanned_entries: 200, in_progress: false, last_reconciled_at: 1000,
      error: 'Speech cache allocation query failed',
    },
  } }] });
  await page.goto('/');
  await page.getByText('音频占用', { exact: true }).click();
  await expect(page.getByText('生词选定音频', { exact: true })).toBeVisible();
  await expect(page.getByText('来源未知的文件（保留）', { exact: true })).toBeVisible();
  await expect(page.getByText('未能测量', { exact: true })).toBeVisible();
  await expect(page.getByText('Speech cache allocation query failed', { exact: true })).toBeVisible();
  await expect(page.getByText('未归属音频超过 200 MiB 软目标，选定音频及保留期继续受保护。', { exact: true })).toBeVisible();
});
