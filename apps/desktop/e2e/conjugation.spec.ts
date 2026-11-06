import { expect, test } from '@playwright/test';
import fixtures from './fixtures/conjugation.json' with { type: 'json' };
import ordinary from './fixtures/conjugation-ordinary.json' with { type: 'json' };

test('ordinary compounds expose every interpretation and complete agreement alternative', async ({ page }) => {
  await page.addInitScript((data) => {
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string, args?: { key: string }) => {
      if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 1, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } };
      return command === 'conjugation_lookup' ? data[args?.key as keyof typeof data] ?? { query: args?.key, tables: [] } : null;
    } } });
  }, ordinary);
  await page.goto('/'); await page.getByRole('button', { name: '动词变位', exact: true }).click();
  const input = page.getByLabel('法语动词或简单变位词形');
  await input.fill('aie aimé'); await input.press('Enter');
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(2);
  for (const code of ['SP', 'YP']) await expect(page.locator(`[data-tense=${code}] [data-matched=true]`).first()).toBeVisible();
  await expect(page.locator('[data-tense=KP]')).toContainText('ayant aimé');
  await expect(page.locator('[data-tense=GP]')).toContainText('en aimant');
  await expect(page.locator('[data-tense=GPP]')).toContainText('en ayant aimé');
  await input.fill('je suis allée'); await input.press('Enter');
  await expect(page.locator('[data-tense=PC]')).toContainText('je suis allé');
  await expect(page.locator('[data-tense=PC]')).toContainText('je suis allée');
  await expect(page.locator('.conjugation-expression[data-matched=true]')).toHaveText(['je suis allée']);
  await input.fill('habite'); await input.press('Enter');
  await expect(page.locator('[data-tense=P]')).not.toContainText('je habite');
  await expect(page.locator('[data-tense=P]')).toContainText('j’habite');
  await expect(page.locator('[data-tense=P]')).not.toContainText('主语省音未确认');
  for (const query of ['j’ai abélisé', 'je suis monté', 'j’ai monté']) {
    await input.fill(query); await input.press('Enter');
    await expect(page.locator('.conjugation-expression[data-matched=true]')).toHaveText([query]);
    await expect(page.locator('[data-tense=PC] .conjugation-cell').first()).not.toContainText('未确认');
  }
  await input.fill('gésir'); await input.press('Enter');
  for (const code of ['PC', 'J', 'YP']) await expect(page.locator(`[data-tense=${code}]`)).toContainText('不适用（已验证）');
  for (const query of ['j’occirai', 'il est avenu', 'ayons chu', 'aie occis', 'il stupéfait', 'aie stupéfait', 'aie forclos']) {
    await input.fill(query); await input.press('Enter');
    await expect(page.locator('.conjugation-expression[data-matched=true]').filter({ hasText: new RegExp(`^${query}$`, 'u') }).first()).toHaveText(query);
  }
  await input.fill('aie occis'); await input.press('Enter');
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(2);
  await expect(page.locator('[data-tense=Y]')).not.toContainText('未确认');
  await input.fill('j’ai pu'); await input.press('Enter');
  await expect(page.getByRole('group', { name: '匹配动词' }).getByRole('button')).toHaveCount(2);
  await page.getByRole('button', { name: /^paître/ }).click();
  await expect(page.locator('[data-tense=PC]')).toContainText('鹰猎用语');
  await expect(page.locator('[data-tense=K]')).toContainText('鹰猎用语');
  await input.fill('je avons aimé'); await input.press('Enter');
  await expect(page.locator('.conjugation-table')).toHaveCount(0);
});

