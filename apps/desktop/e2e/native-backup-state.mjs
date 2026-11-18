import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { resolve, join, relative, isAbsolute } from 'node:path';

function ownedDestination(destination) {
  const root = resolve(destination);
  assert.ok(['issue-75', 'issue-79', 'issue-80', 'issue-81', 'issue-82', 'issue-83'].some(issue => {
    const owned = relative(resolve('../../target', issue), root);
    return owned && !owned.startsWith('..') && !isAbsolute(owned);
  }));
  return root;
}

function packagePython(destination, source, code, extra = []) {
  const root = ownedDestination(destination);
  const input = relative(root, resolve(source));
  assert.ok(input && !input.startsWith('..') && !isAbsolute(input));
  const result = spawnSync('python', ['-B', '-X', 'utf8', '-c', code, source, ...extra], { encoding: 'utf8', timeout: 5000, maxBuffer: 4 * 1024 * 1024 });
  assert.equal(result.status, 0, result.stderr || String(result.error));
  return JSON.parse(result.stdout);
}

export function inspectPackage(destination, source) {
  return packagePython(destination, source, `import zipfile,json,sys
with zipfile.ZipFile(sys.argv[1]) as z:
 print(json.dumps({'manifest':json.loads(z.read('manifest.json')),'snapshot':json.loads(z.read('snapshot.json')),'entries':z.namelist()},ensure_ascii=False))`);
}

export function corruptPackage(destination, source, variant) {
  const variants = ['unsupported-format', 'unsupported-snapshot', 'count', 'unknown-field', 'invalid-number', 'invalid-relation', 'duplicate-json', 'duplicate-entry', 'path-entry', 'truncated', 'missing-audio', 'crc', 'hash', 'compression'];
  assert.ok(variants.includes(variant));
  const target = join(destination, `invalid-${variant}.lexique-backup`);
  const result = packagePython(destination, source, `import zipfile,json,sys,pathlib,struct,warnings
source,target,variant=sys.argv[1:]
raw=pathlib.Path(source).read_bytes()
with zipfile.ZipFile(source) as z:
 entries=[(i,z.read(i)) for i in z.infolist()]
manifest=json.loads(dict((i.filename,b) for i,b in entries)['manifest.json'])
snapshot=json.loads(dict((i.filename,b) for i,b in entries)['snapshot.json'])
audio=next(i for i,b in entries if i.filename.startswith('audio/'))
if variant=='truncated': output=raw[:-20]
elif variant=='crc':
 output=bytearray(raw)
 n,e=struct.unpack_from('<HH',raw,audio.header_offset+26)
 output[audio.header_offset+30+n+e+80]^=1
else:
 if variant=='unsupported-format': manifest['format_version']=2
 elif variant=='unsupported-snapshot': manifest['snapshot_version']=snapshot['snapshot_version']=3
 elif variant=='count': manifest['counts']['cards']+=1
 elif variant=='unknown-field': snapshot['unexpected']=True
 elif variant=='invalid-number': snapshot['records'][0]['id']=0
 elif variant=='invalid-relation': snapshot['cards'][0]['vocabulary_id']=9999999
 import io
 buffer=io.BytesIO()
 with warnings.catch_warnings():
  warnings.simplefilter('ignore',UserWarning)
  with zipfile.ZipFile(buffer,'w',compression=zipfile.ZIP_STORED) as z:
   for info,data in entries:
    if info.filename=='manifest.json': data=json.dumps(manifest,ensure_ascii=False,separators=(',',':')).encode()
    elif info.filename=='snapshot.json':
     data=json.dumps(snapshot,ensure_ascii=False,separators=(',',':')).encode()
     if variant=='duplicate-json': data=b'{"snapshot_version":1,'+data[1:]
    if info.filename==audio.filename:
     if variant=='missing-audio': continue
     if variant=='hash': data=data[:-1]+bytes([data[-1]^1])
    if variant=='compression' and info.filename=='manifest.json': info.compress_type=zipfile.ZIP_DEFLATED
    z.writestr(info,data)
   if variant=='duplicate-entry': z.writestr('manifest.json',json.dumps(manifest).encode())
   if variant=='path-entry': z.writestr('../outside.txt',b'not a backup entry')
 output=buffer.getvalue()
with open(target,'xb') as f: f.write(output)
print(json.dumps({'variant':variant,'byte_length':len(output)}))`, [target, variant]);
  return { ...result, path: target };
}

