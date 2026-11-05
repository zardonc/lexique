/* global process, window, document, performance, AudioContext, URL, Blob, navigator, TextEncoder, atob */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { readFile, rename, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, mode = 'interaction', countText = '1000', scenario = 'new', missingText = '0'] = process.argv.slice(2);
assert.ok(['interaction', 'scale', 'gap', 'gap-red', 'decode'].includes(mode));
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
const executionStarted = Date.now();
// The reusable package can itself be an older synthetic fixture. Keep its log
// separately so this run's lifecycle cannot be confused with historical events.
try { await rename(join(fixture.destination, 'lifecycle.jsonl'), join(fixture.destination, 'inherited-lifecycle.jsonl')); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
const report = {
  binarySha256: createHash('sha256').update(await readFile(resolve(binaryPath))).digest('hex'),
  hardTimeoutSeconds: 60, mode, samples: [], physicalDiskCold: false,
  sourceBoundary: 'fixed native binary, real IPC/SQLite; synthetic data only',
};
try {
  let page = await fixture.start();
  await page.exposeFunction('__s0Record', async (row) => {
    report.metrics ??= [];
    report.metrics.push(row);
    await writeFile(join(fixture.destination, 's0-results.json'), JSON.stringify(report, null, 2));
  });
  report.webview = await page.evaluate(() => navigator.userAgent);
  if (mode === 'decode') {
    const resources = resolve(countText);
    assert.ok(resources.startsWith(resolve('../../target/issue-25') + '\\'));
    const evidence = JSON.parse(await readFile(join(resources, 'results.json'), 'utf8'));
    const inputs = evidence.trials.flatMap(trial => trial.result.results.map(result => ({
      path: join(resources, trial.case, trial.audio_directory ?? '', result.audio_file), duration: result.duration_s,
    })));
    inputs.push({ path: join(resources, 'large-audio', evidence.large_audio.audio_directory ?? '', '0.ogg'), duration: evidence.large_audio.result.duration_s });
    report.decoded = [];
    for (const input of inputs) {
      const bytes = await readFile(input.path);
      const decoded = await page.evaluate(async (base64) => {
        const context = new AudioContext({ sampleRate: 48000 });
        try {
          const start = performance.now();
          const buffer = await context.decodeAudioData(Uint8Array.from(atob(base64), value => value.charCodeAt(0)).buffer);
          const pcm = buffer.getChannelData(0);
          return { duration: buffer.duration, channels: buffer.numberOfChannels,
            finite: pcm.every(Number.isFinite), nonSilent: pcm.some(value => value !== 0), ms: performance.now() - start };
        } finally { await context.close(); }
      }, bytes.toString('base64'));
      assert.ok(decoded.finite && decoded.nonSilent && decoded.channels === 1 && Math.abs(decoded.duration - input.duration) < .05);
      report.decoded.push({ ...decoded, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
    }
    report.headerRejected = await page.evaluate(async () => {
      const context = new AudioContext();
      try {
        const bytes = new Uint8Array(8192).fill(120); bytes.set([79, 103, 103, 83]);
        try { await context.decodeAudioData(bytes.buffer); return false; }
        catch { return true; }
      } finally { await context.close(); }
    });
    assert.ok(report.headerRejected, 'H fixture cannot be used as playable audio evidence');
  } else if (mode !== 'interaction') {
    await fixture.stop(true);
    const count = mode.startsWith('gap') ? 1 : Number(countText);
    assert.ok([1, 100, 1000, 10000].includes(count));
    const seed = spawnSync('python', ['-X', 'utf8', '-c', `import sys,sqlite3,time,json
from pathlib import Path
sys.path.insert(0,sys.argv[1]);from s0_baseline import seed_rows
db=Path(sys.argv[2]);count=int(sys.argv[3]);scenario=sys.argv[4];missing=int(sys.argv[5]);gap=sys.argv[6]=='true'
with sqlite3.connect(db) as c:
 seed_rows(c,count,'intraday' if gap else scenario,int(time.time()))
 if gap:c.execute('UPDATE review_settings SET new_limit=0,review_limit=0')
cache=db.parent.parent/'temp'/'speech';cache.mkdir(parents=True,exist_ok=True)
for i in range(1,count+1):
 if missing==100 or missing==50 and i%2==0:continue
 (cache/f'{i}.ogg').write_bytes(b'OggS'+b'x'*8188)
print(json.dumps({'words':count,'cards':count*2,'missing':missing,'audio':'header-only H; no decode/playback proof'}))`,
      resolve('../../scripts'), join(fixture.destination, 'Lexique-data/data/lexique.sqlite3'), String(count), scenario, missingText, String(mode.startsWith('gap'))],
    { windowsHide: true, encoding: 'utf8', killSignal: 'SIGKILL',
      timeout: Math.max(1, 58_000 - (Date.now() - executionStarted)) });
    assert.equal(seed.status, 0, seed.stderr);
    report.fixture = JSON.parse(seed.stdout);
    page = await fixture.start();
    for (let iteration = 0; iteration < 21; iteration++) {
      let sample;
      try {
        sample = await page.evaluate(async (iteration) => {
        const start = performance.now();
        try {
          const value = await window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] });
          return { iteration, status: 'passed', ms: performance.now() - start,
            cards: value.queue.cards.length, ids: value.queue.cards.map(card => card.id),
            responseBytes: new TextEncoder().encode(JSON.stringify(value)).length,
            reason: value.queue.reason, availableAt: value.queue.available_at };
        } catch (error) { return { iteration, status: 'failed', ms: performance.now() - start, error: String(error) }; }
        }, iteration);
      } catch (error) {
        sample = { iteration, status: 'failed', ms: null, error: String(error) };
      }
      report.samples.push(sample);
      await writeFile(join(fixture.destination, 's0-results.json'), JSON.stringify(report, null, 2));
      if (sample.status === 'failed') throw new Error(sample.error);
    }
    if (mode === 'gap') {
      assert.ok(report.samples.every(row => JSON.stringify(row.ids) === '[1]'));
      report.gap = { test: 'T24-Q04', expectedIds: [1, 2], actualIds: [1], status: 'reproduced', owner: 26 };
    }
  } else {
    const result = await page.evaluate(async () => {
      const invoke = window.__TAURI_INTERNALS__.invoke;
      const metrics = [];
      async function measured(command, args, phase) {
        const start = performance.now();
        try {
          const value = await invoke(command, args);
          const row = { command, phase, status: 'passed', ms: performance.now() - start };
          metrics.push(row); await window.__s0Record(row);
          return value;
        } catch (error) {
          const row = { command, phase, status: 'failed', ms: performance.now() - start, error: String(error) };
          metrics.push(row); await window.__s0Record(row);
          throw error;
        }
      }
      const ids = [];
      for (const [index, headword] of ['école', 'bonjour', 'rouge', 'bleu', 'maison', 'chat'].entries()) {
        const saved = await measured('vocabulary_save', { request: { operation: `s0-create-${index}`, headword, notes: 'synthetic fixture',
          example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } }, 'setup');
        ids.push(saved.record.id);
      }
      const cache = await measured('speech_preview', { text: 'école' }, 'first-model-and-word');
      await measured('review_add', { vocabularyId: ids[0] }, 'setup');
      await measured('dictionary_lookup', { key: 'école' }, 'idle');
      await measured('speech_cached', { vocabularyId: ids[0] }, 'idle');
      const idleCards = await invoke('review_cards', { vocabularyId: ids[0] });
      await measured('review_rate', { request: { operation: 's0-idle-rate', card_id: idleCards[0].id,
        expected_revision: 0, rating: 'good', timezone: 'UTC' } }, 'idle');
      const idleEdit = await measured('vocabulary_save', { request: { operation: 's0-idle-edit', id: ids[0], revision: 1,
        headword: 'école', notes: 'idle synthetic note', example_fr: '', example_zh: '', example_sense: null, more_examples: [] } }, 'idle');
      const batch = await measured('review_add_batch', { request: { operation: 's0-six', vocabulary_ids: ids } }, 'submit');
      const cards = await invoke('review_cards', { vocabularyId: ids[1] });
      const inProgress = await invoke('review_batch_status', { operation: 's0-six' });
      const concurrencyObserved = inProgress.items.some(item => item.audio_state === 'running');
      const actions = await Promise.all([
        measured('speech_preview', { text: 'oui' }, 'batch'),
        measured('dictionary_lookup', { key: 'école' }, 'batch'),
        measured('speech_cached', { vocabularyId: ids[0] }, 'batch'),
        measured('review_queue', { timezone: 'UTC', skipped: [] }, 'batch'),
        measured('vocabulary_save', { request: { operation: 's0-edit', id: ids[0], revision: idleEdit.record.revision, headword: 'école', notes: 'concurrent synthetic note',
          example_fr: '', example_zh: '', example_sense: null, more_examples: [] } }, 'batch'),
        measured('review_rate', { request: { operation: 's0-rate', card_id: cards[0].id, expected_revision: 0, rating: 'good', timezone: 'UTC' } }, 'batch'),
      ]);
      const audioBytes = new Uint8Array(cache);
      const context = new AudioContext({ sampleRate: 48000 });
      let decoded;
      try {
        const start = performance.now();
        const buffer = await context.decodeAudioData(audioBytes.slice().buffer);
        const pcm = buffer.getChannelData(0);
        decoded = { channels: buffer.numberOfChannels, duration: buffer.duration, finite: pcm.every(Number.isFinite), nonSilent: pcm.some(sample => sample !== 0) };
        metrics.push({ command: 'complete-decode', phase: 'real-audio', status: 'passed', ms: performance.now() - start });
      } finally { await context.close(); }
      const audio = document.createElement('audio');
      const url = URL.createObjectURL(new Blob([audioBytes], { type: 'audio/ogg' }));
      audio.src = url;
      try {
        const ended = new Promise((done, reject) => { audio.onended = done; audio.onerror = () => reject(new Error('Actual audio playback failed')); });
        const start = performance.now();
        await audio.play();
        metrics.push({ command: 'actual-play-start', phase: 'real-audio', status: 'passed', ms: performance.now() - start });
        await ended;
      } finally { audio.pause(); URL.revokeObjectURL(url); }
      return { metrics, ids, batch, concurrencyObserved, decoded, cacheBytes: cache.length,
        lookupEntries: actions[1].entries.length, edited: actions[4].record.notes, rated: actions[5].status };
    });
    report.result = result;
    assert.equal(result.rated, 'committed');
    assert.equal(result.edited, 'concurrent synthetic note');
    assert.ok(result.lookupEntries > 0 && result.cacheBytes > 0);
    assert.ok(result.decoded.finite && result.decoded.nonSilent && result.decoded.channels === 1);
  }
  await fixture.stop(true);
  report.ownedWorkersAfterExit = fixture.workerPids().length;
  assert.equal(report.ownedWorkersAfterExit, 0);
  report.status = 'passed';
  if (mode === 'gap-red') {
    report.contractPassed = report.samples.every(row => JSON.stringify(row.ids) === '[1,2]');
    if (!report.contractPassed) { report.status = 'expected-contract-failure'; process.exitCode = 1; }
  }
} catch (error) {
  report.status = 'failed'; report.error = String(error); process.exitCode = 1;
} finally {
  await fixture.finish();
  await writeFile(join(fixture.destination, 's0-results.json'), JSON.stringify(report, null, 2));
}
process.stdout.write(JSON.stringify({ status: report.status, mode, samples: report.samples.length, error: report.error }));
