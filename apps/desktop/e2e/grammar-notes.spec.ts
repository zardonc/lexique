import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const note = { id: 2, title: 'Passé composé', body: 'Action terminée.\nJe suis partie.', category_ids: [1],
  card: null, revision: 1, created_at: 1000, updated_at: 1000 };
const category = { id: 1, name: 'Temps' };

for (const failedPart of ['note', 'card'] as const) {
  test(`failed ${failedPart} saves retry at the revision from the other independent save`, async ({ page }) => {
    const enrolled = { ...note, category_ids: [], card: { id: 3, question: 'Quand ?', answer: 'Avant.' } };
    const body = 'Ma note.', answer = 'Après.';
    const otherSaved = { ...enrolled, revision: 2, ...(failedPart === 'note' ? { card: { ...enrolled.card, answer } } : { body }) };
    const saved = { ...otherSaved, revision: 3, body, card: { ...enrolled.card, answer } };
    const failedCommand = failedPart === 'note' ? 'grammar_save' : 'grammar_card_save';
    const otherCommand = failedPart === 'note' ? 'grammar_card_save' : 'grammar_save';
    await nativeFixture(page, { grammar_list: [{ value: [enrolled] }],
      [failedCommand]: [{ error: 'disk full' }, { value: { outcome: 'updated', record: saved } }],
      [otherCommand]: [{ value: { outcome: 'updated', record: otherSaved } }] });
    await page.goto('/');
    await page.getByRole('button', { name: '笔记', exact: true }).click();
    await page.getByRole('tab', { name: '语法', exact: true }).click();
    await page.locator('.vocabulary-list button').click();
    const failedSave = page.getByRole('button', { name: failedPart === 'note' ? '保存笔记' : '保存问答卡', exact: true });
    const otherSave = page.getByRole('button', { name: failedPart === 'note' ? '保存问答卡' : '保存笔记', exact: true });
    await page.getByLabel(failedPart === 'note' ? '语法正文' : '答案', { exact: true }).fill(failedPart === 'note' ? body : answer);
    await failedSave.click();
    await expect(page.getByRole('alert')).toContainText('disk full');
    await page.getByLabel(failedPart === 'note' ? '答案' : '语法正文', { exact: true }).fill(failedPart === 'note' ? answer : body);
    await otherSave.click();
    await expect(otherSave).toBeDisabled();
    await failedSave.click();
    await expect(failedSave).toBeDisabled();
    await expect(page.getByLabel('语法正文')).toHaveValue(body);
    await expect(page.getByLabel('答案', { exact: true })).toHaveValue(answer);
    const requests = await page.evaluate((command) => window.testNative.calls.filter((call) => call.command === command)
      .map((call) => call.args?.request as { operation: string; revision: number }), failedCommand);
    expect(requests).toHaveLength(2);
    expect(requests[0].revision).toBe(1);
    expect(requests[1].revision).toBe(2);
    expect(requests[1].operation).not.toBe(requests[0].operation);
  });
}

