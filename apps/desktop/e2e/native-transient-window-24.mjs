/* global process, window, console */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
let locker;
try {
  await fixture.start(); await fixture.stop(true);
  const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
  const seeded = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys,hashlib,time
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db);audio=Path(sys.argv[2]).read_bytes();content=hashlib.sha256(audio).hexdigest();now=int(time.time());locator='1'*64
snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'synthetic','version':'1','attribution':'synthetic','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'mot',?,'keep',1000,1000)",(snapshot,))
c.execute("INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(1,1,'listening')")
c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,last_used) VALUES(?,?,?,?,?,1,0,0,1,?,'word','verified',?)",(locator,content,len(audio),now,now,locator,now))
c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(1,?)',(locator,));c.commit()
path=db.parent.parent/'temp'/'speech'/f's-{locator}-{content}.ogg';path.write_bytes(audio);print(path)`, database,
    resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus')], { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(seeded.status, 0, seeded.stderr);
  locker = spawn('python', ['-u', '-c', `import ctypes,sys
from ctypes import wintypes
k=ctypes.WinDLL('kernel32',use_last_error=True);k.CreateFileW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,wintypes.HANDLE];k.CreateFileW.restype=wintypes.HANDLE
h=k.CreateFileW(sys.argv[1],0x80000000,0,None,3,0,None)
if h==wintypes.HANDLE(-1).value:raise ctypes.WinError(ctypes.get_last_error())
print('locked',flush=True);sys.stdin.readline();k.CloseHandle.argtypes=[wintypes.HANDLE];k.CloseHandle(h)`, seeded.stdout.trim()], { windowsHide: true });
  await new Promise((done, reject) => {
    locker.stdout.once('data', (bytes) => { if (bytes.toString().includes('locked')) done(); else reject(new Error(bytes.toString())); });
    locker.once('error', reject); locker.once('exit', (code) => reject(new Error(`File locker exited before readiness: ${code}`)));
    locker.stderr.on('data', (bytes) => reject(new Error(bytes.toString())));
  });
  const page = await fixture.start();
  const read = () => page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }));
  const blocked = await read();
  assert.equal(blocked.queue.cards.length, 0, 'Exclusive sharing failure must exclude the listening card');
  assert.ok(blocked.audio_error, 'Sharing failure must remain visible');
  const throttled = await read();
  assert.equal(throttled.metrics.strict_probes, 0, 'An immediate retry must respect transient failure backoff');
  await delay(1_100);
  const repeated = await read();
  assert.equal(repeated.metrics.strict_probes, 1, 'An expired transient failure must permit one bounded probe');
  assert.equal(repeated.queue.cards.length, 0);
  assert.ok(repeated.audio_error);
  const throttledAgain = await read();
  assert.equal(throttledAgain.metrics.strict_probes, 0, 'A repeated failure must renew the retry backoff');
  const released = new Promise((done) => locker.once('exit', done)); locker.stdin.end('\n'); await released; locker = undefined;
  await delay(1_100);
  let recovered = await read();
  for (let attempt = 0; !recovered.queue.cards.length && attempt < 10; attempt++) { await delay(100); recovered = await read(); }
  const report = { blocked, throttled, repeated, throttledAgain, recovered };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2));
  assert.equal(recovered.queue.cards.length, 1, 'Releasing a transient file lock must allow recovery without restarting the app');
  assert.equal(recovered.audio_error, null);
  assert.ok(recovered.window[0].audio.length > 0);
  console.log(JSON.stringify({ blocked: blocked.audio_error, recovered: recovered.queue.cards, metrics: recovered.metrics }));
  await fixture.stop(true);
} finally {
  if (locker?.exitCode === null) { const exited = new Promise((done) => locker.once('exit', done)); locker.stdin.end('\n'); await exited; }
  await fixture.finish();
}
