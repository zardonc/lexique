/* global process, window */
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected resources, native test host and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((path) => resolve(path));
let status = 200, body = { translations: [{ text: '钢琴', detected_source_language: 'IT' }] };
const requests = [];
const server = createServer((request, response) => {
  assert.equal(request.url, '/v2/translate');
  assert.equal(request.headers.authorization, 'DeepL-Auth-Key fixture-secret');
  let data = '';
  request.on('data', (chunk) => { data += chunk; });
  request.on('end', () => {
    requests.push(JSON.parse(data));
    response.writeHead(status, { 'Content-Type': 'application/json' });
    response.end(JSON.stringify(body));
  });
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const priorEndpoint = process.env.LEXIQUE_TEST_DEEPL_ENDPOINT;
process.env.LEXIQUE_TEST_DEEPL_ENDPOINT = `http://127.0.0.1:${server.address().port}/v2/translate`;
let fixture;
try {
  fixture = await native24Fixture(resources, binary, destination, ['translation_deepl::tests::native_translation_host', '--exact', '--ignored', '--nocapture']);
  const page = await fixture.start();
  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const initial = await invoke('translation_settings', { locale: 'zh', preferences: null, replacement: null });
    await invoke('translation_settings', { locale: 'zh', preferences: { ...initial.preferences, active_provider: 'deepl', source_language: 'auto', target_language: 'zh-CN' }, replacement: { provider: 'deepl', key: 'fixture-secret' } });
  });
  const result = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('translation_execute', { id: 'deepl-auto', text: 'piano', locale: 'zh' }));
  assert.deepEqual(result, { text: '钢琴', source: 'auto', target: 'zh-CN' });
  assert.deepEqual(requests[0], { text: ['piano'], target_lang: 'ZH-HANS' });
  const errors = [];
  for (const [httpStatus, expected] of [[401, 'auth_failed'], [403, 'auth_failed'], [429, 'quota_or_rate_limited'], [456, 'quota_or_rate_limited'], [400, 'invalid_request']]) {
    status = httpStatus;
    const error = await page.evaluate(async () => {
      try { await window.__TAURI_INTERNALS__.invoke('translation_execute', { id: 'deepl-error', text: 'piano', locale: 'zh' }); }
      catch (cause) { return String(cause); }
      return 'unexpected_success';
    });
    assert.equal(error, expected);
    errors.push({ status, error });
  }
  status = 200; body = { translations: [{ text: '你好', detected_source_language: 'FR' }] };
  const connection = await page.evaluate(async () => {
    const initial = await window.__TAURI_INTERNALS__.invoke('translation_settings', { locale: 'zh', preferences: null, replacement: null });
    return window.__TAURI_INTERNALS__.invoke('translation_test_connection', { preferences: initial.preferences, replacement: null });
  });
  assert.equal(connection.host, 'api-free.deepl.com');
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('translation_clear_key', { provider: 'deepl' }));
  await writeFile(join(destination, 'translation-result.json'), JSON.stringify({ nativeIpc: true, mockDeepl: true, result, errors, connection }));
  await fixture.stop(true);
} finally {
  if (fixture) await fixture.finish();
  if (priorEndpoint === undefined) delete process.env.LEXIQUE_TEST_DEEPL_ENDPOINT;
  else process.env.LEXIQUE_TEST_DEEPL_ENDPOINT = priorEndpoint;
  await new Promise((done) => server.close(done));
}
