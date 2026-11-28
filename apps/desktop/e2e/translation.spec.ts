import { expect, test } from '@playwright/test';
import { nativeFixture, record } from './native-fixture';

const configured = { preferences: { active_provider: 'deepl', source_language: 'auto', target_language: 'zh-CN', deepl: { tier: 'free' }, openai_compatible: { base_url: '', model: '' } }, keys: { deepl: 'configured', openai_compatible: 'missing' } };
declare global { interface Window { translationPlays: number } }

for (const item of [
  { locale: 'en', source: 'Source text', language: 'Source language', option: 'French', settings: 'Settings', title: 'Services and keys', service: 'OpenAI-compatible' },
  { locale: 'fr', source: 'Texte source', language: 'Langue source', option: 'Français', settings: 'Réglages', title: 'Services et clés', service: 'Service compatible OpenAI' },
] as const) {
  test(`${item.locale} translation controls and settings are localized`, async ({ page }) => {
    await nativeFixture(page, { interface_preferences: [{ value: { locale: item.locale, theme: 'white' } }], translation_settings: [{ value: configured }, { value: configured }] });
    await page.goto('/');
    await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }));
    await expect(page.getByLabel(item.source, { exact: true })).toBeVisible();
    await expect(page.getByLabel(item.language).getByRole('option', { name: item.option })).toHaveCount(1);
    await page.getByRole('button', { name: item.settings, exact: true }).click();
    await expect(page.getByRole('heading', { name: item.title })).toBeVisible();
    await expect(page.getByRole('group', { name: item.service, exact: true })).toBeVisible();
  });
}

test('saved settings failure reports its safe diagnostic', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ error: 'Invalid saved translation preferences: invalid JSON' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByText(/保存的翻译设置已损坏/)).toBeVisible();
});

for (const diagnostic of ['Could not open database: unable to open database file', 'Invalid credentials file', 'Corrupt credentials file: expected value', 'Invalid credential', 'Could not access translation settings: task panicked']) {
  test(`settings failure retains ${diagnostic}`, async ({ page }) => {
    await nativeFixture(page, { translation_settings: [{ error: diagnostic }] });
    await page.goto('/');
    await page.getByRole('button', { name: '设置', exact: true }).click();
    await expect(page.getByText(diagnostic, { exact: false })).toBeVisible();
  });
}

test('control characters in source text and replacement keys report invalid requests before IPC', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }, { value: configured }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'hello\x07world' }));
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect(page.getByText(/请求无效，请检查文本和服务设置/)).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute'))).toHaveLength(0);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByLabel('替换 API 密钥').first().fill('fixture\x07key');
  await page.getByRole('button', { name: '保存服务设置' }).first().click();
  await expect(page.getByText(/服务设置操作失败: 请求无效，请检查文本和服务设置/)).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_settings' && call.args?.replacement))).toHaveLength(0);
});

test('invalid translation response is distinguished from invalid input', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ value: { text: 'Bonjour', source: 'IT', target: 'fr' } }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Hello' }));
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect(page.getByText(/服务返回了无效响应/)).toBeVisible();
  await expect(page.locator('.lookup-input pre')).toHaveCount(0);
});

test('browser preview reports translation as native-only', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await expect(page.getByText('在线翻译仅在桌面应用中可用')).toBeVisible();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByText('在线翻译设置仅在桌面应用中可用。')).toBeVisible();
});

test('completed translation copies plain text and language changes cancel streaming', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const french = { ...configured, preferences: { ...configured.preferences, source_language: 'fr', target_language: 'en' } };
  await nativeFixture(page, { translation_settings: [{ value: french }], translation_execute: [{ value: { text: 'Hello\nworld', source: 'fr', target: 'en' } }, { gate: 'swap', value: { text: 'late', source: 'fr', target: 'en' } }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }));
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByRole('button', { name: '复制译文' }).click();
  await expect.poll(() => page.evaluate(async () => (await navigator.clipboard.readText()).replaceAll('\r\n', '\n'))).toBe('Hello\nworld');
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(2);
  await page.getByRole('button', { name: '互换语言' }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_cancel').length)).toBe(1);
  const saved = await page.evaluate(() => window.testNative.calls.find((call) => call.command === 'translation_settings' && call.args?.preferences)?.args?.preferences);
  expect(saved).toMatchObject({ source_language: 'en', target_language: 'fr' });
  await page.evaluate(() => window.testNative.release('swap'));
  await expect(page.locator('.lookup-input pre')).toHaveCount(0);
});

