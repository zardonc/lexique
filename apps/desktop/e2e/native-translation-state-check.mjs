/* global process, window */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { join, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected package, binary and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((path) => resolve(path));
let held;
const server = createServer((request, response) => {
  assert.equal(request.url, '/v1/chat/completions');
  response.writeHead(200, { 'Content-Type': 'text/event-stream' });
  response.write('data: {"choices":[{"delta":{"content":"SOURCE=fr;TARGET=en\\nHello\\n"}}]}\n\n');
  held = response;
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
let fixture, page;
const invoke = (command, args = {}) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
function picker(filename, filenameId) {
  const args = ['-NoProfile', '-NonInteractive', '-File', resolve('e2e/native-input-helper.ps1'), '-SourcePid', String(fixture.hostPid()), '-SourceExecutable', join(destination, 'Lexique.exe')];
  const run = (extra) => {
    const result = spawnSync('pwsh', [...args, ...extra], { windowsHide: true, encoding: 'utf8', timeout: 8000 });
    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(result.stdout);
  };
  const controls = run(['-Action', 'picker-controls']);
  assert.equal(controls.pid, fixture.hostPid());
  assert.equal(controls.controls.filter((control) => control.class === 'Edit' && control.id === filenameId && control.visible && control.enabled).length, 1);
  const result = run(['-Action', 'picker-select', '-Text', filename, '-FilenameControlId', String(filenameId)]);
  assert.equal(result.dialog, controls.dialog);
  assert.equal(result.disappeared, true);
}
try {
  fixture = await native24Fixture(resources, binary, destination);
  page = await fixture.start();
  const initial = await invoke('translation_settings', { locale: 'en' });
  const preferences = { ...initial.preferences, active_provider: 'openai_compatible', source_language: 'fr', target_language: 'en', openai_compatible: { base_url: `http://127.0.0.1:${server.address().port}/v1`, model: 'fixture' } };
  await invoke('translation_settings', { locale: 'en', preferences, replacement: { provider: 'deepl', key: 'deepl-fixture:fx' } });
  await invoke('translation_settings', { locale: 'en', preferences, replacement: { provider: 'openai_compatible', key: 'before-backup' } });
  const backup = join(destination, 'translation.lexique-backup');
  const exported = await invoke('backup_export_start');
  picker(backup, 1001);
  await expect.poll(async () => (await invoke('backup_operation_status', { operation: exported.operation })).state, { timeout: 10000 }).toBe('completed');
  const archive = spawnSync('python', ['-B', '-X', 'utf8', '-c', "import zipfile,json,sys; z=zipfile.ZipFile(sys.argv[1]); print(json.dumps({'entries':z.namelist(),'snapshot':json.loads(z.read('snapshot.json'))}))", backup], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
  assert.equal(archive.status, 0, archive.stderr);
  assert.ok(!archive.stdout.includes('translation-preferences'));
  assert.ok(!archive.stdout.includes('before-backup'));
  assert.ok(!archive.stdout.includes('service-credentials'));
  const changed = { ...preferences, target_language: 'zh-CN' };
  await invoke('translation_settings', { locale: 'en', preferences: changed, replacement: { provider: 'openai_compatible', key: 'after-backup' } });
  await page.evaluate(() => {
    window.translationHeld = window.__TAURI_INTERNALS__.invoke('translation_execute', { id: 'restore-concurrency', text: 'Bonjour', locale: 'en' }).catch(() => null);
  });
  await expect.poll(() => Boolean(held)).toBe(true);
  const preview = await invoke('backup_preview_start');
  picker(backup, 1148);
  await expect.poll(async () => (await invoke('backup_preview_status', { operation: preview.operation })).state, { timeout: 10000 }).toBe('preview_ready');
  page = await fixture.followRestart(() => invoke('backup_restore_start', { operation: preview.operation }));
  await expect.poll(async () => (await invoke('startup_status')).state, { timeout: 10000 }).toBe('ready');
  const restored = await invoke('translation_settings', { locale: 'en' });
  assert.deepEqual(restored.preferences, changed);
  assert.deepEqual(restored.keys, { deepl: 'configured', openai_compatible: 'configured' });
  held.end('data: [DONE]\n\n');
  await fixture.stop(true);
  const path = join(destination, 'Lexique-data/config/service-credentials.json');
  const slots = JSON.parse(await readFile(path, 'utf8'));
  slots.openai_compatible = 'invalid-dpapi';
  await writeFile(path, JSON.stringify(slots));
  page = await fixture.start();
  assert.deepEqual((await invoke('translation_settings', { locale: 'en' })).keys, { deepl: 'configured', openai_compatible: 'unreadable' });
  await invoke('translation_clear_key', { provider: 'openai_compatible' });
  assert.deepEqual((await invoke('translation_settings', { locale: 'en' })).keys, { deepl: 'configured', openai_compatible: 'missing' });
  await fixture.stop(true);
  const damaged = (await readFile(path)).subarray(0, -1);
  await writeFile(path, damaged);
  page = await fixture.start();
  assert.deepEqual((await invoke('translation_settings', { locale: 'en' })).keys, { deepl: 'unreadable', openai_compatible: 'unreadable' });
  await invoke('translation_clear_key', { provider: 'deepl' });
  const config = join(destination, 'Lexique-data/config');
  const preserved = (await readdir(config)).filter((name) => name.startsWith('.credentials-corrupt-'));
  assert.equal(preserved.length, 1);
  assert.deepEqual(await readFile(join(config, preserved[0])), damaged);
  assert.deepEqual((await invoke('translation_settings', { locale: 'en' })).keys, { deepl: 'missing', openai_compatible: 'missing' });
  await invoke('translation_settings', { locale: 'en', replacement: { provider: 'openai_compatible', key: 'recovered-fixture' } });
  assert.equal((await invoke('translation_settings', { locale: 'en' })).keys.openai_compatible, 'configured');
  await writeFile(join(destination, 'translation-state-result.json'), JSON.stringify({ backupIsolation: true, restoreWhileStreaming: true, independentSlotRecovery: true, corruptFilePreserved: true }));
  await fixture.stop(true);
} finally {
  if (fixture) await fixture.finish();
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
}
