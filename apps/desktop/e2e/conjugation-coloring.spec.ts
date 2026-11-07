import { expect, test } from '@playwright/test';
import coloring from './fixtures/conjugation-coloring.json' with { type: 'json' };
import pronominal from './fixtures/conjugation-pronominal.json' with { type: 'json' };
import simple from './fixtures/conjugation.json' with { type: 'json' };
import ordinary from './fixtures/conjugation-ordinary.json' with { type: 'json' };

test.beforeEach(async ({ page }) => {
  await page.addInitScript((data) => {
    function preferences(choice?: { mode: string }) {
      const failure = localStorage.getItem('conjugation-failure');
      if (choice) {
        if (failure === 'write') throw 'Coloring preference storage is read-only';
        if (failure === 'mismatch') return { mode: 'ending' };
        localStorage.setItem('conjugation-mode', choice.mode);
      } else if (failure === 'read') throw 'Saved coloring preference is unreadable';
      return { mode: localStorage.getItem('conjugation-mode') ?? 'ending' };
    }
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: { invoke: async (command: string, args?: { key?: string; preferences?: { mode: string } }) => {
      if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 17, storage: 'sqlite', audioMigration: { pending: 0, failed: 0 } };
      if (command === 'conjugation_preferences') return preferences(args?.preferences);
      const table = data[args?.key as keyof typeof data];
      return command === 'conjugation_lookup' ? { query: args?.key, tables: table ? [table] : [] } : null;
    } } });
  }, {
    'nous avions lavé': coloring.laver, lavais: coloring.laver, finir: coloring.finir,
    'se laver': pronominal['se laver'].tables[0], 'lave-toi': pronominal['se laver'].tables[0],
    'sois-toi lavé': pronominal['se laver'].tables[0], 'nous nous étions lavées': pronominal['se laver'].tables[0],
    'en se lavant': pronominal['se laver'].tables[0], 'que je me lave': pronominal['se laver'].tables[0],
    'va-t’en': pronominal['va-t’en'].tables[0],
    paie: simple.paie.tables[0], 'je suis allée': ordinary['je suis allée'].tables[0],
  });
  await page.goto('/'); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
});

