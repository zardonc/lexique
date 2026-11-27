import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const configured = { preferences: { active_provider: 'deepl', source_language: 'auto', target_language: 'zh-CN', deepl: { tier: 'free' }, openai_compatible: { base_url: '', model: '' } }, keys: { deepl: 'configured', openai_compatible: 'missing' } };
declare global { interface Window { translationPlays: number } }

test('captured text is sent only after Translate and editing cancels the request', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ gate: 'translation', value: { text: '迟到译文', source: 'fr', target: 'zh-CN' } }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour\nmonde' });
  });
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('Bonjour\nmonde');
  await expect(page.getByText('DeepL (Free · api-free.deepl.com)')).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute'))).toHaveLength(0);
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
  await page.getByLabel('法语原文', { exact: true }).fill('nouveau');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_cancel').length)).toBe(1);
  await page.evaluate(() => window.testNative.release('translation'));
  await expect(page.getByText('迟到译文')).toHaveCount(0);
});

test('streamed text remains incomplete after a disconnect and cannot be copied', async ({ page }) => {
  const openai = { ...configured, preferences: { ...configured.preferences, active_provider: 'openai_compatible', openai_compatible: { base_url: 'http://localhost:11434/v1', model: 'test' } } };
  await nativeFixture(page, { translation_settings: [{ value: openai }], translation_execute: [{ gate: 'stream', error: 'network_or_timeout' }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' });
  });
  await expect(page.getByText('OpenAI-compatible · localhost:11434 · test')).toBeVisible();
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
  await page.evaluate(() => {
    const call = window.testNative.calls.find((item) => item.command === 'translation_execute');
    window.testNative.emit('translation-progress', { id: call?.args?.id, kind: 'delta', text: 'Hello', source: 'fr', target: 'en' });
  });
  await expect(page.getByText('Hello', { exact: true })).toBeVisible();
  await page.locator('.lookup-input pre').evaluate((node) => { const range = document.createRange(); range.selectNodeContents(node); const selection = window.getSelection()!; selection.removeAllRanges(); selection.addRange(range); node.dispatchEvent(new MouseEvent('mouseup', { bubbles: true })); });
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup'))).toHaveLength(0);
  await expect(page.getByRole('button', { name: '复制译文' })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('stream'));
  await expect(page.getByText('译文不完整，请重试。')).toBeVisible();
  await expect(page.getByRole('button', { name: '重试' })).toBeVisible();
  await expect(page.getByRole('button', { name: '复制译文' })).toBeDisabled();
});

test('looks up selections in both panes and preserves a completed translation', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ value: { text: 'école française', source: 'en', target: 'fr' } }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'school house' });
  });
  const source = page.getByLabel('法语原文', { exact: true });
  await expect(source).toHaveValue('school house');
  await source.focus();
  await page.keyboard.press('Home');
  for (let i = 0; i < 6; i++) await page.keyboard.press('Shift+ArrowRight');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBeGreaterThan(0);
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  const target = page.locator('.lookup-input pre');
  await expect(target).toHaveText('école française');
  await expect(target).toHaveAttribute('lang', 'fr');
  await target.evaluate((node) => { const range = document.createRange(); range.setStart(node.firstChild!, 0); range.setEnd(node.firstChild!, 5); const selection = window.getSelection()!; selection.removeAllRanges(); selection.addRange(range); node.dispatchEvent(new MouseEvent('mouseup', { bubbles: true })); });
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBeGreaterThan(1);
  await expect(page.getByText('学校', { exact: true }).first()).toBeVisible();
  await expect(page.getByLabel('播放例句: école française (机器合成语音)')).toBeVisible();
  await page.getByRole('button', { name: '打开完整词典结果' }).click();
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await expect(source).toHaveValue('school house');
  await expect(target).toHaveText('école française');
});

test('French speech respects the length limit and ignores a preview after editing', async ({ page }) => {
  await page.addInitScript(() => { window.translationPlays = 0; HTMLMediaElement.prototype.play = function () { window.translationPlays++; return Promise.resolve(); }; });
  const french = { ...configured, preferences: { ...configured.preferences, source_language: 'fr' } };
  await nativeFixture(page, { translation_settings: [{ value: french }], speech_preview: [{ gate: 'speech', value: { bytes: [79, 103, 103, 83], voice: 'female' } }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' });
  });
  const source = page.getByLabel('法语原文', { exact: true });
  await expect(source).toHaveValue('Bonjour');
  await page.getByRole('button', { name: /播放例句: Bonjour/ }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').length)).toBe(1);
  await source.fill('é'.repeat(201));
  await expect(page.getByText(/朗读仅支持规范化后最多 200/)).toBeVisible();
  await expect(page.getByRole('button', { name: /播放例句:/ })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('speech'));
  await page.waitForTimeout(100);
  expect(await page.evaluate(() => window.translationPlays)).toBe(0);
  await expect(page.getByRole('button', { name: /播放例句:/ })).toBeDisabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'speech_preview').map((call) => call.args))).toEqual([{ text: 'Bonjour' }]);
});
