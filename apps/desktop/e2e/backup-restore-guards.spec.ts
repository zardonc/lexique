import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';
import { cancelledBatch, confirmRestore, nativeCalls, pendingBatch, pendingRating, pendingUndo, preparing, previewReady, ratingResult, selectBackup } from './backup-restore-fixture';

async function pendingRequests(page: Page) {
  await page.addInitScript(({ rating, undo, batch }) => {
    localStorage.setItem('lexique-pending-rating-88', JSON.stringify(rating));
    localStorage.setItem('lexique-pending-undo', JSON.stringify(undo));
    localStorage.setItem('lexique-pending-review-batch', JSON.stringify(batch));
    localStorage.setItem('reader-preference', 'keep');
  }, { rating: pendingRating, undo: pendingUndo, batch: pendingBatch });
}

test('whole-backup restore waits for cancelled or failed draft saves before confirmation', async ({ page }) => {
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], backup_restore_start: [{ value: preparing }],
    backup_restore_status: [{ value: preparing }], vocabulary_save: [{ error: 'Draft disk unavailable' },
      { value: { outcome: 'created', record: { ...record, notes: 'Current note' } } }] });
  await page.goto('/');
  await selectBackup(page);
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
  await page.getByLabel('独立笔记').fill('Current note');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改', exact: true });
  await dialog.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记')).toHaveValue('Current note');
  expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await dialog.getByRole('button', { name: '保存后离开', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('Draft disk unavailable');
  expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
  await dialog.getByRole('button', { name: '保存后离开', exact: true }).click();
  await expect(page.getByRole('heading', { name: '设置', exact: true })).toBeVisible();
  await confirmRestore(page);
  await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
  expect(await nativeCalls(page, 'backup_preview_start')).toHaveLength(1);
  expect(await nativeCalls(page, 'vocabulary_save')).toHaveLength(2);
  expect(await nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
});

async function savedRequests(page: Page) {
  return page.evaluate(() => Object.fromEntries(Object.keys(localStorage).filter((key) => key.startsWith('lexique-pending-'))
    .map((key) => [key, localStorage.getItem(key)])));
}

const failures: readonly { readonly name: string; readonly message: string; readonly replies: NonNullable<Parameters<typeof nativeFixture>[1]> }[] = [
  { name: 'rating error', message: 'Rating confirmation failed', replies: { review_rate: [{ error: 'Rating confirmation failed' }] } },
  { name: 'rating rejection', message: 'Rating revision changed', replies: { review_rate: [{ value: { status: 'rejected', code: 'conflict', detail: 'Rating revision changed' } }] } },
  { name: 'undo error', message: 'Undo confirmation failed', replies: { review_undo: [{ error: 'Undo confirmation failed' }] } },
  { name: 'batch error', message: 'Batch confirmation failed', replies: { review_batch_status: [{ value: null }, { error: 'Batch confirmation failed' }] } },
  { name: 'wrong batch identity', message: 'Invalid batch response identity', replies: { review_batch_status: [{ value: null }, { value: { ...cancelledBatch, operation: 'another-batch' } }] } },
  { name: 'incomplete batch', message: 'Invalid batch response identity', replies: {
    review_batch_status: [{ value: null }, { value: { ...cancelledBatch, cancelled: false } }],
    review_add_batch: [{ value: { ...cancelledBatch, cancelled: false } }],
  } },
];

for (const failure of failures) {
  test(`whole-backup confirmation preserves every pending key on ${failure.name}`, async ({ page }) => {
    await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], review_rate: [{ value: ratingResult }],
      review_batch_status: [{ value: null }, { value: cancelledBatch }], backup_restore_start: [{ value: preparing }], ...failure.replies });
    await pendingRequests(page);
    await page.goto('/');
    await selectBackup(page);
    const original = await savedRequests(page);
    await confirmRestore(page);
    await expect(page.getByRole('region', { name: '检查备份', exact: true }).getByRole('alert')).toContainText(failure.message);
    expect(await savedRequests(page)).toEqual(original);
    expect(await page.evaluate(() => localStorage.getItem('reader-preference'))).toBe('keep');
    expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
    await expect(page.getByRole('heading', { name: '设置', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: '恢复整份备份', exact: true })).toBeEnabled();
  });
}

test('a fully confirmed batch keeps its request until committed restore cleanup', async ({ page }) => {
  const fullBatch = { operation: 'pending-batch', cancelled: false, items: [
    { vocabulary_id: 10, created_cards: 2, error: null, audio_state: 'queued', audio_error: null },
    { vocabulary_id: 20, created_cards: 2, error: null, audio_state: 'queued', audio_error: null },
    { vocabulary_id: 30, created_cards: 2, error: null, audio_state: 'queued', audio_error: null },
  ] };
  await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], review_rate: [{ value: ratingResult }],
    review_add_batch: [{ value: fullBatch }], backup_restore_start: [{ value: preparing }], backup_restore_status: [{ value: preparing }] });
  await pendingRequests(page);
  await page.goto('/');
  await selectBackup(page);
  const original = await savedRequests(page);
  await confirmRestore(page);
  await expect(page.getByRole('heading', { name: '正在恢复备份', exact: true })).toBeVisible();
  expect(await savedRequests(page)).toEqual(original);
  expect((await nativeCalls(page, 'review_add_batch'))[0].args?.request).toEqual(pendingBatch);
  expect(await nativeCalls(page, 'backup_restore_start')).toHaveLength(1);
});

for (const change of ['replacement', 'new key'] as const) {
  test(`whole-backup confirmation keeps a ${change} and requires confirmation again`, async ({ page }) => {
    await nativeFixture(page, { backup_preview_start: [{ value: previewReady }], review_rate: [{ value: ratingResult, gate: 'confirm-rating' }],
      review_batch_status: [{ value: null }, { value: cancelledBatch }], backup_restore_start: [{ value: preparing }] });
    await pendingRequests(page);
    await page.goto('/');
    await selectBackup(page);
    const original = await savedRequests(page);
    await confirmRestore(page);
    await expect.poll(() => nativeCalls(page, 'review_rate')).toHaveLength(1);
    const key = change === 'replacement' ? 'lexique-pending-rating-88' : 'lexique-pending-rating-99999';
    const request = { ...pendingRating, operation: 'new-rating', card_id: change === 'replacement' ? 88 : 99999 };
    await page.evaluate(({ key, request }) => {
      localStorage.setItem(key, JSON.stringify(request)); window.testNative.release('confirm-rating');
    }, { key, request });
    await expect(page.getByRole('region', { name: '检查备份', exact: true }).getByRole('alert')).toContainText('Confirm pending requests again');
    expect(await savedRequests(page)).toEqual({ ...original, [key]: JSON.stringify(request) });
    expect(await nativeCalls(page, 'backup_restore_start')).toEqual([]);
    await expect(page.getByRole('button', { name: '恢复整份备份', exact: true })).toBeEnabled();
  });
}
