/* global process, window, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath] = process.argv.slice(2);
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
try {
  let page = await fixture.start();
  const before = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke; const ids = [];
    for (const [index, headword] of ['école', 'bonjour', 'rouge', 'bleu', 'maison', 'chat', 'chien', 'oui'].entries()) {
      const saved = await invoke('vocabulary_save', { request: { operation: `life-${index}`, headword, notes: 'keep', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } }); ids.push(saved.record.id);
    }
    await invoke('review_add_batch', { request: { operation: 'life-batch', vocabulary_ids: ids } });
    await invoke('review_pause', { vocabularyId: ids[0], paused: true });
    return { ids, cards: await invoke('review_cards', { vocabularyId: ids[0] }) };
  });
  let status;
  for (let i = 0; i < 100; i++) {
    status = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_batch_status', { operation: 'life-batch' }));
    if (status.items.some((item) => item.audio_state === 'running')) break;
    await delay(50);
  }
  assert.ok(status.items.some((item) => item.audio_state === 'running'));
  const workers = fixture.workerPids(); assert.equal(workers.length, 1);
  const killed = spawnSync('taskkill.exe', ['/PID', String(workers[0]), '/T', '/F'], { windowsHide: true }); assert.equal(killed.status, 0);
  let interrupted;
  for (let i = 0; i < 40; i++) {
    interrupted = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_batch_status', { operation: 'life-batch' }));
    if (interrupted.items.some((item) => item.audio_state === 'retry_wait')) break;
    await delay(25);
  }
  assert.ok(interrupted.items.some((item) => item.audio_state === 'retry_wait'), JSON.stringify(interrupted));
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_cancel_batch', { operation: 'life-batch' }));
  const cancelled = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_batch_status', { operation: 'life-batch' }));
  assert.equal(cancelled.cancelled, true); assert.ok(cancelled.items.some((item) => item.audio_state === 'cancelled'));
  await fixture.stop(true); assert.equal(fixture.workerPids().length, 0);
  // Create a controlled publication/metadata gap using an actual decoded product file.
  const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
  const inject = spawnSync('python', ['-X', 'utf8', '-c', `import hashlib,json,sqlite3,sys,time
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db)
row=c.execute("SELECT semantic_key,content_hash,generated_at FROM speech_assets WHERE content_hash IS NOT NULL LIMIT 1").fetchone()
if row is None:
 print(json.dumps({'publication_gap':'no_completed_file'}));sys.exit(0)
key,content,generated=row;file=db.parent.parent/'temp'/'speech'/f's-{key}-{content}.ogg'
assert hashlib.sha256(file.read_bytes()).hexdigest()==content
c.execute("UPDATE speech_assets SET pending_hash=content_hash,pending_generated_at=generated_at,content_hash=NULL,generated_at=NULL,expires_at=NULL WHERE semantic_key=?",(key,))
c.execute("UPDATE speech_tasks SET state='running',deadline_at=? WHERE semantic_key=?",(int(time.time())+120,key));c.commit()
print(json.dumps({'publication_gap':'prepared','semantic_key':key,'generated_at':generated}))`, database], { windowsHide: true, encoding: 'utf8' });
  assert.equal(inject.status, 0, inject.stderr); const gap = JSON.parse(inject.stdout);
  page = await fixture.start();
  const restored = await page.evaluate(async (id) => { const invoke = window.__TAURI_INTERNALS__.invoke; return {
    status: await invoke('review_batch_status', { operation: 'life-batch' }), cards: await invoke('review_cards', { vocabularyId: id }), paused: await invoke('review_pause', { vocabularyId: id, paused: null }), record: await invoke('vocabulary_find', { headword: 'école' }) }; }, before.ids[0]);
  assert.equal(restored.status.cancelled, true); assert.deepEqual(restored.cards, before.cards); assert.equal(restored.paused, true); assert.equal(restored.record.notes, 'keep');
  assert.ok(restored.status.items.every((item) => ['ready', 'failed', 'cancelled'].includes(item.audio_state)));
  if (gap.publication_gap === 'prepared') assert.ok(restored.status.items.some((item) => item.audio_state === 'ready'));
  assert.equal(fixture.workerPids().length, 0);
  const report = { hardTimeoutSeconds: 60, interrupted, cancelled, restored, gap, ownedWorkersAfterExit: 0 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
