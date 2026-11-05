/* global window, AudioContext, console */
import assert from 'node:assert/strict';
import { Buffer } from 'node:buffer';
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

export async function runAudioOracle(packagePath, binaryPath, destinationPath) {
  const found = spawnSync('uv', ['python', 'find'], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
  assert.equal(found.status, 0, found.stderr);
  const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
  const database = join(fixture.destination, 'Lexique-data/data/lexique.sqlite3');
  const report = { scope: '8 mixed-state words, 16 cards, selected Opus audio, native publication and competing SQLite connection',
    limitations: ['No clean-system offline test', 'No playback UI assertion', 'No crash interruption or simultaneous writer barriers', 'No performance claim'], checks: [] };
  function sql(source, ...args) {
    const child = spawnSync(found.stdout.trim(), ['-X', 'utf8', '-c', `import sqlite3,json,sys,time,hashlib
from pathlib import Path
sys.path.insert(0,sys.argv[2])
c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA foreign_keys=ON')
${source}`, database, resolve('../../scripts'), ...args], { encoding: 'utf8', windowsHide: true, timeout: 8000 });
    assert.equal(child.status, 0, child.stderr); return JSON.parse(child.stdout);
  }
  function snapshot() {
    return sql(`from s0_baseline import oracle
c.execute('BEGIN');now=int(time.time());health={};versions={};cache=Path(sys.argv[1]).parent.parent/'temp'/'speech'
for word,locator,physical,checksum,state,length in c.execute('SELECT s.vocabulary_id,v.locator,v.physical_locator,v.content_hash,v.verification,v.byte_length FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version'):
 path=cache/f's-{physical}-{checksum}.ogg';data=path.read_bytes() if path.exists() else b''
 health[word]='healthy' if data.startswith(b'OggS') and len(data)==length and hashlib.sha256(data).hexdigest()==checksum and state in ('unverified','verified') else 'missing'
 versions[word]=dict(locator=locator,path=str(path),health=health[word],verification=state)
expected=oracle(c,now,'UTC',health)
busy=c.execute("SELECT COUNT(*) FROM speech_tasks WHERE state IN ('queued','running','retry_wait')").fetchone()[0]
print(json.dumps(dict(now=now,expected=expected,versions=versions,busy=busy)))`);
  }
  let page;
  const invoke = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
  async function compare(name) {
    for (let attempt = 0; attempt < 80; attempt++) {
      const before = snapshot();
      const actual = await invoke('review_queue', { timezone: 'UTC', skipped: [] });
      const check = { name, attempt, before, actual };
      report.checks.push(check);
      await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2));
      const wanted = before.expected;
      if (actual.summary.verification_pending || snapshot().busy > 0) { await delay(50); continue; }
      assert.deepEqual(actual.queue.cards.map((card) => card.id), wanted.ids.slice(0, 4), `${name}: visible order`);
      assert.equal(actual.summary.admitted_count, wanted.ids.length, `${name}: admitted count`);
      assert.equal(actual.queue.reason, wanted.reason, `${name}: reason`);
      assert.equal(actual.queue.available_at, wanted.available_at, `${name}: wake`);
      assert.equal(actual.summary.learning_day, wanted.learning_day, `${name}: learning day`);
      for (const entry of actual.window.filter((entry) => entry.card.kind === 'listening')) {
        const version = before.versions[entry.card.vocabulary_id];
        assert.equal(entry.version, version.locator, `${name}: selected version`);
        assert.deepEqual(Buffer.from(entry.audio), await readFile(version.path), `${name}: same selected bytes`);
      }
      check.status = 'passed'; return actual;
    }
    assert.fail(`${name}: pending verification/publication did not converge`);
  }
  try {
    await fixture.start(); await fixture.stop(true);
    report.seed = sql(`from s0_baseline import seed_rows
now=int(time.time());seed_rows(c,8,'mixed',now)
names=['école','bonjour','merci','livre','chien','maison','fleur','voiture'];data=Path(sys.argv[3]).read_bytes();checksum=hashlib.sha256(data).hexdigest();cache=Path(sys.argv[1]).parent.parent/'temp'/'speech'
for word,name in enumerate(names,1):
 c.execute('UPDATE vocabulary SET headword=? WHERE id=?',(name,word));locator=f'{word:064x}'
 c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,sealed,physical_locator,context,verification,last_used) VALUES(?,?,?,?,?,?,1,?,'word','verified',?)",(locator,checksum,len(data),now,now,word,locator,now))
 c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(?,?)',(word,locator));(cache/f's-{locator}-{checksum}.ogg').write_bytes(data)
c.commit();print(json.dumps(dict(now=now,words=8,cards=16,schema=c.execute('PRAGMA user_version').fetchone()[0])))`, resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus'));
    page = await fixture.start();
    const initial = await compare('healthy mixed recognition and listening');
    const listening = initial.window.find((entry) => entry.card.kind === 'listening');
    assert.ok(listening, 'Mixed fixture must expose listening in its first window');
    const decoded = await page.evaluate(async (bytes) => {
      const context = new AudioContext();
      try { const audio = await context.decodeAudioData(new Uint8Array(bytes).buffer); return { duration: audio.duration, audible: audio.getChannelData(0).some((value) => value !== 0) }; }
      finally { await context.close(); }
    }, listening.audio);
    assert.ok(decoded.duration > 0 && decoded.audible); report.decoded = decoded;
    const old = snapshot().versions[listening.card.vocabulary_id];
    const published = await invoke('speech_generate', { vocabularyId: listening.card.vocabulary_id, regenerate: true, operation: 'oracle-selected-replacement' });
    assert.ok(published.length > 4);
    const replacement = await compare('native selected-version publication');
    assert.notEqual(snapshot().versions[listening.card.vocabulary_id].locator, old.locator);
    const candidate = replacement.window.find((entry) => entry.card.kind === 'listening');
    const revision = (await invoke('review_progress', { cardId: candidate.card.id })).revision;
    const request = { operation: 'oracle-competing-pause-rating', card_id: candidate.card.id, expected_revision: revision, rating: 'again', timezone: 'UTC' };
    sql(`c.execute('BEGIN IMMEDIATE');c.execute('INSERT OR IGNORE INTO review_paused VALUES(?)',(int(sys.argv[3]),));c.commit();print('true')`, String(candidate.card.vocabulary_id));
    const rejected = await invoke('review_rate', { request });
    assert.equal(rejected.status, 'rejected'); assert.equal(rejected.code, 'paused'); report.competingPauseRejection = rejected;
    await compare('second connection pause invalidates warmed queue');
    sql(`c.execute('BEGIN IMMEDIATE');c.execute('DELETE FROM review_paused WHERE vocabulary_id=?',(int(sys.argv[3]),));c.execute('UPDATE review_settings SET new_limit=2,review_limit=3 WHERE id=1');c.commit();print('true')`, String(candidate.card.vocabulary_id));
    const limited = await compare('second connection resume and limits');
    const rateCard = limited.queue.cards.find((card) => card.kind === 'listening');
    assert.ok(rateCard, 'Competing limits must retain an eligible listening card');
    const progress = await invoke('review_progress', { cardId: rateCard.id });
    const rated = await invoke('review_rate', { request: { ...request, operation: 'oracle-audio-rating', card_id: rateCard.id, expected_revision: progress.revision } });
    assert.equal(rated.status, 'committed');
    await compare('real rating updates mixed queue');
    assert.equal((await invoke('review_undo', { operation: 'oracle-audio-undo', ratingOperation: 'oracle-audio-rating' })).status, 'committed');
    await compare('undo restores mixed queue');
    await invoke('review_settings', { settings: { retention_percent: 90, new_limit: 100, review_limit: 100, new_cards_ignore_review_limit: false } });
    await fixture.stop(true);
    const missingWord = listening.card.vocabulary_id;
    report.faults = sql(`missing=int(sys.argv[3]);faults=[]
for word in [missing,next(word for word in range(1,9) if word!=missing)]:
 locator,physical,checksum,length=c.execute('SELECT v.locator,v.physical_locator,v.content_hash,v.byte_length FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id=?',(word,)).fetchone();path=Path(sys.argv[1]).parent.parent/'temp'/'speech'/f's-{physical}-{checksum}.ogg'
 if word==missing:path.unlink()
 else:path.write_bytes(path.read_bytes()+b'x')
 faults.append(dict(word=word,locator=locator,fault='missing' if word==missing else 'length mismatch',expected_bytes=length))
print(json.dumps(faults))`, String(missingWord));
    page = await fixture.start();
    const recovered = await compare('restart and missing or wrong-length selected files converge');
    assert.ok(recovered.queue.cards.length > 0);
    const after = snapshot();
    for (const fault of report.faults) assert.ok(after.versions[fault.word].health === 'missing' || after.versions[fault.word].locator !== fault.locator);
    report.status = 'passed';
    await fixture.stop(true);
    console.log(JSON.stringify({ status: report.status, checks: report.checks.filter((check) => check.status === 'passed').length, destination: fixture.destination }));
  } catch (error) { report.status = 'failed'; report.error = String(error); throw error; }
  finally { await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); await fixture.finish(); }
}
