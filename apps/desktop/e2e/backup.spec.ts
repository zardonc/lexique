import { expect, test } from '@playwright/test';
import { collect, nativeFixture, record } from './native-fixture';

const completed = {
  operation: 'backup-1', state: 'completed', phase: null, cancel_requested: false, error: null,
  result: { destination: 'C:\\Backups\\study.lexique-backup', byte_length: 48000, created_at: 1794528000,
    counts: { vocabulary: 4, cards: 8, ratings: 12, batches: 2, custom_dictionaries: 1, legacy_audio: 1,
      included_audio_versions: 3, included_legacy_audio: 1, audio_files: 3, source_missing_audio: 1, source_invalid_audio: 2 } },
};

const collecting = { operation: 'backup-1', state: 'running', phase: 'collecting', cancel_requested: false, result: null, error: null };

const ratingResult = { status: 'committed', result: {
  card: { phase: 'review', memory: { stability: 7, difficulty: 5 }, last_review_at: 1000, due_at: 87400 },
  elapsed_days: 0, step_seconds: null, interval_days: 1, model: { version: 'test-model', parameters: [] }, retention_percent: 90, timezone_database: '2026b',
} };
const pendingRating = { operation: 'pending-rating-88', card_id: 88, expected_revision: 2, rating: 'good', timezone: 'UTC' };
const pendingUndo = { operation: 'pending-undo', ratingOperation: 'earlier-rating' };
const pendingBatch = { operation: 'pending-batch', vocabulary_ids: [10, 20, 30] };
const cancelledBatch = { operation: 'pending-batch', cancelled: true,
  items: [{ vocabulary_id: 10, created_cards: 2, error: null, audio_state: 'cancelled', audio_error: null }] };

test('backup resolves all pending card keys and keeps a confirmed cancelled partial batch', async ({ page }) => {
  await nativeFixture(page, { review_rate: [{ value: ratingResult }], review_batch_status: [{ value: cancelledBatch }],
    backup_export_start: [{ gate: 'backup-start', value: completed }] });
  await page.addInitScript(({ rating, undo, batch }) => {
    localStorage.setItem('lexique-pending-rating-88', JSON.stringify(rating));
    localStorage.setItem('lexique-pending-rating-99', JSON.stringify({ ...rating, operation: 'pending-rating-99', card_id: 99 }));
    localStorage.setItem('lexique-pending-undo', JSON.stringify(undo));
    localStorage.setItem('lexique-pending-review-batch', JSON.stringify(batch));
    localStorage.setItem('unrelated-preference', 'keep');
  }, { rating: pendingRating, undo: pendingUndo, batch: pendingBatch });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(true);
  expect(await page.evaluate(() => ['lexique-pending-rating-88', 'lexique-pending-rating-99', 'lexique-pending-undo', 'lexique-pending-review-batch'].map((key) => localStorage.getItem(key)))).toEqual([null, null, null, null]);
  expect(await page.evaluate(() => localStorage.getItem('unrelated-preference'))).toBe('keep');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate').map((call) => call.args?.request))).toEqual([pendingRating, { ...pendingRating, operation: 'pending-rating-99', card_id: 99 }]);
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_add_batch'))).toBe(false);
  await expect(page.getByRole('region', { name: '批量任务', exact: true })).toContainText('已停止剩余生成');
  await expect(page.getByRole('region', { name: '批量任务', exact: true }).getByRole('button', { name: '重试', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('backup-start'));
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
});

test('cancellation waits for native confirmation before restoring normal controls', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: collecting }], backup_operation_status: [{ value: collecting }],
    backup_operation_cancel: [{ value: { ...collecting, cancel_requested: true } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await panel.getByRole('button', { name: '取消备份', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('已请求取消');
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeDisabled();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), { ...collecting, state: 'cancelled', phase: null, cancel_requested: true });
  await expect(panel.getByRole('status')).toContainText('备份已取消');
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeEnabled();
  await expect(panel).not.toContainText('备份已保存');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_operation_cancel')?.args)).toEqual({ operation: 'backup-1' });
});

test('collection pauses normal controls and packaging resumes them while keeping the final result', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: collecting }], backup_operation_status: [{ value: collecting }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeDisabled();
  await expect(page.getByRole('button', { name: '词典', exact: true })).toBeDisabled();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), { ...collecting, phase: 'packaging' });
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), completed);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('C:\\Backups\\study.lexique-backup');
});

test('manual backup displays the native file result without passing archive bytes or paths', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: completed }], backup_operation_status: [{ value: completed }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await expect(panel.getByRole('status')).toContainText('备份已保存');
  await expect(panel).toContainText('C:\\Backups\\study.lexique-backup');
  await expect(panel).toContainText('48,000 字节');
  await expect(panel).toContainText('原音频缺失：1');
  await expect(panel).toContainText('原音频损坏：2');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_export_start')?.args)).toEqual({});
});

