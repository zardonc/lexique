/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
const clip = resolve('../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus');
function python(source) {
  const result = spawnSync('python', ['-X','utf8','-c', source, database, clip], { windowsHide:true,encoding:'utf8',timeout:10_000 });
  assert.equal(result.status,0,result.stderr); return result.stdout ? JSON.parse(result.stdout) : null;
}
const observe = () => python(`import sqlite3,json,sys
c=sqlite3.connect(sys.argv[1]);expired=c.execute("SELECT COUNT(*) FROM speech_versions WHERE expires_at<strftime('%s','now') AND gc_state='deleted'").fetchone()[0]
unowned=c.execute("SELECT COALESCE(SUM(byte_length),0) FROM audio_version_protection WHERE gc_state<>'deleted' AND NOT owned").fetchone()[0]
print(json.dumps(dict(expired=expired,unowned=unowned,business=c.execute('SELECT id,notes,revision FROM vocabulary').fetchall(),cards=c.execute('SELECT id,vocabulary_id,kind FROM review_cards ORDER BY id').fetchall(),ratings=c.execute('SELECT COUNT(*) FROM review_ratings').fetchone()[0],current=c.execute('SELECT current_version,previous_version FROM word_audio_selections').fetchall())))`);
try {
  await fixture.start(); await fixture.stop(true);
  const seed = python(`import sqlite3,json,sys,time,hashlib
from pathlib import Path
db=Path(sys.argv[1]);cache=db.parent.parent/'temp'/'speech';audio=Path(sys.argv[2]).read_bytes();large=audio*1400;now=int(time.time());c=sqlite3.connect(db)
snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'école','version':'1','attribution':'synthetic','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'école',?,'keep',1000,1000)",(snapshot,))
c.execute("INSERT INTO review_cards VALUES(1,1,'recognition')");c.execute("INSERT INTO review_cards VALUES(2,1,'listening')")
files=[]
for i in range(1,340):
 data=large if i>=304 else audio;content=hashlib.sha256(data).hexdigest();locator=f'{i:064x}';generated=now-604801 if 3<=i<=303 else now-120;expiry=generated+604800
 c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,expires_at,last_used) VALUES(?,?,?,?,?,1,0,0,1,?,'word','verified',?,?)",(locator,content,len(data),generated,generated,locator,expiry,generated))
 name=f's-{locator}-{content}.ogg';(cache/name).write_bytes(data);files.append(name)
c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version,previous_version,previous_until) VALUES(1,?,?,?)',(f'{1:064x}',f'{2:064x}',now+1209600))
(cache/'unregistered-user.ogg').write_bytes(audio);c.commit();print(json.dumps(dict(expired=301,files=len(files),permanent=files[0],previous=files[1],clip_bytes=len(audio),large_bytes=len(large))))`);
  const before=observe(); assert.ok(before.unowned>209_715_200,'The fixture must exert real capacity pressure');
  const page=await fixture.start(); let after=observe();
  for (let index=0;index<40 && after.expired!==301;index++) { await delay(100); after=observe(); }
  let status=await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('runtime_status'));
  for (let index=0;index<40 && status.audioCache?.in_progress;index++) {
    await delay(100); status=await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('runtime_status'));
  }
  after=observe();
  await writeFile(join(fixture.destination,'lifecycle-checkpoint.json'),JSON.stringify({seed,before,after,status},null,2));
  assert.equal(after.expired,301,'Maintenance must continue after the first 100 reclaimable files');
  assert.ok(after.unowned<=209_715_200,'Unprotected temporary content must meet the logical soft target');
  assert.deepEqual(after.business,before.business);assert.deepEqual(after.cards,before.cards);assert.deepEqual(after.current,before.current);assert.equal(after.ratings,0);
  const cache=join(fixture.destination,'Lexique-data','temp','speech');const original=await readFile(clip);
  for (const name of [seed.permanent,seed.previous,'unregistered-user.ogg']) assert.deepEqual(await readFile(join(cache,name)),original);
  assert.ok(status.audioCache,'Capacity and unknown files must be reported');
  assert.equal(status.audioCache.in_progress,false,'The bounded reconciliation must finish');
  assert.equal(status.audioCache.error,null);
  assert.equal(status.audioCache.unknown_files,1);assert.ok(status.audioCache.allocated_bytes>=status.audioCache.file_bytes);
  assert.equal(status.audioCache.file_bytes,status.audioCache.logical_bytes+status.audioCache.unknown_bytes);
  await writeFile(join(fixture.destination,'results.json'),JSON.stringify({seed,after,status},null,2));
  console.log(JSON.stringify({expired:after.expired,unowned:after.unowned,status}));await fixture.stop(true);
} finally { await fixture.finish(); }
