import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('startup reads saved interface language and theme together', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ value: { locale: 'fr', theme: 'dark' } }] });
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Dictionnaire', exact: true })).toBeVisible();
  await expect(page.getByLabel('Langue de l’interface')).toHaveValue('fr');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'dark');
  await expect(page.locator('html')).toHaveAttribute('lang', 'fr');
});

test('failed startup reads disable choices and allow a keyboard retry', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [{ error: 'Settings disk temporarily unavailable' }] });
  await page.goto('/');
  await expect(page.getByRole('banner').getByRole('alert')).toContainText('Settings disk temporarily unavailable');
  await expect(page.getByLabel('界面语言')).toBeDisabled();
  const retry = page.getByRole('button', { name: '重试界面偏好', exact: true });
  await expect(retry).toBeEnabled();
  await page.evaluate(() => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => command === 'interface_preferences' ? { locale: 'en', theme: 'sepia' } : invoke(command, args);
  });
  await retry.focus(); await retry.press('Enter');
  await expect(page.getByRole('heading', { name: 'Dictionary', exact: true })).toBeVisible();
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'sepia');
  await expect(page.getByLabel('Interface language')).toBeEnabled();
  await expect(page.getByRole('banner').getByRole('alert')).toHaveCount(0);
});

test('a pending save keeps the confirmed language and blocks another choice', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [
    { value: { locale: 'zh', theme: 'white' } }, { value: { locale: 'zh', theme: 'white' } },
    { gate: 'interface-save', value: { locale: 'en', theme: 'white' } },
  ] });
  await page.goto('/');
  await page.getByLabel('界面语言').selectOption('en');
  await expect(page.getByLabel('界面语言')).toHaveValue('zh');
  await expect(page.getByLabel('界面语言')).toBeDisabled();
  await expect(page.getByLabel('显示主题')).toBeDisabled();
  await expect(page.getByRole('banner')).toContainText('正在保存界面偏好');
  await page.evaluate(() => window.testNative.release('interface-save'));
  await expect(page.getByRole('heading', { name: 'Dictionary', exact: true })).toBeVisible();
  await expect(page.getByLabel('Interface language')).toBeEnabled();
  await expect(page.getByRole('banner')).toContainText('Interface preferences saved.');
});

test('a failed save preserves the confirmed pair and retries the requested pair', async ({ page }) => {
  await nativeFixture(page, { interface_preferences: [
    { value: { locale: 'zh', theme: 'white' } }, { value: { locale: 'zh', theme: 'white' } },
    { error: 'Interface database is read-only' }, { value: { locale: 'en', theme: 'white' } },
  ] });
  await page.goto('/');
  await page.getByLabel('界面语言').selectOption('en');
  await expect(page.getByRole('banner').getByRole('alert')).toContainText('Interface database is read-only');
  await expect(page.getByLabel('界面语言')).toHaveValue('zh');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'white');
  const retry = page.getByRole('button', { name: '重试界面偏好', exact: true });
  await retry.focus(); await retry.press('Enter');
  await expect(page.getByRole('heading', { name: 'Dictionary', exact: true })).toBeVisible();
  await expect(page.getByLabel('Display theme')).toHaveValue('white');
  await expect(page.getByRole('banner').getByRole('alert')).toHaveCount(0);
});

test('a reply that differs from the requested pair is not confirmed', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByLabel('界面语言').selectOption('en');
  await page.evaluate(() => {
    const native = Reflect.get(window, '__TAURI_INTERNALS__') as { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
    const invoke = native.invoke;
    native.invoke = async (command, args) => command === 'interface_preferences' ? { locale: 'en', theme: 'white' } : invoke(command, args);
  });
  await page.getByLabel('Display theme').selectOption('dark');
  await expect(page.getByRole('banner').getByRole('alert')).toContainText('Saved interface preferences differ from the request');
  await expect(page.getByLabel('Display theme')).toHaveValue('white');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'white');
});

test('native interface controls accept every language and theme with accessible labels', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  for (const [locale, label, tag] of [['zh', '界面语言', 'zh-CN'], ['en', 'Interface language', 'en'], ['fr', 'Langue de l’interface', 'fr']]) {
    await page.getByRole('banner').getByRole('combobox').nth(0).selectOption(locale);
    await expect(page.getByLabel(label)).toHaveValue(locale);
    await expect(page.locator('html')).toHaveAttribute('lang', tag);
    for (const theme of ['white', 'sepia', 'dark']) {
      await page.getByRole('banner').getByRole('combobox').nth(1).selectOption(theme);
      await expect(page.locator('.app')).toHaveAttribute('data-theme', theme);
    }
  }
});

test('browser preview explains its temporary choices and resets on reload', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('banner')).toContainText('语言和主题仅在本次预览中使用，不会保存');
  await page.getByLabel('界面语言').selectOption('fr');
  await page.getByLabel('Thème d’affichage').selectOption('sepia');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'sepia');
  await expect(page.getByRole('banner')).toContainText('la langue et le thème sont temporaires');
  await page.reload();
  await expect(page.getByLabel('界面语言')).toHaveValue('zh');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'white');
});

for (const [name, value] of [
  ['unknown language', { locale: 'de', theme: 'white' }],
  ['unknown theme', { locale: 'zh', theme: 'blue' }],
  ['unknown field', { locale: 'zh', theme: 'white', path: 'external' }],
  ['missing theme', { locale: 'zh' }],
] as const) {
  test(`an invalid saved interface reply with ${name} stays visibly unavailable`, async ({ page }) => {
    await nativeFixture(page, { interface_preferences: [{ value }] });
    await page.goto('/');
    await expect(page.getByRole('banner').getByRole('alert')).toBeVisible();
    await expect(page.getByLabel('界面语言')).toBeDisabled();
    await expect(page.getByRole('button', { name: '重试界面偏好', exact: true })).toBeEnabled();
    await expect(page.getByRole('banner')).not.toContainText('界面偏好已保存');
  });
}

test('the runtime rejects invalid interface requests before native IPC', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  const accepted = await page.evaluate(async () => {
    const path = '/src/platform/runtime.ts';
    const { runtime } = await import(path);
    const accepted: unknown[] = [];
    for (const value of [{ locale: 'de', theme: 'white' }, { locale: 'zh', theme: 'blue' }, { locale: 'zh', theme: 'white', path: 'external' }]) {
      try { await runtime.interfacePreferences(value); accepted.push(value); } catch { /* Rejected at the public request boundary. */ }
    }
    return accepted;
  });
  expect(accepted).toEqual([]);
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'interface_preferences' && call.args?.preferences !== null))).toEqual([]);
});
