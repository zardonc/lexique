import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const ready = {
  operation: 'preview-1', state: 'preview_ready', phase: null, cancel_requested: false, error: null,
  preview: { filename: 'études.lexique-backup', byte_length: 48000, created_at: 1794528000, application_version: '0.1.0', requires_restart: true,
    counts: { vocabulary: 4, cards: 8, ratings: 12, batches: 2, custom_dictionaries: 1, legacy_audio: 1,
      included_audio_versions: 3, included_legacy_audio: 1, audio_files: 3, source_missing_audio: 1, source_invalid_audio: 2 },
    pending_dictionaries: [{ id: 'mdx-7', title: '法语词典.mdx' }] },
};
const validating = { operation: 'preview-1', state: 'running', phase: 'validating', cancel_requested: false, preview: null, error: null };

test('preview displays fully verified backup facts and resource needs without replacing learning data', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: ready }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel).toContainText('études.lexique-backup');
  await expect(panel).toContainText('48,000 字节');
  await expect(panel).toContainText('2026/11/13');
  await expect(panel).toContainText('0.1.0');
  await expect(panel).toContainText('生词：4');
  await expect(panel).toContainText('卡片：8');
  await expect(panel).toContainText('评分：12');
  await expect(panel).toContainText('批量请求：2');
  await expect(panel).toContainText('自定义词典：1');
  await expect(panel).toContainText('旧音频关联：1');
  await expect(panel).toContainText('已包含音频版本：3');
  await expect(panel).toContainText('已包含旧音频：1');
  await expect(panel).toContainText('音频文件：3');
  await expect(panel).toContainText('原音频缺失：1');
  await expect(panel).toContainText('原音频损坏：2');
  await expect(panel).toContainText('法语词典.mdx');
  await expect(panel).toContainText('恢复需要重启');
  await expect(panel).toContainText('词典文件和模型需在目标设备另行提供');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command.startsWith('backup_')));
  expect(calls.map((call) => ({ command: call.command, args: call.args }))).toEqual([{ command: 'backup_preview_start', args: {} }]);
});

test('preview recovers completion before the start reply from native operation status', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ gate: 'preview-start', value: validating }], backup_preview_status: [{ value: ready }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_start'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-preview-progress', value); window.testNative.release('preview-start'); }, ready);
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_preview_status')?.args)).toEqual({ operation: 'preview-1' });
});

test('confirmed preview cancellation survives racing ready replies and permits a new selection', async ({ page }) => {
  const cancelled = { ...validating, state: 'cancelled', phase: null, cancel_requested: true };
  const second = { ...ready, operation: 'preview-2', preview: { ...ready.preview, filename: 'nouvelle.lexique-backup' } };
  await nativeFixture(page, { backup_preview_start: [{ value: validating }, { value: second }],
    backup_preview_status: [{ gate: 'first-status', value: ready }], backup_preview_cancel: [{ gate: 'cancel-preview', value: cancelled }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_status'))).toBe(true);
  await panel.getByRole('button', { name: '取消检查', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_cancel'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-preview-progress', value); window.testNative.release('cancel-preview'); }, ready);
  await expect(panel.getByRole('status')).toHaveText('备份检查已取消。');
  await expect(panel.getByRole('button', { name: '选择备份', exact: true })).toBeEnabled();
  await page.evaluate((value) => { window.testNative.release('first-status'); window.testNative.emit('lexique-backup-preview-progress', value); }, ready);
  await expect(panel.getByRole('status')).toHaveText('备份检查已取消。');
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel).toContainText('nouvelle.lexique-backup');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), cancelled);
  await expect(panel).not.toContainText('études.lexique-backup');
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'backup_preview_cancel')?.args)).toEqual({ operation: 'preview-1' });
});

