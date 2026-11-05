/* global process, window, console, performance */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile, writeFile, readdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data/data/lexique.sqlite3');
const started = performance.now();
const report = { scope: 'bounded mixed-history resource run; not hours of soak or physical sleep', samples: [], timings: [] };
try {
  await fixture.start(); await fixture.stop(true);
  const seeded = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys,time,hashlib
from pathlib import Path
sys.path.insert(0,'../../scripts');from s0_baseline import seed_rows
c=sqlite3.connect(sys.argv[1]);now=int(time.time());audio=Path(sys.argv[2]).read_bytes();content=hashlib.sha256(audio).hexdigest();locator='a'*64
triggers=c.execute("SELECT name,sql FROM sqlite_master WHERE type='trigger' AND name LIKE 'queue_revision_%'").fetchall()
for name,_ in triggers:c.execute(f'DROP TRIGGER "{name}"')
seed_rows(c,50000,'mixed',now)
c.execute("UPDATE vocabulary SET notes='private-notes-fixture-24',headword='private-headword-fixture-24-'||id")
c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,last_used) VALUES(?,?,?,?,?,1,0,0,1,?,'word','verified',?)",(locator,content,len(audio),now,now,locator,now))
c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version) SELECT id,? FROM vocabulary',(locator,))
(Path(sys.argv[1]).parent.parent/'temp/speech'/f's-{locator}-{content}.ogg').write_bytes(audio)
before=json.dumps({'phase':'review','memory':{'stability':10.0,'difficulty':5.0},'last_review_at':now-172800,'due_at':now-86400})
request=json.dumps({'operation':'historical','card_id':1,'expected_revision':0,'rating':'good','now':now-86400,'timezone':'UTC'})
c.execute("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<100000) INSERT INTO review_ratings(operation,card_id,request,before_state,result,reviewed_at,learning_day,quota,queue_before) SELECT 'history-'||x,1,?,?,?,?,'2026-01-01','review',NULL FROM n",(request,before,'{}',now-86400))
c.execute('UPDATE review_settings SET new_limit=1000,review_limit=1000')
for _,sql in triggers:c.execute(sql)
c.commit();print(json.dumps(dict(words=50000,cards=100000,history=100000,sharedSelectedVersion=True,fixtureJournalSuppressed=True)))`, database,
    resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus')], { windowsHide: true, encoding: 'utf8', timeout: 15_000 });
  assert.equal(seeded.status, 0, seeded.stderr); report.seed = JSON.parse(seeded.stdout);
  let page = await fixture.start();
  async function sample(label) {
    const cdp = await page.context().newCDPSession(page);
    const heap = await cdp.send('Runtime.getHeapUsage');
    const dom = await cdp.send('Memory.getDOMCounters'); await cdp.detach();
    const lifecycle = (await readFile(join(fixture.destination, 'lifecycle.jsonl'), 'utf8')).trim().split('\n').map(JSON.parse);
    const pid = lifecycle.findLast(row => row.event === 'host-start').pid;
    const host = spawnSync('pwsh', ['-NoProfile', '-NonInteractive', '-Command', `Get-Process -Id ${pid} | Select-Object WorkingSet64,PrivateMemorySize64,HandleCount,CPU | ConvertTo-Json -Compress`], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
    assert.equal(host.status, 0, host.stderr);
    report.samples.push({ label, elapsed_ms: performance.now() - started, heap, dom, host: JSON.parse(host.stdout), worker_pids: fixture.workerPids() });
    await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2));
  }
  await page.evaluate(() => { window.resourceQueue = { previous: null, entries: [], skipped: [], index: 0 }; });
  await sample('before-ratings');
  for (let batch = 0; batch < 4; batch++) {
    report.timings.push(...await page.evaluate(async () => {
      const state = window.resourceQueue;
      const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
      async function read() {
        const cursor = state.previous ? { revision: state.previous.summary.revision, window: state.entries.map(entry => ({ card: entry.card, record_revision: entry.record.revision, progress_revision: entry.progress.revision, version: entry.version, content_hash: entry.content_hash })) } : null;
        const start = performance.now();
        const next = await invoke('review_queue', { timezone: 'UTC', skipped: state.skipped, cursor });
        const entries = new Map(state.entries.map(entry => [entry.card.id, entry]));
        for (const id of next.delta.removed) entries.delete(id);
        for (const entry of next.window) entries.set(entry.card.id, entry);
        state.entries = next.delta.order.map(id => entries.get(id)); state.previous = next;
        return { ms: performance.now() - start, ...next.metrics };
      }
      const samples = [];
      if (!state.previous) await read();
      for (let index = 0; index < 25; index++) {
        let selected = state.entries.find(entry => entry.card.kind === 'recognition');
        if (!selected) {
          state.skipped.push(...state.entries.map(entry => entry.card.id)); await read();
          selected = state.entries.find(entry => entry.card.kind === 'recognition');
        }
        if (!selected) throw new Error('Mixed fixture has no recognition card');
        const request = { operation: `resource-rate-${state.index++}`, card_id: selected.card.id, expected_revision: selected.progress.revision, rating: 'good', timezone: 'UTC' };
        const start = performance.now();
        const result = await invoke('review_rate', { request });
        if (result.status !== 'committed') throw new Error('Rating must commit');
        const rate_ms = performance.now() - start;
        samples.push({ rate_ms, ...await read() });
      }
      return samples;
    }));
    await sample(`after-${(batch + 1) * 25}-ratings`);
  }
  assert.equal(report.timings.length, 100);
  assert.ok(report.timings.every(row => row.strict_probes <= 8 && row.compressed_bytes <= 33_554_432));
  assert.ok(report.timings.every(row => !row.metadata_rebuilt && row.metadata_loaded_rows <= 8 && row.parsed_progress <= 4), 'Ratings must apply bounded metadata deltas');
  report.latency = Object.fromEntries(['rate_ms', 'ms'].map(key => {
    const values = report.timings.map(row => row[key]).sort((a, b) => a - b);
    return [key, { attempts: values.length, p50: values[Math.ceil(values.length * .5) - 1], p95: values[Math.ceil(values.length * .95) - 1], p99: values[Math.ceil(values.length * .99) - 1], max: values.at(-1) }];
  }));
  await fixture.stop(true); assert.deepEqual(fixture.workerPids(), []);
  page = await fixture.start(); await sample('after-restart');
  const restarted = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }));
  assert.ok(restarted.queue.cards.length > 0);
  await fixture.stop(true); assert.deepEqual(fixture.workerPids(), []);
  const logs = join(fixture.destination, 'Lexique-data/logs');
  const names = await readdir(logs).catch(error => { if (error.code === 'ENOENT') return []; throw error; });
  for (const name of names) {
    const content = await readFile(join(logs, name), 'utf8');
    assert.ok(!/private-notes-fixture-24|private-headword-fixture-24|https:\/\/example.org/.test(content), 'Owned diagnostics must omit learning text and source URLs');
  }
  report.log_files_checked = names; report.elapsed_ms = performance.now() - started; report.status = 'passed';
  console.log(JSON.stringify({ status: report.status, seed: report.seed, latency: report.latency, elapsed_ms: report.elapsed_ms }));
} finally {
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2));
  await fixture.finish();
}
