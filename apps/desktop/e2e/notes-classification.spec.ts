import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const categories = [{ id: 1, name: 'Temps' }, { id: 2, name: 'À revoir' }];
const note = { id: 2, title: 'Passé composé', body: 'Je suis partie.', card: null, category_ids: [1], revision: 1, created_at: 1000, updated_at: 1000 };
const other = { ...note, id: 3, title: 'Pronoms', category_ids: [2] };

test('shared category filters combine with both note types and sorting retains selected identities', async ({ page }) => {
  const classified = { ...note, revision: 2, category_ids: [1, 2] };
  await nativeFixture(page, { learning_categories: [{ value: categories }], vocabulary_list: [{ value: [{ ...record, category_ids: [1] }] }],
    grammar_list: [{ value: [note, other] }, { value: [other, note] }, { value: [other, classified] }],
    category_bulk: [{ value: [{ id: note.id, status: 'saved', error: null }] }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByLabel('筛选分类').selectOption('1');
  await expect(page.locator('.vocabulary-list button')).toContainText('école');
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await expect(page.locator('.vocabulary-list button')).toHaveCount(1);
  await expect(page.locator('.vocabulary-list button')).toContainText(note.title);
  await page.getByLabel('筛选分类').selectOption('all');
  await page.getByLabel(`选择 ${note.title}`, { exact: true }).check();
  await page.getByLabel('排序方式').selectOption('headword-asc');
  await expect(page.locator('.vocabulary-list button').first()).toContainText(other.title);
  await expect(page.getByLabel(`选择 ${note.title}`, { exact: true })).toBeChecked();
  const bulk = page.getByRole('region', { name: '批量整理分类' });
  await bulk.getByLabel('批量整理分类').selectOption('2');
  await bulk.getByRole('button', { name: '添加分类', exact: true }).click();
  await expect(bulk).toContainText(`${note.title}: 成功`);
  await page.getByLabel('筛选分类').selectOption('2');
  await expect(page.locator('.vocabulary-list button')).toHaveCount(2);
  await expect(bulk.getByRole('button', { name: '添加分类', exact: true })).toBeDisabled();
});

test('sorting keeps off-page selections and classification retries only failed records', async ({ page }) => {
  const notes = Array.from({ length: 21 }, (_, index) => ({ ...note, id: index + 2, title: `Point ${index}` }));
  const reversed = [...notes].reverse();
  await nativeFixture(page, { learning_categories: [{ value: categories }],
    grammar_list: [{ value: notes }, { value: reversed }], category_bulk: [
      { value: [{ id: notes[0].id, status: 'failed', error: 'database locked' }, { id: reversed[0].id, status: 'saved', error: null }] },
      { value: [{ id: notes[0].id, status: 'saved', error: null }] },
    ] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByLabel(`选择 ${notes[0].title}`, { exact: true }).check();
  await page.getByLabel('排序方式').selectOption('headword-desc');
  await expect(page.locator('.vocabulary-list button').first()).toContainText(reversed[0].title);
  await expect(page.getByLabel(`选择 ${notes[0].title}`, { exact: true })).toHaveCount(0);
  await page.getByLabel('选择本页', { exact: true }).check();
  await page.getByLabel('选择本页', { exact: true }).uncheck();
  await page.getByLabel(`选择 ${reversed[0].title}`, { exact: true }).check();
  const bulk = page.getByRole('region', { name: '批量整理分类' });
  await bulk.getByLabel('批量整理分类').selectOption('2');
  await bulk.getByRole('button', { name: '添加分类', exact: true }).click();
  await expect(bulk.getByRole('listitem').filter({ hasText: 'Point 0:' })).toContainText('database locked');
  await expect(bulk.getByRole('listitem').filter({ hasText: 'Point 20:' })).toContainText('成功');
  await bulk.getByRole('button', { name: '重试分类' }).click();
  await expect(bulk.getByRole('button', { name: '重试分类' })).toHaveCount(0);
  const requests = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'category_bulk').map((call) => call.args?.request));
  expect(requests).toEqual([
    { record_kind: 'grammar', ids: [notes[0].id, reversed[0].id], category_id: 2, action: 'add' },
    { record_kind: 'grammar', ids: [notes[0].id], category_id: 2, action: 'add' },
  ]);
  await page.getByLabel('搜索语法标题或正文').fill('Point');
  await expect(bulk.getByRole('button', { name: '添加分类', exact: true })).toBeDisabled();
});

test('category rename and confirmed deletion keep notes and recover from visible failures', async ({ page }) => {
  const renamed = { ...categories[0], name: 'Règles' };
  await nativeFixture(page, { learning_categories: [{ value: categories }, { value: categories }, { value: [renamed, categories[1]] }, { value: [categories[1]] }],
    grammar_list: [{ value: [note] }, { value: [note] }, { value: [{ ...note, revision: 2, category_ids: [] }] }],
    category_rename: [{ error: 'Category name already exists' }, { value: renamed }],
    category_delete: [{ error: 'database locked' }, { value: null }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByLabel('筛选分类').selectOption('1');
  await page.getByText('管理分类', { exact: true }).click();
  const row = page.locator('.category-manager li').filter({ hasText: 'Temps' });
  await row.getByLabel('分类名称: Temps').fill(renamed.name);
  await row.getByRole('button', { name: '重命名', exact: true }).click();
  await expect(row.getByRole('alert')).toContainText('already exists');
  await expect(row.getByLabel('分类名称: Temps')).toHaveValue(renamed.name);
  await row.getByRole('button', { name: '重命名', exact: true }).click();
  const renamedRow = page.locator('.category-manager li').filter({ hasText: 'Règles' });
  await expect(renamedRow.getByLabel('分类名称: Règles')).toBeVisible();
  await expect(page.getByLabel('筛选分类')).toHaveValue('1');
  await expect(page.locator('.vocabulary-list button')).toContainText(note.title);
  await renamedRow.getByRole('button', { name: '删除分类', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '删除这个分类？' });
  await expect(dialog).toContainText('笔记、卡片和学习进度全部保留');
  await dialog.getByRole('button', { name: '取消', exact: true }).click();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'category_delete'))).toHaveLength(0);
  await renamedRow.getByRole('button', { name: '删除分类', exact: true }).click();
  await dialog.getByRole('button', { name: '确认删除', exact: true }).click();
  await expect(dialog.getByRole('alert')).toContainText('database locked');
  await dialog.getByRole('button', { name: '确认删除', exact: true }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByLabel('筛选分类')).toHaveValue('all');
  await page.getByLabel('筛选分类').selectOption('uncategorized');
  await expect(page.locator('.vocabulary-list button')).toContainText(note.title);
});

test('word category edits discard and save with the rest of the note', async ({ page }) => {
  const saved = { ...record, revision: 2, notes: 'À apprendre.', category_ids: [1] };
  await nativeFixture(page, { learning_categories: [{ value: categories }], vocabulary_list: [{ value: [{ ...record, category_ids: [] }] }],
    vocabulary_save: [{ error: 'disk full' }, { value: { outcome: 'updated', record: saved } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  const picker = page.getByRole('group', { name: '记录分类' });
  await picker.getByLabel('Temps').check();
  await page.getByLabel('独立笔记').fill(saved.notes);
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('dialog', { name: '有尚未保存的修改' }).getByRole('button', { name: '放弃修改' }).click();
  await page.getByRole('tab', { name: '生词', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(picker.getByLabel('Temps')).not.toBeChecked();
  await expect(page.getByLabel('独立笔记')).toHaveValue('');
  await picker.getByLabel('Temps').check();
  await page.getByLabel('独立笔记').fill(saved.notes);
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('disk full');
  await expect(picker.getByLabel('Temps')).toBeChecked();
  await expect(page.getByLabel('独立笔记')).toHaveValue(saved.notes);
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeDisabled();
  const saves = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'vocabulary_save'));
  expect(saves[0].args).toEqual(saves[1].args);
  expect((saves[1].args?.request as { category_ids: number[] }).category_ids).toEqual([1]);
});
