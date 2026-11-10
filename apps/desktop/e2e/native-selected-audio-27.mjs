/* global process, window, document, AudioContext, URL, Blob, setTimeout, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function inspect(corrupt = false) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import json,sqlite3,sys
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db)
row=c.execute('SELECT s.current_version,s.previous_version,s.previous_until,s.revision,v.semantic_key,v.content_hash,v.generation,a.spec FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version LEFT JOIN speech_assets a ON a.semantic_key=v.semantic_key WHERE s.vocabulary_id=1').fetchone();assert row
locator,previous,until,revision,key,content,generation,spec=row
if sys.argv[2]=='true': (db.parent.parent/'temp'/'speech'/f's-{locator}-{content}.ogg').write_bytes(b'OggSdamaged-selected-audio')
print(json.dumps(dict(locator=locator,previous=previous,previous_until=until,revision=revision,key=key,hash=content,generation=generation,voice=json.loads(spec)['voice'],versions=c.execute('SELECT COUNT(*) FROM speech_versions').fetchone()[0],notes=c.execute('SELECT notes FROM vocabulary WHERE id=1').fetchone()[0])))`, database, String(corrupt)], { windowsHide: true, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
try {
  let page = await fixture.start();
  const original = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const { bytes } = await invoke('speech_preview', { text: 'école' });
    const request = { operation: 'selected-save', headword: 'école', notes: 'original', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null };
    const saved = await invoke('vocabulary_save', { request });
    await invoke('review_add_batch', { request: { operation: 'selected-cards', vocabulary_ids: [saved.record.id] } });
    const cards = await invoke('review_cards', { vocabularyId: saved.record.id });
    await invoke('review_rate', { request: { operation: 'selected-rate', card_id: cards[0].id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    const progress = await invoke('review_progress', { cardId: cards[0].id });
    await invoke('speech_preferences', { preferences: { voice: 'male', wpm: 100 } });
    const { bytes: male } = await invoke('speech_preview', { text: 'école' });
    await invoke('vocabulary_save', { request: { ...request, operation: 'selected-edit', id: saved.record.id, revision: 1, notes: 'keep after repair' } });
    return { bytes, male, cached: (await invoke('speech_cached', { vocabularyId: saved.record.id })).bytes, id: saved.record.id, cards, progress };
  });
  assert.deepEqual(original.cached, original.bytes);
  assert.notDeepEqual(original.male, original.bytes, 'The alternate voice must be a distinct real output');
  await fixture.stop(true);
  const before = inspect();
  assert.equal(before.voice, 'female'); assert.equal(before.versions, 2, 'Selected female and temporary male content are both registered');
  page = await fixture.start();
  const restored = await page.evaluate(async ({ id, cardId }) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const { bytes } = await invoke('speech_cached', { vocabularyId: id });
    const context = new AudioContext({ sampleRate: 48000 });
    const decoded = await context.decodeAudioData(new Uint8Array(bytes).buffer);
    const audio = document.createElement('audio');
    const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'audio/ogg' }));
    try {
      audio.src = url;
      for (const rate of [1, 0.6]) {
        audio.currentTime = 0; audio.playbackRate = rate;
        const ended = new Promise((done, reject) => { audio.onended = done; audio.onerror = reject; });
        await audio.play(); await ended;
      }
      return { bytes, duration: decoded.duration, audible: decoded.getChannelData(0).some((sample) => sample !== 0), cards: await invoke('review_cards', { vocabularyId: id }), progress: await invoke('review_progress', { cardId }) };
    } finally { audio.pause(); URL.revokeObjectURL(url); await context.close(); }
  }, { id: original.id, cardId: original.cards[0].id });
  assert.deepEqual(restored.bytes, original.bytes); assert.deepEqual(restored.cards, original.cards); assert.deepEqual(restored.progress, original.progress);
  assert.ok(restored.duration > 0 && restored.audible);
  assert.equal(fixture.workerPids().length, 0, 'Cache-only playback must not start synthesis');
  await fixture.stop(true);
  assert.deepEqual(inspect(true), before, 'Normal and slow playback must not create a version');
  page = await fixture.start();
  const repair = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    let error;
    try { await invoke('speech_cached', { vocabularyId: id }); } catch (cause) { error = String(cause); }
    const queued = await invoke('speech_retry_missing', { vocabularyId: id });
    let bytes;
    for (let attempt = 0; attempt < 200; attempt++) {
      try { ({ bytes } = await invoke('speech_cached', { vocabularyId: id })); if (bytes) break; } catch { /* The old selected copy remains visibly damaged until commit. */ }
      await new Promise((done) => setTimeout(done, 50));
    }
    return { error, queued, bytes, cards: await invoke('review_cards', { vocabularyId: id }) };
  }, original.id);
  assert.match(repair.error, /integrity check failed/); assert.equal(repair.queued, 1); assert.ok(repair.bytes?.length > 4); assert.deepEqual(repair.cards, original.cards);
  await fixture.stop(true);
  const after = inspect();
  assert.equal(after.voice, 'female'); assert.notEqual(after.locator, before.locator);
  assert.equal(after.previous, before.locator); assert.ok(after.previous_until > Date.now() / 1000 + 1_209_500);
  assert.equal(after.revision, before.revision + 1); assert.equal(after.versions, before.versions + 1); assert.equal(after.notes, 'keep after repair');
  assert.equal(fixture.workerPids().length, 0);
  const digest = (bytes) => createHash('sha256').update(new Uint8Array(bytes)).digest('hex');
  const report = { hardTimeoutSeconds: 60, before, after, originalHash: digest(original.bytes), alternateHash: digest(original.male), repairedHash: digest(repair.bytes), normalAndSlowPlayback: true, restartPreservedProgress: true, ownedWorkersAfterExit: 0 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
