import { expect, test } from '@playwright/test';
import { nativeFixture } from './native-fixture';

const configured = { preferences: { active_provider: 'deepl', source_language: 'auto', target_language: 'zh-CN', deepl: { tier: 'free' }, openai_compatible: { base_url: '', model: '' } }, keys: { deepl: 'configured', openai_compatible: 'missing' } };

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
  await expect(page.getByRole('button', { name: '复制译文' })).toBeDisabled();
  await page.evaluate(() => window.testNative.release('stream'));
  await expect(page.getByText('译文不完整，请重试。')).toBeVisible();
  await expect(page.getByRole('button', { name: '重试' })).toBeVisible();
  await expect(page.getByRole('button', { name: '复制译文' })).toBeDisabled();
});
