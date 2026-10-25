import { expect, test } from '@playwright/test';
import { collect, nativeFixture, record } from './native-fixture';

test('browser notebook clearly reports unavailable native persistence', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByText('浏览器预览无法读写生词本，请使用桌面应用。')).toBeVisible();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toHaveCount(0);
});

test('explicit save preserves failed input and retries the same operation before linking collection', async ({ page }) => {
  const saved = { ...record, notes: 'Mon école\n学校 😀' };
  await nativeFixture(page, {
    vocabulary_save: [{ error: 'disk full' }, { value: { outcome: 'created', record: saved } }],
  });
  await collect(page);
  await expect(page.getByRole('region', { name: '基本词汇信息' }).first()).toContainText('未提供');
  await expect(page.locator('.dictionary-original')).toContainText('学派');
  await page.getByLabel('独立笔记', { exact: true }).fill(saved.notes);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save'))).toHaveLength(0);
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('disk full');
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue(saved.notes);
  await expect(page.getByText('已保存', { exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '查看现有记录' })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save'));
  expect(calls[0].args).toEqual(calls[1].args);
  await page.screenshot({ path: 'test-results/notebook-saved.png', fullPage: true });
});

test('empty notes save, existing records can be cleared, and resource failures preserve saved content', async ({ page }) => {
  await nativeFixture(page, {
    vocabulary_find: [{ value: record }], vocabulary_list: [{ value: [{ ...record, notes: 'old notes' }] }],
    vocabulary_save: [{ value: { outcome: 'updated', record: { ...record, revision: 2 } } }],
    dictionary_lookup: [{ error: { code: 'missing', message: 'resource removed' } }],
  });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('old notes');
  await expect(page.locator('.dictionary-original')).toContainText('内置词典资源不存在');
  await expect(page.locator('.vocabulary-editor')).toContainText('学校');
  await page.getByLabel('独立笔记', { exact: true }).fill('');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('');
});

test('stale updates keep input and retain conflict checks until current content is adopted', async ({ page }) => {
  const current = { ...record, notes: 'other editor', revision: 2 };
  await nativeFixture(page, { vocabulary_find: [{ value: record }],
    vocabulary_save: [{ value: { outcome: 'conflict', record: current } }] });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '查看现有记录' }).click();
  await page.getByLabel('独立笔记', { exact: true }).fill('my changes');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('other editor');
  await page.getByRole('button', { name: '返回编辑' }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('my changes');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByText('采用现有内容将放弃本次未保存修改。')).toBeVisible();
  await page.getByRole('button', { name: '采用现有内容', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('other editor');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save').map((call) =>
    (call.args?.request as { revision: number }).revision))).toEqual([1, 1]);
});

test('navigation and native close callbacks offer save, discard or keep editing', async ({ page }) => {
  await nativeFixture(page, { vocabulary_save: [{ error: 'database unavailable' }] });
  await collect(page); await page.getByLabel('独立笔记', { exact: true }).fill('keep me');
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: '继续编辑' }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('keep me');
  await page.evaluate(() => window.testNative.close());
  await page.getByRole('button', { name: '保存后离开' }).click();
  await expect(page.getByRole('dialog')).toContainText('database unavailable');
  expect(await page.evaluate(() => window.testNative.calls.some((call) => call.command === 'finish_close'))).toBe(false);
  await page.getByRole('button', { name: '放弃修改', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.some((call) => call.command === 'finish_close'))).toBe(true);
});

test('Unicode limits preserve overlong input and invalid IPC responses remain visible', async ({ page }) => {
  await nativeFixture(page, { vocabulary_save: [{ value: { outcome: 'created', record: { invalid: true } } }] });
  await collect(page);
  const notes = page.getByLabel('独立笔记', { exact: true });
  await notes.fill('😀'.repeat(20_000));
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeEnabled();
  await notes.fill('😀'.repeat(20_000) + 'e\u0301');
  await expect(page.getByRole('alert')).toContainText('内容未被截断');
  await expect(notes).toHaveValue('😀'.repeat(20_000) + 'e\u0301');
  await notes.fill('valid'); await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('保存失败');
  await expect(notes).toHaveValue('valid');
});

test('notebook restores sort and sends literal searches while read failures never appear empty', async ({ page }) => {
  await nativeFixture(page, { vocabulary_sort: [{ value: 'headword-desc' }],
    vocabulary_list: [{ value: [record] }, { error: 'read failed' }, { value: [record] }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByLabel('排序方式')).toHaveValue('headword-desc');
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('ÉCOLE%_');
  await expect(page.getByRole('alert')).toContainText('read failed');
  await expect(page.getByText('没有匹配的生词记录。')).toHaveCount(0);
  await page.getByRole('button', { name: '重试', exact: true }).click();
  await expect(page.locator('.vocabulary-list')).toContainText('école');
  await page.getByRole('searchbox', { name: '搜索词头、摘要或笔记' }).fill('');
  await expect(page.getByLabel('排序方式')).toHaveValue('headword-desc');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_list')
    .some((call) => call.args?.query === 'ÉCOLE%_' && call.args.sort === 'headword-desc'))).toBe(true);
});

test('an uncertain update remains guarded when input returns to the originally loaded text', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }],
    vocabulary_save: [{ error: 'response lost after commit' }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('独立笔记', { exact: true }).fill('committed but unconfirmed');
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('response lost after commit');
  await page.getByLabel('独立笔记', { exact: true }).fill('');
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeEnabled();
  await page.evaluate(() => window.testNative.close());
  await expect(page.getByRole('dialog')).toBeVisible();
});

test('all sort directions persist through the adapter and notebook works in three languages on mobile', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByLabel('排序方式')).toHaveValue('updated-desc');
  for (const sort of ['headword-asc', 'headword-desc', 'created-asc', 'created-desc', 'updated-asc', 'updated-desc']) {
    await page.getByLabel('排序方式').selectOption(sort);
    await expect(page.getByLabel('排序方式')).toBeEnabled();
    await expect.poll(() => page.evaluate((expected) => window.testNative.calls.filter((call) =>
      call.command === 'vocabulary_list').at(-1)?.args?.sort === expected, sort)).toBe(true);
  }
  await page.getByLabel('界面语言').selectOption('en');
  await expect(page.getByRole('searchbox', { name: 'Search headwords, summaries or notes' })).toBeVisible();
  await page.getByLabel('Interface language').selectOption('fr');
  await expect(page.getByRole('searchbox', { name: 'Rechercher dans les mots, résumés ou notes' })).toBeVisible();
  await page.getByLabel('Langue de l’interface').selectOption('zh');
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toBeFocused();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: 'test-results/notebook-mobile.png', fullPage: true });
});
