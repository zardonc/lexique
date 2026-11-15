import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { nativeFixture } from './native-fixture';
import { cleanup, completed, confirmRestore, nativeCalls, preserving, previewReady, ready, restarting } from './backup-restore-fixture';

const failedStartup = { ...ready, state: 'recovery_required', error: 'Learning initialization failed: file is not a database' };
const rawRequests = [
  { key: 'lexique-pending-rating-not-a-card', value: '{malformed' },
  { key: 'lexique-pending-undo', value: '' },
  { key: 'lexique-pending-review-batch', value: 'null' },
  { key: 'lexique-pending-rating-123', value: ' \n原始请求🙂\n ' },
];
const rawCopies = [{ id: 'recovery-raw-1', kind: 'raw_files', path: 'C:\\Lexique-data\\data\\retained-restores\\preview-1\\raw-files', byte_length: 12500 }];

async function seedRawRequests(page: Page) {
  await page.addInitScript((requests) => {
    for (const request of requests) localStorage.setItem(request.key, request.value);
    localStorage.setItem('reader-preference', 'keep');
  }, rawRequests);
}

async function storedRawRequests(page: Page) {
  return page.evaluate((requests) => requests.map(({ key }) => ({ key, value: localStorage.getItem(key) })), rawRequests);
}

