import { expect, test } from '@playwright/test';
import { collect, dictionary, nativeFixture, record } from './native-fixture';

test('editable translation text uses the dictionary and opens the full result', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('法语原文', { exact: true }).fill('école');
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  await page.getByRole('button', { name: '打开完整词典结果', exact: true }).click();
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语词或词组')).toHaveValue('école');
  await expect(page.getByRole('button', { name: '加入生词本', exact: true })).toBeEnabled();
});

test('captured selection enters translation without automatic dictionary lookup', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: '  école  ' });
  });
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('  école  ');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
});

test('shortcut conflicts stay visible and another shortcut can be saved', async ({ page }) => {
  await nativeFixture(page, { selection_shortcut: [
    { value: { shortcut: 'Ctrl+Alt+Shift+D', registered: true, error: null } },
    { value: { shortcut: 'Ctrl+Alt+Shift+D', registered: true, error: null } },
    { error: 'Could not register shortcut; choose another: already registered' },
    { value: { shortcut: 'Ctrl+Alt+K', registered: true, error: null } },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('取词快捷键', { exact: true })).toHaveValue('Ctrl+Alt+Shift+D');
  await page.getByLabel('取词快捷键', { exact: true }).fill('Ctrl+Alt+X');
  await page.getByRole('button', { name: '保存取词快捷键', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'already registered' })).toBeVisible();
  await expect(page.getByText('当前使用：Ctrl+Alt+Shift+D', { exact: true })).toBeVisible();
  await page.getByLabel('取词快捷键', { exact: true }).fill('Ctrl+Alt+K');
  await page.getByRole('button', { name: '保存取词快捷键', exact: true }).click();
  await expect(page.getByText('取词快捷键已保存。', { exact: true })).toBeVisible();
  await expect(page.getByText('当前使用：Ctrl+Alt+K', { exact: true })).toBeVisible();
});

test('cancelled and older captures cannot replace a newer input', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'pending', request: 10 }));
  await page.getByRole('button', { name: '取消', exact: true }).click();
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 10, text: 'stale' }));
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 11, text: 'école' }));
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 10, text: 'stale' }));
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 11, text: 'late' }));
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
});

test('editing invalidates a delayed dictionary result and permits another lookup', async ({ page }) => {
  await nativeFixture(page, { dictionary_lookup: [{ value: dictionary, gate: 'first' }, { value: dictionary }] });
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('法语原文', { exact: true }).fill('école');
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByRole('button', { name: '取消', exact: true })).toBeVisible();
  await page.getByLabel('法语原文', { exact: true }).fill('côte');
  await page.evaluate(() => window.testNative.release('first'));
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
  await page.getByLabel('法语原文', { exact: true }).fill('école');
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
});

test('a long selection stays editable and only its selected word is queried', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  const raw = `école\n${'à'.repeat(600)}`;
  await page.evaluate((text) => window.testNative.emit('selection-input', { kind: 'captured', request: 20, text }), raw);
  const field = page.getByLabel('法语原文', { exact: true });
  await expect(field).toHaveValue(raw);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
  await field.evaluate((node) => { (node as HTMLTextAreaElement).setSelectionRange(0, 5); });
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
  await expect(field).toHaveValue(raw);
  expect(await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'dictionary_lookup')?.args?.key)).toBe('école');
});

test('translation input and captured text keep only the first 5,000 characters', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  const field = page.getByLabel('法语原文', { exact: true });
  const prefix = `${'à'.repeat(4_999)}😀`;
  await field.focus();
  await page.keyboard.insertText(`${prefix}fin`);
  await expect(field).toHaveValue(prefix);
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByRole('button', { name: '查询输入', exact: true })).toBeEnabled();
  await page.evaluate((text) => window.testNative.emit('selection-input', { kind: 'captured', request: 25, text }), `${prefix}stale`);
  await expect(field).toHaveValue(prefix);
  await field.fill('côte');
  await expect(field).toHaveValue('côte');
});

test('capture uses the unsaved-note guard and does not save or collect text', async ({ page }) => {
  await nativeFixture(page);
  await collect(page);
  await page.getByLabel('独立笔记', { exact: true }).fill('保留原有修改');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 30 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 30, text: 'école' });
  });
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('保留原有修改');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 31, text: 'école' }));
  await page.getByRole('button', { name: '放弃修改', exact: true }).click();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save').length)).toBe(0);
});

test('a capture during note saving opens translation after the save completes', async ({ page }) => {
  await nativeFixture(page, { vocabulary_save: [{ gate: 'save', value: { outcome: 'created', record: { ...record, notes: '保留笔记' } } }] });
  await collect(page);
  await page.getByLabel('独立笔记', { exact: true }).fill('保留笔记');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '正在保存…', exact: true })).toBeDisabled();
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 35 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 35, text: 'école' });
    window.testNative.release('save');
  });
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  const saves = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save'));
  expect(saves).toHaveLength(1);
  expect(saves[0]?.args?.request).toMatchObject({ notes: '保留笔记' });
});

test('a failed note save keeps the latest capture behind the leave guard', async ({ page }) => {
  await nativeFixture(page, { vocabulary_save: [{ gate: 'save', error: 'disk full' }] });
  await collect(page);
  await page.getByLabel('独立笔记', { exact: true }).fill('保留未保存笔记');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'captured', request: 36, text: 'stale' });
    window.testNative.emit('selection-input', { kind: 'captured', request: 37, text: 'école' });
    window.testNative.release('save');
  });
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await expect(dialog.getByRole('alert').filter({ hasText: 'disk full' })).toBeVisible();
  await page.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('保留未保存笔记');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 38, text: 'école' }));
  await page.getByRole('button', { name: '放弃修改', exact: true }).click();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
});

for (const code of ['no_selection', 'unsupported', 'permission', 'read_failed', 'busy'] as const) {
  test(`selection ${code} remains visible without inventing dictionary results`, async ({ page }) => {
    await nativeFixture(page);
    await page.goto('/');
    await page.evaluate((failure) => window.testNative.emit('selection-input', { kind: 'failed', request: 40, code: failure, message: 'source status' }), code);
    await expect(page.getByRole('alert').filter({ hasText: 'source status' })).toBeVisible();
    expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(0);
    await page.getByLabel('法语原文', { exact: true }).fill('école');
    await page.getByRole('button', { name: '查询输入', exact: true }).click();
    await expect(page.getByText('学校', { exact: true })).toBeVisible();
  });
}

test('invalid capture payloads stay visible and cannot populate input', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 50, text: 123 }));
  await expect(page.getByRole('alert').filter({ hasText: '选择返回的数据无效' })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('');
});

test('application text selection enters translation with editable text', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByText('école', { exact: true }).first().evaluate((node) => {
    const range = document.createRange(); range.selectNodeContents(node);
    window.getSelection()?.removeAllRanges(); window.getSelection()?.addRange(range);
  });
  await page.getByRole('button', { name: '查询应用内选中文字', exact: true }).click();
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('école');
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
});