test('completion before the start reply is recovered from native status', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ gate: 'start-reply', value: collecting }], backup_operation_status: [{ value: completed }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-progress', value); window.testNative.release('start-reply'); }, completed);
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_operation_status')?.args)).toEqual({ operation: 'backup-1' });
});

test('a late status reply cannot replace a completed event', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: collecting }], backup_operation_status: [{ gate: 'status-reply', value: collecting }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_operation_status'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-progress', value); window.testNative.release('status-reply'); }, completed);
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await expect(panel.getByRole('status')).toContainText('备份已保存');
  await expect(panel.getByRole('button', { name: '创建备份', exact: true })).toBeEnabled();
});

test('an older collecting event cannot pause controls after packaging was observed', async ({ page }) => {
  const packaging = { ...collecting, phase: 'packaging' };
  await nativeFixture(page, { backup_export_start: [{ value: packaging }], backup_operation_status: [{ value: packaging }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('status')).toContainText('正在打包备份');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), collecting);
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeEnabled();
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('status')).toContainText('正在打包备份');
});

test('an older event cannot clear a confirmed cancellation request', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: collecting }], backup_operation_status: [{ value: collecting }],
    backup_operation_cancel: [{ value: { ...collecting, cancel_requested: true } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await panel.getByRole('button', { name: '取消备份', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('已请求取消');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), collecting);
  await expect(panel.getByRole('button', { name: '取消备份', exact: true })).toBeDisabled();
  await expect(panel.getByRole('status')).toContainText('已请求取消');
});

test('a completed publication stays completed when a cancellation reply arrives later', async ({ page }) => {
  const packaging = { ...collecting, phase: 'packaging' };
  await nativeFixture(page, { backup_export_start: [{ value: packaging }], backup_operation_status: [{ value: packaging }],
    backup_operation_cancel: [{ gate: 'cancel-reply', value: { ...packaging, cancel_requested: true } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await page.getByRole('button', { name: '取消备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_operation_cancel'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-progress', value); window.testNative.release('cancel-reply'); }, completed);
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await expect(panel).toContainText('备份已保存');
  await expect(panel).not.toContainText('备份已取消');
});

test('unrelated events are ignored and an invalid event can be recovered with a status check', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: collecting }], backup_operation_status: [{ value: collecting }, { value: completed }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  const panel = page.getByRole('region', { name: '手动备份', exact: true });
  await expect(panel.getByRole('progressbar')).toBeVisible();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), { ...completed, operation: 'another-backup' });
  await expect(panel).not.toContainText('备份已保存');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), { ...completed, result: null });
  await expect(panel.getByRole('alert')).toContainText('备份失败');
  await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeDisabled();
  await panel.getByRole('button', { name: '检查备份状态', exact: true }).click();
  await expect(panel).toContainText('备份已保存');
  await expect(panel.getByRole('alert')).toHaveCount(0);
});

test('repeated creation clicks keep one native operation active', async ({ page }) => {
  const choosing = { ...collecting, state: 'choosing', phase: null };
  await nativeFixture(page, { backup_export_start: [{ value: choosing }], backup_operation_status: [{ value: choosing }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).evaluate((button: HTMLButtonElement) => { button.click(); button.click(); });
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('status')).toContainText('请选择新备份文件的位置');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'backup_export_start').length)).toBe(1);
  await expect(page.getByRole('button', { name: '创建备份', exact: true })).toBeDisabled();
});

for (const item of [
  { name: 'rating', key: 'lexique-pending-rating-88', request: pendingRating, command: 'review_rate' },
  { name: 'undo', key: 'lexique-pending-undo', request: pendingUndo, command: 'review_undo' },
  { name: 'batch status', key: 'lexique-pending-review-batch', request: pendingBatch, command: 'review_batch_status' },
]) {
  test(`failed pending ${item.name} stays stored and prevents export`, async ({ page }) => {
    await nativeFixture(page, { [item.command]: [{ error: 'Pending result is unavailable' }], backup_export_start: [{ value: completed }] });
    await page.addInitScript(({ key, request }) => localStorage.setItem(key, JSON.stringify(request)), item);
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    await page.getByRole('button', { name: '创建备份', exact: true }).click();
    await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('Pending result is unavailable');
    expect(await page.evaluate((key) => localStorage.getItem(key), item.key)).toBe(JSON.stringify(item.request));
    expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
    await expect(page.getByRole('button', { name: '保存复习设置', exact: true })).toBeEnabled();
  });
}

