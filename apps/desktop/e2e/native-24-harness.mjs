/* global process, window, document, fetch, setTimeout, clearTimeout */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

export async function native24Fixture(packagePath, binaryPath, destinationPath) {
  const destination = resolve(destinationPath);
  assert.ok(destination.startsWith(resolve('../../target/issue-24') + '\\'));
  await mkdir(destination);
  await cp(resolve(packagePath), destination, { recursive: true, filter: (path) => path !== join(resolve(packagePath), 'Lexique-data') });
  await cp(resolve(binaryPath), join(destination, 'Lexique.exe'));
  const server = createServer();
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const port = server.address().port; await new Promise((done) => server.close(done));
  let child, browser, page;
  const timer = setTimeout(() => {
    if (child?.pid) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
    process.stderr.write('Native #24 execution exceeded the hard 60-second limit\n'); process.exit(1);
  }, 60_000);
  async function start() {
    child = spawn(join(destination, 'Lexique.exe'), [], { cwd: destination, windowsHide: true,
      env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --autoplay-policy=no-user-gesture-required --remote-debugging-port=${port}` } });
    child.on('error', (error) => { process.stderr.write(String(error)); });
    child.on('exit', (code, signal) => { process.stderr.write(`Owned desktop exited: ${code}, ${signal}\n`); });
    let endpoint;
    for (let attempt = 0; attempt < 80; attempt++) {
      try { endpoint = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl; if (endpoint) break; }
      catch { /* Own WebView2 is starting. */ }
      await delay(100);
    }
    assert.ok(endpoint, 'Own WebView2 endpoint unavailable');
    browser = await chromium.connectOverCDP(endpoint); page = browser.contexts()[0].pages()[0];
    try {
      await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__) && window.location.origin !== 'null' && document.readyState === 'complete', undefined, { timeout: 10_000 });
    } catch (error) { throw new Error(`Own WebView2 did not finish navigation (${page.url()}): ${error}`); }
    return page;
  }
  async function stop(normal = false) {
    if (normal && page) {
      try { await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('finish_close')); }
      catch (error) { assert.match(String(error), /closed|Target|context|session|Frame|destroy/i); }
      for (let i = 0; child?.exitCode === null && i < 50; i++) await delay(100);
      assert.notEqual(child?.exitCode, null, 'Normal exit must terminate the application');
    }
    if (browser) await browser.close(); browser = undefined; page = undefined;
    if (child?.pid && child.exitCode === null) spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true });
    child = undefined; await delay(200);
  }
  function workerPids() {
    const expected = join(destination, 'speech', 'runtime', 'python.exe').replaceAll("'", "''");
    const result = spawnSync('pwsh', ['-NoProfile', '-NonInteractive', '-Command', `ConvertTo-Json -Compress -InputObject @(Get-Process -Name python -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${expected}' } | Select-Object -ExpandProperty Id)`], { windowsHide: true, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(result.stdout);
  }
  return { destination, start, stop, workerPids, finish: async () => { clearTimeout(timer); await stop(); } };
}
