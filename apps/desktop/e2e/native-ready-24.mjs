/* global process, window, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';
const fixture=await native24Fixture(...process.argv.slice(2));
try {
  let page=await fixture.start();
  const id=await page.evaluate(async () => {const invoke=window.__TAURI_INTERNALS__.invoke;await invoke('speech_preview',{text:'école'});const saved=await invoke('vocabulary_save',{request:{operation:'ready-save',headword:'école',notes:'keep',example_fr:'',example_zh:'',example_sense:null,more_examples:[],id:null,revision:null}});await invoke('review_add_batch',{request:{operation:'ready-one',vocabulary_ids:[saved.record.id]}});return saved.record.id;});
  await fixture.stop(true);
  const removed=spawnSync('python',['-X','utf8','-c',`from pathlib import Path
import sqlite3,sys
db=Path(sys.argv[1]);c=sqlite3.connect(db);physical,content=c.execute('SELECT v.physical_locator,v.content_hash FROM speech_assets a JOIN speech_versions v ON v.locator=a.current_version').fetchone();(db.parent.parent/'temp'/'speech'/f's-{physical}-{content}.ogg').unlink()`,join(fixture.destination,'Lexique-data','data','lexique.sqlite3')],{windowsHide:true,encoding:'utf8'});assert.equal(removed.status,0,removed.stderr);
  page=await fixture.start();
  const batch=await page.evaluate((id)=>window.__TAURI_INTERNALS__.invoke('review_add_batch',{request:{operation:'ready-two',vocabulary_ids:[id]}}),id);
  assert.equal(batch.items[0].audio_state,'queued','Missing formerly ready audio must be queued for verification/generation');assert.equal(batch.items[0].created_cards,0);
  await writeFile(join(fixture.destination,'results.json'),JSON.stringify({batch},null,2));console.log(JSON.stringify({batch}));
} finally {await fixture.finish();}
