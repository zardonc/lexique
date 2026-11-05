/* global process, window, console */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { spawn, spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destination, helperPath, point] = process.argv.slice(2);
assert.ok(['intent_committed', 'tmp_written', 'tmp_synced', 'renamed', 'readback', 'db_complete', 'gc_marked', 'gc_deleted', 'gc_finished'].includes(point));
const fixture = await native24Fixture(packagePath, binaryPath, destination);
const database = join(fixture.destination, 'Lexique-data/data/lexique.sqlite3');
let child;
process.once('exit', () => { if (child?.exitCode === null) child.kill(); });
function sql(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', source, database], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
  assert.equal(result.status, 0, result.stderr); return result.stdout ? JSON.parse(result.stdout) : null;
}
const observe = () => sql(`import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);print(json.dumps(dict(words=c.execute('SELECT id,notes,revision FROM vocabulary').fetchall(),cards=c.execute('SELECT * FROM review_cards ORDER BY id').fetchall(),pending=c.execute('SELECT COUNT(*) FROM word_audio_publications').fetchone()[0],selected=c.execute('SELECT current_version,revision FROM word_audio_selections WHERE vocabulary_id=1').fetchone(),versions=c.execute('SELECT locator,sealed,gc_state FROM speech_versions ORDER BY locator').fetchall())))`);
try {
  await fixture.start(); await fixture.stop(true);
  sql(`import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'école','version':'1','attribution':'fixture','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'école',?,'keep',1,1)",(snapshot,));c.execute("INSERT INTO review_cards VALUES(1,1,'recognition')");c.execute("INSERT INTO review_cards VALUES(2,1,'listening')");c.commit()`);
  const before = observe(); const marker = join(fixture.destination, 'crash-boundary.txt');
  child = spawn(resolve(helperPath), ['--exact', 'publication_crash_fixture', '--ignored', '--nocapture'], { windowsHide: true,
    env: { ...process.env, LEXIQUE_TEST_CRASH_DATABASE: database, LEXIQUE_TEST_CRASH_POINT: point, LEXIQUE_TEST_CRASH_MARKER: marker } });
  let errors = ''; child.stderr.on('data', bytes => { errors += bytes.toString(); });
  let reached;
  for (let index = 0; index < 100 && !reached; index++) {
    reached = await readFile(marker, 'utf8').catch(() => null);
    assert.equal(child.exitCode, null, errors); if (!reached) await delay(50);
  }
  assert.equal(reached, point, errors);
  const interrupted = observe();
  const exited = new Promise(done => child.once('exit', done)); child.kill(); await exited;
  const page = await fixture.start();
  let after;
  for (let index = 0; index < 100; index++) {
    after = observe();
    if (point.startsWith('gc_') ? after.versions.every(row => row[2] === 'deleted') : after.pending === 0) break;
    await delay(50);
  }
  assert.deepEqual(after.words, before.words); assert.deepEqual(after.cards, before.cards);
  assert.equal(after.pending, 0);
  if (point.startsWith('gc_')) assert.ok(after.versions.length && after.versions.every(row => row[2] === 'deleted'));
  else if (['renamed', 'readback', 'db_complete'].includes(point)) {
    assert.ok(after.selected?.[0]); assert.equal(after.selected[1], 1);
    const bytes = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 }));
    assert.deepEqual(Buffer.from(bytes), await readFile(resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus')));
  } else assert.equal(after.selected?.[0] ?? null, null);
  await fixture.stop(true);
  await fixture.start(); await fixture.stop(true);
  assert.deepEqual(observe(), after, 'A second restart must be idempotent');
  const result = { point, before, interrupted, after, processKilled: true, nativeRestarts: 2 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(result, null, 2));
  console.log(JSON.stringify({ point, passed: true }));
} finally { if (child?.exitCode === null) child.kill(); await fixture.finish(); }