test('independent recovery restores exact raw pending strings without opening learning or exporting', async ({ page }, testInfo) => {
  await nativeFixture(page, { startup_status: [{ value: failedStartup }], backup_preview_start: [{ value: previewReady }],
    backup_preview_status: [{ value: previewReady }], backup_restore_start: [{ value: preserving }], backup_restore_status: [{ value: preserving }] });
  await seedRawRequests(page);
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '需要恢复学习数据', exact: true })).toBeVisible();
  await expect(page.getByRole('alert')).toContainText('file is not a database');
  await expect(page.getByRole('button', { name: '创建备份', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await page.getByRole('button', { name: '恢复整份备份', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '确认恢复整份备份', exact: true });
  await expect(dialog).toContainText('不解析或提交它们');
  await expect(dialog).toContainText('本机独立配置');
  await dialog.getByRole('checkbox').check();
  await page.keyboard.press('Escape');
  await expect(dialog).not.toBeVisible();
  expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
  await page.getByRole('button', { name: '恢复整份备份', exact: true }).click();
  await expect(dialog.getByRole('checkbox')).not.toBeChecked();
  await page.screenshot({ path: testInfo.outputPath('recovery-confirmation.png'), fullPage: true });
  await dialog.getByRole('checkbox').focus();
  await page.keyboard.press('Space');
  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: '确认并恢复', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect.poll(() => nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
  const calls = await nativeCalls(page, 'backup_restore_start');
  expect(calls[0].args?.operation).toBe('preview-1');
  expect((calls[0].args?.pendingRequests as typeof rawRequests).sort((a, b) => a.key.localeCompare(b.key)))
    .toEqual([...rawRequests].sort((a, b) => a.key.localeCompare(b.key)));
  for (const command of ['runtime_status', 'vocabulary_list', 'review_rate', 'review_undo', 'review_add_batch', 'review_batch_status', 'backup_start']) {
    expect(await nativeCalls(page, command)).toEqual([]);
  }
  expect(await storedRawRequests(page)).toEqual(rawRequests);
  expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
});

declare global { interface Window { repairRecoveryStorage: () => void } }

async function failRawStorage(page: Page, fault: string) {
  await seedRawRequests(page);
  await page.addInitScript((fault) => {
    let active = true, reads = 0;
    const key = Storage.prototype.key, getItem = Storage.prototype.getItem;
    const length = Object.getOwnPropertyDescriptor(Storage.prototype, 'length')!;
    Storage.prototype.key = function (index) {
      if (active && fault === 'enumeration') throw new Error('Recovery storage enumeration denied');
      return key.call(this, index);
    };
    Object.defineProperty(Storage.prototype, 'length', { ...length, get() {
      if (active && fault === 'length') throw new Error('Recovery storage length denied');
      return length.get!.call(this);
    } });
    Storage.prototype.getItem = function (name) {
      if (active && name === 'lexique-pending-undo') {
        if (fault === 'read') throw new Error('Recovery storage read denied');
        if (fault === 'disappearing') return null;
        if (fault === 'changed' && ++reads > 1) return 'changed between snapshot and confirmation';
      }
      return getItem.call(this, name);
    };
    window.repairRecoveryStorage = () => { active = false; };
  }, fault);
}

for (const fault of ['enumeration', 'length', 'read', 'disappearing', 'changed']) {
  test(`recovery blocks restore when raw pending storage ${fault} fails and retries without replay`, async ({ page }) => {
    await nativeFixture(page, { startup_status: [{ value: failedStartup }], backup_preview_start: [{ value: previewReady }],
      backup_preview_status: [{ value: previewReady }], backup_restore_start: [{ value: preserving }], backup_restore_status: [{ value: preserving }] });
    await failRawStorage(page, fault);
    await page.goto('/');
    await page.getByRole('button', { name: '选择备份', exact: true }).click();
    await confirmRestore(page);
    await expect(page.getByRole('alert').filter({ hasText: '恢复前确认失败' })).toBeVisible();
    expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
    for (const command of ['review_rate', 'review_undo', 'review_batch_status', 'review_add_batch', 'runtime_status']) expect(await nativeCalls(page, command)).toEqual([]);
    await page.evaluate(() => window.repairRecoveryStorage());
    expect(await storedRawRequests(page)).toEqual(rawRequests);
    await confirmRestore(page);
    await expect.poll(() => nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
    expect(await storedRawRequests(page)).toEqual(rawRequests);
    expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
  });
}

test('failed raw preservation shows its location and allows a newly verified recovery attempt', async ({ page }) => {
  const failure = { ...preserving, state: 'failed', phase: null, error: 'Raw-file preservation failed: source sharing violation', retained_copies: rawCopies };
  const nextPreview = { ...previewReady, operation: 'preview-2' };
  const nextRestore = { ...preserving, operation: 'preview-2' };
  await nativeFixture(page, { startup_status: [{ value: failedStartup }],
    backup_preview_start: [{ value: previewReady }, { value: nextPreview }], backup_preview_status: [{ value: previewReady }, { value: nextPreview }],
    backup_restore_start: [{ value: failure }, { value: nextRestore }], backup_restore_status: [{ value: nextRestore }] });
  await seedRawRequests(page);
  await page.goto('/');
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await confirmRestore(page);
  await expect(page.getByRole('alert')).toContainText('source sharing violation');
  const copies = page.getByRole('region', { name: '保留的副本', exact: true });
  await expect(copies).toContainText('原始文件保留副本');
  await expect(copies).toContainText(rawCopies[0].path);
  await expect(copies).toContainText('12,500 字节');
  await expect(copies).toContainText('不能通过备份选择器导入');
  expect(await storedRawRequests(page)).toEqual(rawRequests);
  await page.getByRole('button', { name: '返回恢复入口', exact: true }).click();
  await expect(page.getByRole('heading', { name: '需要恢复学习数据', exact: true })).toBeVisible();
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await confirmRestore(page);
  await expect.poll(() => nativeCalls(page, 'backup_restore_start')).toHaveLength(2);
  expect((await nativeCalls(page, 'backup_restore_start')).map((call) => call.args?.operation)).toEqual(['preview-1', 'preview-2']);
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
  expect(await storedRawRequests(page)).toEqual(rawRequests);
});

test('raw recovery retains exact requests through both cleanup acknowledgements and keeps unrelated storage', async ({ page }, testInfo) => {
  const rawRestarting = { ...restarting, retained_copies: rawCopies };
  const rawCleanup = { ...cleanup, retained_copies: rawCopies }, rawCompleted = { ...completed, retained_copies: rawCopies };
  await nativeFixture(page, { startup_status: [{ value: failedStartup }], backup_preview_start: [{ value: previewReady }], backup_preview_status: [{ value: previewReady }],
    backup_restore_start: [{ value: preserving }], backup_restore_status: [{ value: preserving }],
    backup_restore_cleanup_begin: [{ value: rawCleanup, gate: 'raw-cleanup-begin' }],
    backup_restore_cleanup_complete: [{ value: rawCompleted, gate: 'raw-cleanup-complete' }] });
  await seedRawRequests(page);
  await page.goto('/');
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await confirmRestore(page);
  await expect.poll(() => nativeCalls(page, 'backup_restore_status')).toHaveLength(1);
  await expect(page.getByRole('status')).toContainText('正在保留原数据库');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-restore-progress', value), rawRestarting);
  await expect(page.getByRole('status')).toContainText('已请求重启');
  await page.screenshot({ path: testInfo.outputPath('raw-retained-restart.png'), fullPage: true });
  await page.getByRole('button', { name: '打开保留目录', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'backup_retained_directory_open')).toHaveLength(1);
  expect((await nativeCalls(page, 'backup_retained_directory_open'))[0].args).toEqual({ copy: rawCopies[0].id });
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), rawCleanup);
  await expect.poll(() => nativeCalls(page, 'backup_restore_cleanup_begin')).toHaveLength(1);
  expect(await storedRawRequests(page)).toEqual(rawRequests);
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
  await page.evaluate(() => window.testNative.release('raw-cleanup-begin'));
  await expect.poll(() => nativeCalls(page, 'backup_restore_cleanup_complete')).toHaveLength(1);
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('lexique-pending-')))).toEqual([]);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), rawCompleted);
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
  await page.evaluate(() => window.testNative.release('raw-cleanup-complete'));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
  for (const command of ['review_rate', 'review_undo', 'review_add_batch']) expect(await nativeCalls(page, command)).toEqual([]);
  expect((await nativeCalls(page, 'review_batch_status')).filter((call) => call.args?.operation !== null)).toEqual([]);
});

