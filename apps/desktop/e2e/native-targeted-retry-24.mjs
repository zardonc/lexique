/* global process, window, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
try {
  let page = await fixture.start();
  const ids = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke, ids = [];
    for (const [index, headword] of ['école', 'bonjour'].entries()) {
      const saved = await invoke('vocabulary_save', { request: { operation: `target-${index}`, headword,
        notes: 'keep', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } });
      ids.push(saved.record.id);
    }
    await invoke('review_add_batch', { request: { operation: 'target-batch', vocabulary_ids: ids } });
    return ids;
  });
  await fixture.stop(true);
  const seeded = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,sys
c=sqlite3.connect(sys.argv[1]);c.execute("UPDATE speech_tasks SET state='cancelled',cancel_requested=1");c.execute("UPDATE speech_assets SET content_hash=NULL,generated_at=NULL,expires_at=NULL");c.commit()`,
    join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3')], { windowsHide: true, encoding: 'utf8' });
  assert.equal(seeded.status, 0, seeded.stderr);
  page = await fixture.start();
  const result = await page.evaluate(async (ids) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const view = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
    const retried = await invoke('speech_retry_missing', { vocabularyId: ids[0] });
    const batch = await invoke('review_batch_status', { operation: 'target-batch' });
    let omitted;
    try { await invoke('speech_retry_missing'); } catch (error) { omitted = String(error); }
    return { view, retried, batch, omitted };
  }, ids);
  assert.equal(result.retried, 1);
  assert.equal(result.view.audio_vocabulary_id, ids[0]);
  assert.notEqual(result.batch.items[0].audio_state, 'cancelled');
  assert.equal(result.batch.items[1].audio_state, 'cancelled', 'Explicit retry must preserve unrelated cancellation');
  assert.match(result.omitted, /vocabularyId|vocabulary_id/);
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));
} finally { await fixture.finish(); }
