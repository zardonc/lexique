/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
try {
  let page = await fixture.start();
  const id = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const saved = await invoke('vocabulary_save', { request: { operation: 'corrupt-word', headword: 'école', notes: 'keep', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } });
    await invoke('review_add_batch', { request: { operation: 'corrupt-batch', vocabulary_ids: [saved.record.id] } });
    return saved.record.id;
  });
  await fixture.stop(true);
  const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
  const seed = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,sys,time
c=sqlite3.connect(sys.argv[1]);c.execute("UPDATE speech_tasks SET state='queued',attempts=0,wake_at=?,deadline_at=NULL,cancel_requested=0,refresh_requested=0",(int(time.time()),));c.execute('UPDATE speech_assets SET content_hash=NULL');c.commit()`, database], { windowsHide: true, encoding: 'utf8' });
  assert.equal(seed.status, 0, seed.stderr);
  await writeFile(join(fixture.destination, 'Lexique-data', 'temp', 'speech', `${id}.ogg`), 'BAD!');
  page = await fixture.start();
  const result = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const retried = await invoke('speech_retry_missing', { vocabularyId: id });
    const { bytes: audio } = await invoke('speech_preview', { text: 'bonjour' });
    return { retried, header: audio.slice(0, 4), bytes: audio.length };
  }, id);
  assert.equal(result.retried, 1, 'Corrupt legacy cache must not terminate the supervisor');
  assert.deepEqual(result.header, [79, 103, 103, 83]);
  await fixture.stop(true);
  assert.equal(fixture.workerPids().length, 0);
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));
} finally { await fixture.finish(); }
