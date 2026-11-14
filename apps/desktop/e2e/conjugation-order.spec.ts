import { expect, test } from '@playwright/test';
import coloring from './fixtures/conjugation-coloring.json' with { type: 'json' };
import simple from './fixtures/conjugation.json' with { type: 'json' };
import defaultOrder from './fixtures/conjugation-order.json' with { type: 'json' };

test.beforeEach(async ({ page }) => {
  await page.addInitScript(({ tables, order }) => {
    function preferences(choice?: { mode: string }) {
      if (choice) localStorage.setItem('conjugation-mode', choice.mode);
      return { mode: localStorage.getItem('conjugation-mode') ?? 'ending' };
    }
    function blockOrder(choice?: string[]) {
      const failure = localStorage.getItem('conjugation-order-failure');
      if (choice) {
        if (failure === 'write') throw 'Block order storage is read-only';
        if (failure === 'mismatch') return order;
        localStorage.setItem('conjugation-order', JSON.stringify(choice));
      } else if (failure === 'read') throw 'Saved block order is unreadable';
      const saved = localStorage.getItem('conjugation-order');
      return saved ? JSON.parse(saved) : order;
    }
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string,
      args?: { key?: string; order?: string[]; preferences?: { mode: string } }) => {
      const systemReplies = { runtime_status: () => ({ version: '0.1.0', schemaVersion: 17, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } }),
        interface_preferences: () => Reflect.get(args ?? {}, 'preferences') ?? { locale: 'zh', theme: 'white' } }; if (Object.hasOwn(systemReplies, command)) return Reflect.get(systemReplies, command)();
      if (command === 'conjugation_preferences') return preferences(args?.preferences);
      if (command === 'conjugation_block_order') return blockOrder(args?.order);
      const table = tables[args?.key as keyof typeof tables];
      return command === 'conjugation_lookup' ? { query: args?.key, tables: table ? [table] : [] } : null;
    } } });
  }, { tables: { 'nous avions lavé': coloring.laver,
    falloir: { ...simple.falloir.tables[0], verb: { ...simple.falloir.tables[0].verb, blocks: { Y: 'not_applicable' } } },
  }, order: defaultOrder });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
});