test('lost card-save replies replay the original operation without another save', async ({ page }) => {
  const enrolled = { ...note, card: { id: 3, question: 'Quand ?', answer: 'Avant.' } };
  const saved = { ...enrolled, revision: 2, card: { ...enrolled.card, answer: 'Après.' } };
  await nativeFixture(page, { grammar_list: [{ value: [enrolled] }], learning_categories: [{ value: [category] }],
    grammar_card_save: [{ error: 'Response lost' }, { value: { outcome: 'replayed', record: saved } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('答案', { exact: true }).fill(saved.card.answer);
  const save = page.getByRole('button', { name: '保存问答卡', exact: true });
  await save.click();
  await expect(page.getByRole('alert')).toContainText('Response lost');
  await expect(page.getByLabel('答案', { exact: true })).toHaveValue(saved.card.answer);
  await save.click();
  await expect(save).toBeDisabled();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'grammar_card_save'));
  expect(calls).toHaveLength(2);
  expect(calls[1].args).toEqual(calls[0].args);
});

test('grammar saves retain failed input and create an independent editable question-answer card', async ({ page }) => {
  const enrolled = { ...note, revision: 2, card: { id: 3, question: 'Quand utiliser le passé composé ?', answer: 'Action terminée.' } };
  await nativeFixture(page, { learning_categories: [{ value: [category] }],
    grammar_save: [{ error: 'disk full' }, { value: { outcome: 'created', record: note } }],
    grammar_card_save: [{ value: { outcome: 'created', record: enrolled } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '新建语法笔记' }).click();
  await page.getByLabel('语法标题', { exact: true }).fill(note.title);
  await page.getByLabel('语法正文').fill(note.body);
  await page.getByRole('group', { name: '记录分类' }).getByLabel('Temps').check();
  await page.getByRole('button', { name: '保存笔记', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('disk full');
  await expect(page.getByLabel('语法正文')).toHaveValue(note.body);
  await expect(page.getByRole('group', { name: '记录分类' }).getByLabel('Temps')).toBeChecked();
  await page.getByRole('button', { name: '保存笔记', exact: true }).click();
  await page.getByRole('button', { name: '制作问答卡' }).click();
  await expect(page.getByLabel('题面')).toHaveValue(note.title);
  await expect(page.getByLabel('答案', { exact: true })).toHaveValue(note.body);
  await page.getByLabel('题面').fill(enrolled.card.question);
  await page.getByLabel('答案', { exact: true }).fill(enrolled.card.answer);
  await page.getByRole('button', { name: '保存并加入复习' }).click();
  await expect(page.getByRole('button', { name: '暂停复习' })).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls);
  const saves = calls.filter((call) => call.command === 'grammar_save');
  expect(saves[0].args).toEqual(saves[1].args);
  expect((saves[1].args?.request as { category_ids: number[] }).category_ids).toEqual([1]);
  expect(calls.filter((call) => call.command === 'review_add_batch')).toHaveLength(0);
});

test('grammar and card drafts share save-discard-stay protection and failed close saves retain drafts', async ({ page }) => {
  const enrolled = { ...note, card: { id: 3, question: 'Quand ?', answer: 'Avant.' } };
  await nativeFixture(page, { grammar_list: [{ value: [enrolled] }], learning_categories: [{ value: [category] }],
    grammar_card_save: [{ error: 'read-only database' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('答案', { exact: true }).fill('Après.\nUne autre réponse.');
  await page.evaluate(() => window.testNative.close());
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await dialog.getByRole('button', { name: '保存后离开' }).click();
  await expect(dialog).toContainText('read-only database');
  await dialog.getByRole('button', { name: '继续编辑' }).click();
  await expect(page.getByLabel('答案', { exact: true })).toHaveValue('Après.\nUne autre réponse.');
  await page.getByRole('tab', { name: '生词', exact: true }).click();
  await dialog.getByRole('button', { name: '放弃修改' }).click();
  await expect(page.getByLabel('答案', { exact: true })).toHaveCount(0);
  await expect(page.getByRole('tab', { name: '生词', exact: true })).toHaveAttribute('aria-selected', 'true');
});

test('grammar fields enforce Unicode limits and require a complete question-answer draft', async ({ page }) => {
  const saved = { ...note, title: '📖'.repeat(200), body: '', category_ids: [] };
  await nativeFixture(page, { grammar_save: [{ value: { outcome: 'created', record: saved } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  await page.getByRole('button', { name: '新建语法笔记' }).click();
  const save = page.getByRole('button', { name: '保存笔记', exact: true });
  await page.getByLabel('语法标题', { exact: true }).fill(saved.title + '📖');
  await expect(save).toBeDisabled();
  await page.getByLabel('语法标题', { exact: true }).fill(saved.title);
  await page.getByLabel('语法正文').fill('é'.repeat(19_999) + '\n');
  await expect(save).toBeEnabled();
  await page.getByLabel('语法正文').fill('é'.repeat(20_001));
  await expect(save).toBeDisabled();
  await page.getByLabel('语法正文').fill('');
  await save.click();
  await page.getByRole('button', { name: '制作问答卡' }).click();
  const enroll = page.getByRole('button', { name: '保存并加入复习', exact: true });
  await expect(enroll).toBeDisabled();
  await page.getByLabel('答案', { exact: true }).fill(' \n ');
  await expect(enroll).toBeDisabled();
  await page.getByLabel('答案', { exact: true }).fill(saved.title + 'é'.repeat(19_800));
  await expect(enroll).toBeEnabled();
  await page.getByLabel('题面').fill('é'.repeat(20_001));
  await expect(enroll).toBeDisabled();
  await page.getByLabel('题面').fill('Question\nUne autre ligne.');
  await expect(enroll).toBeEnabled();
});

for (const editedCard of [false, true]) {
  test(`lost note-save replies preserve ${editedCard ? 'a conflicting card draft' : 'a card edited elsewhere'}`, async ({ page }) => {
    const enrolled = { ...note, card: { id: 3, question: 'Quand ?', answer: 'Initiale.' } };
    const current = { ...enrolled, body: 'Ma note.', revision: 3, card: { ...enrolled.card, answer: 'Réponse externe.' } };
    await nativeFixture(page, { grammar_list: [{ value: [enrolled] }], learning_categories: [{ value: [category] }],
      grammar_save: [{ error: 'Response lost' }, { value: { outcome: 'replayed', record: current } }] });
    await page.goto('/');
    await page.getByRole('button', { name: '笔记', exact: true }).click();
    await page.getByRole('tab', { name: '语法', exact: true }).click();
    await page.locator('.vocabulary-list button').click();
    await page.getByLabel('语法正文').fill(current.body);
    if (editedCard) await page.getByLabel('答案', { exact: true }).fill('Ma réponse.');
    await page.getByRole('button', { name: '保存笔记', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText('Response lost');
    await page.getByRole('button', { name: '保存笔记', exact: true }).click();
    if (editedCard) {
      await expect(page.getByRole('alert')).toContainText(current.card.answer);
      await expect(page.getByLabel('答案', { exact: true })).toHaveValue('Ma réponse.');
    } else {
      await expect(page.getByLabel('答案', { exact: true })).toHaveValue(current.card.answer);
      await expect(page.getByRole('button', { name: '保存问答卡', exact: true })).toBeDisabled();
    }
    expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'grammar_card_save'))).toHaveLength(0);
    const saves = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'grammar_save'));
    expect(saves).toHaveLength(2);
    expect(saves[1].args).toEqual(saves[0].args);
  });
}
