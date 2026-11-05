/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function sql(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', source, database], { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout);
}
try {
  let page = await fixture.start();
  const bytes = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_preview', { text: 'bonjour' }));
  await fixture.stop(true);
  const before = sql(`import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);c.execute('UPDATE speech_versions SET last_used=0');c.commit()
print(json.dumps(c.execute('SELECT locator,expires_at,last_used FROM speech_versions').fetchall()))`);
  page = await fixture.start();
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_preview', { text: 'bonjour' })), bytes);
  const after = sql(`import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);print(json.dumps(c.execute('SELECT locator,expires_at,last_used FROM speech_versions').fetchall()))`);
  assert.equal(after.length, 1);
  assert.equal(after[0][0], before[0][0]); assert.equal(after[0][1], before[0][1], 'Reading must not extend absolute expiry');
  assert.ok(after[0][2] > 0, 'A real cache hit must update LRU use time');
  assert.equal(fixture.workerPids().length, 0, 'Cache hit must not start the model');
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ before, after }, null, 2));
  console.log(JSON.stringify({ before, after }));
} finally { await fixture.finish(); }
