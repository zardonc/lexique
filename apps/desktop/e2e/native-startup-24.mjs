/* global process, window, performance, console */
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, mode = 'repeat'] = process.argv.slice(2);
assert.ok(['repeat', 'disconnect'].includes(mode));
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
try {
  const samples = [];
  for (let iteration = 0; iteration < (mode === 'repeat' ? 15 : 1); iteration++) {
    const page = await fixture.start();
    if (mode === 'disconnect') {
      await page.close();
      await assert.rejects(page.evaluate(() => window.__TAURI_INTERNALS__.invoke('runtime_status')), /closed|Target|context|session|destroy/i);
      const events = (await readFile(join(fixture.destination, 'lifecycle.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
      assert.ok(events.some(row => row.event === 'page-closed' && !row.stopping));
      samples.push({ injectedPageClose: true, failureObserved: true });
    } else {
      samples.push(await page.evaluate(async () => {
        const start = performance.now();
        const status = await window.__TAURI_INTERNALS__.invoke('runtime_status');
        const queue = await window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] });
        return { ms: performance.now() - start, schema: status.schema_version, cards: queue.queue.cards.length };
      }));
      assert.equal(samples.at(-1).cards, 0);
      await fixture.stop(true);
    }
  }
  const events = (await readFile(join(fixture.destination, 'lifecycle.jsonl'), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
  assert.ok(!events.some(row => row.event === 'host-stderr' && row.message.includes('WebView2 ProcessFailed')));
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ mode, samples }, null, 2));
  console.log(JSON.stringify({ mode, samples }));
} finally {
  await fixture.finish();
}