test('simple lookup keeps all candidates, cells and spelling alternatives accessible', async ({ page }) => {
  await page.addInitScript((data) => {
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string, args?: { key: string }) => {
      if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 1, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } };
      if (command === 'conjugation_lookup') return data[args?.key as keyof typeof data];
      return null;
    } } });
  }, fixtures);
  await page.goto('/');
  await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  const input = page.getByLabel('French verb or simple form');
  const search = page.getByRole('button', { name: 'Look up conjugation', exact: true });
  await input.fill('sommes'); await search.click();
  await expect(page.getByRole('status').filter({ hasText: '2 verbs · 3 matching cells' })).toBeVisible();
  const choices = page.getByRole('group', { name: 'Matching verbs' });
  await expect(choices.getByRole('button')).toHaveText([/être/, /sommer/]);
  await expect(choices.getByRole('button', { name: /^être/ })).toContainText('matching cells: 1');
  await expect(choices.getByRole('button', { name: /^sommer/ })).toContainText('matching cells: 2');
  await expect(page.locator('.conjugation-table')).toHaveCount(0);
  await choices.getByRole('button', { name: /^sommer/ }).click();
  await expect(page.locator('.conjugation-block')).toHaveCount(23);
  await expect(page.locator('.conjugation-block h4').first()).toHaveText('Présent de l’indicatif');
  await expect(page.locator('.conjugation-block h4')).toHaveText([
    'Présent de l’indicatif', 'Passé composé', 'Imparfait', 'Futur simple', 'Conditionnel présent', 'Subjonctif présent', 'Plus-que-parfait',
    'Passé simple', 'Passé antérieur', 'Futur antérieur', 'Conditionnel passé', 'Subjonctif passé', 'Subjonctif imparfait',
    'Subjonctif plus-que-parfait', 'Impératif présent', 'Impératif passé', 'Infinitif présent', 'Infinitif passé',
    'Participe présent', 'Participe passé', 'Participe composé', 'Gérondif présent', 'Gérondif passé',
  ]);
  await expect(page.locator('.conjugation-block[data-priority=true]')).toHaveCount(7);
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(2);
  await expect(page.getByText('English meaning unavailable', { exact: true })).toBeVisible();
  await choices.getByRole('button', { name: /^être/ }).click();
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(1);
  await input.fill('suis'); await search.click();
  await expect(choices.getByRole('button', { name: /^suivre/ })).toContainText('matching cells: 3');
  await page.getByRole('button', { name: /^suivre/ }).click();
  await expect(page.locator('.conjugation-cell[data-matched=true]')).toHaveCount(3);
  await expect(page.locator('[data-tense=Y] .conjugation-cell[data-matched=true]')).toContainText('Exact match');
  await expect(page.locator('[data-tense=P] .conjugation-cell[data-matched=true]').first()).not.toContainText('poursuis');
  await input.fill('paie'); await search.click();
  const cell = page.locator('[data-tense=P] [data-person="1s"]');
  await expect(cell.locator('.conjugation-expression')).toHaveText(['je paie', 'je paye']);
  await expect(cell.locator('[data-matched=true]')).toHaveText('je paie');
  await expect(page.locator('[data-tense=P] [data-person="2s"]')).not.toHaveAttribute('data-matched', 'true');
});

test('preview and failures keep the query and never retain an earlier table', async ({ page }) => {
  await page.goto('/'); await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('suis'); await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.locator('.conjugation')).toContainText('浏览器预览无法读取内置变位数据');
  await page.addInitScript((data) => {
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string, args?: { key: string }) => {
      if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 1, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } };
      if (command !== 'conjugation_lookup') return null;
      const key = args?.key;
      if (key === 'missing' || key === 'corrupt' || key === 'query_failed') throw { code: key, message: 'Test resource error' };
      if (key === 'invalid') return { query: key, tables: 'invalid response' };
      if (key === 'wrong') return data.paie;
      return data[key as keyof typeof data] ?? { query: key, tables: [] };
    } } });
  }, fixtures);
  await page.reload(); await page.getByRole('button', { name: '动词变位', exact: true }).click();
  const input = page.getByLabel('法语动词或简单变位词形'), submit = page.getByRole('button', { name: '查询变位', exact: true });
  for (const [query, expected] of [['unknown', '未找到完全匹配'], ['missing', '内置变位资源不存在'], ['corrupt', '内置变位资源已损坏'],
    ['query_failed', '变位查询失败'], ['invalid', '变位查询失败'], ['wrong', '变位查询失败'], [' ', '请输入有效']]) {
    await input.fill('paie'); await submit.click(); await expect(page.locator('.conjugation-table')).toHaveCount(1);
    await input.fill(query); await input.press('Enter');
    await expect(page.locator('.conjugation')).toContainText(expected);
    await expect(input).toHaveValue(query); await expect(page.locator('.conjugation-table')).toHaveCount(0);
  }
});

test('source gaps, pronominal-only uses, themes and languages stay explicit', async ({ page }) => {
  await page.addInitScript((data) => {
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string, args?: { key: string }) => {
      if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 1, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } };
      return command === 'conjugation_lookup' ? data[args?.key as keyof typeof data] : null;
    } } });
  }, fixtures);
  await page.goto('/'); await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('falloir'); await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.locator('[data-tense=P]')).toContainText('没有已确认的源词形');
  await expect(page.locator('[data-tense=P]')).toContainText('il faut');
  await expect(page.locator('[data-tense=P]')).not.toContainText('elle faut');
  await expect(page.locator('[data-tense=PC]')).toContainText('未确认');
  await page.getByLabel('法语动词或简单变位词形').fill('enfuir'); await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await expect(page.locator('.conjugation-table')).toContainText('不适用（已验证）');
  await expect(page.locator('.conjugation-table')).toContainText('代词式结构未确认');
  await expect(page.locator('[data-tense=P]')).not.toContainText('je enfuis');
  await page.getByLabel('界面语言').selectOption('fr');
  await expect(page.getByLabel('Verbe français ou forme simple')).toHaveValue('enfuir');
  for (const theme of ['white', 'sepia', 'dark']) {
    await page.getByLabel('Thème d’affichage').selectOption(theme);
    await expect(page.locator('.app')).toHaveAttribute('data-theme', theme);
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator('.conjugation-block').first()).toBeVisible();
});
