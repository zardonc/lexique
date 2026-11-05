/* global process, window, console */
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function inject(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', source, database], { windowsHide: true, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout);
}
try {
  let page = await fixture.start();
  const before = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const bytes = await invoke('speech_preview', { text: 'école' });
    const saved = await invoke('vocabulary_save', { request: { operation: 'recovery-save', headword: 'école', notes: 'keep', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } });
    await invoke('review_add_batch', { request: { operation: 'recovery-batch', vocabulary_ids: [saved.record.id] } });
    await invoke('review_pause', { vocabularyId: saved.record.id, paused: true });
    return { bytes, id: saved.record.id, cards: await invoke('review_cards', { vocabularyId: saved.record.id }) };
  });
  await fixture.stop(true);
  const legacy = inject(`import json,sqlite3,sys,shutil,hashlib
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db);cache=db.parent.parent/'temp'/'speech'
row=c.execute('SELECT v.physical_locator,a.spec,a.context,v.content_hash,v.generated_at FROM speech_assets a JOIN speech_versions v ON v.locator=a.current_version').fetchone();assert row
key,spec,context,content,generated=row;src=cache/f's-{key}-{content}.ogg';assert hashlib.sha256(src.read_bytes()).hexdigest()==content
shutil.copy2(src,cache/'1.ogg')
for (name,) in c.execute("SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'queue_revision_%'").fetchall():c.execute(f'DROP TRIGGER {name}')
c.executescript('DROP VIEW audio_version_protection; DROP TABLE review_queue_changes; DROP TABLE word_audio_repairs; DROP TABLE word_audio_operations; DROP TABLE word_audio_publications; DROP TABLE word_audio_imports; DROP TABLE word_audio_selections; DROP TABLE speech_versions; DROP TABLE review_batch_items; DROP TABLE review_batches; DROP TABLE speech_tasks; DROP TABLE speech_owners; DROP TABLE legacy_speech; DROP TABLE speech_assets; PRAGMA user_version=10;');c.commit()
print(json.dumps(dict(semantic_key=key,spec=spec,context=context,content_hash=content,generated_at=generated)))`);
  page = await fixture.start();
  const restored = await page.evaluate(async (id) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    return { bytes: await invoke('speech_cached', { vocabularyId: id }), cards: await invoke('review_cards', { vocabularyId: id }), paused: await invoke('review_pause', { vocabularyId: id, paused: null }), schema: (await invoke('runtime_status')).schemaVersion };
  }, before.id);
  assert.equal(restored.schema, 17); assert.deepEqual(restored.bytes, before.bytes); assert.deepEqual(restored.cards, before.cards); assert.equal(restored.paused, true);
  assert.equal(fixture.workerPids().length, 0);
  await fixture.stop(true);
  const state = inject(`import json,sqlite3,sys
c=sqlite3.connect(sys.argv[1]);row=c.execute('SELECT vocabulary_id,content_hash FROM speech_versions WHERE semantic_key IS NULL AND sealed=1').fetchone();assert row
assert c.execute('SELECT COUNT(*) FROM speech_assets').fetchone()[0]==0
asset=json.loads(${JSON.stringify(JSON.stringify(legacy))})
c.execute('INSERT INTO speech_assets(semantic_key,spec,context,content_hash,generated_at,expires_at,gc_pending) VALUES(?,?,?,?,1000,605800,1)',(asset['semantic_key'],asset['spec'],asset['context'],asset['content_hash']))
c.execute("INSERT INTO speech_versions(locator,semantic_key,content_hash,generated_at,created_at,physical_locator,context,gc_state,gc_generation,expires_at,last_used) VALUES(?,?,?,1000,1000,?,'word','marked',1,605800,1000)",('e'*64,asset['semantic_key'],asset['content_hash'],asset['semantic_key']))
c.execute('UPDATE speech_assets SET current_version=?',('e'*64,));c.commit()
print(json.dumps({'legacy_registered':True,'gc_gap_prepared':True}))`);
  page = await fixture.start();
  assert.deepEqual(await page.evaluate((id) => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: id }), before.id), before.bytes);
  await fixture.stop(true);
  const gc = inject(`import json,sqlite3,sys
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db);cache=db.parent.parent/'temp'/'speech'
assert c.execute('SELECT COUNT(*) FROM speech_assets WHERE gc_pending=1 OR content_hash IS NOT NULL').fetchone()[0]==0
assert (cache/'1.ogg').exists()
selected=c.execute('SELECT v.locator,v.content_hash FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id=1').fetchone();assert selected
assert (cache/f's-{selected[0]}-{selected[1]}.ogg').exists()
assert not (cache/f's-{${JSON.stringify(legacy.semantic_key)}}-{${JSON.stringify(legacy.content_hash)}}.ogg').exists()
assert c.execute('SELECT notes FROM vocabulary WHERE id=1').fetchone()[0]=='keep'
print(json.dumps({'gc_recovered':True,'legacy_preserved':True,'notes_preserved':True}))`);
  assert.equal(fixture.workerPids().length, 0);
  const report = { legacy, state, gc, preservedCards: restored.cards, schema: restored.schema, ownedWorkersAfterExit: 0 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
