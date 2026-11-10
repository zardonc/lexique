/* global process, window, document, URL, fetch */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { spawn, spawnSync } from 'node:child_process';
import { cp, writeFile } from 'node:fs/promises';
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
const helper = resolve('e2e/native-input-helper.ps1'), sourceTitle = 'Lexique OCR source 59';
const results = [], blocked = [], diagnostics = [];
const monitors = JSON.parse(input('monitors'));
const snapshot = () => learningSnapshot(join(destination, 'speech/runtime/python.exe'), join(destination, 'Lexique-data/data/lexique.sqlite3'));
const captureLabels = {
  'zh-CN': { code: 'zh', image: '冻结屏幕', title: 'Lexique — 截图框选' },
  en: { code: 'en', image: 'Frozen screen', title: 'Lexique — screen selection' },
  fr: { code: 'fr', image: 'Écran figé', title: 'Lexique — sélection de l’écran' },
};
let editor, page;
function input(action, ...args) {
  const run = spawnSync('pwsh', ['-NoProfile', '-File', helper, '-Action', action, ...args],
    { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(run.status, 0, run.stderr); return run.stdout.trim();
}
function shortcut(key) {
  input('shortcut', '-Title', sourceTitle, '-Key', String(key), '-WithShift', '-OcrSample');
}
async function overlays() {
  const locale = captureLabels[await page.locator('html').getAttribute('lang')];
  const pages = () => page.context().browser().contexts().flatMap((context) => context.pages()).filter((candidate) => new URL(candidate.url()).searchParams.has('capture'));
  await expect.poll(() => pages().length, { timeout: 12_000 }).toBe(monitors.length);
  for (const overlay of pages()) {
    assert.equal(new URL(overlay.url()).searchParams.get('locale'), locale.code);
    await expect(overlay.getByRole('img', { name: locale.image, exact: true })).toBeVisible();
  }
  return pages();
}
async function capture(monitor, key, name) {
  const locale = captureLabels[await page.locator('html').getAttribute('lang')];
  input('position', '-Title', sourceTitle, '-X', String(monitor.x + 80), '-Y', String(monitor.y + 80));
  const source = JSON.parse(input('bounds', '-Title', sourceTitle))[0];
  shortcut(key); const screens = await overlays();
  const bounds = JSON.parse(input('bounds', '-Title', locale.title, '-SourcePid', String(fixture.hostPid())));
  const rectangles = (values) => values.map(({ x, y, width, height }) => ({ x, y, width, height })).sort((a, b) => a.x - b.x || a.y - b.y);
  assert.deepEqual(rectangles(bounds), rectangles(monitors));
  const viewports = [];
  for (const screen of screens) viewports.push(await screen.evaluate(() => {
    const image = document.querySelector('.frozen-screen');
    return { width: window.innerWidth, height: window.innerHeight, scale: window.devicePixelRatio,
      bitmapWidth: image.naturalWidth, bitmapHeight: image.naturalHeight };
  }));
  input('drag', '-X', String(source.x + 4), '-Y', String(source.y + 4),
    '-EndX', String(source.x + 824), '-EndY', String(source.y + 224));
  await page.locator('.preferences select').first().selectOption('zh');
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  const preview = page.getByRole('img', { name: '截图预览', exact: true });
  await expect(preview).toBeVisible();
  const data = await preview.getAttribute('src');
  await writeFile(join(destination, `${name}.png`), Buffer.from(data.split(',')[1], 'base64'));
  const field = page.getByLabel('法语原文', { exact: true });
  try { await expect(field).toHaveValue(/L'école d'aujourd'hui\.\nOù est le café \?\nÇa coûte 10 euros\./, { timeout: 12_000 }); }
  catch (cause) { throw new Error(`Native OCR alerts: ${(await page.getByRole('alert').allTextContents()).join('; ')}; input: ${await field.inputValue()}; ${cause}`); }
  const dimensions = await preview.evaluate((image) => [image.naturalWidth, image.naturalHeight]);
  // Native pointer coordinates can round by one physical pixel at fractional DPI.
  assert.ok(Math.abs(dimensions[0] - 820) <= 1 && Math.abs(dimensions[1] - 220) <= 1, `Physical crop: ${dimensions}`);
  await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
  await field.evaluate((node) => { const offset = node.value.indexOf('école'); node.setSelectionRange(offset, offset + 5); });
  await page.getByRole('button', { name: '查询输入', exact: true }).click();
  await expect(page.getByText('学校', { exact: true })).toBeVisible();
  results.push({ check: `${name}: localized real global shortcut → physical screen region → automatic offline OCR → explicit dictionary lookup`, passed: true, locale: locale.code, source, bounds, viewports, dimensions });
}
try {
  page = await fixture.start();
  page.on('console', (message) => { if (['error', 'warning'].includes(message.type())) diagnostics.push(message.text().slice(0, 2000)); });
  page.on('requestfailed', (request) => diagnostics.push({ url: request.url(), failure: request.failure() }));
  await page.context().route('**/*', (route) => {
    const url = new URL(route.request().url());
    if (!['http:', 'https:'].includes(url.protocol) || ['ipc.localhost', 'tauri.localhost', '127.0.0.1', 'localhost'].includes(url.hostname)) return route.continue();
    blocked.push(url.hostname); return route.abort();
  });
  diagnostics.push(await page.evaluate(async () => {
    const resource = await fetch('/ocr/worker.min.js');
    return { workerStatus: resource.status, type: resource.headers.get('content-type'), prefix: (await resource.text()).slice(0, 64) };
  }));
  const before = snapshot();
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByLabel('导入图片', { exact: true }).setInputFiles(resolve('e2e/fixtures/ocr-french.png'));
  await expect(page.getByRole('img', { name: '截图预览', exact: true })).toBeVisible();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('');
  await page.getByRole('button', { name: '识别文字', exact: true }).click();
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue(/Où est le café \?/, { timeout: 12_000 });
  results.push({ check: 'packaged local WASM and French model recognize imported PNG with external requests blocked', passed: true });
  await page.locator('.preferences select').first().selectOption('fr');
  await page.getByRole('button', { name: 'Rechercher par capture', exact: true }).click();
  const localized = await overlays();
  assert.equal(JSON.parse(input('bounds', '-Title', captureLabels.fr.title, '-SourcePid', String(fixture.hostPid()))).length, monitors.length);
  await localized[0].getByRole('button', { name: 'Annuler la capture', exact: true }).click();
  await expect.poll(() => page.context().browser().contexts().flatMap((context) => context.pages())
    .filter((candidate) => new URL(candidate.url()).searchParams.has('capture')).length).toBe(0);
  await page.locator('.preferences select').first().selectOption('zh');
  results.push({ check: 'screen button carries the French interface language to every overlay and native title; localized cancel closes all overlays', passed: true });
  const initial = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('ocr_shortcut', { shortcut: null }));
  assert.equal(initial.shortcut, 'Ctrl+Alt+Shift+O');
  editor = spawn('pwsh', ['-NoProfile', '-File', helper, '-Action', 'editor', '-Title', sourceTitle, '-OcrSample',
    '-Text', "L'école d'aujourd'hui.\r\nOù est le café ?\r\nÇa coûte 10 euros."], { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] });
  let editorError = ''; editor.stderr.on('data', (bytes) => { editorError += bytes.toString(); });
  await delay(1200); assert.equal(editor.exitCode, null, editorError);
  if (!initial.registered) {
    assert.ok(initial.error); await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('ocr_shortcut', { shortcut: 'Ctrl+Alt+Shift+P' }));
  }
  const primary = monitors.find((monitor) => monitor.primary);
  assert.ok(primary);
  await page.locator('.preferences select').first().selectOption('en');
  await capture(primary, initial.registered ? 79 : 80, 'primary-default');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  const selection = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('selection_shortcut', { shortcut: null }));
  if (selection.registered) {
    await page.getByLabel('截图快捷键', { exact: true }).fill(selection.shortcut);
    await page.getByRole('button', { name: '保存截图快捷键', exact: true }).click();
    await expect(page.getByRole('alert').filter({ hasText: 'another Lexique action' })).toBeVisible();
    const retained = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('ocr_shortcut', { shortcut: null }));
    assert.equal(retained.shortcut, initial.registered ? initial.shortcut : 'Ctrl+Alt+Shift+P');
    results.push({ check: 'selection and OCR shortcut conflict is visible and keeps the previous registration', passed: true });
  }
  await page.getByLabel('截图快捷键', { exact: true }).fill('Ctrl+Alt+Shift+P');
  await page.getByRole('button', { name: '保存截图快捷键', exact: true }).click();
  await expect(page.getByText('截图快捷键已保存。', { exact: true })).toBeVisible();
  const secondary = monitors.filter((monitor) => !monitor.primary);
  await page.locator('.preferences select').first().selectOption('fr');
  if (!secondary.length) {
    results.push({ check: 'secondary monitor with negative origin and independent scaling', status: 'skipped', reason: 'Only the primary monitor is currently connected' });
    await capture(primary, 80, 'primary-custom');
  } else for (let index = 0; index < secondary.length; index++) await capture(secondary[index], 80, `secondary-${index}`);
  shortcut(80); const screens = await overlays();
  await screens[0].keyboard.press('Escape');
  await expect.poll(() => page.context().pages().filter((candidate) => new URL(candidate.url()).searchParams.has('capture')).length).toBe(0);
  await expect(page.getByLabel('法语原文', { exact: true })).toHaveValue('');
  results.push({ check: 'Escape closes both capture windows and cancels the request', passed: true });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list li').filter({ hasText: 'école' }).getByRole('button').click();
  await page.getByLabel('独立笔记', { exact: true }).fill('Draft to preserve during capture close');
  shortcut(80); await overlays();
  input('close', '-Title', 'Lexique', '-SourcePid', String(fixture.hostPid()));
  await expect(page.getByRole('dialog')).toBeVisible();
  await expect.poll(() => page.context().browser().contexts().flatMap((context) => context.pages())
    .filter((candidate) => new URL(candidate.url()).searchParams.has('capture')).length).toBe(0);
  assert.equal(JSON.parse(input('bounds', '-Title', 'Lexique', '-SourcePid', String(fixture.hostPid()))).length, 1);
  await page.getByRole('button', { name: '继续编辑', exact: true }).click();
  await expect(page.getByLabel('独立笔记', { exact: true })).toHaveValue('Draft to preserve during capture close');
  await page.getByRole('button', { name: '翻译', exact: true }).click();
  await page.getByRole('button', { name: '放弃修改', exact: true }).click();
  results.push({ check: 'native close cancels capture, reveals the main save dialog and preserves an unsaved note when editing continues', passed: true });
  assert.deepEqual(snapshot(), before);
  shortcut(80); await overlays();
  await fixture.stop(true); page = await fixture.start();
  const restored = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('ocr_shortcut', { shortcut: null }));
  assert.equal(restored.shortcut, 'Ctrl+Alt+Shift+P'); assert.equal(restored.registered, true);
  assert.deepEqual(snapshot(), before);
  results.push({ check: 'authorized close terminates all capture windows; OCR shortcut, learning and audio relationships survive restart', passed: true });
  assert.deepEqual(blocked, []);
  await writeFile(join(destination, 'results.json'), JSON.stringify({ initial, restored, blocked, monitors,
    retained: Object.fromEntries(['vocabulary', 'review_cards', 'review_progress', 'review_ratings', 'word_audio_selections', 'protected'].map((table) => [table, before[table].length])), results }, null, 2));
  process.stdout.write(JSON.stringify(results, null, 2) + '\n');
} catch (cause) {
  const alerts = page && !page.isClosed() ? await page.getByRole('alert').allTextContents() : [];
  const pages = page && !page.isClosed() ? page.context().browser().contexts().map((context) => context.pages().map((page) => page.url())) : [];
  const documents = page && !page.isClosed() ? await Promise.all(page.context().browser().contexts().flatMap((context) => context.pages()).map(async (view) => {
    try { return await Promise.race([view.evaluate(() => ({ url: window.location.href, state: document.readyState,
      title: document.title, frozen: Boolean(document.querySelector('.frozen-screen')) })), delay(1500).then(() => ({ timeout: true }))]); }
    catch (error) { return { error: String(error) }; }
  })) : [];
  let windows;
  try { windows = JSON.parse(input('bounds', '-Title', 'Lexique — 截图框选', '-SourcePid', String(fixture.hostPid()))); }
  catch { windows = []; }
  await writeFile(join(destination, 'failure.json'), JSON.stringify({ alerts, diagnostics, blocked, pages, documents, windows, error: String(cause) }, null, 2));
  process.stderr.write(JSON.stringify({ alerts, diagnostics, blocked }) + '\n'); throw cause;
} finally {
  if (editor?.pid && editor.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(editor.pid), '/T', '/F'], { windowsHide: true });
  await fixture.finish();
}