test('preview progress stays mounted during learning and does not move back from staging', async ({ page }) => {
  const staging = { ...validating, phase: 'staging' };
  await nativeFixture(page, { backup_preview_start: [{ value: staging }], backup_preview_status: [{ gate: 'old-status', value: validating }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('status')).toHaveText('正在准备隔离学习数据和音频，可继续学习。');
  await expect(panel.getByRole('progressbar')).toHaveAccessibleName('正在准备隔离学习数据和音频，可继续学习。');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_status'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-preview-progress', value); window.testNative.release('old-status'); }, validating);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), { ...ready, operation: 'unrelated-preview' });
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await expect(page.getByLabel('法语词或词组')).toBeEnabled();
  await expect(panel).toBeVisible();
  await expect(panel.getByRole('status')).toHaveText('正在准备隔离学习数据和音频，可继续学习。');
  await expect(panel).not.toContainText('études.lexique-backup');
  const input = page.getByLabel('法语词或词组');
  await input.fill('école');
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), ready);
  await expect(input).toBeFocused();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(panel).toContainText('études.lexique-backup');
});

test('export waits for an active preview and clears its ready facts after native export accepts', async ({ page }) => {
  const exported = { operation: 'export-1', state: 'completed', phase: null, cancel_requested: false, error: null,
    result: { destination: 'C:\\Backups\\fresh.lexique-backup', byte_length: 64000, created_at: 1794528000, counts: ready.preview.counts } };
  await nativeFixture(page, { backup_preview_start: [{ value: validating }], backup_preview_status: [{ value: validating }],
    backup_export_start: [{ gate: 'export-start', value: exported }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const preview = page.getByRole('region', { name: '检查备份', exact: true }), backup = page.getByRole('region', { name: '手动备份', exact: true });
  await preview.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(preview.getByRole('status')).toContainText('正在检查备份内容');
  await expect(backup.getByRole('button', { name: '创建备份', exact: true })).toBeDisabled();
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), ready);
  await expect(backup.getByRole('button', { name: '创建备份', exact: true })).toBeEnabled();
  await backup.getByRole('button', { name: '创建备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_export_start'))).toBe(true);
  await expect(preview.getByRole('button', { name: '选择备份', exact: true })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('export-start'));
  await expect(backup.getByRole('status')).toContainText('备份已保存');
  await expect(preview).not.toContainText('études.lexique-backup');
  await expect(preview.getByRole('status')).toHaveCount(0);
});

test('an invalid preview event is announced with focus and can be retried from the keyboard', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: validating }], backup_preview_status: [{ value: validating }, { value: ready }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_status'))).toBe(true);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), { ...validating, unexpected: true });
  const error = panel.getByRole('alert');
  await expect(error).toContainText('检查备份失败');
  await expect(error).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(error.getByRole('button', { name: '重试', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel.getByRole('status')).toBeFocused();
  await expect(error).toHaveCount(0);
});

test('a failed native preview start keeps its cause visible and permits another selection', async ({ page }) => {
  const browserErrors: string[] = [];
  page.on('pageerror', (cause) => browserErrors.push(cause.message));
  await nativeFixture(page, { backup_preview_start: [{ error: 'Cannot stage backup: no space available' }, { value: ready }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('alert')).toContainText('Cannot stage backup: no space available');
  await expect(panel.getByRole('alert')).toBeFocused();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'plugin:event|unlisten' && call.args?.event === 'lexique-backup-preview-progress'))).toBe(true);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), { ...validating, unexpected: true });
  await expect(panel.getByRole('alert')).toContainText('Cannot stage backup: no space available');
  await panel.getByRole('alert').getByRole('button', { name: '重试', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel.getByRole('alert')).toHaveCount(0);
  expect(browserErrors).toEqual([]);
});