export function fixtureSql(destination, code, parameters = {}, readOnly = false) {
  ownedDestination(destination);
  const result = spawnSync('python', ['-B', '-X', 'utf8', '-c',
    `import sqlite3,json,sys,pathlib\np=json.loads(sys.argv[2])\ndb=pathlib.Path(sys.argv[1])\nc=sqlite3.connect(db.as_uri()+'?mode=ro' if ${readOnly ? 'True' : 'False'} else db,uri=${readOnly ? 'True' : 'False'})\nc.row_factory=sqlite3.Row\n` + code,
    join(destination, 'Lexique-data/data/lexique.sqlite3'), JSON.stringify(parameters)], { windowsHide: true, encoding: 'utf8', timeout: 10_000 });
  assert.equal(result.status, 0, result.error?.message || result.stderr);
  return result.stdout.trim() ? JSON.parse(result.stdout) : null;
}

export function seedStartupFailure(destination, kind) {
  assert.ok(['corrupt', 'unsupported', 'migration'].includes(kind));
  return fixtureSql(destination, `
import os
if p['kind']=='corrupt':
 c.close()
 db.write_bytes(b'Lexique synthetic unreadable learning database\\n')
 for suffix in ['-wal','-shm','-journal']:
  pathlib.Path(str(db)+suffix).write_bytes(('Original opaque '+suffix+' bytes\\n').encode())
else:
 c.execute('PRAGMA journal_mode=WAL')
 c.execute('PRAGMA wal_autocheckpoint=0')
 c.execute('PRAGMA user_version='+('999' if p['kind']=='unsupported' else '17'))
 c.execute('INSERT OR REPLACE INTO app_metadata(key,value) VALUES(?,?)',('native75-raw-source',json.dumps('Preserve exact raw startup files')))
 c.commit()
files=[{'name':pathlib.Path(str(db)+suffix).name,'bytes':pathlib.Path(str(db)+suffix).stat().st_size} for suffix in ['', '-wal','-shm','-journal'] if pathlib.Path(str(db)+suffix).is_file()]
print(json.dumps({'kind':p['kind'],'files':files}),flush=True)
os._exit(0)`, { kind });
}

export function readState(destination) {
  return fixtureSql(destination, `
queries={
 'records':'SELECT id,headword,snapshot,notes,example_fr,example_zh,example_source,example_sense,more_examples,revision,created_at,updated_at FROM vocabulary ORDER BY id',
 'saves':'SELECT operation,request,outcome,record_id FROM save_operations ORDER BY operation',
 'cards':'SELECT id,vocabulary_id,kind FROM review_cards ORDER BY id',
 'progress':'SELECT card_id,revision,state FROM review_progress ORDER BY card_id',
 'ratings':'SELECT operation,card_id,request,before_state,result,reviewed_at,learning_day,quota,queue_before,undo_operation,undo_queue_rebuilt FROM review_ratings ORDER BY rowid',
 'queue_context':'SELECT learning_day,timezone FROM review_queue_state WHERE id=1',
 'queue':'SELECT card_id,position FROM review_queue_entries ORDER BY position',
 'paused':'SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id',
 'batches':'SELECT operation,request,voice,created_at,cancelled FROM review_batches ORDER BY rowid',
 'batch_items':'SELECT operation,ordinal,vocabulary_id,semantic_key,created_cards,error FROM review_batch_items ORDER BY operation,ordinal',
 'sources':'SELECT semantic_key,spec,context,current_version,pending_version,retain FROM speech_assets ORDER BY semantic_key',
 'owners':'SELECT semantic_key,vocabulary_id FROM speech_owners ORDER BY semantic_key,vocabulary_id',
 'tasks':'SELECT id,semantic_key,priority,state,attempts,wake_at,deadline_at,cancel_requested,refresh_requested,successor_requested,interactive_requested,execution_generation,demand_class,requested_at,error FROM speech_tasks ORDER BY id',
 'versions':'SELECT locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,execution_generation,context,verification,gc_state,expires_at,orphan_until FROM speech_versions ORDER BY locator',
 'selections':'SELECT vocabulary_id,current_version,previous_version,previous_until,revision,desired_generation,desired_key,current_generation FROM word_audio_selections ORDER BY vocabulary_id',
 'imports':'SELECT vocabulary_id,state,error FROM word_audio_imports ORDER BY vocabulary_id',
 'legacy':'SELECT vocabulary_id,content_hash FROM legacy_speech ORDER BY vocabulary_id',
 'audio_operations':'SELECT operation,vocabulary_id,semantic_key,generation,refresh FROM word_audio_operations ORDER BY operation',
 'publications':'SELECT locator,vocabulary_id,expected_revision,generation,execution_generation,error FROM word_audio_publications ORDER BY locator',
 'repairs':'SELECT vocabulary_id,failure_identity,semantic_key,generation,error FROM word_audio_repairs ORDER BY vocabulary_id,failure_identity',
 'protection':'SELECT locator,owned,protected_until,publishing,migrating,expires_at,orphan_until,gc_state FROM audio_version_protection ORDER BY locator',
 'settings':'SELECT retention_percent,new_limit,review_limit FROM review_settings WHERE id=1',
 'preferences':"SELECT key,value FROM app_metadata WHERE key IN ('new-cards-ignore-review-limit','review-auto-enroll','speech-preferences','interface-preferences','conjugation-preferences','conjugation-block-order','vocabulary-sort','dictionary-settings') ORDER BY key",
 'devices':"SELECT key,value FROM app_metadata WHERE key IN ('selection-shortcut','ocr-shortcut','dictionary-files') ORDER BY key",
 'absent_progress':'SELECT c.id FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE p.card_id IS NULL ORDER BY c.id'
}
json_columns={'snapshot','request','state','before_state','result','queue_before','example_source','more_examples','spec','voice','value'}
c.execute('BEGIN')
result={}
for name,query in queries.items():
 result[name]=[]
 for row in c.execute(query):
  item=dict(row)
  for key in json_columns.intersection(item):
   if key=='value' and item.get('key')=='vocabulary-sort': continue
   if item[key] is not None and (key!='state' or name=='progress'): item[key]=json.loads(item[key])
  result[name].append(item)
result['schema']=c.execute('PRAGMA user_version').fetchone()[0]
result['integrity']=[r[0] for r in c.execute('PRAGMA integrity_check')]
result['foreign_keys']=[list(r) for r in c.execute('PRAGMA foreign_key_check')]
result['physical']=[dict(r) for r in c.execute('SELECT locator,physical_locator,content_hash FROM speech_versions ORDER BY locator')]
print(json.dumps(result,ensure_ascii=False))`, {}, true);
}