test('retry uses the original pending rating after a failed response', async ({ page }) => {
  await nativeFixture(page, { review_rate: [{ error: 'Reply lost' }, { value: ratingResult }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)), pendingRating);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const create = page.getByRole('button', { name: '创建备份', exact: true });
  await create.click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('Reply lost');
  await create.focus(); await create.press('Enter');
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBeNull();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_rate').map((call) => call.args?.request))).toEqual([pendingRating, pendingRating]);
});

for (const [name, key, saved] of [
  ['mismatched card', 'lexique-pending-rating-99', JSON.stringify(pendingRating)],
  ['unreadable rating', 'lexique-pending-rating-88', '{'],
  ['invalid undo', 'lexique-pending-undo', JSON.stringify({ ...pendingUndo, extra: true })],
] as const) {
  test(`invalid pending ${name} is retained before any export`, async ({ page }) => {
    await nativeFixture(page, { backup_export_start: [{ value: completed }] });
    await page.addInitScript(({ key, saved }) => localStorage.setItem(key, saved), { key, saved });
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    await page.getByRole('button', { name: '创建备份', exact: true }).click();
    await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('备份失败');
    expect(await page.evaluate((key) => localStorage.getItem(key), key)).toBe(saved);
    expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
  });
}

test('an incomplete unconfirmed batch is retried with its original request', async ({ page }) => {
  await nativeFixture(page, { review_batch_status: [{ value: { ...cancelledBatch, cancelled: false } }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-review-batch', JSON.stringify(request)), pendingBatch);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'review_add_batch')?.args?.request)).toEqual(pendingBatch);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-review-batch'))).toBeNull();
});

test('concurrent batch cancellation confirms only its returned partial items', async ({ page }) => {
  await nativeFixture(page, { review_add_batch: [{ value: { ...cancelledBatch, operation: 'fixture' } }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-review-batch', JSON.stringify(request)), pendingBatch);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-review-batch'))).toBeNull();
  await expect(page.locator('.page-content')).toContainText('已创建卡片: 2');
});

for (const [name, value] of [
  ['completed without result', { ...completed, result: null }],
  ['unsafe size', { ...completed, result: { ...completed.result, byte_length: 9007199254740992 } }],
  ['negative count', { ...completed, result: { ...completed.result, counts: { ...completed.result.counts, cards: -1 } } }],
  ['unknown field', { ...completed, unexpected: true }],
  ['incomplete counts', { ...completed, result: { ...completed.result, counts: { vocabulary: 0 } } }],
  ['running without phase', { ...collecting, phase: null }],
  ['failed without error', { ...collecting, state: 'failed', phase: null, error: '' }],
  ['cancelled with result', { ...completed, state: 'cancelled' }],
] as const) {
  test(`malformed backup response ${name} is not reported as saved`, async ({ page }) => {
    await nativeFixture(page, { backup_export_start: [{ value }] });
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    await page.getByRole('button', { name: '创建备份', exact: true }).click();
    const panel = page.getByRole('region', { name: '手动备份', exact: true });
    await expect(panel.getByRole('alert')).toContainText('备份失败');
    await expect(panel).not.toContainText('备份已保存');
    await expect(panel.getByRole('button', { name: '创建备份', exact: true })).toBeEnabled();
  });
}

for (const item of [
  { locale: 'zh', settings: '设置', title: '手动备份', create: '创建备份', progress: '正在等待已确认操作完成…', error: '备份失败' },
  { locale: 'en', settings: 'Settings', title: 'Manual backup', create: 'Create backup', progress: 'Waiting for accepted operations to finish…', error: 'Backup failed' },
  { locale: 'fr', settings: 'Réglages', title: 'Sauvegarde manuelle', create: 'Créer une sauvegarde', progress: 'Attente de la fin des opérations acceptées…', error: 'Échec de la sauvegarde' },
]) {
  test(`${item.locale} backup progress and failures are accessible by keyboard`, async ({ page }) => {
    const draining = { ...collecting, phase: 'draining' };
    await nativeFixture(page, { interface_preferences: [{ value: { locale: item.locale, theme: 'white' } }], backup_export_start: [{ value: draining }], backup_operation_status: [{ value: draining }] });
    await page.goto('/');
    const settings = page.getByRole('button', { name: item.settings, exact: true });
    await settings.focus(); await settings.press('Enter');
    const panel = page.getByRole('region', { name: item.title, exact: true }), create = panel.getByRole('button', { name: item.create, exact: true });
    await create.focus(); await create.press('Enter');
    await expect(panel.getByRole('progressbar', { name: item.progress, exact: true })).toBeVisible();
    await page.evaluate((value) => window.testNative.emit('lexique-backup-progress', value), { ...collecting, state: 'failed', phase: null, error: 'Destination is full' });
    await expect(panel.getByRole('alert')).toContainText(`${item.error}: Destination is full`);
    await expect(create).toBeEnabled();
  });
}

