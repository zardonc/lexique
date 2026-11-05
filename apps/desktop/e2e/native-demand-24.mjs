/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function rows() {
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);print(json.dumps(c.execute('SELECT a.spec,t.state,t.attempts,t.deadline_at FROM speech_tasks t JOIN speech_assets a USING(semantic_key) ORDER BY t.id').fetchall()))`, database], { windowsHide: true, encoding: 'utf8', timeout: 3000 });
  assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout);
}
try {
  await fixture.start(); await fixture.stop(true);
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'synthetic','version':'1','attribution':'synthetic','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
for i in range(1,21):
 c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?,?,?,'keep',1000,1000)",(i,f'mot{i}',snapshot))
 for card,kind in ((i*2-1,'recognition'),(i*2,'listening')):c.execute('INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?,?,?)',(card,i,kind))
c.commit()`, database], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
  assert.equal(result.status, 0, result.stderr);
  const page = await fixture.start();
  const view = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }));
  let tasks = rows();
  for (let index = 0; index < 20 && tasks.length === 0; index++) { await delay(100); tasks = rows(); }
  await writeFile(join(fixture.destination, 'demand-checkpoint.json'), JSON.stringify({ view, tasks }, null, 2));
  assert.ok(tasks.length > 0, 'Reading a window with missing audio must persist bounded review demand');
  assert.ok(tasks.length <= 20, 'A refresh must not enqueue the future library');
  const before = tasks.map((task) => ({ text: JSON.parse(task[0]).text, attempts: task[2], deadline: task[3] }));
  await page.evaluate(async () => { for (let index = 0; index < 5; index++) await window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }); });
  tasks = rows();
  assert.equal(new Set(tasks.map((task) => JSON.parse(task[0]).text)).size, tasks.length);
  for (const original of before) {
    const task = tasks.find((task) => JSON.parse(task[0]).text === original.text);
    assert.ok(task[2] >= original.attempts, 'Automatic refresh cannot reset attempts');
    if (original.deadline !== null) assert.equal(task[3], original.deadline);
  }
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ view: view.summary, tasks }, null, 2));
  console.log(JSON.stringify({ missing: view.summary.audio_pending_count, tasks: tasks.length }));
  await fixture.stop(true);
} finally { await fixture.finish(); }
