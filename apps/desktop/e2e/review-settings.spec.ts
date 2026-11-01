import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('review settings save and reopen', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('目标保持率 (%)')).toHaveValue('90');
  await page.getByLabel('目标保持率 (%)').fill('80');
  await page.getByLabel('每日新卡上限').fill('0');
  await page.getByLabel('每日复习卡上限').fill('12');
  await page.getByLabel('新卡不受复习上限限制').check();
  await page.getByRole('button', { name: '保存复习设置' }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('目标保持率 (%)')).toHaveValue('80');
  await expect(page.getByLabel('每日新卡上限')).toHaveValue('0');
  await expect(page.getByLabel('新卡不受复习上限限制')).toBeChecked();
});

test('review settings retain unsaved values after a failed save', async ({ page }) => {
  await nativeFixture(page, { review_settings: [
    { value: { retention_percent: 90, new_limit: 20, review_limit: 50, new_cards_ignore_review_limit: false } },
    { value: { retention_percent: 90, new_limit: 20, review_limit: 50, new_cards_ignore_review_limit: false } },
    { error: 'disk is read-only' },
  ] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('每日新卡上限')).toBeVisible();
  await page.getByLabel('每日新卡上限').fill('0');
  await page.getByRole('button', { name: '保存复习设置' }).click();
  await expect(page.getByRole('alert')).toContainText('disk is read-only');
  await expect(page.getByLabel('每日新卡上限')).toHaveValue('0');
});