test('browser preview explains desktop-only backups in every locale', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  for (const [locale, title, create, preview] of [
    ['zh', '手动备份', '创建备份', '手动备份仅在桌面应用中可用。'],
    ['en', 'Manual backup', 'Create backup', 'Manual backup is available in the desktop app only.'],
    ['fr', 'Sauvegarde manuelle', 'Créer une sauvegarde', 'La sauvegarde manuelle est disponible uniquement dans l’application de bureau.'],
  ]) {
    await page.getByRole('banner').getByRole('combobox').first().selectOption(locale);
    const panel = page.getByRole('region', { name: title, exact: true });
    await expect(panel).toContainText(preview);
    await expect(panel.getByRole('button', { name: create, exact: true })).toBeDisabled();
  }
});

test('failed draft saves keep the editor until the user saves before backup', async ({ page }) => {
  await nativeFixture(page, { vocabulary_save: [{ error: 'Draft disk is unavailable' }, { value: { outcome: 'created', record: { ...record, notes: 'My pending note' } } }], backup_export_start: [{ value: completed }] });
  await collect(page);
  await page.getByLabel('独立笔记').fill('My pending note');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await dialog.getByRole('button', { name: '保存后离开', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('Draft disk is unavailable');
  await expect(page.getByLabel('独立笔记')).toHaveValue('My pending note');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
  await dialog.getByRole('button', { name: '保存后离开', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
});

test('keeping a draft delays backup until the user discards it explicitly', async ({ page }) => {
  await nativeFixture(page, { backup_export_start: [{ value: completed }] });
  await collect(page);
  await page.getByLabel('独立笔记').fill('Keep this draft');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await dialog.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记')).toHaveValue('Keep this draft');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await dialog.getByRole('button', { name: '放弃修改', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'vocabulary_save'))).toBe(false);
});

test('a rejected rating remains pending until its outcome can be resolved', async ({ page }) => {
  await nativeFixture(page, { review_rate: [{ value: { status: 'rejected', code: 'conflict', detail: 'Review revision changed' } }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)), pendingRating);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('Review revision changed');
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe(JSON.stringify(pendingRating));
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
});

test('a pending request replaced during confirmation is kept and stops export', async ({ page }) => {
  await nativeFixture(page, { review_rate: [{ gate: 'rating-reply', value: ratingResult }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)), pendingRating);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_rate'))).toBe(true);
  const replacement = { ...pendingRating, operation: 'replacement-rating' };
  await page.evaluate((request) => { localStorage.setItem('lexique-pending-rating-88', JSON.stringify(request)); window.testNative.release('rating-reply'); }, replacement);
  await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('Pending review request changed');
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-rating-88'))).toBe(JSON.stringify(replacement));
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
});

for (const [name, value] of [
  ['another operation', { ...cancelledBatch, operation: 'another-batch' }],
  ['unrequested item', { ...cancelledBatch, items: [{ ...cancelledBatch.items[0], vocabulary_id: 99 }] }],
  ['duplicate items', { ...cancelledBatch, items: [cancelledBatch.items[0], cancelledBatch.items[0]] }],
] as const) {
  test(`pending batch status with ${name} does not clear the request`, async ({ page }) => {
    await nativeFixture(page, { review_batch_status: [{ value }], backup_export_start: [{ value: completed }] });
    await page.addInitScript((request) => localStorage.setItem('lexique-pending-review-batch', JSON.stringify(request)), pendingBatch);
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    await page.getByRole('button', { name: '创建备份', exact: true }).click();
    await expect(page.getByRole('region', { name: '手动备份', exact: true }).getByRole('alert')).toContainText('response identity');
    expect(await page.evaluate(() => localStorage.getItem('lexique-pending-review-batch'))).toBe(JSON.stringify(pendingBatch));
    expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(false);
  });
}

test('a complete batch preserves confirmed failed items without replaying it', async ({ page }) => {
  const batch = { operation: 'pending-batch', cancelled: false, items: [cancelledBatch.items[0],
    { vocabulary_id: 20, created_cards: 0, error: 'Unknown vocabulary item', audio_state: null, audio_error: null },
    { vocabulary_id: 30, created_cards: 0, error: null, audio_state: 'ready', audio_error: null }] };
  await nativeFixture(page, { review_batch_status: [{ value: batch }], backup_export_start: [{ value: completed }] });
  await page.addInitScript((request) => localStorage.setItem('lexique-pending-review-batch', JSON.stringify(request)), pendingBatch);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect(page.getByRole('region', { name: '手动备份', exact: true })).toContainText('备份已保存');
  await expect(page.getByRole('region', { name: '批量任务', exact: true })).toContainText('#20: Unknown vocabulary item');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_add_batch'))).toBe(false);
  expect(await page.evaluate(() => localStorage.getItem('lexique-pending-review-batch'))).toBeNull();
});
