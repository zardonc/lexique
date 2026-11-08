/* global process, window, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function sql(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', source, database], { windowsHide: true, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout);
}
try {
  let page = await fixture.start();
  const before = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const words = [];
    for (const [index, headword] of ['école', 'bonjour'].entries()) {
      const { bytes } = await invoke('speech_preview', { text: headword });
      const saved = await invoke('vocabulary_save', { request: { operation: `recover-version-${index}`, headword, notes: 'keep', example_fr: 'Bonjour.', example_zh: '你好。', example_sense: 0, more_examples: [], id: null, revision: null } });
      await invoke('review_add_batch', { request: { operation: `recover-cards-${index}`, vocabulary_ids: [saved.record.id] } });
      words.push({ id: saved.record.id, bytes, cards: await invoke('review_cards', { vocabularyId: saved.record.id }) });
    }
    return words;
  });
  await fixture.stop(true);
  const prepared = sql(`import json,sqlite3,sys,shutil
from pathlib import Path
db=Path(sys.argv[1]);c=sqlite3.connect(db);cache=db.parent.parent/'temp'/'speech'
row=c.execute('SELECT v.locator,v.semantic_key,v.content_hash,v.byte_length,v.generated_at FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id=2').fetchone();assert row
old,key,content,size,generated=row;locator='f'*64
shutil.copyfile(cache/f's-{old}-{content}.ogg',cache/f's-{locator}-{content}.ogg')
c.execute('UPDATE word_audio_selections SET current_version=NULL,revision=0 WHERE vocabulary_id=2')
for i in range(205):
 missing=f'{i:064x}'
 c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,created_at,vocabulary_id,physical_locator,context,last_used) VALUES(?,?,4,1,1,?,'word',1)",(missing,'a'*64,missing))
 c.execute('INSERT INTO word_audio_publications(locator,vocabulary_id,expected_revision,generation) VALUES(?,1,0,0)',(missing,))
c.execute("INSERT INTO speech_versions(locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,physical_locator,context,last_used) VALUES(?,?,?,?,?,?,2,?,'word',?)",(locator,key,content,size,generated,generated,locator,generated))
c.execute('INSERT INTO word_audio_publications(locator,vocabulary_id,expected_revision,generation) VALUES(?,2,0,0)',(locator,))
source=c.execute('SELECT v.semantic_key FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id=1').fetchone()[0]
c.execute('UPDATE speech_assets SET gc_pending=1 WHERE semantic_key=?',(source,))
c.commit()
print(json.dumps(dict(locator=locator,hash=content,missingIntents=205,ownedGcKey=source)))`);
  for (let restart = 0; restart < 2; restart++) {
    page = await fixture.start();
    for (const word of before) {
      const restored = await page.evaluate(async (id) => {
        const invoke = window.__TAURI_INTERNALS__.invoke;
        return { bytes: (await invoke('speech_cached', { vocabularyId: id })).bytes, cards: await invoke('review_cards', { vocabularyId: id }) };
      }, word.id);
      assert.deepEqual(restored.bytes, word.bytes); assert.deepEqual(restored.cards, word.cards);
    }
    assert.equal(fixture.workerPids().length, 0);
    await fixture.stop(true);
    const state = sql(`import json,sqlite3,sys
c=sqlite3.connect(sys.argv[1]);row=c.execute('SELECT current_version,previous_version,previous_until,revision FROM word_audio_selections WHERE vocabulary_id=2').fetchone()
assert row==('f'*64,None,None,1),row
assert c.execute("SELECT COUNT(*) FROM speech_versions WHERE sealed=0").fetchone()[0]==205
assert c.execute('SELECT COUNT(*) FROM word_audio_publications').fetchone()[0]==0
assert c.execute('SELECT COUNT(*) FROM speech_assets WHERE gc_pending=1').fetchone()[0]==0
assert c.execute("SELECT COUNT(*) FROM vocabulary WHERE notes='keep' AND example_fr='Bonjour.' AND example_zh='你好。'").fetchone()[0]==2
print(json.dumps(dict(revision=row[3],pendingMissing=205,notesAndExamplesPreserved=True,ownedGcRecovered=True)))`);
    assert.equal(state.revision, 1);
  }
  const report = { prepared, repeatedRecoveryPreservedSelection: true, normalShutdowns: 3, ownedWorkersAfterExit: 0, hardTimeoutSeconds: 60 };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2)); console.log(JSON.stringify(report));
} finally { await fixture.finish(); }