test('a new raw recovery owns startup notifications while stale previous recovery notifications are ignored', async ({ page }, testInfo) => {
  const previous = { ...failedStartup, operation: 'previous-raw' };
  await nativeFixture(page, { startup_status: [{ value: previous }], backup_preview_start: [{ value: previewReady }], backup_preview_status: [{ value: previewReady }],
    backup_restore_start: [{ value: preserving }], backup_restore_status: [{ value: preserving }],
    backup_restore_cleanup_begin: [{ value: cleanup }], backup_restore_cleanup_complete: [{ value: completed }] });
  await seedRawRequests(page);
  await page.goto('/');
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await confirmRestore(page);
  await expect.poll(() => nativeCalls(page, 'backup_restore_status')).toHaveLength(1);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), { ...cleanup, operation: 'previous-raw' });
  await page.screenshot({ path: testInfo.outputPath('raw-identity-before-cleanup.png') });
  expect(await nativeCalls(page, 'backup_restore_cleanup_begin')).toEqual([]);
  expect(await storedRawRequests(page)).toEqual(rawRequests);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), cleanup);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  expect((await nativeCalls(page, 'backup_restore_cleanup_begin')).map((call) => call.args?.operation)).toEqual(['preview-1']);
  await page.evaluate((value) => window.testNative.emit('lexique-startup-status', value), previous);
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
});

test('recovery preview keeps learning unavailable, gates retry and reports native close failures', async ({ page }) => {
  const running = { ...previewReady, state: 'running', phase: 'validating', preview: null };
  const cancelled = { ...running, state: 'cancelled', phase: null, cancel_requested: true };
  await nativeFixture(page, { startup_status: [{ value: failedStartup }], backup_preview_start: [{ value: running }], backup_preview_status: [{ value: running }],
    backup_preview_cancel: [{ value: cancelled }], finish_close: [{ error: 'backup_busy: Finish or cancel the backup before closing' }, { value: null }] });
  await page.goto('/');
  await page.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('正在检查备份内容');
  await expect(page.getByRole('status')).not.toContainText('可继续学习');
  await expect(page.getByRole('button', { name: '重试恢复', exact: true })).toBeDisabled();
  await page.evaluate(() => window.testNative.close());
  await expect(page.getByRole('alert').filter({ hasText: 'backup_busy' })).toBeVisible();
  await page.getByRole('button', { name: '取消检查', exact: true }).click();
  await expect(page.getByRole('button', { name: '重试恢复', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '关闭应用', exact: true }).click();
  await expect.poll(() => nativeCalls(page, 'finish_close')).toHaveLength(2);
  expect(await nativeCalls(page, 'runtime_status')).toEqual([]);
});
