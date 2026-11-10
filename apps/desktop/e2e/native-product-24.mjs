/* global process, window, document, performance, AudioContext, URL, Blob, setTimeout, console */
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath] = process.argv.slice(2);
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
try {
  let page = await fixture.start();
  const result = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const metrics = [];
    async function measured(command, args) { const start = performance.now(); const value = await invoke(command, args); metrics.push({ command, ms: performance.now() - start }); return value; }
    const original = await measured('speech_preview', { text: 'école' });
    const ids = [];
    for (const [index, headword] of ['école', 'bonjour', 'rouge', 'bleu', 'maison', 'chat'].entries()) {
      const saved = await measured('vocabulary_save', { request: { operation: `product-save-${index}`, headword, notes: 'isolated', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } });
      ids.push(saved.record.id);
    }
    const reuse = await measured('speech_preview', { text: 'école' });
    if (JSON.stringify(original) !== JSON.stringify(reuse)) throw new Error('Favorite did not reuse the existing temporary word asset');
    const batch = await measured('review_add_batch', { request: { operation: 'product-six', vocabulary_ids: ids } });
    const during = await measured('review_queue', { timezone: 'UTC', skipped: [] });
    const cards = await invoke('review_cards', { vocabularyId: ids[0] });
    const hot = measured('speech_preview', { text: 'oui' });
    const lookup = measured('dictionary_lookup', { key: 'école' });
    const notes = measured('vocabulary_save', { request: { operation: 'product-notes', headword: 'école', id: ids[0], revision: 1, notes: 'concurrent', example_fr: '', example_zh: '', example_sense: null, more_examples: [] } });
    const rating = measured('review_rate', { request: { operation: 'product-rating', card_id: cards[0].id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    const [interactive, dictionary, edited, rated] = await Promise.all([hot, lookup, notes, rating]);
    let status;
    for (let attempt = 0; attempt < 150; attempt++) { status = await invoke('review_batch_status', { operation: 'product-six' }); if (status.items.every((item) => ['ready', 'failed', 'cancelled'].includes(item.audio_state))) break; await new Promise((done) => setTimeout(done, 50)); }
    const decoded = [];
    for (const id of ids) {
      const { bytes } = await measured('speech_cached', { vocabularyId: id });
      const context = new AudioContext({ sampleRate: 48000 });
      try { const audio = await context.decodeAudioData(new Uint8Array(bytes).buffer); decoded.push({ channels: audio.numberOfChannels, duration: audio.duration, finite: audio.getChannelData(0).every(Number.isFinite), audible: audio.getChannelData(0).some((sample) => sample !== 0) }); } finally { await context.close(); }
    }
    const { bytes } = await invoke('speech_cached', { vocabularyId: ids[0] });
    const audio = document.createElement('audio'); const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'audio/ogg' })); audio.src = url;
    const start = performance.now(); await audio.play(); metrics.push({ command: 'actual-play-start', ms: performance.now() - start });
    await new Promise((done, reject) => { audio.onended = done; audio.onerror = reject; }); URL.revokeObjectURL(url);
    return { metrics, ids, batch, duringCardCount: during.queue.cards.length, duringAdmittedCount: during.summary.admitted_count, status, decoded, interactiveBytes: interactive.bytes.length, dictionaryCount: dictionary.entries.length, edited: edited.record.notes, rated: rated.status };
  });
  assert.ok(result.batch.items.every((item) => item.created_cards === 2));
  assert.ok(result.duringCardCount <= 4);
  assert.ok(result.duringAdmittedCount >= 6, 'Recognition cards must be admitted before all audio completes');
  assert.ok(result.status.items.every((item) => item.audio_state === 'ready'), JSON.stringify(result.status));
  assert.equal(result.edited, 'concurrent'); assert.equal(result.rated, 'committed'); assert.ok(result.dictionaryCount > 0 && result.interactiveBytes > 0);
  assert.ok(result.decoded.every((audio) => audio.channels === 1 && audio.finite && audio.audible && audio.duration > 0));
  await writeFile(join(fixture.destination, 'learning-results.json'), JSON.stringify(result, null, 2));
  const workers = fixture.workerPids(); assert.equal(workers.length, 1, 'Exactly one resident model process');
  await fixture.stop(true); assert.equal(fixture.workerPids().length, 0, 'Normal exit leaves no owned speech worker');
  page = await fixture.start();
  const restored = await page.evaluate(async (ids) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const status = await invoke('review_batch_status', { operation: 'product-six' });
    const { bytes: cached } = await invoke('speech_cached', { vocabularyId: ids[0] });
    const cards = await invoke('review_cards', { vocabularyId: ids[0] });
    return { status, cachedBytes: cached.length, progress: await invoke('review_progress', { cardId: cards[0].id }) };
  }, result.ids);
  assert.deepEqual(restored.status, result.status); assert.equal(restored.progress.revision, 1); assert.ok(restored.cachedBytes > 0);
  assert.equal(fixture.workerPids().length, 0, 'Ready/cache-only restart must not load the model');
  const report = { binarySha256: createHash('sha256').update(await readFile(resolve(binaryPath))).digest('hex'), hardTimeoutSeconds: 60, result, restored, residentProcesses: workers.length, ownedWorkersAfterExit: 0 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