test('a confirmed cancelled event permits reselection before a late cancellation error returns', async ({ page }) => {
  const cancelled = { ...validating, state: 'cancelled', phase: null, cancel_requested: true };
  const second = { ...ready, operation: 'preview-2', preview: { ...ready.preview, filename: 'nouvelle.lexique-backup' } };
  await nativeFixture(page, { backup_preview_start: [{ value: validating }, { value: second }], backup_preview_status: [{ value: validating }],
    backup_preview_cancel: [{ gate: 'old-cancellation', error: 'Old cancellation reply unavailable' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await panel.getByRole('button', { name: '取消检查', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_cancel'))).toBe(true);
  await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), cancelled);
  await expect(panel.getByRole('status')).toHaveText('备份检查已取消。');
  await expect(panel.getByRole('status')).toBeFocused();
  await expect(panel.getByRole('button', { name: '选择备份', exact: true })).toBeEnabled();
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel).toContainText('nouvelle.lexique-backup');
  await page.evaluate(() => window.testNative.release('old-cancellation'));
  await expect(panel.getByRole('button', { name: '取消检查', exact: true })).toBeEnabled();
  await expect(panel.getByRole('alert')).toHaveCount(0);
  await expect(panel.getByRole('status')).toContainText('备份已验证');
});

test('preview subscription failure blocks native selection until its keyboard retry succeeds', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: ready }] });
  await page.addInitScript(() => {
    const boundary = (window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>, options?: unknown) => Promise<unknown> } }).__TAURI_INTERNALS__;
    const invoke = boundary.invoke;
    let fail = true;
    boundary.invoke = (command, args, options) => {
      if (fail && command === 'plugin:event|listen' && args?.event === 'lexique-backup-preview-progress') {
        fail = false; return Promise.reject('Preview event channel unavailable');
      }
      return invoke(command, args, options);
    };
  });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('alert')).toContainText('Preview event channel unavailable');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_start'))).toBe(false);
  await panel.getByRole('alert').getByRole('button', { name: '重试', exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(panel.getByRole('status')).toContainText('备份已验证');
});

test('a wrong preview status identity stays visible until a real status retry succeeds', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: validating }],
    backup_preview_status: [{ value: { ...ready, operation: 'wrong-preview' } }, { value: ready }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('alert')).toContainText('response identity differs');
  await expect(panel).not.toContainText('études.lexique-backup');
  await panel.getByRole('alert').getByRole('button', { name: '重试', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel.getByRole('alert')).toHaveCount(0);
});

test('a delayed status failure cannot replace a verified preview event', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: validating }], backup_preview_status: [{ gate: 'late-status-error', error: 'Old status read failed' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'backup_preview_status'))).toBe(true);
  await page.evaluate((value) => { window.testNative.emit('lexique-backup-preview-progress', value); window.testNative.release('late-status-error'); }, ready);
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel.getByRole('status')).toBeFocused();
  await expect(panel.getByRole('alert')).toHaveCount(0);
});

for (const [name, reply] of [
  ['native failure', { error: 'Preview cancellation unavailable' }],
  ['wrong identity', { value: { ...validating, operation: 'wrong-preview', state: 'cancelled', phase: null, cancel_requested: true } }],
  ['unconfirmed cancellation', { value: { ...validating, state: 'cancelled', phase: null, cancel_requested: false } }],
] as const) {
  test(`preview cancellation ${name} preserves the active operation until cancellation is confirmed`, async ({ page }) => {
    const cancelled = { ...validating, state: 'cancelled', phase: null, cancel_requested: true };
    await nativeFixture(page, { backup_preview_start: [{ value: validating }], backup_preview_status: [{ value: validating }], backup_preview_cancel: [reply, { value: cancelled }] });
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    const panel = page.getByRole('region', { name: '检查备份', exact: true });
    await panel.getByRole('button', { name: '选择备份', exact: true }).click();
    await panel.getByRole('button', { name: '取消检查', exact: true }).click();
    await expect(panel.getByRole('alert')).toContainText('检查备份失败');
    await expect(panel.getByRole('button', { name: '选择备份', exact: true })).toBeDisabled();
    await expect(page.getByRole('button', { name: '创建备份', exact: true })).toBeDisabled();
    await panel.getByRole('button', { name: '取消检查', exact: true }).click();
    await expect(panel.getByRole('status')).toHaveText('备份检查已取消。');
    await expect(panel.getByRole('status')).toBeFocused();
    await expect(panel.getByRole('alert')).toHaveCount(0);
  });
}

