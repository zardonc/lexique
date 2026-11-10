import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';

export function learningSnapshot(python, database) {
  const result = spawnSync(python, ['-B', '-X', 'utf8', '-c', `import sqlite3,json,sys
c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True)
c.row_factory=sqlite3.Row
tables=['vocabulary','review_cards','review_progress','review_ratings','word_audio_selections']
r={name:[dict(row) for row in c.execute('SELECT * FROM '+name+' ORDER BY 1')] for name in tables}
r['protected']=[dict(row) for row in c.execute('SELECT locator,owned,expires_at,gc_state FROM audio_version_protection WHERE owned=1 ORDER BY locator')]
r['schema']=c.execute('PRAGMA user_version').fetchone()[0]
print(json.dumps(r))`, database], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  const snapshot = JSON.parse(result.stdout);
  for (const table of ['vocabulary', 'review_cards', 'review_progress', 'review_ratings', 'word_audio_selections', 'protected'])
    assert.ok(snapshot[table].length > 0, `Retention fixture must contain ${table}`);
  assert.ok(snapshot.vocabulary.some((record) => record.notes.length > 0), 'Retention fixture must contain saved notes');
  return snapshot;
}
