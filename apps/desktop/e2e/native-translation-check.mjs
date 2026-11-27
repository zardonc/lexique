/* global process, window, setTimeout */
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected package, binary and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((path) => resolve(path));
const server = createServer((request, response) => {
  if (request.url === '/slow/v1/chat/completions') {
    response.writeHead(200, { 'Content-Type': 'text/event-stream' });
    response.flushHeaders();
    return;
  }
  assert.equal(request.url, '/v1/chat/completions');
  response.writeHead(200, { 'Content-Type': 'text/event-stream' });
  response.end('data: {"choices":[{"delta":{"reasoning_content":"private","content":"SOURCE=fr;TARGET=en\\nHello"}}]}\n\ndata: [DONE]\n\n');
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
let fixture;
try {
  fixture = await native24Fixture(resources, binary, destination);
  const page = await fixture.start();
  const baseUrl = `http://127.0.0.1:${server.address().port}/v1`;
  const result = await page.evaluate(async (url) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const kinds = [];
    const handler = window.__TAURI_INTERNALS__.transformCallback((event) => { if (event.payload.id === 'native-test') kinds.push(event.payload.kind); });
    const eventId = await invoke('plugin:event|listen', { event: 'translation-progress', target: { kind: 'Any' }, handler });
    const initial = await invoke('translation_settings', { locale: 'en', preferences: null, replacement: null });
    const preferences = { ...initial.preferences, active_provider: 'openai_compatible', source_language: 'fr', target_language: 'en', openai_compatible: { base_url: url, model: 'fixture' } };
    let rejected = false;
    try { await invoke('translation_settings', { locale: 'en', preferences: { ...preferences, openai_compatible: { base_url: 'https://user:secret@example.com/v1', model: 'fixture' } }, replacement: null }); }
    catch { rejected = true; }
    if (!rejected) throw new Error('URL credentials were accepted');
    const saved = await invoke('translation_settings', { locale: 'en', preferences, replacement: { provider: 'openai_compatible', key: 'fixture-secret' } });
    await invoke('translation_test_connection', { preferences, replacement: null });
    const translation = await invoke('translation_execute', { id: 'native-test', text: 'Bonjour', locale: 'en' });
    for (let i = 0; i < 40 && !kinds.includes('completed'); i++) await new Promise((done) => setTimeout(done, 25));
    await invoke('plugin:event|unlisten', { event: 'translation-progress', eventId });
    const slow = { ...preferences, openai_compatible: { ...preferences.openai_compatible, base_url: url.replace('/v1', '/slow/v1') } };
    await invoke('translation_settings', { locale: 'en', preferences: slow, replacement: null });
    const started = Date.now();
    let timeoutError = '';
    try { await invoke('translation_execute', { id: 'native-timeout', text: 'Bonjour', locale: 'en' }); }
    catch (error) { timeoutError = String(error); }
    const timeoutMs = Date.now() - started;
    await invoke('translation_settings', { locale: 'en', preferences, replacement: null });
    return { saved, translation, kinds, timeoutError, timeoutMs };
  }, baseUrl);
  assert.equal(result.saved.keys.openai_compatible, 'configured');
  assert.deepEqual(result.translation, { text: 'Hello', source: 'fr', target: 'en' });
  assert.deepEqual(result.kinds, ['started', 'delta', 'completed']);
  assert.equal(result.timeoutError, 'network_or_timeout');
  assert.ok(result.timeoutMs >= 29_000 && result.timeoutMs < 36_000);
  const ciphertext = await readFile(join(destination, 'Lexique-data/config/service-credentials.json'), 'utf8');
  assert.ok(!ciphertext.includes('fixture-secret'));
  await fixture.stop(true);
  const restarted = await fixture.start();
  const state = await restarted.evaluate(() => window.__TAURI_INTERNALS__.invoke('translation_settings', { locale: 'en', preferences: null, replacement: null }));
  assert.equal(state.keys.openai_compatible, 'configured');
  assert.equal(state.preferences.openai_compatible.base_url, baseUrl);
  await restarted.evaluate(() => window.__TAURI_INTERNALS__.invoke('translation_clear_key', { provider: 'openai_compatible' }));
  const cleared = await restarted.evaluate(() => window.__TAURI_INTERNALS__.invoke('translation_settings', { locale: 'en', preferences: null, replacement: null }));
  assert.equal(cleared.keys.openai_compatible, 'missing');
  await writeFile(join(destination, 'translation-result.json'), JSON.stringify({ nativeIpc: true, sse: true, dpapi: true, restart: true, clear: true }));
  await fixture.stop(true);
} finally {
  if (fixture) await fixture.finish();
  await new Promise((done) => server.close(done));
}