test('translation navigation honors unsaved notes and Lite reports unavailable speech', async ({ page }) => {
  await nativeFixture(page, { vocabulary_list: [{ value: [record] }], translation_settings: [{ value: { ...configured, preferences: { ...configured.preferences, source_language: 'fr' } } }], speech_preview: [{ error: 'Local speech resources are unavailable' }] });
  await page.goto('/');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  await page.getByLabel('独立笔记', { exact: true }).fill('unsaved');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: '有尚未保存的修改' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: '继续编辑' }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('unsaved');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await dialog.getByRole('button', { name: '放弃修改', exact: true }).click();
  await page.getByLabel('原文', { exact: true }).fill('Bonjour');
  await page.getByRole('button', { name: /播放例句: Bonjour/ }).click();
  await expect(page.locator('.lookup-input [role="alert"]')).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute'))).toHaveLength(0);
});

test('disabled provider has a direct save and incomplete configuration is identified', async ({ page }) => {
  const openai = { ...configured, preferences: { ...configured.preferences, active_provider: 'openai_compatible' } };
  await nativeFixture(page, { translation_settings: [{ value: openai }, { value: openai }, { value: { ...openai, preferences: { ...openai.preferences, active_provider: 'disabled' } } }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }));
  await expect(page.getByText('服务未配置')).toBeVisible();
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const services = page.locator('section').filter({ has: page.getByRole('heading', { name: '服务与密钥' }) });
  await services.getByLabel('当前翻译服务').selectOption('disabled');
  await services.getByRole('button', { name: '保存当前翻译服务' }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_settings' && (call.args?.preferences as { active_provider?: string } | undefined)?.active_provider === 'disabled').length)).toBe(1);
});

test('both services can be configured and tested while translation is disabled', async ({ page }) => {
  const disabled = { ...configured, preferences: { ...configured.preferences, active_provider: 'disabled' } };
  await nativeFixture(page, { translation_settings: [{ value: disabled }], translation_test_connection: [{ value: { host: 'api-free.deepl.com', elapsed_ms: 42 } }, { value: { host: 'localhost', elapsed_ms: 24 } }] });
  await page.goto('/');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const services = page.locator('section').filter({ has: page.getByRole('heading', { name: '服务与密钥' }) });
  await expect(services.getByLabel('服务地址')).toBeVisible();
  await expect(services.getByLabel('DeepL 套餐')).toBeVisible();
  await expect(services.getByRole('button', { name: '测试连接' })).toHaveCount(2);
  await services.getByRole('button', { name: '测试连接' }).first().click();
  await expect(services.getByText(/api-free\.deepl\.com.*42/)).toBeVisible();
  await services.getByLabel('服务地址').fill('http://localhost:1234/v1');
  await services.getByLabel('模型').fill('test-model');
  await services.getByRole('button', { name: '测试连接' }).last().click();
  await expect(services.getByText(/localhost.*24/)).toBeVisible();
  const calls = await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_test_connection').map((call) => call.args));
  expect(calls.map((call) => (call?.preferences as { active_provider: string }).active_provider)).toEqual(['deepl', 'openai_compatible']);
  await services.getByLabel('DeepL 套餐').selectOption('pro');
  await services.getByLabel('替换 API 密钥').first().fill('fixture:fx');
  await expect(services.getByLabel('DeepL 套餐')).toHaveValue('free');
  await services.getByLabel('替换 API 密钥').last().fill('other-slot-fixture');
  await services.getByRole('button', { name: '保存服务设置' }).first().click();
  await expect(services.getByLabel('替换 API 密钥').last()).toHaveValue('other-slot-fixture');
});

test('dictionary lookup waits for an explicit valid query', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }] });
  await page.goto('/');
  await page.evaluate(() => { window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }); });
  const source = page.getByLabel('原文', { exact: true });
  const lookup = page.getByRole('button', { name: '查询输入' });
  await source.fill('a'.repeat(513));
  await expect(lookup).toBeDisabled();
  await source.focus();
  await page.keyboard.press('Home');
  for (let i = 0; i < 3; i++) await page.keyboard.press('Shift+ArrowRight');
  await expect(lookup).toBeEnabled();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup'))).toHaveLength(0);
  await lookup.click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBe(1);
});

