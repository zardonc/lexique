/* global process, window, document, fetch, setTimeout, clearTimeout */
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { appendFileSync } from 'node:fs';
import { resolve, join, basename } from 'node:path';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';

export async function native24Fixture(packagePath, binaryPath, destinationPath) {
  const destination = resolve(destinationPath);
  assert.ok(['issue-9', 'issue-24', 'issue-56', 'issue-57', 'issue-58', 'issue-59', 'issue-65', 'issue-66', 'issue-75', 'issue-79'].some((issue) => destination.startsWith(resolve('../../target/' + issue) + '\\')));
  await mkdir(destination);
  // Reused test packages also contain old results. Only copy distribution resources:
  // a failed run must never inherit a successful report or someone else's data.
  for (const name of ['dictionary', 'speech', 'LICENSE', 'README.txt']) {
    await cp(join(resolve(packagePath), name), join(destination, name), { recursive: true });
  }
  await cp(resolve(binaryPath), join(destination, 'Lexique.exe'));
  await writeFile(join(destination, 'run-metadata.json'), JSON.stringify({
    startedAt: new Date().toISOString(), binarySha256: createHash('sha256').update(await readFile(join(destination, 'Lexique.exe'))).digest('hex'),
    command: process.argv.slice(1), hardTimeoutSeconds: 60,
  }, null, 2));
  const server = createServer();
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const port = server.address().port; await new Promise((done) => server.close(done));
  const executable = join(destination, 'Lexique.exe');
  let child, browser, page, currentHostPid, lastEndpoint;
  const helpers = new Set();
  let stopping = false;
  const started = Date.now();
  function diagnostic(event, detail = {}) {
    appendFileSync(join(destination, 'lifecycle.jsonl'), JSON.stringify({ ms: Date.now() - started, event, stopping, ...detail }) + '\n');
  }
  function executablePids(path) {
    const expected = path.replaceAll("'", "''");
    const name = basename(path, '.exe').replaceAll("'", "''");
    const result = spawnSync('pwsh', ['-NoProfile', '-NonInteractive', '-Command', `ConvertTo-Json -Compress -InputObject @(Get-Process -Name '${name}' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${expected}' } | Select-Object -ExpandProperty Id)`], { windowsHide: true, encoding: 'utf8', timeout: 4000 });
    assert.equal(result.status, 0, result.error?.message || result.stderr);
    return JSON.parse(result.stdout);
  }
  function killOwned() {
    for (const helper of helpers) helper.kill();
    for (const path of [executable, join(destination, 'speech', 'runtime', 'python.exe')]) {
      for (const pid of executablePids(path)) spawnSync('taskkill.exe', ['/PID', String(pid), '/T', '/F'], { windowsHide: true, timeout: 4000 });
    }
  }
  const onExit = () => { try { killOwned(); } catch (error) { process.stderr.write(String(error)); } };
  process.once('exit', onExit);
  const timer = setTimeout(() => {
    killOwned();
    process.stderr.write('Native execution exceeded the hard 60-second limit\n'); process.exit(1);
  }, 60_000);
  async function startHost(additionalBrowserArguments = '') {
    assert.deepEqual(executablePids(executable), [], 'Do not launch over an existing owned host');
    stopping = false;
    child = spawn(executable, [], { cwd: destination, windowsHide: true,
      env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--disable-gpu --autoplay-policy=no-user-gesture-required --remote-debugging-port=${port} ${additionalBrowserArguments}` } });
    currentHostPid = child.pid;
    diagnostic('host-start', { pid: child.pid });
    child.stderr.on('data', (bytes) => diagnostic('host-stderr', { message: bytes.toString().slice(0, 4096) }));
    child.on('error', (error) => { diagnostic('host-error', { message: String(error) }); process.stderr.write(String(error)); });
    child.on('exit', (code, signal) => { diagnostic('host-exit', { code, signal }); process.stderr.write(`Owned desktop exited: ${code}, ${signal}\n`); });
    let endpoint;
    for (let attempt = 0; attempt < 80; attempt++) {
      try { endpoint = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl; if (endpoint) break; }
      catch { /* Own WebView2 is starting. */ }
      await delay(100);
    }
    assert.ok(endpoint, 'Own WebView2 endpoint unavailable');
    lastEndpoint = endpoint;
    return endpoint;
  }
  async function attach(endpoint) {
    browser = await chromium.connectOverCDP(endpoint); page = browser.contexts()[0].pages()[0];
    browser.on('disconnected', () => diagnostic('cdp-disconnected'));
    page.on('crash', () => diagnostic('renderer-crash'));
    page.on('close', () => diagnostic('page-closed'));
    page.on('pageerror', (error) => diagnostic('page-error', { message: String(error).slice(0, 4096) }));
    try {
      await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__) && window.location.origin !== 'null' && document.readyState === 'complete', undefined, { timeout: 10_000 });
    } catch (error) { throw new Error(`Own WebView2 did not finish navigation (${page.url()}): ${error}`); }
    diagnostic('navigation-ready');
    return page;
  }
  async function start(additionalBrowserArguments = '') {
    return attach(await startHost(additionalBrowserArguments));
  }
  async function exitWatcher(pid) {
    const expected = executable.replaceAll("'", "''");
    const helper = spawn('pwsh', ['-NoProfile', '-NonInteractive', '-Command', `$ErrorActionPreference='Stop'; $owned=Get-Process -Id ${pid}; if($owned.Path -ne '${expected}'){throw 'Exit waiter executable mismatch'}; $null=$owned.Handle; Write-Output ready; if(-not $owned.WaitForExit(10000)){throw 'Owned host exit timed out'}; Write-Output $owned.ExitCode`], { windowsHide: true });
    helpers.add(helper);
    let output = '', stderr = '';
    helper.stdout.on('data', (bytes) => { output += bytes.toString(); });
    helper.stderr.on('data', (bytes) => { stderr += bytes.toString(); });
    const deadline = setTimeout(() => helper.kill(), 12_000);
    const finished = new Promise((done, reject) => {
      helper.once('error', reject);
      helper.once('exit', (code) => { clearTimeout(deadline); helpers.delete(helper); done(code); });
    });
    for (let i = 0; i < 50 && !output.startsWith('ready'); i++) {
      assert.equal(helper.exitCode, null, stderr || 'Exit waiter stopped before acquiring the process handle');
      await delay(50);
    }
    assert.ok(output.startsWith('ready'), 'Exit waiter did not acquire the owned process handle');
    return async () => { assert.equal(await finished, 0, stderr); assert.equal(output.trim().split(/\s+/).at(-1), '0', 'Owned host must exit successfully'); };
  }
  async function followRestart(trigger) {
    const predecessor = currentHostPid;
    assert.deepEqual(executablePids(executable), [predecessor], 'Restart requires exactly one owned predecessor');
    const awaitExit = await exitWatcher(predecessor);
    const previousEndpoint = lastEndpoint;
    let triggerError;
    try { await trigger(); } catch (error) { triggerError = error; }
    let successor;
    for (let i = 0; i < 16; i++) {
      const pids = executablePids(executable);
      assert.ok(pids.length <= 2 && (pids.length < 2 || pids.includes(predecessor)), 'Restart created multiple successors');
      if (pids.length === 1 && pids[0] !== predecessor) { successor = pids[0]; break; }
      await delay(100);
    }
    assert.ok(successor, `No owned restart successor: ${triggerError || ''}`);
    currentHostPid = successor;
    diagnostic('restart-adopted', { predecessor, successor });
    await awaitExit();
    if (triggerError) assert.match(String(triggerError), /closed|Target|context|session|Frame|destroy/i);
    let endpoint;
    for (let i = 0; i < 80; i++) {
      try {
        const candidate = (await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()).webSocketDebuggerUrl;
        if (candidate && candidate !== previousEndpoint) { endpoint = candidate; break; }
      } catch { /* The successor owns the same private CDP port. */ }
      await delay(100);
    }
    assert.ok(endpoint, 'Restart successor did not expose a fresh owned WebView');
    lastEndpoint = endpoint;
    return attach(endpoint);
  }
  async function stop(normal = false, trigger) {
    stopping = true; diagnostic('stop-request', { normal });
    if (normal && currentHostPid) {
      const awaitExit = await exitWatcher(currentHostPid);
      if (page) {
        try {
          if (trigger) await trigger();
          else await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('finish_close'));
        }
        catch (error) { assert.match(String(error), /closed|Target|context|session|Frame|destroy/i); }
      }
      await awaitExit();
      if (currentHostPid === child?.pid) assert.equal(child.exitCode, 0, 'Normal exit must terminate the application successfully');
      assert.deepEqual(executablePids(executable), [], 'Normal close left an owned host');
    }
    if (browser) await browser.close(); browser = undefined; page = undefined;
    killOwned();
    child = undefined; currentHostPid = undefined; await delay(200);
  }
  function workerPids() {
    return executablePids(join(destination, 'speech', 'runtime', 'python.exe'));
  }
  return { destination, start, startHost, followRestart, stop, workerPids, hostPid: () => currentHostPid,
    ownHelper: (helper) => { helpers.add(helper); helper.once('exit', () => helpers.delete(helper)); }, finish: async () => {
    try {
      await stop();
      const cleanup = { hosts: executablePids(executable), workers: workerPids(), helpers: helpers.size };
      await writeFile(join(destination, 'cleanup.json'), JSON.stringify(cleanup));
      assert.deepEqual(cleanup, { hosts: [], workers: [], helpers: 0 });
    } finally { killOwned(); clearTimeout(timer); process.removeListener('exit', onExit); }
  } };
}
