import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const source = { title: 'Lexique 3.83', version: '3.83', attribution: 'Boris New; Christophe Pallier',
  license: 'CC BY-SA 4.0', source_url: 'https://chrplr.github.io/openlexicon/datasets-info/Lexique383/README-Lexique.html',
  license_url: 'https://creativecommons.org/licenses/by-sa/4.0/' };

const entries = [0, 9.99, 10, 99.99, 100].map((per_million, index) => ({ key: `mot ${index}`, lemma: `mot ${index}`,
  per_million, reason: null, candidates: [`mot ${index}`], parts: [{ part_of_speech: 'NOM', per_million, reason: null }] }));

test('notebook frequency labels include zero and both product thresholds without changing notes', async ({ page }) => {
  const rows = entries.map((entry, index) => ({ ...record, id: index + 1, headword: entry.key, notes: `note ${index}` }));
  await nativeFixture(page, { vocabulary_list: [{ value: rows }], frequency_lookup: [{ value: { source, metric: 'freqlemfilms2', entries } }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  for (const [index, label] of ['低频', '低频', '中频', '中频', '高频'].entries()) {
    const row = page.locator('.vocabulary-list li').nth(index);
    await expect(row.locator('.frequency-badge')).toHaveText(label);
    await expect(row).toContainText(`note ${index}`);
  }
});

test('frequency reads only the visible notebook page up to one hundred words', async ({ page }) => {
  const rows = Array.from({ length: 101 }, (_, index) => ({ ...record, id: index + 1, headword: `mot ${index}` }));
  await nativeFixture(page, { vocabulary_list: [{ value: rows }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.locator('.frequency-badge')).toHaveCount(20);
  await expect(page.getByText('正在读取词频…', { exact: true })).toHaveCount(0);
  await page.getByLabel('每页词数').selectOption('100');
  await expect(page.locator('.frequency-badge')).toHaveCount(100);
  await expect(page.getByText('正在读取词频…', { exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: '下一页', exact: true }).click();
  await expect(page.locator('.frequency-badge')).toHaveCount(1);
  await expect(page.getByText('正在读取词频…', { exact: true })).toHaveCount(0);
  await expect(page.locator('.vocabulary-list strong')).toHaveText('mot 100');
  await expect(page.getByRole('alert')).toHaveCount(0);
});

test('a late frequency failure from the previous page cannot replace the current result', async ({ page }) => {
  const rows = Array.from({ length: 21 }, (_, index) => ({ ...record, id: index + 1, headword: `mot ${index}` }));
  const entry = { ...entries[4]!, key: 'mot 20', lemma: 'mot 20', candidates: ['mot 20'] };
  await nativeFixture(page, { vocabulary_list: [{ value: rows }], frequency_lookup: [
    { error: { code: 'query_failed', message: 'stale frequency failure' }, gate: 'first-frequency' },
    { value: { source, metric: 'freqlemfilms2', entries: [entry] } },
  ] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.locator('.vocabulary-list li')).toHaveCount(20);
  await expect(page.getByText('正在读取词频…', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '下一页', exact: true }).click();
  await expect(page.locator('.frequency-badge')).toHaveText('高频');
  await page.evaluate(() => window.testNative.release('first-frequency'));
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.locator('.vocabulary-list strong')).toHaveText('mot 20');
  await expect(page.locator('.frequency-badge')).toHaveText('高频');
});

test('saved details retain a missing POS value and explain the unknown total', async ({ page }) => {
  const entry = { key: 'o', lemma: 'o', per_million: null, reason: 'missing_pos', candidates: ['o'],
    parts: [{ part_of_speech: null, per_million: 0.01, reason: 'missing_pos' }, { part_of_speech: 'NOM', per_million: 57.07, reason: null }] };
  const lookup = { source, metric: 'freqlemfilms2', entries: [entry] };
  await nativeFixture(page, { vocabulary_list: [{ value: [{ ...record, headword: 'o', notes: '原有笔记' }] }],
    frequency_lookup: [{ value: lookup }, { value: lookup }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.locator('.vocabulary-summary .frequency-badge')).toHaveText('暂无词频');
  await page.getByText('词频数值与来源', { exact: true }).click();
  await expect(page.locator('.frequency-details')).toContainText('合计暂不可用');
  await expect(page.locator('.frequency-details')).toContainText('0.01');
  await expect(page.locator('.frequency-details')).toContainText('57.07');
  await expect(page.locator('.frequency-details')).toContainText('CC BY-SA 4.0');
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('原有笔记');
  await expect(page.getByRole('alert')).toHaveCount(0);
});

test('a missing resource leaves notes editable and retry only reads frequency', async ({ page }) => {
  const entry = { ...entries[4]!, key: record.headword, lemma: record.headword, candidates: [record.headword] };
  const lookup = { source, metric: 'freqlemfilms2', entries: [entry] };
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], frequency_lookup: [{ value: lookup },
    { error: { code: 'missing', message: 'frequency resource missing' } }, { value: lookup }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await expect(page.getByRole('alert')).toContainText('frequency resource missing');
  await page.getByLabel('独立笔记', { exact: true }).fill('未保存输入');
  await page.getByRole('button', { name: '重试词频' }).click();
  await expect(page.locator('.vocabulary-summary .frequency-badge')).toHaveText('高频');
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('未保存输入');
  await expect(page.getByRole('button', { name: '保存', exact: true })).toBeEnabled();
});

for (const [name, entry] of [
  ['negative', { ...entries[0]!, per_million: -1 }],
  ['missing reason', { ...entries[0]!, per_million: null }],
  ['wrong identity', { ...entries[0]!, key: 'different query' }],
] as const) test(`invalid frequency response is visible: ${name}`, async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [{ ...record, headword: entries[0]!.key }] }],
    frequency_lookup: [{ value: { source, metric: 'freqlemfilms2', entries: [entry] } }] });
  await page.goto('/'); await page.getByRole('button', { name: '生词本', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('词频读取失败');
  await expect(page.locator('.vocabulary-list .frequency-badge')).toHaveText('暂无词频');
  await expect(page.locator('.vocabulary-list button')).toBeEnabled();
});

for (const theme of ['white', 'sepia', 'dark']) for (const [locale, labels] of [
  ['zh', ['低频', '中频', '高频', '暂无词频']], ['en', ['Low frequency', 'Medium frequency', 'High frequency', 'Frequency unavailable']],
  ['fr', ['Fréquence faible', 'Fréquence moyenne', 'Fréquence élevée', 'Fréquence indisponible']],
] as const) test(`${locale} frequency text and contrast in ${theme}`, async ({ page }) => {
  const values = [entries[0]!, entries[2]!, entries[4]!, { key: 'suis', lemma: null, per_million: null, reason: 'ambiguous', candidates: ['suivre', 'être'], parts: [] }];
  await nativeFixture(page, { vocabulary_list: [{ value: values.map((entry, index) => ({ ...record, id: index + 1, headword: entry.key })) }],
    frequency_lookup: [{ value: { source, metric: 'freqlemfilms2', entries: values } }] });
  await page.goto('/'); await page.getByLabel('显示主题').selectOption(theme);
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.getByLabel('界面语言').selectOption(locale);
  await expect(page.locator('.frequency-badge')).toHaveText([...labels]);
  const contrasts = await page.locator('.frequency-badge').evaluateAll((badges) => {
    const rgb = (color: string) => color.match(/[\d.]+/g)!.slice(0, 3).map(Number);
    const luminance = (color: number[]) => color.map((c) => c / 255).map((c) => c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2)
      .reduce((sum, c, i) => sum + c * [0.2126, 0.7152, 0.0722][i]!, 0);
    return badges.map((badge) => {
      let background: Element = badge;
      while (getComputedStyle(background).backgroundColor === 'rgba(0, 0, 0, 0)' && background.parentElement) background = background.parentElement;
      const a = luminance(rgb(getComputedStyle(badge).color)), b = luminance(rgb(getComputedStyle(background).backgroundColor));
      return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
    });
  });
  expect(contrasts.every((value) => value >= 4.5)).toBe(true);
  if ((locale === 'en' && theme === 'white') || (locale === 'zh' && theme === 'sepia') || (locale === 'fr' && theme === 'dark'))
    await page.screenshot({ path: `test-results/frequency-${locale}-${theme}.png`, fullPage: true });
});
