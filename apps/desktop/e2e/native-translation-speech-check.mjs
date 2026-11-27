/* global process, window, document, HTMLMediaElement, MouseEvent */
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected package, binary and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((path) => resolve(path));
const server = createServer((request, response) => {
  assert.equal(request.url, '/v1/chat/completions');
  response.writeHead(200, { 'Content-Type': 'text/event-stream' });
  response.end('data: {"choices":[{"delta":{"content":"SOURCE=zh-CN;TARGET=fr\\nbonjour"}}]}\n\ndata: [DONE]\n\n');
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
let fixture;
try {
  fixture = await native24Fixture(resources, binary, destination);
  const page = await fixture.start();
  await page.evaluate(async (url) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const initial = await invoke('translation_settings', { locale: 'zh', preferences: null, replacement: null });
    await invoke('translation_settings', { locale: 'zh', replacement: null, preferences: { ...initial.preferences,
      active_provider: 'openai_compatible', source_language: 'zh-CN', target_language: 'fr',
      openai_compatible: { base_url: url, model: 'fixture' } } });
    window.translationAudioPlays = 0;
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () { window.translationAudioPlays++; return play.call(this); };
  }, `http://127.0.0.1:${server.address().port}/v1`);
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('法语原文', { exact: true }).fill('你好');
  await page.locator('.lookup-input').getByRole('button', { name: '翻译', exact: true }).click();
  const output = page.locator('.lookup-input pre');
  await output.waitFor();
  assert.equal(await output.textContent(), 'bonjour');
  await output.evaluate((node) => {
    const range = document.createRange(); range.selectNodeContents(node);
    const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
    node.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
  });
  await page.locator('.lookup-input .dictionary-original').waitFor({ timeout: 10_000 });
  await page.getByRole('button', { name: /播放例句: bonjour/ }).click();
  await page.waitForFunction(() => window.translationAudioPlays > 0, undefined, { timeout: 20_000 });
  await fixture.stop(true);
} finally {
  if (fixture) await fixture.finish();
  await new Promise((done) => server.close(done));
}
