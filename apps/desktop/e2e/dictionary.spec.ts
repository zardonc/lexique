import { expect, test } from '@playwright/test';

const result = {
  source: { title: 'FreeDict+WikDict français-中文', version: '2025.11.23',
    attribution: 'Karl Bartel; Wiktionary contributors', license: 'CC-BY-SA-3.0',
    source_url: 'https://download.freedict.org/dictionaries/fra-zho/2025.11.23/',
    license_url: 'https://creativecommons.org/licenses/by-sa/3.0/' },
  sha256: 'a'.repeat(64),
  entries: [{ ordinal: 1, key: 'école', pronunciations: ['e.kɔl'], grammar: ['n', 'fem'],
    senses: [{ translations: ['学校'], definitions: [], senses: [
      { translations: [], definitions: ['Lieu dédié à l’apprentissage'], senses: [] },
    ] }] }],
};

test('dictionary page keeps browser preview explicit', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('œil');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.locator('.dictionary')).toContainText('浏览器预览无法读取内置词典');
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
});

test('native adapter displays results, no match and resource errors', async ({ page }) => {
  await page.addInitScript((fixture) => {
    Object.defineProperty(window, 'isTauri', { value: true });
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {
      invoke: async (command: string, args?: { key: string }) => {
        if (command === 'runtime_status') return { version: '0.1.0', schemaVersion: 1, storage: 'sqlite' };
        const key = args?.key;
        const errors: Record<string, { code: string; message: string }> = {
          missing: { code: 'missing', message: 'Dictionary file not found' },
          broken: { code: 'corrupt', message: 'Dictionary checksum mismatch' },
          failed: { code: 'query_failed', message: 'SQLite query failed' },
        };
        if (key && errors[key]) throw errors[key];
        if (key === 'invalid') return { ...fixture, entries: 'invalid IPC payload' };
        return key === 'école' ? fixture : { ...fixture, entries: [] };
      },
    } });
  }, result);
  await page.goto('/');
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.locator('.dictionary-entry').getByText('学校', { exact: true })).toBeVisible();
  await expect(page.getByText('Lieu dédié à l’apprentissage', { exact: true })).toBeVisible();
  await expect(page.locator('.dictionary-source')).toContainText('CC-BY-SA-3.0');
  for (const [key, message] of [['sommes', '未找到完全匹配的词条'], ['missing', '内置词典资源不存在'],
    ['broken', '内置词典资源已损坏'], ['failed', '词典查询失败'], ['invalid', '词典查询失败']]) {
    await page.getByLabel('法语词或词组').fill(key);
    await page.getByRole('button', { name: '查词', exact: true }).click();
    await expect(page.locator('.dictionary')).toContainText(message);
    await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
  }
});
