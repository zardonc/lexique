/* global process, window, document, URL */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { learningSnapshot } from './native-learning-snapshot.mjs';

assert.equal(process.argv.length, 8, 'Expected package, binary, conjugation, frequency, existing test learning data and new destination');
const [resources, binary, conjugation, frequency, learningData, destination] = process.argv.slice(2).map((path) => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
await cp(conjugation, join(destination, 'conjugation'), { recursive: true });
await cp(frequency, join(destination, 'frequency'), { recursive: true });
await cp(learningData, join(destination, 'Lexique-data'), { recursive: true });
const helper = resolve('e2e/native-input-helper.ps1');
const results = [];
const snapshot = () => learningSnapshot(join(destination, 'speech/runtime/python.exe'), join(destination, 'Lexique-data/data/lexique.sqlite3'));
let sourceFixture, editor;
function shortcut(title, pid = 0, defaultKey = false) {
  const target = pid ? ['-SourcePid', String(pid)] : ['-Title', title];
  const run = spawnSync('pwsh', ['-NoProfile', '-File', helper, '-Action', 'shortcut', ...target,
    '-Key', defaultKey ? '68' : '75', ...(defaultKey ? ['-WithShift'] : [])],
    { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(run.status, 0, run.stderr);
}
function focusSource(pid) {
  const run = spawnSync('pwsh', ['-NoProfile', '-File', helper, '-Action', 'focus', '-SourcePid', String(pid)],
    { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(run.status, 0, run.stderr);
}
async function captured(page, raw) {
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  try { await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(raw, { timeout: 8000 }); }
  catch (error) { throw new Error(`Native selection: ${(await page.getByRole('alert').allTextContents()).join('; ')}; ${error}`); }
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
  await page.getByLabel('法语原文', { exact: true }).evaluate((node) => {
    const offset = node.value.indexOf('école'); node.setSelectionRange(offset, offset + 5);
  });
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
}
try {
  let page = await fixture.start();
  await page.context().route('**/*', (route) => {
    const host = new URL(route.request().url()).hostname;
    return ['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(host) ? route.continue() : route.abort();
  });
  const initial = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('selection_shortcut', { shortcut: null }));
  assert.equal(initial.shortcut, 'Ctrl+Alt+Shift+D');
  const before = snapshot();
  const raw = '  école  ';
  editor = spawn('pwsh', ['-NoProfile', '-File', helper, '-Action', 'editor', '-Title', 'Lexique selection editor', '-Text', raw],
    { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] });
  let editorError = ''; editor.stderr.on('data', (bytes) => { editorError += bytes.toString(); });
  await delay(1500);
  assert.equal(editor.exitCode, null, editorError);
  if (initial.registered) {
    shortcut('Lexique selection editor', 0, true);
    await captured(page, raw);
    results.push({ check: 'default Ctrl+Alt+Shift+D → translation input → explicit offline dictionary lookup', passed: true });
  } else {
    assert.match(initial.error, /already registered/i);
    results.push({ check: 'default shortcut conflict is returned without simulated registration', passed: true });
  }
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await expect(page.getByLabel('取词快捷键', { exact: true })).toHaveValue('Ctrl+Alt+Shift+D');
  if (!initial.registered) await expect(page.getByRole('alert').filter({ hasText: 'already registered' })).toBeVisible();
  await page.getByLabel('取词快捷键', { exact: true }).fill('Ctrl+Alt+K');
  await page.getByRole('button', { name: '保存取词快捷键', exact: true }).click();
  await expect(page.getByText('取词快捷键已保存。', { exact: true })).toBeVisible();
  const saved = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('selection_shortcut', { shortcut: null }));
  assert.equal(saved.registered, true); assert.equal(saved.shortcut, 'Ctrl+Alt+K');
  sourceFixture = await native24Fixture(resources, binary, join(destination, 'source'));
  const source = await sourceFixture.start('--force-renderer-accessibility');
  await source.setContent('<!doctype html><title>Lexique selection browser</title><p id="word" style="white-space:pre">  école d’aujourd’hui  </p>');
  focusSource(sourceFixture.hostPid());
  await source.locator('#word').click();
  await source.locator('#word').evaluate((node) => {
    const range = document.createRange(); range.selectNodeContents(node);
    window.getSelection().removeAllRanges(); window.getSelection().addRange(range);
  });
  shortcut('', sourceFixture.hostPid());
  await captured(page, '  école d’aujourd’hui  ');
  results.push({ check: 'separate WebView2 browser webpage → custom shortcut → translation input → explicit dictionary lookup', passed: true });
  const prefix = `école ${'à'.repeat(4_993)}😀`;
  focusSource(sourceFixture.hostPid());
  await source.locator('#word').evaluate((node, text) => {
    node.textContent = text;
    const range = document.createRange(); range.selectNodeContents(node);
    window.getSelection().removeAllRanges(); window.getSelection().addRange(range);
  }, `${prefix}discarded`);
  shortcut('', sourceFixture.hostPid());
  await captured(page, prefix);
  results.push({ check: 'native capture truncates at 5,000 Unicode characters and preserves the last character', passed: true });
  shortcut('Lexique selection editor');
  await captured(page, raw);
  results.push({ check: 'Win32 RichEdit → custom shortcut → translation input → explicit dictionary lookup', passed: true });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list li').filter({ hasText: 'école' }).getByRole('button').click();
  await page.getByLabel('独立笔记', { exact: true }).fill('Draft to preserve');
  shortcut('Lexique selection editor');
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('Draft to preserve');
  shortcut('Lexique selection editor');
  await page.getByRole('button', { name: '放弃修改', exact: true }).click();
  await captured(page, raw);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'native unsaved-note guard preserves learning and selected audio', passed: true });
  const dictionary = join(destination, 'dictionary/dictionary.sqlite3'), backup = dictionary + '.original';
  await rename(dictionary, backup);
  try {
    shortcut('Lexique selection editor');
    await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(raw);
    await page.getByRole('button', { name: '查询输入', exact: true }).click();
    await expect(page.getByRole('alert').filter({ hasText: 'Could not read resource' })).toBeVisible();
    await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(raw);
  } finally { await rename(backup, dictionary); }
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
  results.push({ check: 'missing dictionary keeps captured input; restoration permits retry', passed: true });
  await fixture.stop(true);
  page = await fixture.start();
  const restored = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('selection_shortcut', { shortcut: null }));
  assert.deepEqual(restored, saved);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'shortcut registration and setting survive normal restart', passed: true });
  await writeFile(join(destination, 'results.json'), JSON.stringify({ initial, saved, retained: Object.fromEntries(
    ['vocabulary', 'review_cards', 'review_progress', 'review_ratings', 'word_audio_selections', 'protected']
      .map((table) => [table, before[table].length])), results }, null, 2));
  process.stdout.write(JSON.stringify(results, null, 2) + '\n');
} finally {
  if (editor?.pid && editor.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(editor.pid), '/T', '/F'], { windowsHide: true });
  if (sourceFixture) await sourceFixture.finish();
  await fixture.finish();
}
