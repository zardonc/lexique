/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, sequenceTimezone = 'Europe/Paris', mode = 'recognition'] = process.argv.slice(2);
if (mode === 'audio') {
  const { runAudioOracle } = await import('./native-oracle-audio-24.mjs');
  await runAudioOracle(packagePath, binaryPath, destinationPath);
  process.exit(0);
}
const found = spawnSync('uv', ['python', 'find'], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
assert.equal(found.status, 0, found.stderr);
const python = found.stdout.trim();
const scripts = resolve('../../scripts');
const hash = async (path) => createHash('sha256').update(await readFile(path)).digest('hex');
const identity = { binary: resolve(binaryPath), binary_sha256: await hash(resolve(binaryPath)), node_tz: process.versions.tz, node_icu: process.versions.icu,
  oracle_sha256: await hash(join(scripts, 's0_baseline.py')), clock_sha256: await hash(join(scripts, 's0-clock.mjs')) };
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
const results = { identity, sequenceTimezone, scope: '40 mixed-state words; recognition only; six baseline cases plus 16 seeded native operations and raw-state queue comparisons',
  gaps: ['listening/audio validation and publication', 'independent scheduling-output oracle', 'long/random-fuzz and multi-connection races', 'DST/04:00 boundary crossings', 'large scale/performance', 'crash recovery'], cases: [] };
function runPython(source, ...args) {
  const result = spawnSync(python, ['-X', 'utf8', '-c', source, database, scripts, ...args],
    { windowsHide: true, encoding: 'utf8', timeout: 8000 });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
  return JSON.parse(result.stdout);
}
function oracle(now, timezone, skipped) {
  return runPython(`import sqlite3,json,sys
sys.path.insert(0,sys.argv[2])
from s0_baseline import oracle
c=sqlite3.connect(sys.argv[1]);c.execute('BEGIN')
result=oracle(c,int(sys.argv[3]),sys.argv[4],{},json.loads(sys.argv[5]))
print(json.dumps(result))`, String(now), timezone, JSON.stringify(skipped));
}
try {
  await fixture.start(); await fixture.stop(true);
  results.seed = runPython(`import sqlite3,json,sys,time
sys.path.insert(0,sys.argv[2])
from s0_baseline import seed_rows,SEED
c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA foreign_keys=ON');now=int(time.time())
seed_rows(c,40,'mixed',now)
c.execute("DELETE FROM review_progress WHERE card_id IN (SELECT id FROM review_cards WHERE kind='listening')")
c.execute("DELETE FROM review_cards WHERE kind='listening'");c.commit()
print(json.dumps(dict(seed=SEED,now=now,scenario='mixed',words=40,cards=c.execute('SELECT COUNT(*) FROM review_cards').fetchone()[0],schema=c.execute('PRAGMA user_version').fetchone()[0])))`);
  await writeFile(join(fixture.destination, 'seed.json'), JSON.stringify(results.seed, null, 2));
  assert.equal(results.seed.cards, 40);
  let page = await fixture.start();
  const invoke = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
  const setLimits = (newLimit, reviewLimit) => invoke('review_settings', { settings: {
    retention_percent: 90, new_limit: newLimit, review_limit: reviewLimit, new_cards_ignore_review_limit: false } });
  async function compare(name, timezone = 'UTC', skipped = []) {
    const now = Math.floor(Date.now() / 1000);
    const expected = oracle(now, timezone, skipped);
    const actual = await invoke('review_queue', { timezone, skipped });
    const item = { name, now, timezone, skipped, expected, actual };
    results.cases.push(item);
    await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(results, null, 2));
    assert.deepEqual(actual.queue.cards.map((card) => card.id), expected.ids.slice(0, 4), `${name}: visible order`);
    assert.equal(actual.summary.admitted_count, expected.ids.length, `${name}: admitted count`);
    assert.equal(actual.queue.reason, expected.reason, `${name}: ending reason`);
    assert.equal(actual.queue.available_at, expected.available_at, `${name}: future wake`);
    assert.equal(actual.summary.learning_day, expected.learning_day, `${name}: learning day`);
    assert.equal(actual.summary.audio_pending_count, 0, `${name}: recognition needs no audio demand`);
    assert.equal(actual.metrics.strict_probes, 0, `${name}: recognition needs no file probes`);
    item.status = 'passed';
    return actual;
  }
  await setLimits(20, 50);
  const initial = await compare('default UTC');
  await compare('default Vancouver', 'America/Vancouver');
  await setLimits(0, 0);
  const unlimited = await compare('zero quotas retain intraday');
  assert.ok(unlimited.summary.admitted_count > 0, 'Fixture must actually exercise intraday repetitions');
  const pausedWord = initial.queue.cards[0].vocabulary_id;
  await invoke('review_pause', { vocabularyId: pausedWord, paused: true });
  await setLimits(4, 7);
  const paused = await compare('reduced quotas and paused current word');
  assert.ok(!paused.queue.cards.some((card) => card.vocabulary_id === pausedWord));
  const skipped = paused.queue.cards.map((card) => card.id);
  await compare('session skips preserve saved anchors', 'UTC', skipped);
  await invoke('review_pause', { vocabularyId: pausedWord, paused: false });
  await setLimits(20, 50);
  const restored = await compare('resume and restore quotas');
  results.operations = runPython(`import json,sys
sys.path.insert(0,sys.argv[2])
from s0_baseline import operations
print(json.dumps(operations(16)))`);
  const state = { timezone: 'UTC', skipped: [], view: restored, lastRating: null, pausedWord: null };
  const actions = {
    async rate(operation) {
      const card = state.view.queue.cards[operation.value % state.view.queue.cards.length];
      assert.ok(card, 'Seeded rating must have an eligible visible card');
      const progress = await invoke('review_progress', { cardId: card.id });
      const request = { operation: `oracle-${results.seed.now}-${operation.step}-rate`, card_id: card.id,
        expected_revision: progress.revision, rating: 'again', timezone: state.timezone };
      const outcome = await invoke('review_rate', { request });
      assert.equal(outcome.status, 'committed');
      assert.deepEqual(await invoke('review_rate', { request }), outcome, 'Original operation retry must be idempotent');
      state.lastRating = request.operation;
      operation.request = request; operation.outcome = outcome;
    },
    async undo(operation) {
      assert.equal(await invoke('review_undo_target'), state.lastRating);
      const args = { operation: `oracle-${results.seed.now}-${operation.step}-undo`, ratingOperation: state.lastRating };
      const outcome = await invoke('review_undo', args);
      assert.equal(outcome.status, 'committed');
      assert.deepEqual(await invoke('review_undo', args), outcome, 'Original undo retry must be idempotent');
      operation.request = args; operation.outcome = outcome;
    },
    async pause(operation) {
      state.pausedWord = operation.word;
      assert.equal(await invoke('review_pause', { vocabularyId: state.pausedWord, paused: true }), true);
    },
    async resume(operation) {
      operation.resumedWord = state.pausedWord;
      assert.equal(await invoke('review_pause', { vocabularyId: state.pausedWord, paused: false }), false);
    },
    async limits(operation) { await setLimits(operation.value % 9, operation.value % 13); },
    async timezone(operation) { state.timezone = operation.value % 2 === 0 ? 'UTC' : sequenceTimezone; },
    async skip(operation) {
      const card = state.view.queue.cards[operation.value % state.view.queue.cards.length];
      assert.ok(card, 'Seeded skip must have a visible card');
      state.skipped.push(card.id); operation.skippedCard = card.id;
    },
    async reopen() {
      await fixture.stop(true); page = await fixture.start(); state.skipped = [];
    },
  };
  for (const operation of results.operations) {
    // The shared plan's audio slot becomes a session skip in this recognition-only fixture.
    const kind = operation.kind === 'audio' ? 'skip' : operation.kind;
    operation.executedKind = kind;
    await actions[kind](operation);
    state.view = await compare(`seeded ${operation.step}: ${kind}`, state.timezone, [...state.skipped]);
  }
  assert.ok(results.cases.some((item) => item.expected.used.review > 0), 'Sequence must observe a committed quota charge before undo');
  results.status = 'passed';
  await fixture.stop(true);
  console.log(JSON.stringify({ status: results.status, cases: results.cases.length, seed: results.seed, identity }));
} catch (error) {
  results.status = 'failed'; results.error = String(error); throw error;
} finally {
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(results, null, 2));
  await fixture.finish();
}
