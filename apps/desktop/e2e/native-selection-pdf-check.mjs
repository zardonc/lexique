/* global process, window, document */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { spawnSync } from 'node:child_process';
import { cp, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { resolve, join } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';
import { learningSnapshot } from './native-learning-snapshot.mjs';

assert.equal(process.argv.length, 8, 'Expected package, binary, conjugation, frequency, learning data and new destination');
const [resources, binary, conjugation, frequency, learningData, destination] = process.argv.slice(2).map((path) => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
for (const [source, name] of [[conjugation, 'conjugation'], [frequency, 'frequency'], [learningData, 'Lexique-data']])
  await cp(source, join(destination, name), { recursive: true });
const snapshot = () => learningSnapshot(join(destination, 'speech/runtime/python.exe'), join(destination, 'Lexique-data/data/lexique.sqlite3'));
const content = "BT /F1 40 Tf 70 700 Td (école d'aujourd'hui) Tj ET";
const objects = ['<< /Type /Catalog /Pages 2 0 R >>', '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
  '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
  `<< /Length ${Buffer.byteLength(content, 'latin1')} >>\nstream\n${content}\nendstream`,
  '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>'];
let pdf = '%PDF-1.4\n'; const offsets = [0];
for (const [index, object] of objects.entries()) { offsets.push(Buffer.byteLength(pdf, 'latin1')); pdf += `${index + 1} 0 obj\n${object}\nendobj\n`; }
const xref = Buffer.byteLength(pdf, 'latin1');
pdf += `xref\n0 6\n0000000000 65535 f \n${offsets.slice(1).map((offset) => `${String(offset).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
const bytes = Buffer.from(pdf, 'latin1');
await writeFile(join(destination, 'selection.pdf'), bytes);
const server = createServer((_, response) => { response.writeHead(200, { 'Content-Type': 'application/pdf' }); response.end(bytes); });
await new Promise((done) => server.listen(0, '127.0.0.1', done));
let sourceFixture;
try {
  const page = await fixture.start();
  const before = snapshot();
  const shortcut = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('selection_shortcut', { shortcut: 'Ctrl+Alt+K' }));
  assert.equal(shortcut.registered, true, shortcut.error);
  sourceFixture = await native24Fixture(resources, binary, join(destination, 'source'));
  const source = await sourceFixture.start('--force-renderer-accessibility');
  await source.goto(`http://127.0.0.1:${server.address().port}/selection.pdf`);
  await source.waitForTimeout(1500);
  await source.mouse.click(500, 210);
  await source.keyboard.press('Control+a');
  await source.screenshot({ path: join(destination, 'pdf-selection.png') });
  const run = spawnSync('pwsh', ['-NoProfile', '-File', resolve('e2e/native-input-helper.ps1'), '-Action', 'shortcut',
    '-SourcePid', String(sourceFixture.hostPid()), '-Key', '75'], { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(run.status, 0, run.stderr);
  await expect(page.getByRole('heading', { name: '翻译', exact: true })).toBeVisible();
  await page.waitForFunction(() => document.querySelector('#lookup-input-raw')?.value || document.querySelector('[role="alert"]'));
  const raw = await page.getByLabel('法语原文', { exact: true }).inputValue();
  const failures = await page.getByRole('alert').allTextContents();
  if (raw) {
    assert.match(raw, /école/);
    await expect(page.getByText('学校', { exact: true })).toHaveCount(0);
    await page.getByLabel('法语原文', { exact: true }).evaluate((node) => { const offset = node.value.indexOf('école'); node.setSelectionRange(offset, offset + 5); });
    await page.getByRole('button', { name: '查询输入', exact: true }).click();
    await expect(page.getByText('学校', { exact: true })).toBeVisible();
  } else assert.ok(failures.some((failure) => /来源不提供|没有选中文字/.test(failure)), failures.join('; '));
  assert.deepEqual(snapshot(), before);
  await writeFile(join(destination, 'results.json'), JSON.stringify({ source: 'WebView2 built-in PDF viewer',
    supported: Boolean(raw), raw, failures, retained: true }, null, 2));
  process.stdout.write(JSON.stringify({ supported: Boolean(raw), raw, failures }) + '\n');
} finally {
  await new Promise((done) => server.close(done));
  if (sourceFixture) await sourceFixture.finish();
  await fixture.finish();
}
