import { expect, test } from '@playwright/test';

test('shell navigation, themes and mobile layout remain usable', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '词典', exact: true })).toBeVisible();
  await expect(page.getByRole('status')).toContainText('浏览器预览');
  await page.getByLabel('界面语言').selectOption('fr');
  await expect(page.getByRole('heading', { name: 'Dictionnaire', exact: true })).toBeVisible();
  await page.getByLabel('Langue de l’interface').selectOption('zh');
  await page.getByLabel('显示主题').selectOption('sepia');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'sepia');
  await page.getByLabel('显示主题').selectOption('dark');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'dark');
  await page.getByRole('button', { name: '收起导航' }).click();
  await expect(page.getByRole('button', { name: '展开导航' })).toBeVisible();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  await expect(page.getByRole('heading', { name: '闪卡复习', exact: true })).toBeVisible();
  await expect(page.getByText('浏览器预览无法读取复习队列。')).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByRole('heading', { name: '设置', exact: true })).toBeVisible();
  await expect(page.getByText('Baidu', { exact: false })).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(errors).toEqual([]);
  await page.screenshot({ path: 'test-results/shell-mobile.png', fullPage: true });
});

test('desktop white theme preserves the darker navigation', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('.app')).toHaveAttribute('data-theme', 'white');
  const colors = await page.locator('.sidebar').evaluate((sidebar) => ({
    sidebar: getComputedStyle(sidebar).backgroundColor,
    main: getComputedStyle(document.querySelector('main')!).backgroundColor,
  }));
  expect(colors.sidebar).not.toBe(colors.main);
  await page.screenshot({ path: 'test-results/shell-desktop.png', fullPage: true });
});
