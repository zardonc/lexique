import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const source = { format: 'csv', filename: 'external.csv' };
const fresh = { content: { headword: 'nouveau', notes: '外部笔记', examples: [{ fr: 'Une phrase.', zh: '' }] }, rows: [2], existing: null, status: 'create', error: null };
const changed = { ...fresh, content: { ...fresh.content, headword: record.headword, notes: '修改笔记' }, rows: [3], existing: record, status: 'different' };
const invalid = { ...fresh, content: { ...fresh.content, headword: 'conflit' }, rows: [4, 6], status: 'invalid', error: 'Rows 4, 6: conflicting notes' };
const mapping = { headword: 0, notes: 1, example_fr: 2, example_zh: null };
const preview = { operation: 'batch-fixture', source, columns: ['mot', 'texte', 'phrase', 'ignored'], mapping, groups: [fresh, changed, invalid], error: null };

test('batch CSV mapping and explicit update and invalid exclusion submit one selected batch', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, mapping: { ...mapping, headword: null }, groups: [], error: 'Map the required headword column' } }],
    note_import_remap: [{ value: preview }], note_import_commit: [{ value: { created: 1, updated: 1, skipped: 1, records: [record] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  await expect(page.getByLabel('headword', { exact: true })).toBeVisible();
  await page.getByLabel('headword', { exact: true }).selectOption('0');
  await page.getByLabel('分隔符', { exact: true }).selectOption('semicolon');
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(page.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
  await page.getByLabel(`导入决定：${record.headword}`, { exact: true }).selectOption('update');
  await page.getByLabel('排除无效组：conflit', { exact: true }).check();
  await expect(page.getByText('例句将整组替换；未提供的语言侧为空。').first()).toBeVisible();
  await page.getByRole('button', { name: '确认导入', exact: true }).click();
  await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('新建 1 · 更新 1 · 跳过 1');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command.startsWith('note_import_')));
  expect(calls.find((call) => call.command === 'note_import_remap')?.args?.options).toEqual({ delimiter: 'semicolon', mapping });
  expect(calls.find((call) => call.command === 'note_import_commit')?.args?.selections).toEqual([{ index: 0, action: 'create' }, { index: 1, action: 'update' }, { index: 2, action: 'skip' }]);
});

test('stale CSV preview remains visible and remapping requires fresh decisions before retry', async ({ page }) => {
  const revised = { ...record, revision: 2, notes: '同时修改' };
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, groups: [changed] } }],
    note_import_remap: [{ value: { ...preview, groups: [{ ...changed, existing: revised }] } }],
    note_import_commit: [{ error: 'Vocabulary revision changed; preview again' }, { value: { created: 0, updated: 1, skipped: 0, records: [revised] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  const decision = page.getByLabel(`导入决定：${record.headword}`, { exact: true });
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  await decision.selectOption('update'); await confirm.click();
  await expect(page.getByRole('alert')).toContainText('Vocabulary revision changed');
  await expect(page.getByRole('region', { name: 'CSV 导入预览', exact: true })).toBeVisible();
  await page.getByLabel('notes', { exact: true }).selectOption('');
  await expect(confirm).toBeDisabled();
  await page.getByRole('button', { name: '重新预览', exact: true }).click();
  await expect(page.getByText('同时修改', { exact: true })).toBeVisible();
  await expect(confirm).toBeDisabled(); await expect(decision).toHaveValue('');
  await decision.selectOption('update'); await confirm.click();
  await expect(page.getByRole('region', { name: '导入 CSV', exact: true }).getByRole('status')).toContainText('更新 1');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'));
  expect(calls).toHaveLength(2);
  expect(calls[0].args?.operation).toBe(calls[1].args?.operation);
});

test('one CSV commit stays pending until its response and review enrollment is an explicit action', async ({ page }) => {
  await nativeFixture(page, { note_import_preview: [{ value: { ...preview, groups: [fresh] } }],
    note_import_commit: [{ gate: 'commit', value: { created: 1, updated: 0, skipped: 0, records: [record] } }],
    review_add_batch: [{ gate: 'enroll', value: { operation: 'fixture', cancelled: false,
      items: [{ vocabulary_id: record.id, created_cards: 2, error: null, audio_state: 'queued', audio_error: null }] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('button', { name: '导入 CSV', exact: true }).click();
  const confirm = page.getByRole('button', { name: '确认导入', exact: true });
  await confirm.click(); await expect(confirm).toBeDisabled();
  await expect(page.getByRole('button', { name: '重新预览', exact: true })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'note_import_commit'))).toHaveLength(1);
  await page.evaluate(() => window.testNative.release('commit'));
  const enroll = page.getByRole('button', { name: '将导入记录加入双卡复习', exact: true });
  await expect(enroll).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'review_add_batch'))).toBe(false);
  await enroll.click(); await expect(enroll).toBeDisabled();
  const enrollment = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_add_batch'));
  expect(enrollment).toHaveLength(1); expect(enrollment[0].args?.request).toMatchObject({ vocabulary_ids: [record.id] });
  await page.evaluate(() => window.testNative.release('enroll'));
  await expect.poll(() => page.evaluate(() => ({ pending: localStorage.getItem('lexique-pending-review-batch'), errors: Array.from(document.querySelectorAll('[role=alert]')).map((node) => node.textContent) }))).toEqual({ pending: null, errors: [] });
  await expect(enroll).toBeEnabled();
});
