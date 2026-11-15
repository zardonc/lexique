import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { dictionary, nativeFixture } from './native-fixture';
import simple from './fixtures/conjugation.json' with { type: 'json' };
import pronominal from './fixtures/conjugation-pronominal.json' with { type: 'json' };
import type { DictionaryLookup } from '../src/platform/dictionary';

async function lookups(page: Page, replies: Readonly<Record<string, DictionaryLookup>>, conjugations: Record<string, { tables: readonly unknown[] }>) {
  await page.addInitScript(({ replies, conjugations, empty }) => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: { key: string; dictionary_id?: string }) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => {
      const value = await invoke(command, args), key = args?.key ?? '', dictionaryId = args?.dictionary_id;
      if (command === 'dictionary_lookup') return replies[`${dictionaryId}/${key}`] ?? replies[key] ?? { ...empty, dictionary_id: dictionaryId };
      if (command === 'conjugation_lookup') return { query: key, tables: conjugations[key]?.tables ?? [] };
      return value;
    };
  }, { replies, conjugations, empty: { ...dictionary, entries: [] } });
}

const payer = { ...dictionary, entries: [{ ...dictionary.entries[0], key: 'payer', grammar: ['v'] }] };
const paie = { ...dictionary, entries: [{ ...dictionary.entries[0], key: 'paie', grammar: ['n'], senses: [{ translations: ['薪资'], definitions: [], senses: [] }] }] };

test('a unique verb lemma stays an explicit choice beside a homograph and protects its note', async ({ page }) => {
  await nativeFixture(page);
  await lookups(page, { paie, payer }, { paie: simple.paie });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('paie');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  const choice = page.getByRole('button', { name: '查询原形：payer', exact: true });
  await expect(choice).toBeVisible();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('paie');
  await expect(page.locator('.dictionary-entry h3')).toHaveText('paie');
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
  await page.getByLabel('独立笔记', { exact: true }).fill('keep the salary note');
  await choice.click();
  await page.getByRole('dialog').getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('keep the salary note');
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('paie');
  await choice.click();
  await page.getByRole('dialog').getByRole('button', { name: '放弃修改', exact: true }).click();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('payer');
  await expect(page.locator('.dictionary-entry h3')).toHaveText('payer');
});

test('ambiguous forms only offer lemmas present in the active target dictionary', async ({ page }) => {
  const source = { ...dictionary, dictionary_id: 'mdx-1', entries: [] };
  const suivre = { ...source, entries: [{ ordinal: 2, key: 'suivre', pronunciations: [], grammar: [], senses: [], html: '<p>to follow</p>' }] };
  const etre = { ...payer, dictionary_id: 'mdx-2', entries: [{ ...payer.entries[0], key: 'être' }] };
  await nativeFixture(page, { dictionary_settings: [{ value: [{ id: 'mdx-1', title: 'first.mdx', content_hash: 'a'.repeat(64), availability: 'available' },
    { id: 'mdx-2', title: 'second.mdx', content_hash: 'b'.repeat(64), availability: 'available' },
    { id: 'builtin', title: 'FreeDict', content_hash: null, availability: 'available' },
    { id: 'builtin-fra-eng', title: 'FreeDict fra-eng 0.4.1', content_hash: null, availability: 'available' }] }] });
  await lookups(page, { 'mdx-1/suis': source, 'mdx-1/suivre': suivre, 'mdx-2/être': etre }, { suis: simple.suis });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('suis');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.getByRole('button', { name: '查询原形：suivre', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '查询原形：être', exact: true })).toHaveCount(0);
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('suis');
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup'));
  expect([...new Set(calls.map((call) => call.args?.dictionary_id))]).toEqual(['mdx-1']);
  expect(new Set(calls.map((call) => call.args?.key))).toEqual(new Set(['suis', 'être', 'suivre']));
  await page.getByRole('button', { name: '查询原形：suivre', exact: true }).click();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('suivre');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').at(-1)?.args?.dictionary_id)).toBe('mdx-1');
});

test('a noun-only target cannot be classified as the matched verb lemma', async ({ page }) => {
  const noun = { ...payer, entries: [{ ...payer.entries[0], grammar: ['n'] }] };
  await nativeFixture(page);
  await lookups(page, { payer: noun }, { paie: simple.paie });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('paie');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.getByText('当前词典中没有已核对的动词原形词条。', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: '查询原形：payer', exact: true })).toHaveCount(0);
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('paie');
});

test('pronominal expression links keep the exact apostrophe spelling of the target', async ({ page }) => {
  const target = { ...payer, entries: [{ ...payer.entries[0], key: "s'abstenir" }] };
  await nativeFixture(page);
  await lookups(page, { "s'abstenir": target }, { 'elles se sont abstenues': pronominal['elles se sont abstenues'] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('elles se sont abstenues');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  const choice = page.getByRole('button', { name: "查询原形：s'abstenir", exact: true });
  await expect(choice).toBeVisible();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('elles se sont abstenues');
  await choice.click();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue("s'abstenir");
  await expect(page.locator('.dictionary-entry h3')).toHaveText("s'abstenir");
});
