import { expect, test } from '@playwright/test';
import { dictionary, nativeFixture } from './native-fixture';
import type { navigationProbe } from './fixtures/navigation-probe';

const catalog = [{ id: 'mdx-1', title: 'first.mdx', content_hash: 'a'.repeat(64), availability: 'available' },
  { id: 'mdx-2', title: 'second.mdx', content_hash: 'b'.repeat(64), availability: 'available' },
  { id: 'builtin', title: 'FreeDict fra-zho', content_hash: null, availability: 'available' },
  { id: 'builtin-fra-eng', title: 'FreeDict fra-eng 0.4.1', content_hash: null, availability: 'available' }];
const first = { ...dictionary, dictionary_id: 'mdx-1', source: { ...dictionary.source, title: 'first.mdx', attribution: null, license: null, source_url: null, license_url: null },
  entries: [{ ordinal: 1, key: 'double', pronunciations: [], grammar: [], senses: [], html: '<p>first sense</p>' }] };

test('dictionary tabs load one body first and keep a failed dictionary isolated', async ({ page }) => {
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_lookup: [{ value: first }, { error: { code: 'corrupt', message: 'second dictionary is damaged' } }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('double');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.getByRole('tab', { name: 'first.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  await expect(page.frameLocator('iframe[title="first.mdx"]').getByText('first sense', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').map((call) => call.args?.dictionary_id))).toEqual(['mdx-1']);
  await page.getByRole('tab', { name: 'second.mdx', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('second dictionary is damaged');
  await page.getByRole('tab', { name: 'first.mdx', exact: true }).click();
  await expect(page.frameLocator('iframe[title="first.mdx"]').getByText('first sense', { exact: true })).toBeVisible();
  expect(await page.getByLabel('法语词或词组', { exact: true }).inputValue()).toBe('double');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(2);
});

test('MDX formatting survives while scripts, navigation and external resources are isolated', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => { if (request.url().includes('mdx.invalid')) requests.push(request.url()); });
  const html = `<p style="color:red;background-image:url(https://mdx.invalid/image)" onclick="parent.pwned=true"><b>L’école</b> d’aujourd’hui.</p>
    <a href="https://mdx.invalid/nav" target="_top">Où est le café ?</a><script>parent.pwned=true</script>
    <img src="https://mdx.invalid/img" onerror="parent.pwned=true"><iframe src="https://mdx.invalid/frame"></iframe>
    <meta http-equiv="refresh" content="0;url=https://mdx.invalid/refresh"><style>@import url(https://mdx.invalid/css)</style>
    <svg onload="parent.pwned=true"><foreignObject><img src="https://mdx.invalid/svg"></foreignObject></svg>`;
  const lookup = { ...first, entries: [{ ...first.entries[0], html }] };
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_lookup: [{ value: lookup }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('double');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  const iframe = page.locator('iframe[title="first.mdx"]');
  await expect(iframe).toHaveAttribute('sandbox', '');
  const frame = iframe.contentFrame();
  await expect(frame.getByText('L’école d’aujourd’hui.', { exact: true })).toBeVisible();
  await expect(frame.locator('b')).toHaveText('L’école');
  await expect(frame.locator('script, img, iframe, svg, a, [onclick], [onerror]')).toHaveCount(0);
  await expect(frame.locator('p')).toHaveAttribute('style', 'color:red');
  await frame.getByText('Où est le café ?').click();
  expect(await frame.locator('body').evaluate(() => {
    try { return typeof Reflect.get(window.parent, '__TAURI_INTERNALS__'); } catch (error) { return (error as Error).name; }
  })).toBe('SecurityError');
  expect(requests).toEqual([]);
  expect(await page.evaluate(() => Object.hasOwn(window, 'pwned'))).toBe(false);
  expect(new URL(page.url()).pathname).toBe('/');
});

test('cancelled lookup cannot replace a newer query with a late body', async ({ page }) => {
  const next = { ...first, entries: [{ ...first.entries[0], key: 'école', html: '<p>new query</p>' }] };
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_lookup: [{ value: first, gate: 'old' }, { value: next }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('double');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '取消查询', exact: true }).click();
  await page.getByLabel('法语词或词组', { exact: true }).fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await expect(page.frameLocator('iframe[title="first.mdx"]').getByText('new query', { exact: true })).toBeVisible();
  await page.evaluate(() => window.testNative.release('old'));
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => resolve())));
  await expect(page.frameLocator('iframe[title="first.mdx"]').getByText('new query', { exact: true })).toBeVisible();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('école');
});

