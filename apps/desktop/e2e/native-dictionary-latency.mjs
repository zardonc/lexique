/* global process, URL, WebSocket, fetch */
import assert from 'node:assert/strict';
import { cp, writeFile } from 'node:fs/promises';
import { join, resolve, basename } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

assert.ok(process.argv.length >= 8, 'Expected package, binary, conjugation, frequency, new destination and local MDX paths');
const [resources, binary, conjugation, frequency, destination, ...corpora] = process.argv.slice(2).map(path => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
await cp(conjugation, join(destination, 'conjugation'), { recursive: true });
await cp(frequency, join(destination, 'frequency'), { recursive: true });
let socket, next = 0;
const pending = new Map(), results = [];

function send(method, params = {}) {
  const id = ++next;
  return new Promise((done, reject) => { pending.set(id, { done, reject }); socket.send(JSON.stringify({ id, method, params })); });
}
async function evaluate(expression) {
  const value = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  assert.equal(value.exceptionDetails, undefined, JSON.stringify(value.exceptionDetails));
  return value.result?.value;
}
async function start() {
  const endpoint = new URL(await fixture.startHost());
  let target;
  for (let attempt = 0; attempt < 100; attempt++) {
    const tabs = await (await fetch(`http://${endpoint.host}/json/list`)).json();
    target = tabs.find(tab => tab.type === 'page' && tab.url.startsWith('http://tauri.localhost'));
    if (target) break;
    await delay(100);
  }
  assert.ok(target, 'Own WebView2 application target must finish its initial navigation');
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((done, reject) => { socket.onopen = done; socket.onerror = reject; });
  socket.onmessage = event => {
    const message = JSON.parse(event.data);
    if (message.id && pending.has(message.id)) {
      const { done, reject } = pending.get(message.id); pending.delete(message.id);
      if (message.error) reject(new Error(JSON.stringify(message.error))); else done(message.result);
    }
  };
  socket.onclose = () => { for (const { reject } of pending.values()) reject(new Error('CDP disconnected')); pending.clear(); };
  // Network debugger events duplicate large binary uploads; observe only DOM and scalar IPC results.
  await send('Runtime.enable');
  for (let attempt = 0; attempt < 100; attempt++) {
    if (await evaluate("Boolean(window.__TAURI_INTERNALS__) && document.readyState === 'complete' && Boolean(document.querySelector('nav button'))")) return;
    await delay(100);
  }
  throw new Error('Own WebView2 navigation did not finish');
}
async function stop() {
  try { await evaluate("window.__TAURI_INTERNALS__.invoke('finish_close')"); }
  catch (error) { assert.match(String(error), /disconnected/); }
  await fixture.stop(true);
  socket?.close();
}

try {
  await start();
  await evaluate("Array.from(document.querySelectorAll('nav button')).find(button => button.textContent === '设置').click()");
  for (let attempt = 0; attempt < 100; attempt++) {
    if (await evaluate("Boolean(document.querySelector('#dictionary-import'))")) break;
    await delay(100);
  }
  for (const path of corpora) {
    const { root } = await send('DOM.getDocument');
    const { nodeId } = await send('DOM.querySelector', { nodeId: root.nodeId, selector: '#dictionary-import' });
    await send('DOM.setFileInputFiles', { nodeId, files: [path] });
    let imported;
    for (let attempt = 0; attempt < 200; attempt++) {
      imported = await evaluate(`Array.from(document.querySelectorAll('.dictionary-settings li span')).some(span => span.textContent === ${JSON.stringify(basename(path))}) && !document.querySelector('#dictionary-import').disabled`);
      if (imported) break;
      await delay(100);
    }
    assert.ok(imported, 'Native MDX import must finish and enable its controls');
  }
  const sources = await evaluate("window.__TAURI_INTERNALS__.invoke('dictionary_settings')");
  assert.equal(sources.length, corpora.length + 1);
  await stop(); await start();
  for (const source of sources.filter(item => item.id !== 'builtin')) {
    const measured = await evaluate(`(async () => {
      const timings = []; let records = 0, bodyBytes = 0;
      for (let index = 0; index < 21; index++) {
        const started = performance.now();
        const value = await window.__TAURI_INTERNALS__.invoke('dictionary_lookup', { key: 'être', dictionary_id: ${JSON.stringify(source.id)} });
        timings.push(performance.now() - started);
        records = value.entries.length;
        bodyBytes = value.entries.reduce((sum, entry) => sum + new TextEncoder().encode(entry.html ?? '').length, 0);
      }
      return { timings, records, bodyBytes };
    })()`);
    assert.ok(measured.records > 0 && measured.bodyBytes > 0, 'Measure a matching real body');
    assert.ok(measured.timings[0] <= 1500, 'First IPC query exceeds 1500 ms');
    assert.ok(Math.max(...measured.timings.slice(1)) <= 250, 'Cached IPC query exceeds 250 ms');
    results.push({ dictionary: source.title, first_ms: measured.timings[0], cached_max_ms: Math.max(...measured.timings.slice(1)),
      records: measured.records, body_bytes: measured.bodyBytes, samples: 21, status: 'passed' });
  }
  await writeFile(join(destination, 'result.json'), JSON.stringify({ mode: 'latency', results }, null, 2));
  process.stdout.write(JSON.stringify({ mode: 'latency', results }) + '\n');
  await stop();
} finally { socket?.close(); await fixture.finish(); }
