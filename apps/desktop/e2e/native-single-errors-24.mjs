/* global process, window, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';

const fixture=await native24Fixture(...process.argv.slice(2));
try {
  let page=await fixture.start();
  const unknown=await page.evaluate(async () => {try {return {success:await window.__TAURI_INTERNALS__.invoke('review_add',{vocabularyId:999})};} catch(error){return {error:String(error)};} });
  assert.ok(unknown.error,'Unknown single enrollment must reject, never return empty cards as success');
  const saved=await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('vocabulary_save',{request:{operation:'single-save',headword:'école',notes:'keep',example_fr:'',example_zh:'',example_sense:null,more_examples:[],id:null,revision:null}}));
  await fixture.stop(true);
  const database=join(fixture.destination,'Lexique-data','data','lexique.sqlite3');
  const seed=spawnSync('python',['-X','utf8','-c',`import sqlite3,sys,time
c=sqlite3.connect(sys.argv[1]);wake=int(time.time())+86400
for i in range(10000):
 key=f'{i:064x}';c.execute("INSERT INTO speech_assets(semantic_key,spec,context) VALUES(?,'{}','word')",(key,));c.execute("INSERT INTO speech_tasks(semantic_key,priority,state,wake_at) VALUES(?,1,'queued',?)",(key,wake))
c.commit()`,database],{windowsHide:true,encoding:'utf8'});assert.equal(seed.status,0,seed.stderr);
  page=await fixture.start();
  const full=await page.evaluate(async (id) => {const invoke=window.__TAURI_INTERNALS__.invoke;const added=await invoke('review_add',{vocabularyId:id});return {added,cards:await invoke('review_cards',{vocabularyId:id}),batch:await invoke('review_batch_status',{operation:null})};},saved.record.id);
  assert.equal(full.cards.length,2);assert.equal(full.added.length,2);
  assert.equal(full.batch.items[0].error,null);assert.equal(full.batch.items[0].audio_state,'failed');assert.match(full.batch.items[0].audio_error,/Speech backlog is full/);assert.equal(fixture.workerPids().length,0);
  const report={unknown,full,workers:0};await writeFile(join(fixture.destination,'results.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
} finally {await fixture.finish();}
