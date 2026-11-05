/* global process, window, console, setTimeout */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
function inspect(seed = false) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import hashlib,json,sqlite3,sys,time
c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA foreign_keys=ON');now=int(time.time())
if sys.argv[2]=='true':
 original=json.loads(c.execute('SELECT a.spec FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version JOIN speech_assets a USING(semantic_key) WHERE s.vocabulary_id=1').fetchone()[0])
 changes={'engine':'archived-engine/1','model':'archived-model/1','preprocessing':'archived-preprocessing/1','speed':2,'steps':4,'encoding':'archived-encoder/1','bitrate':24000,'language':'en','context':'sentence','voice':'archived-voice'}
 for i,(field,value) in enumerate([*changes.items(),('text','bonjour')]):
  spec=dict(original);spec[field]=value;serialized=json.dumps(spec,ensure_ascii=False,separators=(',',':'));key=hashlib.sha256(serialized.encode()).hexdigest()
  c.execute('INSERT INTO speech_assets(semantic_key,spec,context) VALUES(?,?,?)',(key,serialized,spec['context']))
  state=('queued','retry_wait','running')[i%3];attempts=1 if state=='running' else 0
  c.execute('INSERT INTO speech_tasks(semantic_key,priority,state,attempts,wake_at,deadline_at,execution_generation,requested_at) VALUES(?,1,?,?,?,?,?,?)',(key,state,attempts,now,now+120,attempts,now))
  if field=='model':
   c.execute('INSERT INTO speech_owners VALUES(?,1)',(key,));c.execute('UPDATE word_audio_selections SET desired_key=?,desired_generation=1 WHERE vocabulary_id=1',(key,))
 c.commit()
tasks=[]
for state,attempts,error,spec,content in c.execute('SELECT t.state,t.attempts,t.error,a.spec,a.content_hash FROM speech_tasks t JOIN speech_assets a USING(semantic_key) ORDER BY t.id'):
 tasks.append(dict(state=state,attempts=attempts,error=error,spec=json.loads(spec),content_hash=content))
selected=c.execute('SELECT current_version,revision FROM word_audio_selections WHERE vocabulary_id=1').fetchone()
print(json.dumps(dict(tasks=tasks,selected=selected,notes=c.execute('SELECT notes FROM vocabulary WHERE id=1').fetchone()[0])))`, database, String(seed)], { windowsHide: true, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
try {
  let page = await fixture.start();
  const original = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const bytes = await invoke('speech_preview', { text: 'école' });
    await invoke('vocabulary_save', { request: { operation: 'saved-spec-word', headword: 'école', notes: 'keep', example_fr: '', example_zh: '', example_sense: null, more_examples: [], id: null, revision: null } });
    await invoke('review_add_batch', { request: { operation: 'saved-spec-cards', vocabulary_ids: [1] } });
    await invoke('speech_cached', { vocabularyId: 1 });
    return bytes;
  });
  await fixture.stop(true);
  const before = inspect(true);
  page = await fixture.start();
  let observed;
  for (let attempt = 0; attempt < 100; attempt++) {
    observed = inspect();
    if (observed.tasks.every((task) => ['ready', 'failed', 'cancelled'].includes(task.state))) break;
    await new Promise((done) => setTimeout(done, 100));
  }
  await writeFile(join(fixture.destination, 'observed.json'), JSON.stringify(observed, null, 2));
  const unsupported = observed.tasks.filter((task) => task.spec.text === 'école').slice(1);
  assert.equal(unsupported.length, 10);
  for (const task of unsupported) {
    assert.equal(task.state, 'failed', `Unsupported saved specification was executed: ${JSON.stringify(task.spec)}`);
    assert.match(task.error, /specification is unsupported/);
    assert.equal(task.content_hash, null, 'Unsupported execution must not publish content');
  }
  assert.equal(observed.tasks.find((task) => task.spec.text === 'bonjour').state, 'ready');
  assert.deepEqual(observed.selected, before.selected, 'Failed replacement keeps the selected content and revision');
  assert.equal(observed.notes, 'keep');
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 })), original);
  await fixture.stop(true);
  assert.equal(fixture.workerPids().length, 0);
  page = await fixture.start();
  assert.deepEqual(inspect(), observed, 'Restart must retain terminal failures and attempt budgets');
  assert.deepEqual(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('speech_cached', { vocabularyId: 1 })), original);
  assert.equal(fixture.workerPids().length, 0, 'Cache-only recovery must not restart unsupported work');
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ unsupportedSpecifications: unsupported.length, supportedExecution: 'ready', selectedPreserved: true, restartPreserved: true, ownedWorkersAfterExit: 0 }, null, 2));
  console.log('Persisted specification validation and recovery passed');
} finally { await fixture.finish(); }