test('failed writes and mismatched responses preserve the displayed order and independent coloring', async ({ page }) => {
  await page.getByLabel('French verb or simple form').fill('nous avions lavé');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  await page.getByRole('button', { name: '1 · Complete verbal part', exact: true }).click();
  await page.getByLabel('Block to move', { exact: true }).selectOption('PQP');
  const controls = page.locator('.conjugation-order');
  for (const [failure, message] of [['write', 'read-only'], ['mismatch', 'differs from the request']]) {
    await page.evaluate((value) => localStorage.setItem('conjugation-order-failure', value), failure);
    await controls.getByRole('button', { name: 'Move later', exact: true }).click();
    await expect(controls.getByRole('alert')).toContainText(message);
    await expect(page.locator('.conjugation-block').nth(6)).toHaveAttribute('data-tense', 'PQP');
    await expect(page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText('avions lavé');
  }
  await page.evaluate(() => localStorage.removeItem('conjugation-order-failure'));
  await controls.getByRole('button', { name: 'Move later', exact: true }).click();
  await expect(controls.getByRole('alert')).toHaveCount(0);
  await expect(page.locator('.conjugation-block').nth(7)).toHaveAttribute('data-tense', 'PQP');
  await page.getByRole('button', { name: '3 · Main verb ending', exact: true }).click();
  await expect(page.locator('.conjugation-block').nth(7)).toHaveAttribute('data-tense', 'PQP');
  await expect(page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText('é');
});

test('unreadable, malformed, duplicate, missing and unknown blocks require repair without losing coloring', async ({ page }) => {
  await page.getByRole('button', { name: '2 · Main verb', exact: true }).click();
  for (const saved of ['read', 'not-json', JSON.stringify(defaultOrder.slice(1)),
    JSON.stringify(['P', ...defaultOrder.slice(1, -1), 'P']), JSON.stringify([...defaultOrder.slice(0, -1), 'unknown'])]) {
    await page.evaluate((value) => {
      localStorage.removeItem('conjugation-order-failure');
      if (value === 'read') localStorage.setItem('conjugation-order-failure', 'read');
      else localStorage.setItem('conjugation-order', value);
    }, saved);
    await page.reload(); await page.getByLabel('界面语言').selectOption('en');
    await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
    const controls = page.locator('.conjugation-order');
    await expect(controls.getByRole('alert')).toContainText('Block order failed');
    await expect(controls.getByRole('combobox')).toBeDisabled();
    for (const label of ['Move earlier', 'Move later', 'Reset block order']) await expect(controls.getByRole('button', { name: label, exact: true })).toBeDisabled();
    await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
    await page.evaluate((order) => {
      localStorage.removeItem('conjugation-order-failure'); localStorage.setItem('conjugation-order', JSON.stringify(order));
    }, defaultOrder);
    await controls.getByRole('button', { name: 'Reload block order', exact: true }).click();
    await expect(controls.getByRole('combobox')).toBeEnabled();
    await expect(controls.getByRole('alert')).toHaveCount(0);
  }
});

test('first and last blocks move by keyboard and controls remain readable in every language and theme', async ({ page }) => {
  await page.getByLabel('French verb or simple form').fill('nous avions lavé');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  const select = page.getByLabel('Block to move', { exact: true });
  await select.focus(); await select.press('End');
  await expect(select).toHaveValue('GPP');
  await expect(page.getByRole('button', { name: 'Move later', exact: true })).toBeDisabled();
  const earlier = page.getByRole('button', { name: 'Move earlier', exact: true });
  await earlier.focus(); await earlier.press('Space');
  await expect(page.locator('.conjugation-block').nth(21)).toHaveAttribute('data-tense', 'GPP');
  await expect(earlier).toBeFocused();
  await select.focus(); await select.press('Home');
  await expect(select).toHaveValue('P');
  await expect(earlier).toBeDisabled();
  await page.getByRole('button', { name: 'Move later', exact: true }).focus();
  await page.getByRole('button', { name: 'Move later', exact: true }).press('Space');
  await expect(page.locator('.conjugation-block').nth(1)).toHaveAttribute('data-tense', 'P');
  for (const [locale, languageLabel, themeLabel, blockLabel, labels] of [
    ['en', 'Interface language', 'Display theme', 'Block to move', ['Move earlier', 'Move later', 'Reset block order']],
    ['fr', 'Interface language', 'Thème d’affichage', 'Bloc à déplacer', ['Déplacer avant', 'Déplacer après', 'Réinitialiser l’ordre des blocs']],
    ['zh', 'Langue de l’interface', '显示主题', '要移动的区块', ['向前移动', '向后移动', '重置区块顺序']],
  ] as const) {
    await page.getByLabel(languageLabel).selectOption(locale);
    const controls = page.locator('.conjugation-order');
    await expect(page.getByLabel(blockLabel, { exact: true })).toHaveValue('P');
    await expect(controls.getByRole('option')).toHaveCount(23);
    for (const theme of ['white', 'sepia', 'dark']) {
      await page.getByLabel(themeLabel).selectOption(theme);
      for (const label of labels) await expect(controls.getByRole('button', { name: label, exact: true })).toBeVisible();
    }
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByLabel('要移动的区块', { exact: true })).toBeVisible();
  await expect(page.locator('.conjugation')).toHaveJSProperty('scrollWidth', await page.locator('.conjugation').evaluate((element) => element.clientWidth));
});

test('keyboard ordering crosses the priority boundary and follows defective verbs without changing coloring', async ({ page }) => {
  const input = page.getByLabel('French verb or simple form');
  await input.fill('nous avions lavé'); await input.press('Enter');
  await page.getByRole('button', { name: '2 · Main verb', exact: true }).click();
  const blocks = page.locator('.conjugation-block');
  await expect(blocks).toHaveCount(23);
  expect(await blocks.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')))).toEqual(defaultOrder);
  await page.getByLabel('Block to move', { exact: true }).selectOption('PQP');
  const later = page.getByRole('button', { name: 'Move later', exact: true });
  await later.focus(); await later.press('Space');
  const reordered = ['P', 'PC', 'I', 'F', 'C', 'S', 'J', 'PQP', 'PA', 'FA', 'CP', 'SP', 'T', 'SQP', 'Y', 'YP', 'W', 'WP', 'G', 'K', 'KP', 'GP', 'GPP'];
  await expect(blocks.nth(7)).toHaveAttribute('data-tense', 'PQP');
  expect(await blocks.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')))).toEqual(reordered);
  await expect(later).toBeFocused();
  await expect(page.locator('[data-tense=PQP]')).toHaveAttribute('data-priority', 'true');
  await expect(page.locator('[data-tense=J]')).toHaveAttribute('data-priority', 'false');
  await expect(page.locator('.conjugation-block[data-priority=true]')).toHaveCount(7);
  const match = page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true]');
  await expect(match).toHaveText('nous avions lavé');
  await expect(match.locator('.conjugation-colored')).toHaveText('lavé');
  await expect(match).toHaveCSS('text-decoration-line', 'underline');
  await input.fill('falloir'); await input.press('Enter');
  await expect(blocks).toHaveCount(23);
  expect(await blocks.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')))).toEqual(reordered);
  await expect(page.locator('[data-tense=Y]')).toContainText('Not applicable (verified)');
  await page.reload(); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.getByLabel('French verb or simple form').fill('nous avions lavé');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  expect(await blocks.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')))).toEqual(reordered);
  const reset = page.getByRole('button', { name: 'Reset block order', exact: true });
  await reset.focus(); await reset.press('Space');
  await expect(blocks.nth(6)).toHaveAttribute('data-tense', 'PQP');
  expect(await blocks.evaluateAll((elements) => elements.map((element) => element.getAttribute('data-tense')))).toEqual(defaultOrder);
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
});
