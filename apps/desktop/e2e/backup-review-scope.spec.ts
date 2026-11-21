import { expect, test } from '@playwright/test';
import type { ReviewScope } from '../src/platform/review-scope';
import { nativeFixtureSetup } from './native-fixture';
import { confirmRestore, previewReady, restarting, selectBackup } from './backup-restore-fixture';

const cases: readonly { readonly name: string; readonly scope: ReviewScope; readonly label: string }[] = [
  { name: 'All reviews', scope: { kind: 'all' }, label: '全部复习' },
  { name: 'single category identity', scope: { kind: 'vocabulary', category_ids: [8], include_uncategorized: false }, label: 'École' },
  { name: 'category union', scope: { kind: 'vocabulary', category_ids: [7, 8], include_uncategorized: false }, label: 'Voyage, École' },
  { name: 'Uncategorized', scope: { kind: 'vocabulary', category_ids: [], include_uncategorized: true }, label: '未分类' },
  { name: 'category and Uncategorized', scope: { kind: 'vocabulary', category_ids: [7], include_uncategorized: true }, label: 'Voyage, 未分类' },
  { name: 'needs selection', scope: { kind: 'needs_selection' }, label: '请选择复习范围' },
];

for (const item of cases) {
  test(`full backup restore restart displays ${item.name} with backed-up categories`, async ({ page }) => {
    await nativeFixtureSetup(page, ({ scope, preview, restart }) => {
      const native = Reflect.get(window, '__TAURI_INTERNALS__') as {
        invoke: (command: string, args?: Record<string, unknown>, options?: unknown) => Promise<unknown>;
      };
      const invoke = native.invoke;
      const restored = () => sessionStorage.getItem('test-scope-restored') === 'yes';
      const applied = () => restored() ? scope : { kind: 'vocabulary', category_ids: [8], include_uncategorized: false };
      native.invoke = async (command, args, options) => {
        if (['review_scope', 'learning_categories'].includes(command) || command.startsWith('backup_')) {
          window.testNative.calls.push({ command, args });
        }
        if (command === 'review_scope') return applied();
        if (command === 'learning_categories') return restored()
          ? [{ id: 7, name: 'Voyage' }, { id: 8, name: 'École' }] : [{ id: 8, name: 'Destination' }];
        if (['backup_preview_start', 'backup_preview_status'].includes(command)) return preview;
        if (command === 'backup_restore_start') {
          sessionStorage.setItem('test-scope-restored', 'yes');
          return restart;
        }
        if (command === 'backup_restore_status') return restart;
        const response = await invoke(command, args, options);
        if (command === 'review_queue') {
          const view = response as { readonly summary: Record<string, unknown> };
          return { ...view, summary: { ...view.summary, scope: applied() } };
        }
        return response;
      };
    }, { scope: item.scope, preview: previewReady, restart: restarting });
    await page.goto('/');
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    await expect(page.getByText('当前范围：Destination', { exact: true })).toBeVisible();
    await selectBackup(page);
    await confirmRestore(page);
    await expect(page.getByRole('status')).toContainText('已请求重启');
    await page.reload();
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    await expect(page.getByText(`当前范围：${item.label}`, { exact: true })).toBeVisible();
    const selector = page.getByRole('group', { name: '复习范围', exact: true });
    await expect(selector.getByRole('checkbox', { name: 'Destination', exact: true })).toHaveCount(0);
    for (const category of [{ id: 7, name: 'Voyage' }, { id: 8, name: 'École' }]) {
      const option = selector.getByRole('checkbox', { name: category.name, exact: true });
      if (item.scope.kind === 'vocabulary' && item.scope.category_ids.includes(category.id)) await expect(option).toBeChecked();
      else await expect(option).not.toBeChecked();
    }
    expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'review_scope' && call.args?.scope))).toEqual([]);
  });
}