test('deselected translation does not leave a stale dictionary query', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ value: { text: 'école', source: 'en', target: 'fr' } }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'a'.repeat(513) }));
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  const target = page.locator('.lookup-input pre');
  await expect(target).toHaveText('école');
  await target.evaluate((node) => { const range = document.createRange(); range.selectNodeContents(node); window.getSelection()!.removeAllRanges(); window.getSelection()!.addRange(range); });
  const lookup = page.getByRole('button', { name: '查询输入' });
  await expect(lookup).toBeEnabled();
  await page.evaluate(() => window.getSelection()!.removeAllRanges());
  await expect(lookup).toBeDisabled();
});

test('auto source remains undetermined while streaming and after a headerless result', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ gate: 'language', value: { text: 'Bonjour', source: 'auto', target: 'fr' } }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Hello' }));
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
  await page.evaluate(() => { const call = window.testNative.calls.find((item) => item.command === 'translation_execute'); window.testNative.emit('translation-progress', { id: call?.args?.id, kind: 'delta', text: 'Bon', source: 'fr', target: 'fr' }); });
  await expect(page.getByLabel('原文', { exact: true })).not.toHaveAttribute('lang', 'fr');
  await expect(page.getByLabel(/播放例句: Hello/)).toHaveCount(0);
  await page.evaluate(() => window.testNative.release('language'));
  await expect(page.locator('.lookup-input pre')).toHaveText('Bonjour');
  await expect(page.getByLabel('原文', { exact: true })).not.toHaveAttribute('lang', 'fr');
  await expect(page.getByLabel(/播放例句: Hello/)).toHaveCount(0);
});

test('remote OpenAI requires a configured key', async ({ page }) => {
  const openai = { ...configured, preferences: { ...configured.preferences, active_provider: 'openai_compatible', openai_compatible: { base_url: 'https://example.com/v1', model: 'model' } } };
  await nativeFixture(page, { translation_settings: [{ value: openai }] });
  await page.goto('/');
  await page.evaluate(() => { window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }); });
  await expect(page.getByText('密钥未配置')).toBeVisible();
  await expect(page.getByRole('button', { name: '前往设置' })).toBeVisible();
  await expect(page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true })).toBeDisabled();
});

test('explicit French source speech is hidden during streaming and incomplete translation', async ({ page }) => {
  const french = { ...configured, preferences: { ...configured.preferences, source_language: 'fr' } };
  await nativeFixture(page, { translation_settings: [{ value: french }], translation_execute: [{ gate: 'incomplete-source', error: 'network_or_timeout' }] });
  await page.goto('/');
  await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }));
  await expect(page.getByRole('button', { name: /播放例句: Bonjour/ })).toBeVisible();
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
  await page.evaluate(() => { const call = window.testNative.calls.find((item) => item.command === 'translation_execute'); window.testNative.emit('translation-progress', { id: call?.args?.id, kind: 'delta', text: 'Hello', source: 'fr', target: 'en' }); });
  await expect(page.getByRole('button', { name: /播放例句: Bonjour/ })).toHaveCount(0);
  await expect(page.getByLabel('原文', { exact: true })).toHaveAttribute('lang', 'fr');
  await page.evaluate(() => window.testNative.release('incomplete-source'));
  await expect(page.getByText('译文不完整，请重试。')).toBeVisible();
  await expect(page.getByRole('button', { name: /播放例句: Bonjour/ })).toHaveCount(0);
  await expect(page.getByLabel('原文', { exact: true })).toHaveAttribute('lang', 'fr');
});