for (const [name, value] of [
  ['ready without preview', { ...ready, preview: null }],
  ['unknown status field', { ...ready, unexpected: true }],
  ['unknown preview field', { ...ready, preview: { ...ready.preview, staging_path: 'not-a-frontend-capability' } }],
  ['unknown count field', { ...ready, preview: { ...ready.preview, counts: { ...ready.preview.counts, unexpected: 1 } } }],
  ['running without phase', { ...validating, phase: null }],
  ['choosing with phase', { ...validating, state: 'choosing' }],
  ['cancelled with preview', { ...ready, state: 'cancelled' }],
  ['failed with blank error', { ...validating, state: 'failed', phase: null, error: '   ' }],
  ['blank operation', { ...ready, operation: '   ' }],
  ['unsafe size', { ...ready, preview: { ...ready.preview, byte_length: 9007199254740992 } }],
  ['zero size', { ...ready, preview: { ...ready.preview, byte_length: 0 } }],
  ['unsafe creation time', { ...ready, preview: { ...ready.preview, created_at: 9007199254740992 } }],
  ['negative count', { ...ready, preview: { ...ready.preview, counts: { ...ready.preview.counts, cards: -1 } } }],
  ['incomplete counts', { ...ready, preview: { ...ready.preview, counts: { vocabulary: 0 } } }],
  ['missing source version', { ...ready, preview: { ...ready.preview, application_version: undefined } }],
  ['missing restart requirement', { ...ready, preview: { ...ready.preview, requires_restart: false } }],
  ['blank dictionary title', { ...ready, preview: { ...ready.preview, pending_dictionaries: [{ id: 'mdx-7', title: ' ' }] } }],
  ['duplicate dictionary identity', { ...ready, preview: { ...ready.preview, pending_dictionaries: [ready.preview.pending_dictionaries[0], ready.preview.pending_dictionaries[0]] } }],
  ['too many pending dictionaries', { ...ready, preview: { ...ready.preview, pending_dictionaries: Array.from({ length: 16 }, (_, index) => ({ id: `mdx-${index + 1}`, title: 'dictionary.mdx' })) } }],
  ...['builtin', 'mdx-0', 'mdx-01', 'mdx-4294967295'].map((id) => [`invalid pending identity ${id}`, { ...ready, preview: { ...ready.preview, pending_dictionaries: [{ id, title: 'dictionary.mdx' }] } }] as const),
] as const) {
  test(`malformed backup preview ${name} is not reported as verified`, async ({ page }) => {
    await nativeFixture(page, { backup_preview_start: [{ value }] });
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    const panel = page.getByRole('region', { name: '检查备份', exact: true });
    await panel.getByRole('button', { name: '选择备份', exact: true }).click();
    await expect(panel.getByRole('alert')).toContainText('检查备份失败');
    await expect(panel).not.toContainText('备份已验证');
    await expect(panel).not.toContainText('études.lexique-backup');
    await expect(panel.getByRole('button', { name: '选择备份', exact: true })).toBeEnabled();
  });
}

for (const item of [
  { locale: 'zh', settings: '设置', title: '检查备份', choose: '选择备份', retry: '重试', progress: '正在检查备份内容，可继续学习。', failure: '检查备份失败', ready: '备份已验证', restart: '恢复需要重启' },
  { locale: 'en', settings: 'Settings', title: 'Check a backup', choose: 'Choose backup', retry: 'Retry', progress: 'Checking the backup contents. You can continue learning.', failure: 'Backup check failed', ready: 'Backup verified', restart: 'Restoring requires a restart' },
  { locale: 'fr', settings: 'Réglages', title: 'Vérifier une sauvegarde', choose: 'Choisir une sauvegarde', retry: 'Réessayer', progress: 'Vérification du contenu. Vous pouvez continuer à étudier.', failure: 'Échec de la vérification', ready: 'Sauvegarde vérifiée', restart: 'La restauration nécessite un redémarrage' },
]) {
  test(`${item.locale} backup preview announces native progress, failure and keyboard retry`, async ({ page }) => {
    await nativeFixture(page, { interface_preferences: [{ value: { locale: item.locale, theme: 'white' } }], backup_preview_start: [{ value: validating }, { value: ready }], backup_preview_status: [{ value: validating }] });
    await page.goto('/');
    const settings = page.getByRole('button', { name: item.settings, exact: true });
    await settings.focus(); await settings.press('Enter');
    const panel = page.getByRole('region', { name: item.title, exact: true }), choose = panel.getByRole('button', { name: item.choose, exact: true });
    await choose.focus(); await choose.press('Enter');
    await expect(panel.getByRole('progressbar', { name: item.progress, exact: true })).toBeVisible();
    await page.evaluate((value) => window.testNative.emit('lexique-backup-preview-progress', value), { ...validating, state: 'failed', phase: null, error: 'Staging directory is read-only' });
    await expect(panel.getByRole('alert')).toContainText(`${item.failure}: Staging directory is read-only`);
    await expect(panel.getByRole('alert')).toBeFocused();
    await panel.getByRole('alert').getByRole('button', { name: item.retry, exact: true }).focus();
    await page.keyboard.press('Enter');
    await expect(panel.getByRole('status')).toContainText(item.ready);
    await expect(panel).toContainText(item.restart);
    await expect(panel).toContainText('法语词典.mdx');
  });
}