test('three direct modes color the specified parts and preserve match and priority cues', async ({ page }) => {
  const input = page.getByLabel('French verb or simple form');
  const ending = page.getByRole('button', { name: '3 · Main verb ending', exact: true });
  await expect(ending).toHaveAttribute('aria-pressed', 'true');
  await input.fill('nous avions lavé'); await input.press('Enter');
  const match = page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true]');
  await expect(match).toHaveText(['nous avions lavé']);
  await expect(match.locator('.conjugation-colored')).toHaveText(['é']);
  for (const [label, expected] of [['1 · Complete verbal part', 'avions lavé'], ['2 · Main verb', 'lavé'], ['3 · Main verb ending', 'é']]) {
    const choice = page.getByRole('button', { name: label, exact: true });
    await choice.focus(); await choice.press('Space');
    await expect(choice).toHaveAttribute('aria-pressed', 'true');
    await expect(match.locator('.conjugation-colored')).toHaveText([expected]);
    await expect(page.locator('.conjugation-cell[data-matched=true] .conjugation-match')).toContainText('✓ Exact match');
    await expect(page.locator('.conjugation-block[data-priority=true]')).toHaveCount(7);
    await expect(match).toHaveCSS('text-decoration-line', 'underline');
  }
  await input.fill('lavais'); await input.press('Enter');
  await expect(page.locator('.conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText(['ais', 'ais']);
});

test('pronouns stay plain in compounds, imperatives and nonfinite expressions', async ({ page }) => {
  const input = page.getByLabel('French verb or simple form');
  for (const [query, verbal, main, ending] of [
    ['lave-toi', ['lave'], ['lave'], ['e']],
    ['sois-toi lavé', ['sois', 'lavé'], ['lavé'], ['é']],
    ['nous nous étions lavées', ['étions lavées'], ['lavées'], ['ées']],
    ['en se lavant', ['lavant'], ['lavant'], ['ant']],
    ['que je me lave', ['lave'], ['lave'], ['e']],
  ] as const) {
    await input.fill(query); await input.press('Enter');
    const match = page.locator('.conjugation-expression[data-matched=true]');
    await expect(match.first()).toHaveText(query);
    for (const [label, expected] of [
      ['1 · Complete verbal part', verbal], ['2 · Main verb', main], ['3 · Main verb ending', ending],
    ] as const) {
      await page.getByRole('button', { name: label, exact: true }).click();
      await expect(match.first().locator('.conjugation-colored')).toHaveText([...expected]);
      await expect(match.first()).toHaveText(query);
    }
  }
});

test('spelling and gender alternatives retain separate spans and group 3 stays plain in mode 3', async ({ page }) => {
  const input = page.getByLabel('French verb or simple form');
  await input.fill('paie'); await input.press('Enter');
  const alternatives = page.locator('[data-tense=P] [data-person="1s"] .conjugation-expression');
  await expect(alternatives).toHaveText(['je paie', 'je paye']);
  await expect(alternatives.locator('.conjugation-colored')).toHaveText(['e', 'e']);
  for (const label of ['1 · Complete verbal part', '2 · Main verb']) {
    await page.getByRole('button', { name: label, exact: true }).click();
    await expect(alternatives.locator('.conjugation-colored')).toHaveText(['paie', 'paye']);
    await expect(alternatives.first()).toHaveAttribute('data-matched', 'true');
  }
  await page.getByRole('button', { name: '3 · Main verb ending', exact: true }).click();
  await input.fill('se laver'); await input.press('Enter');
  const gender = page.locator('[data-tense=PC] [data-person="1s"] .conjugation-expression');
  await expect(gender).toHaveText(['je me suis lavée', 'je me suis lavées', 'je me suis lavé', 'je me suis lavés']);
  await expect(gender.locator('.conjugation-colored')).toHaveText(['ée', 'ées', 'é', 'és']);
  await input.fill('va-t’en'); await input.press('Enter');
  await expect(page.locator('.conjugation-table .conjugation-colored')).toHaveCount(0);
  await expect(page.locator('.conjugation-table')).toContainText('Third-group verbs stay uncolored in mode 3.');
  const match = page.locator('[data-tense=Y] .conjugation-expression[data-matched=true]');
  await expect(match).toHaveText('va-t’en');
  await expect(match).toHaveCSS('text-decoration-line', 'underline');
  for (const label of ['1 · Complete verbal part', '2 · Main verb']) {
    await page.getByRole('button', { name: label, exact: true }).click();
    await expect(match.locator('.conjugation-colored')).toHaveText(['va']);
    await expect(match).toHaveText('va-t’en');
  }
});

test('the chosen mode follows different verbs and fixture reloads with three languages and themes', async ({ page }) => {
  await page.getByRole('button', { name: '2 · Main verb', exact: true }).click();
  for (const [query, expected] of [['nous avions lavé', ['lavé']], ['finir', ['finir']]] as const) {
    const input = page.getByLabel('French verb or simple form');
    await input.fill(query); await input.press('Enter');
    await expect(page.locator('.conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText([...expected]);
    await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  }
  await page.reload(); await page.getByLabel('界面语言').selectOption('en');
  await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
  await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await page.getByLabel('French verb or simple form').fill('nous avions lavé');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  for (const [locale, languageLabel, themeLabel, group, modes] of [
    ['en', 'Interface language', 'Display theme', 'Verb coloring', ['1 · Complete verbal part', '2 · Main verb', '3 · Main verb ending']],
    ['fr', 'Interface language', 'Thème d’affichage', 'Coloration du verbe', ['1 · Partie verbale complète', '2 · Verbe principal', '3 · Terminaison du verbe principal']],
    ['zh', 'Langue de l’interface', '显示主题', '动词着色', ['1 · 完整动词部分', '2 · 主要动词', '3 · 主要动词词尾']],
  ] as const) {
    await page.getByLabel(languageLabel).selectOption(locale);
    await expect(page.getByRole('group', { name: group, exact: true }).getByRole('button')).toHaveText([...modes]);
    for (const theme of ['white', 'sepia', 'dark']) {
      await page.getByLabel(themeLabel).selectOption(theme);
      const colored = page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored');
      await expect(colored).toHaveText(['lavé']);
      await expect(colored).toHaveCSS('font-style', 'italic');
      await expect(colored).toHaveCSS('color', theme === 'dark' ? 'rgb(120, 216, 193)' : theme === 'sepia' ? 'rgb(9, 94, 74)' : 'rgb(0, 91, 86)');
      await expect(page.locator('.conjugation-block[data-priority=true]')).toHaveCount(7);
    }
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole('button', { name: '2 · 主要动词', exact: true })).toBeVisible();
  await expect(page.locator('.conjugation')).toHaveJSProperty('scrollWidth', await page.locator('.conjugation').evaluate((element) => element.clientWidth));
});

test('preference write failures and mismatched responses keep the prior rendering and can be retried', async ({ page }) => {
  await page.getByLabel('French verb or simple form').fill('nous avions lavé');
  await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
  for (const [failure, message] of [['write', 'read-only'], ['mismatch', 'differs from the request']]) {
    await page.evaluate((value) => localStorage.setItem('conjugation-failure', value), failure);
    await page.getByRole('button', { name: '1 · Complete verbal part', exact: true }).click();
    await expect(page.locator('.conjugation-coloring').getByRole('alert')).toContainText(message);
    await expect(page.getByRole('button', { name: '3 · Main verb ending', exact: true })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText(['é']);
  }
  await page.evaluate(() => localStorage.removeItem('conjugation-failure'));
  await page.getByRole('button', { name: '1 · Complete verbal part', exact: true }).click();
  await expect(page.locator('.conjugation-coloring').getByRole('alert')).toHaveCount(0);
  await expect(page.locator('[data-tense=PQP] .conjugation-expression[data-matched=true] .conjugation-colored')).toHaveText(['avions lavé']);
});

test('unreadable and corrupt preferences stay visible until the learner reloads repaired settings', async ({ page }) => {
  for (const fault of ['read', 'corrupt']) {
    await page.evaluate((value) => {
      localStorage.removeItem('conjugation-failure'); localStorage.removeItem('conjugation-mode');
      localStorage.setItem(value === 'read' ? 'conjugation-failure' : 'conjugation-mode', value === 'read' ? 'read' : 'broken');
    }, fault);
    await page.reload(); await page.getByLabel('界面语言').selectOption('en');
    await page.getByRole('button', { name: 'Conjugation', exact: true }).click();
    const controls = page.locator('.conjugation-coloring');
    await expect(controls.getByRole('alert')).toContainText('Coloring preference failed');
    for (const label of ['1 · Complete verbal part', '2 · Main verb', '3 · Main verb ending']) await expect(page.getByRole('button', { name: label, exact: true })).toBeDisabled();
    await page.evaluate(() => {
      localStorage.removeItem('conjugation-failure'); localStorage.setItem('conjugation-mode', 'main');
    });
    await controls.getByRole('button', { name: 'Reload coloring preference', exact: true }).click();
    await expect(page.getByRole('button', { name: '2 · Main verb', exact: true })).toHaveAttribute('aria-pressed', 'true');
    await expect(controls.getByRole('alert')).toHaveCount(0);
  }
});

test('invalid resource spans and forbidden irregular endings fail visibly at the desktop boundary', async ({ page }) => {
  const tables = [
    { ...coloring.laver, cells: coloring.laver.cells.map((cell, index) => index === 0 ? { ...cell, coloring: {} } : cell) },
    { ...coloring.laver, cells: coloring.laver.cells.map((cell, index) => index === 0 ? { ...cell, coloring: Object.fromEntries(
      Object.keys(cell.coloring).map((text) => [text, { verbal: [[0, 999]], main: [0, 999], ending: null }]),
    ) } : cell) },
    { ...coloring.laver, verb: { ...coloring.laver.verb, group: 3 } },
  ];
  for (const table of tables) {
    await page.evaluate((table) => {
      const internals = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: { key?: string }) => Promise<unknown> };
      const invoke = internals.invoke;
      internals.invoke = async (command: string, args?: { key?: string }) => command === 'conjugation_lookup'
        ? { query: args?.key, tables: [table] } : invoke(command, args);
    }, table);
    await page.getByLabel('French verb or simple form').fill('nous avions lavé');
    await page.getByRole('button', { name: 'Look up conjugation', exact: true }).click();
    await expect(page.locator('.conjugation').getByRole('alert')).toContainText('Conjugation lookup failed.');
    await expect(page.locator('.conjugation-table')).toHaveCount(0);
    await expect(page.getByLabel('French verb or simple form')).toHaveValue('nous avions lavé');
  }
});