export function compareLearning(actual, expected) {
  assert.equal(actual.schema, 18); assert.deepEqual(actual.integrity, ['ok']); assert.deepEqual(actual.foreign_keys, []);
  for (const key of ['records', 'saves', 'cards', 'progress', 'ratings', 'queue_context', 'queue', 'paused', 'batches', 'batch_items', 'settings', 'preferences', 'absent_progress']) {
    assert.deepEqual(actual[key], expected[key], `Independent restored ${key}`);
  }
}

export function seedAudio(destination, ids, timeline, mediaRoot) {
  return fixtureSql(destination, `
import hashlib,time
audio=db.parent.parent/'temp'/'speech'; audio.mkdir(parents=True,exist_ok=True)
now=int(time.time()); current=('a' if p['timeline']=='A' else 'c')*64; previous=('b' if p['timeline']=='A' else 'd')*64
files=['female-1.ogg','female-2.ogg'] if p['timeline']=='A' else ['male-1.ogg','male-2.ogg']
media=[]
for locator,name in zip([current,previous],files):
 data=(pathlib.Path(p['media'])/name).read_bytes(); digest=hashlib.sha256(data).hexdigest()
 c.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,physical_locator,context,verification,gc_state,expires_at,orphan_until,last_used) VALUES(?,?,?,NULL,?,?,0,0,1,?,'word','verified','live',?,?,?)",(locator,digest,len(data),now,p['ids'][0],locator,now+604800,now+604800,now))
 (audio/('s-'+locator+'-'+digest+'.ogg')).write_bytes(data)
 media.append({'locator':locator,'hash':digest,'bytes':len(data),'source':name})
for record in p['ids'][:2]:
 c.execute('INSERT INTO word_audio_selections(vocabulary_id,current_version,previous_version,previous_until,revision,desired_generation,current_generation) VALUES(?,?,?,?,1,0,0) ON CONFLICT(vocabulary_id) DO UPDATE SET current_version=excluded.current_version,previous_version=excluded.previous_version,previous_until=excluded.previous_until,revision=word_audio_selections.revision+1,desired_key=NULL,desired_generation=0,current_generation=0',(record,current,previous,now+86400))
c.execute("UPDATE word_audio_imports SET state='finished',error=NULL")
c.commit()
print(json.dumps({'timeline':p['timeline'],'media':media,'previousUntil':now+86400}))`, { ids, timeline, media: mediaRoot });
}

export function seedUnknownDictionary(destination, id) {
  return fixtureSql(destination, `
catalog=json.loads(c.execute("SELECT value FROM app_metadata WHERE key='dictionary-settings'").fetchone()[0])
files=json.loads(c.execute("SELECT value FROM app_metadata WHERE key='dictionary-files'").fetchone()[0])
entry=next(entry for entry in catalog['entries'] if entry['id']==p['id']); entry['content_hash']=None
files['bindings'].pop(p['id'],None)
for key,value in [('dictionary-settings',catalog),('dictionary-files',files)]: c.execute('UPDATE app_metadata SET value=? WHERE key=?',(json.dumps(value),key))
c.commit(); print(json.dumps(entry))`, { id });
}