test('captured text is sent only after Translate and editing cancels the request', async ({ page }) => {
  await nativeFixture(page, { translation_settings: [{ value: configured }], translation_execute: [{ gate: 'translation', value: { text: '迟到译文', source: 'fr', target: 'zh-CN' } }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour\nmonde' });
  });
  await expect(page.getByLabel('原文', { exact: true })).toHaveValue('Bonjour\nmonde');
  await expect(page.getByText('DeepL (Free · api-free.deepl.com)')).toBeVisible();
  expect(await page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute'))).toHaveLength(0);
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
  await page.getByLabel('原文', { exact: true }).fill('nouveau');
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_cancel').length)).toBe(1);
  await page.evaluate(() => window.testNative.release('translation'));
  await expect(page.getByText('迟到译文')).toHaveCount(0);
});

for (const action of ['cancel', 'edit', 'language'] as const) {
  test(`translation cancellation failure stays visible after ${action}`, async ({ page }) => {
    await nativeFixture(page, {
      translation_settings: [{ value: configured }],
      translation_execute: [{ gate: 'cancelled-translation', value: { text: '迟到译文', source: 'fr', target: 'zh-CN' } }],
      translation_cancel: [{ error: 'Could not cancel translation' }],
    });
    await page.goto('/');
    await page.evaluate(() => window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' }));
    await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'translation_execute').length)).toBe(1);
    if (action === 'cancel') await page.getByRole('button', { name: '取消', exact: true }).click();
    else if (action === 'edit') await page.getByLabel('原文', { exact: true }).fill('nouveau');
    else await page.getByLabel('原文语言').selectOption('fr');
    await expect(page.getByRole('status').filter({ hasText: 'Could not cancel translation' })).toBeVisible();
    await page.evaluate(() => window.testNative.release('cancelled-translation'));
    await expect(page.getByText('迟到译文')).toHaveCount(0);
    await expect(page.getByRole('button', { name: '取消', exact: true })).toHaveCount(0);
  });
}

test('streamed text remains incomplete after a disconnect and cannot be copied', async ({ page }) => {
  const openai = { ...configured, preferences: { ...configured.preferences, active_provider: 'openai_compatible', openai_compatible: { base_url: 'http://localhost:11434/v1', model: 'test' } } };
  await nativeFixture(page, { translation_settings: [{ value: openai }], translation_execute: [{ gate: 'stream', error: 'network_or_timeout' }] });
  await page.goto('/');
  await page.evaluate(() => {
    window.testNative.emit('selection-input', { kind: 'pending', request: 1 });
    window.testNative.emit('selection-input', { kind: 'captured', request: 1, text: 'Bonjour' });
  });
  await expect(page.getByText('OpenAI 兼容服务 · localhost:11434 · test')).toBeVisible();
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
  const source = page.getByLabel('原文', { exact: true });
  await expect(source).toHaveValue('school house');
  await source.focus();
  await page.keyboard.press('Home');
  for (let i = 0; i < 6; i++) await page.keyboard.press('Shift+ArrowRight');
  await page.getByRole('button', { name: '查询输入' }).click();
  await expect.poll(() => page.evaluate(() => window.testNative.calls.filter((call) => call.command === 'dictionary_lookup').length)).toBeGreaterThan(0);
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  const target = page.locator('.lookup-input pre');
  await expect(target).toHaveText('école française');
  await expect(target).toHaveAttribute('lang', 'fr');
  await target.evaluate((node) => { const range = document.createRange(); range.setStart(node.firstChild!, 0); range.setEnd(node.firstChild!, 5); const selection = window.getSelection()!; selection.removeAllRanges(); selection.addRange(range); node.dispatchEvent(new MouseEvent('mouseup', { bubbles: true })); });
  await page.getByRole('button', { name: '查询输入' }).click();
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
  const source = page.getByLabel('原文', { exact: true });
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