test('browser-only preview explains its desktop capability in every locale', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  for (const [locale, title, choose, message] of [
    ['zh', '检查备份', '选择备份', '备份检查仅在桌面应用中可用。'],
    ['en', 'Check a backup', 'Choose backup', 'Backup checks are available in the desktop app only.'],
    ['fr', 'Vérifier une sauvegarde', 'Choisir une sauvegarde', 'La vérification est disponible uniquement dans l’application de bureau.'],
  ]) {
    await page.getByRole('banner').getByRole('combobox').first().selectOption(locale);
    const panel = page.getByRole('region', { name: title, exact: true });
    await expect(panel).toContainText(message);
    await expect(panel.getByRole('button', { name: choose, exact: true })).toBeDisabled();
    await expect(panel.getByRole('status')).toHaveCount(0);
  }
});

test('preview leaves pending learning requests untouched instead of resolving them before selection', async ({ page }) => {
  const pending = { 'lexique-pending-rating-88': '{"operation":"unconfirmed-rating","rating":99}',
    'lexique-pending-rating-99': '{"operation":"second-unconfirmed-rating"}', 'lexique-pending-undo': '{"not_a_confirmed_undo":true}' };
  await nativeFixture(page, { backup_preview_start: [{ value: ready }] });
  await page.addInitScript((values) => { for (const [key, value] of Object.entries(values)) localStorage.setItem(key, value); }, pending);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  expect(await page.evaluate((keys) => Object.fromEntries(keys.map((key) => [key, localStorage.getItem(key)])), Object.keys(pending))).toEqual(pending);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => ['review_rate', 'review_undo', 'review_add_batch', 'vocabulary_save'].includes(call.command)))).toEqual([]);
});

test('an empty verified backup displays zero contents and no pending dictionary list', async ({ page }) => {
  const empty = { ...ready, preview: { ...ready.preview, filename: 'empty.lexique-backup', pending_dictionaries: [],
    counts: { vocabulary: 0, cards: 0, ratings: 0, batches: 0, custom_dictionaries: 0, legacy_audio: 0,
      included_audio_versions: 0, included_legacy_audio: 0, audio_files: 0, source_missing_audio: 0, source_invalid_audio: 0 } } };
  await nativeFixture(page, { backup_preview_start: [{ value: empty }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
  await expect(panel).toContainText('empty.lexique-backup');
  await expect(panel).toContainText('生词：0');
  await expect(panel).toContainText('卡片：0');
  await expect(panel).toContainText('原音频缺失：0');
  await expect(panel).toContainText('原音频损坏：0');
  await expect(panel.getByRole('heading', { name: '待补齐词典', exact: true })).toHaveCount(0);
});

test('a cancelled picker keeps a native cleanup lease failure visible until a later selection succeeds', async ({ page }) => {
  const choosing = { ...validating, state: 'choosing', phase: null }, cancelled = { ...choosing, state: 'cancelled', cancel_requested: true };
  await nativeFixture(page, { backup_preview_start: [{ value: choosing }, { error: 'Backup cleanup is still in progress' }, { value: ready }],
    backup_preview_status: [{ value: choosing }], backup_preview_cancel: [{ value: cancelled }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const panel = page.getByRole('region', { name: '检查备份', exact: true });
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('status')).toHaveText('请选择要检查的备份文件。');
  await panel.getByRole('button', { name: '取消检查', exact: true }).click();
  await expect(panel.getByRole('status')).toHaveText('备份检查已取消。');
  await panel.getByRole('button', { name: '选择备份', exact: true }).click();
  await expect(panel.getByRole('alert')).toContainText('Backup cleanup is still in progress');
  await expect(panel).not.toContainText('备份已验证');
  await panel.getByRole('alert').getByRole('button', { name: '重试', exact: true }).click();
  await expect(panel.getByRole('status')).toContainText('备份已验证');
});
