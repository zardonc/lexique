/* global process, window, performance, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath,binaryPath,destinationPath,countText] = process.argv.slice(2);
const count = Number(countText); assert.ok([100,1000,10000].includes(count));
const fixture = await native24Fixture(packagePath,binaryPath,destinationPath);
try {
  await fixture.start(); await fixture.stop(true);
  const database=join(fixture.destination,'Lexique-data','data','lexique.sqlite3');
  const seed=spawnSync('python',['-X','utf8','-c',`import json,sqlite3,sys,time
from pathlib import Path
db=Path(sys.argv[1]);count=int(sys.argv[2]);c=sqlite3.connect(db);cache=db.parent.parent/'temp'/'speech';cache.mkdir(parents=True,exist_ok=True)
snapshot=json.dumps({'reference':{'dictionary':'synthetic','lookup_key':'synthetic','version':'1','attribution':'synthetic','license':'MIT','source_url':'https://example.org','license_url':'https://example.org','sha256':'a'*64},'digest':[]})
for i in range(1,count+1):
 c.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?,?,?,'',1000,1000)",(i,f'mot{i}',snapshot))
 for card,kind in ((i*2-1,'recognition'),(i*2,'listening')):c.execute('INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?,?,?)',(card,i,kind))
 (cache/f'{i}.ogg').write_bytes(b'OggS'+b'0'*8188)
c.commit();print(json.dumps({'words':count,'fixture_bytes':count*8192,'cache':'Ogg header fixture; not decodable playback evidence'}))`,database,String(count)],{windowsHide:true,encoding:'utf8'});
  assert.equal(seed.status,0,seed.stderr);
  const page=await fixture.start();
  const passes=count===10000?1:4;
  const samples=await page.evaluate(async (passes) => {
    const invoke=window.__TAURI_INTERNALS__.invoke;const rows=[];
    for (let i=0;i<passes;i++) { const start=performance.now();const result=await invoke('review_queue',{timezone:'UTC',skipped:[]});rows.push({state:'due',iteration:i,ms:performance.now()-start,cards:result.queue.cards.length}); }
    return rows;
  },passes);
  await writeFile(join(fixture.destination,'checkpoint-due.json'),JSON.stringify(samples,null,2));
  await fixture.stop(true);
  const future=spawnSync('python',['-X','utf8','-c',`import sqlite3,json,sys,time
c=sqlite3.connect(sys.argv[1]);now=int(time.time());state=json.dumps({'phase':'review','memory':{'stability':10.0,'difficulty':5.0},'last_review_at':now-86400,'due_at':now+86400})
c.execute('INSERT INTO review_progress SELECT id,1,? FROM review_cards',(state,));c.commit()`,database],{windowsHide:true,encoding:'utf8'});
  assert.equal(future.status,0,future.stderr);
  const resumed=await fixture.start();
  samples.push(...await resumed.evaluate(async (passes) => {const rows=[];for(let i=0;i<passes;i++){const start=performance.now();const result=await window.__TAURI_INTERNALS__.invoke('review_queue',{timezone:'UTC',skipped:[]});rows.push({state:'future',iteration:i,ms:performance.now()-start,cards:result.queue.cards.length,available:result.queue.available_at});}return rows;},passes));
  assert.ok(samples.filter(row=>row.state==='future').every(row=>row.cards===0&&row.available));
  const report={count,seed:JSON.parse(seed.stdout),samples,physicalDiskCold:false,sourceBoundary:'real native IPC and SQLite; synthetic Ogg header files',workers:fixture.workerPids().length};
  assert.equal(report.workers,0);await writeFile(join(fixture.destination,'results.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
