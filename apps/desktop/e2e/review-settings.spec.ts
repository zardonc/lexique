import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

test('review settings save and reopen', async ({ page }) => {
  await nativeFixture(page);
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('目标保持率 (%)')).toHaveValue('90');
  await page.getByLabel('目标保持率 (%)').fill('80');
  await page.getByLabel('每日新卡上限').fill('0');
  await page.getByLabel('每日复习作答上限').fill('12');
  await page.getByRole('button', { name: '保存复习设置' }).click();
  await expect(page.getByText('已保存', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('目标保持率 (%)')).toHaveValue('80');
  await expect(page.getByLabel('每日新卡上限')).toHaveValue('0');
});

test('review settings retain unsaved values after a failed save', async ({ page }) => {
  await nativeFixture(page, { review_settings: [
    { value: { retention_percent: 90, new_limit: 20, review_limit: 50 } },
    { value: { retention_percent: 90, new_limit: 20, review_limit: 50 } },
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