test('a dictionary tab change protects an unsaved independent note', async ({ page }) => {
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_lookup: [{ value: first }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('double');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  await page.getByRole('button', { name: '加入生词本', exact: true }).click();
  await page.getByLabel('独立笔记', { exact: true }).fill('keep this note');
  await page.getByRole('tab', { name: 'second.mdx', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('keep this note');
  await expect(page.getByRole('tab', { name: 'first.mdx', exact: true })).toHaveAttribute('aria-selected', 'true');
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(1);
});

test('dictionary tabs support arrow keys and retain the query', async ({ page }) => {
  const second = { ...first, dictionary_id: 'mdx-2', source: { ...first.source, title: 'second.mdx' } };
  await nativeFixture(page, { dictionary_settings: [{ value: catalog }], dictionary_lookup: [{ value: first }, { value: second }] });
  await page.goto('/');
  await page.getByLabel('法语词或词组', { exact: true }).fill('double');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  const firstTab = page.getByRole('tab', { name: 'first.mdx', exact: true });
  await firstTab.focus(); await firstTab.press('ArrowRight');
  const secondTab = page.getByRole('tab', { name: 'second.mdx', exact: true });
  await expect(secondTab).toHaveAttribute('aria-selected', 'true');
  await expect(secondTab).toBeFocused();
  await secondTab.press('ArrowLeft');
  await expect(firstTab).toHaveAttribute('aria-selected', 'true');
  await expect(firstTab).toBeFocused();
  await expect(page.getByLabel('法语词或词组', { exact: true })).toHaveValue('double');
});
test('releases navigation-held dictionary bodies for a new query or page', async ({ page }) => {
  await page.goto('/');
  const retained = await page.evaluate(async result => {
    const path = '/e2e/fixtures/navigation-probe.tsx';
    const module = await import(path) as { navigationProbe: typeof navigationProbe };
    const probe = module.navigationProbe();
    await probe.ready;
    const context = { key: 'sommes', selected: 'être', dictionaryId: 'mdx-1', dictionaryKey: 'être' };
    try {
      await probe.act(navigation => navigation.openDictionaryResult({ result, ordinal: 1, conjugation: context }));
      await probe.act(navigation => navigation.openConjugation(context));
      const onDeparture = probe.state().dictionary !== undefined;
      await probe.act(navigation => navigation.openDictionaryResult({ result, ordinal: 1, conjugation: context }));
      let searched = false;
      await probe.act(navigation => navigation.beforeSearch(() => { searched = true; }));
      const onSearch = probe.state().dictionary !== undefined;
      await probe.act(navigation => navigation.openDictionaryResult({ result, ordinal: 1, conjugation: context }));
      await probe.act(navigation => navigation.openInput(() => {}));
      return { onDeparture, onSearch, onInput: probe.state().dictionary !== undefined, searched, context: probe.state().conjugation };
    } finally { probe.close(); }
  }, { ...first, entries: [{ ...first.entries[0], key: 'être' }] });
  expect(retained).toMatchObject({ onDeparture: false, onSearch: false, onInput: false, searched: true });
  expect(retained.context).toMatchObject({ key: 'sommes', dictionaryId: 'mdx-1', dictionaryKey: 'être' });
});
