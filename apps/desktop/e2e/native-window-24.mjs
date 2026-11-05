/* global process, window, console, performance */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, countText = '100', mode = 'healthy', sampleText] = process.argv.slice(2);
const count = Number(countText);
const sampleCount = Number(sampleText ?? (mode === 'corrupt' ? 50 : 100));
assert.ok([50, 100].includes(sampleCount));
assert.ok([100, 1000, 10000, 50000].includes(count));
assert.ok(['healthy', 'missing-half', 'missing-all', 'corrupt'].includes(mode));
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
try {
  await fixture.start(); await fixture.stop(true);
  const seeded = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys,hashlib,time
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db);audio=Path(sys.argv[2]).read_bytes();content=hashlib.sha256(audio).hexdigest();now=int(time.time());cache=db.parent.parent/'temp'/'speech'
snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'synthetic','version':'1','attribution':'synthetic','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
schema=c.execute('PRAGMA user_version').fetchone()[0]
triggers=c.execute("SELECT name,sql FROM sqlite_master WHERE type='trigger' AND name LIKE 'queue_revision_%'").fetchall()
for name,_ in triggers:c.execute(f'DROP TRIGGER "{name}"')
count=int(sys.argv[3]);mode=sys.argv[4]
for i in range(1,count+1):
 c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?,?,?,'keep',1000,1000)",(i,f'mot{i}',snapshot))
 for card,kind in ((i*2-1,'recognition'),(i*2,'listening')):c.execute('INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?,?,?)',(card,i,kind))
 locator=f'{i:064x}'
 if schema>=15:c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,last_used) VALUES(?,?,?,?,?,?,0,0,1,?,'word','verified',?)",(locator,content,len(audio),now,now,i,locator,now))
 else:c.execute('INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed) VALUES(?,?,?,?,?,?,0,0,1)',(locator,content,len(audio),now,now,i))
 c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(?,?)',(i,locator))
 if (count<50000 or i<=100) and mode!='missing-all' and not(mode=='missing-half' and i%2==0): (cache/f's-{locator}-{content}.ogg').write_bytes(b'OggSdamaged' if mode=='corrupt' else audio)
for _,sql in triggers:c.execute(sql)
c.commit();print(json.dumps(dict(words=count,cards=count*2,bytes=len(audio),hash=content,schema=schema,fixtureJournalSuppressed=True,audioFilesLimitedToFirst100=count==50000)))`, database,
    resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus'), String(count), mode], { windowsHide: true, encoding: 'utf8', timeout: 30_000 });
  assert.equal(seeded.status, 0, seeded.stderr);
  const page = await fixture.start();
  let view;
  const initialFailures = [];
  for (let attempt = 0; attempt < 3 && !view; attempt++) {
    const started = performance.now();
    try { view = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] })); }
    catch (error) {
      if (mode === 'healthy') throw error;
      assert.match(String(error), /queue changed.*retry/i);
      initialFailures.push({ error: String(error), ms: performance.now() - started });
    }
  }
  assert.ok(view, 'The bounded initial retries must produce a queue response');
  await writeFile(join(fixture.destination, 'window-checkpoint.json'), JSON.stringify({ seed: JSON.parse(seeded.stdout), view }, null, 2));
  assert.ok(view.queue.cards.length <= 4, `Expected at most four verified cards, received ${view.queue.cards.length}`);
  assert.equal(view.summary.admitted_count, 20, 'Window size must not replace the quota-admitted count');
  assert.equal(view.window.length, view.queue.cards.length);
  assert.ok(view.metrics.strict_probes <= 8);
  assert.ok(view.metrics.compressed_bytes <= 33_554_432);
  const warm = await page.evaluate(async ({ first, sampleCount }) => {
    let previous = first; let entries = new Map(first.window.map((entry) => [entry.card.id, entry]));
    const samples = [];
    for (let index = 0; index < 5 + sampleCount; index++) {
      const cursor = { revision: previous.summary.revision, window: [...entries.values()].map((entry) => ({ card: entry.card,
        record_revision: entry.record.revision, progress_revision: entry.progress.revision, version: entry.version, content_hash: entry.content_hash })) };
      const start = performance.now();
      let next;
      try { next = await window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [], cursor }); }
      catch (error) {
        if (index >= 5) samples.push({ status: 'failed', error: String(error), ms: performance.now() - start });
        continue;
      }
      for (const id of next.delta.removed) entries.delete(id);
      for (const entry of next.window) { const old = entries.get(entry.card.id); entries.set(entry.card.id, { ...entry, audio: entry.audio ?? old?.audio ?? null }); }
      entries = new Map(next.delta.order.map((id) => [id, entries.get(id)])); previous = next;
      if (index >= 5) samples.push({ status: 'passed', ms: performance.now() - start, changes: next.window.length, ...next.metrics, revision: next.summary.revision });
    }
    return { samples, versions: [...entries.values()].map((entry) => entry.version), cards: previous.queue.cards };
  }, { first: view, sampleCount });
  assert.equal(warm.samples.length, sampleCount);
  assert.ok(warm.samples.filter(sample => sample.status === 'passed').every(sample => sample.strict_probes <= 8 && sample.compressed_bytes <= 33_554_432), 'Every response must respect the file and memory budgets');
  assert.ok(warm.samples.filter(sample => sample.status === 'failed').every(sample => /queue changed.*retry/i.test(sample.error)), 'Unexpected failures must not enter timing results as success');
  if (mode === 'healthy') assert.ok(warm.samples.every((sample) => sample.changes === 0 && sample.payload_bytes === 0 && sample.strict_probes === 0 && sample.metadata_loaded_rows === 0 && sample.parsed_progress === 0), 'Warm unchanged deltas must reuse bytes and decoded progress');
  const sorted = warm.samples.map(sample => sample.ms).sort((a, b) => a - b);
  const latency = { attempts: sorted.length, failures: warm.samples.filter(sample => sample.status === 'failed').length, method: 'nearest rank over all attempts', p50: sorted[Math.ceil(sorted.length * .5) - 1], p95: sorted[Math.ceil(sorted.length * .95) - 1], max: sorted.at(-1), physicalDiskCold: false };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ mode, seed: JSON.parse(seeded.stdout), cards: view.queue.cards.length, summary: view.summary, metrics: view.metrics, initialFailures, latency, warm }, null, 2));
  console.log(JSON.stringify({ count, mode, cards: view.queue.cards.length, summary: view.summary, metrics: view.metrics, latency }));
  await fixture.stop(true);
} finally { await fixture.finish(); }
