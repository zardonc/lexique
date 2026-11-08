/* global process, window, document, AudioContext, URL, Blob, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { appendFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, mode] = process.argv.slice(2);
assert.ok(['quota', 'header', 'bytes', 'codec'].includes(mode));
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
const database = join(fixture.destination, 'Lexique-data/data/lexique.sqlite3');
function python(source, args = [], executable = 'python') {
  const result = spawnSync(executable, ['-X', 'utf8', '-c', source, database, ...args], { encoding: 'utf8', windowsHide: true, timeout: 15_000 });
  assert.equal(result.status, 0, result.stderr); return result.stdout ? JSON.parse(result.stdout) : null;
}
try {
  await fixture.start(); await fixture.stop(true);
  const clip = resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus');
  let source = clip;
  if (mode === 'codec') {
    source = join(fixture.destination, 'codec-16s.opus');
    python(`import sys,numpy as np
from pathlib import Path
sys.path.insert(0,sys.argv[2]);from speech_worker import encode
samples=np.random.default_rng(24).normal(0,0.05,48000*16).astype(np.float32)
Path(sys.argv[4]).write_bytes(encode(Path(sys.argv[3]),samples,48000))`, [resolve('src-tauri'), join(fixture.destination, 'speech/opusenc.dll'), source], join(fixture.destination, 'speech/runtime/python.exe'));
  }
  const seed = python(`import sqlite3,json,sys,time,hashlib
from pathlib import Path
c=sqlite3.connect(sys.argv[1]);audio=Path(sys.argv[2]).read_bytes();mode=sys.argv[3];now=int(time.time());cache=Path(sys.argv[1]).parent.parent/'temp/speech'
if mode=='header':audio=b'OggS'+b'x'*8188
if mode=='bytes':audio=b'OggS'+b'x'*9999996
content=hashlib.sha256(audio).hexdigest();count=100 if mode=='quota' else 8 if mode=='bytes' else 1
snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'synthetic','version':'1','attribution':'fixture','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[{'pronunciations':[],'grammar':[],'translation':'fixture'}]})
for i in range(1,count+1):
 c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?,?,?,'keep',1,1)",(i,f'mot{i}',snapshot))
 for identity,kind in [(i*2-1,'recognition'),(i*2,'listening')]:
  c.execute('INSERT INTO review_cards VALUES(?,?,?)',(identity,i,kind))
  if mode=='quota' and i==1:c.execute('INSERT INTO review_progress VALUES(?,1,?)',(identity,json.dumps(dict(phase='learning_first',memory=dict(stability=10,difficulty=5),last_review_at=now-120,due_at=now-60))))
 locator=f'{i:064x}'
 c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,last_used) VALUES(?,?,?,?,?,?,0,0,1,?,'word','verified',?)",(locator,content,len(audio),now,now,i,locator,now))
 c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(?,?)',(i,locator))
 if mode!='quota' or i==1:(cache/f's-{locator}-{content}.ogg').write_bytes(audio)
if mode=='quota':c.execute('UPDATE review_settings SET new_limit=0,review_limit=0')
else:c.execute("DELETE FROM review_cards WHERE kind='recognition'");c.execute('UPDATE review_settings SET new_limit=100,review_limit=100')
c.commit();print(json.dumps(dict(mode=mode,words=count,bytes=len(audio),locator=f'{1:064x}',hash=content)))`, [source, mode]);
  const page = await fixture.start();
  const view = await page.evaluate(async () => {
    const view = await window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] });
    return { summary: view.summary, cards: view.queue.cards, metrics: view.metrics, audio: view.window.map(entry => entry.audio?.bytes.length ?? 0) };
  });
  assert.ok(view.metrics.strict_probes <= 8); assert.ok(view.metrics.compressed_bytes <= 33_554_432); assert.ok(view.metrics.payload_bytes <= 33_554_432);
  let playback;
  if (mode === 'quota') {
    assert.equal(view.summary.admitted_count, 2); assert.deepEqual(view.cards.map(card => card.id), [1, 2]);
    assert.equal(view.metrics.strict_probes, 1, 'Quota-blocked new cards must not consume the file budget');
    const next = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }));
    assert.equal(next.metrics.strict_probes, 0);
  } else if (mode === 'bytes') {
    assert.equal(seed.bytes, 10_000_000);
    assert.ok(view.audio.filter(Boolean).length <= 3);
    await fixture.stop(true);
    const file = join(fixture.destination, 'Lexique-data/temp/speech', `s-${seed.locator}-${seed.hash}.ogg`);
    await appendFile(file, new Uint8Array([0]));
    const reopened = await fixture.start();
    await assert.rejects(reopened.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 })), /too large|Invalid cached speech audio/i);
  } else if (mode === 'header') {
    assert.equal(view.cards[0]?.kind, 'listening'); assert.equal(view.audio[0], 8192);
    await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
    const card = page.getByRole('region', { name: '复习卡片' });
    await card.getByRole('alert').first().waitFor();
    await card.getByRole('button', { name: '显示答案', exact: true }).click();
    assert.equal(await card.getByRole('button', { name: '忘记', exact: true }).isDisabled(), true);
    assert.equal(python("import sqlite3,sys,json;c=sqlite3.connect(sys.argv[1]);print(json.dumps(c.execute('SELECT COUNT(*) FROM review_ratings').fetchone()[0]))"), 0);
  } else {
    assert.ok(seed.bytes >= 65_536);
    playback = await page.evaluate(async () => {
      const { bytes } = await window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 });
      const context = new AudioContext({ sampleRate: 48000 });
      const decoded = await context.decodeAudioData(new Uint8Array(bytes).buffer);
      const audio = document.createElement('audio'); const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'audio/ogg' }));
      try {
        audio.src = url; const ended = new Promise((done, reject) => { audio.onended = done; audio.onerror = reject; });
        await audio.play(); await ended;
        return { duration: decoded.duration, channels: decoded.numberOfChannels, finite: decoded.getChannelData(0).every(Number.isFinite), audible: decoded.getChannelData(0).some(sample => sample !== 0) };
      } finally { audio.pause(); URL.revokeObjectURL(url); await context.close(); }
    });
    assert.ok(playback.duration >= 16 && playback.finite && playback.audible);
  }
  await fixture.stop(true); assert.equal(fixture.workerPids().length, 0);
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ seed, view, playback, ownedWorkersAfterExit: 0 }, null, 2));
  console.log(JSON.stringify({ mode, passed: true, bytes: seed.bytes, playback }));
} finally { await fixture.finish(); }
