import { expect, test } from '@playwright/test';
import { dictionary, nativeFixture, record } from './native-fixture';

test('dictionary excerpt freezes source and enters the shared draft only on transfer', async ({ page }) => {
  await nativeFixture(page, { vocabulary_find: [{ value: record }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '摘录例句', exact: true }).click();
  await page.getByLabel('摘录法语').fill('Une école.');
  await page.getByLabel('摘录中文').fill('一所学校。');
  await page.getByRole('button', { name: '加入草稿', exact: true }).click();
  await expect(page.locator('#example-fr')).toHaveValue('Une école.');
  await expect(page.locator('#example-zh')).toHaveValue('一所学校。');
  await expect(page.locator('.vocabulary-editor')).toContainText('2025.11.23');
  expect(await page.evaluate(() => window.testNative.calls.filter((c) => c.command === 'vocabulary_save'))).toEqual([]);
});


for (const main of [{ example_fr: 'Une phrase.', example_zh: '' }, { example_fr: '', example_zh: '旧译文' }, { example_fr: 'Une phrase.', example_zh: '旧译文' }]) {
  test(`excerpt preserves partial or full main ${JSON.stringify(main)}`, async ({ page }) => {
    await nativeFixture(page, { vocabulary_find: [{ value: { ...record, ...main } }] });
    await page.goto('/'); await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
    await page.getByRole('button', { name: '摘录例句', exact: true }).click();
    await page.getByLabel('摘录中文').fill('新译文'); await page.getByRole('button', { name: '加入草稿', exact: true }).click();
    await expect(page.locator('#example-fr')).toHaveValue(main.example_fr); await expect(page.locator('#example-zh')).toHaveValue(main.example_zh);
    await expect(page.locator('#example-fr-0')).toHaveValue(''); await expect(page.locator('#example-zh-0')).toHaveValue('新译文');
  });
}

test('exact duplicate focuses original; translation and source differences survive save failures', async ({ page }) => {
  const source = { ...record.reference, dictionary_id: 'builtin' };
  const saved = { ...record, example_fr: 'Une école.', example_zh: '学校', example_source: source, example_sense: 0 };
  await nativeFixture(page, { vocabulary_find: [{ value: saved }], vocabulary_save: [{ error: 'disk full' }] });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '摘录例句', exact: true }).click();
  await page.getByLabel('摘录法语').fill('Une école.'); await page.getByLabel('摘录中文').fill('学校');
  await page.getByRole('button', { name: '加入草稿', exact: true }).click();
  await expect(page.getByText('例句已存在，已定位原例句。')).toBeVisible(); await expect(page.locator('#example-fr')).toBeFocused();
  await page.getByLabel('摘录中文').fill('另一译文'); await page.getByRole('button', { name: '加入草稿', exact: true }).click();
  await page.getByRole('button', { name: '保存', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'disk full' })).toBeVisible(); await expect(page.locator('#example-zh-0')).toHaveValue('另一译文');
  expect(await page.evaluate(() => window.testNative.calls.find((c) => c.command === 'vocabulary_save')?.args?.request)).toMatchObject({ id: 1, more_examples: [{ fr: 'Une école.', zh: '另一译文', source }] });
});

test('switching dictionary retains frozen excerpt source and pasted input', async ({ page }) => {
  const second = { ...dictionary, dictionary_id: 'builtin-fra-eng', source: { ...dictionary.source, title: 'Second dictionary' } };
  await nativeFixture(page, { dictionary_settings: [{ value: [{ id: 'builtin', title: 'First' }, { id: 'builtin-fra-eng', title: 'Second' }] }],
    dictionary_lookup: [{ value: dictionary }, { value: second }], vocabulary_find: [{ value: record }] });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '摘录例句', exact: true }).click(); await page.getByLabel('摘录法语').fill('Une école.');
  await page.getByRole('tab', { name: 'Second', exact: true }).click(); await expect(page.getByLabel('摘录法语')).toHaveValue('Une école.');
  await page.getByRole('button', { name: '加入草稿', exact: true }).click();
  await expect(page.locator('.vocabulary-editor')).not.toContainText('Second dictionary');
  await page.getByRole('button', { name: '摘录例句', exact: true }).click(); await page.getByLabel('摘录法语').fill('Une école.');
  await page.getByRole('button', { name: '加入草稿', exact: true }).click(); await expect(page.locator('#example-fr-0')).toHaveValue('Une école.');
  await expect(page.locator('.vocabulary-editor')).toContainText('Second dictionary');
});

test('capacity and Unicode length errors retain pasted input and leave guard protects it', async ({ page }) => {
  await nativeFixture(page, { vocabulary_find: [{ value: { ...record, example_fr: 'Existing', more_examples: Array.from({ length: 100 }, () => ({ fr: '', zh: '' })) } }] });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '摘录例句', exact: true }).click(); await page.getByLabel('摘录法语').fill('𠮷'.repeat(20_001));
  await page.getByRole('button', { name: '加入草稿', exact: true }).click(); await expect(page.getByRole('alert').filter({ hasText: '20,000' })).toBeVisible();
  await page.getByLabel('摘录法语').fill('𠮷'.repeat(20_000)); await page.getByRole('button', { name: '加入草稿', exact: true }).click();
  await expect(page.getByText('更多例句已达 100 组，摘录内容已保留。')).toBeVisible(); await expect(page.getByLabel('摘录法语')).toHaveValue('𠮷'.repeat(20_000));
  await page.getByRole('button', { name: '生词本', exact: true }).click(); await expect(page.getByRole('dialog')).toBeVisible();
});

test('new word excerpt is created only by explicit save with the original collection preference', async ({ page }) => {
  const source = { ...record.reference, dictionary_id: 'builtin' };
  await nativeFixture(page, { vocabulary_save: [{ value: { outcome: 'created', record: { ...record, example_fr: 'Une école.', example_source: source, example_sense: 0 } } }] });
  await page.goto('/'); await page.getByLabel('法语词或词组').fill('école'); await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '摘录例句', exact: true }).click(); await page.getByLabel('摘录法语').fill('Une école.');
  await page.getByRole('button', { name: '加入草稿', exact: true }).click(); await page.getByLabel('例句对应释义').selectOption('0');
  expect(await page.evaluate(() => window.testNative.calls.filter((c) => c.command === 'vocabulary_save'))).toEqual([]);
  await page.getByRole('button', { name: '保存', exact: true }).click(); await expect(page.getByRole('status').filter({ hasText: '已保存' })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  expect(calls.find((c) => c.command === 'vocabulary_save')?.args?.request).toMatchObject({ id: null, example_source: source, example_fr: 'Une école.', example_zh: '' });
  expect(calls.filter((c) => c.command === 'review_add')).toEqual([]);
});
