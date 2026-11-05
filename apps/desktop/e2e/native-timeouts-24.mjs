/* global process, window, console */
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { native24Fixture } from './native-24-harness.mjs';

const [packagePath, binaryPath, destinationPath, mode] = process.argv.slice(2);
assert.ok(['startup', 'runtime'].includes(mode));
const fixture = await native24Fixture(packagePath, binaryPath, destinationPath);
const database = join(fixture.destination, 'Lexique-data/data/lexique.sqlite3');
let helper;
function sql(source) {
  const result = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,json,sys,time\nc=sqlite3.connect(sys.argv[1])\n${source}`, database], { windowsHide: true, encoding: 'utf8', timeout: 5000 });
  assert.equal(result.status, 0, result.stderr); return JSON.parse(result.stdout);
}
async function suspender(pid) {
  const expected = join(fixture.destination, 'speech/runtime/python.exe').replaceAll("'", "''");
  const marker = join(fixture.destination, 'suspender-ready').replaceAll("'", "''");
  const script = `Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class OwnedWorker { [DllImport("ntdll.dll")] public static extern int NtSuspendProcess(IntPtr handle); }'
[IO.File]::WriteAllText('${marker}','ready')
$deadline=[DateTime]::UtcNow.AddSeconds(8)
do {
  $owned=Get-Process -Name python -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${expected}' ${pid ? `-and $_.Id -eq ${pid}` : ''} } | Select-Object -First 1
  if ($owned) { $result=[OwnedWorker]::NtSuspendProcess($owned.Handle); if ($result -ne 0) { throw "Suspend failed: $result" }; $owned.Id; exit 0 }
  Start-Sleep -Milliseconds 10
} while ([DateTime]::UtcNow -lt $deadline)
throw 'Owned worker unavailable'`;
  helper = spawn('pwsh', ['-NoProfile', '-NonInteractive', '-Command', script], { windowsHide: true });
  let output = '', errors = '';
  helper.stdout.on('data', bytes => { output += bytes; }); helper.stderr.on('data', bytes => { errors += bytes; });
  const done = new Promise((resolve, reject) => {
    helper.on('error', reject);
    helper.on('exit', code => code === 0 ? resolve(Number(output.trim())) : reject(new Error(errors)));
  });
  for (let attempt = 0; attempt < 100; attempt++) {
    if (await readFile(join(fixture.destination, 'suspender-ready'), 'utf8').catch(() => '') === 'ready') return { done };
    await delay(20);
  }
  throw new Error('Owned suspender did not initialize');
}
try {
  let page = await fixture.start();
  const invoke = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
  let suspended, expected;
  if (mode === 'startup') {
    // Arm the monitor before native creation, so startup cannot finish first.
    const monitor = await suspender();
    const result = invoke('speech_preview', { text: 'école' }).then(() => ({ success: true }), error => ({ error: String(error) }));
    suspended = await monitor.done;
    const failed = await result;
    assert.match(failed.error, /Offline model startup timed out/);
  } else {
    await invoke('speech_preview', { text: 'école' });
    const pids = fixture.workerPids(); assert.equal(pids.length, 1);
    suspended = await (await suspender(pids[0])).done;
    expected = sql(`now=int(time.time());deadline=now+4
c.execute("UPDATE speech_tasks SET state='retry_wait',attempts=3,wake_at=?,deadline_at=?,refresh_requested=1 WHERE id=(SELECT MIN(id) FROM speech_tasks)",(now,deadline));c.commit()
print(json.dumps(dict(deadline=deadline,id=c.execute('SELECT MIN(id) FROM speech_tasks').fetchone()[0])))`);
    const recovered = await invoke('speech_preview', { text: 'bonjour' });
    assert.ok(recovered.length > 4, 'A new request must recover after the timed-out worker exits');
    const row = sql(`print(json.dumps(c.execute('SELECT state,attempts,deadline_at,error FROM speech_tasks WHERE id=${expected.id}').fetchone()))`);
    assert.equal(row[0], 'failed'); assert.equal(row[1], 4); assert.equal(row[2], expected.deadline);
    assert.match(row[3], /Offline synthesis execution timed out/);
    expected.row = row;
  }
  const workers = fixture.workerPids(); assert.ok(!workers.includes(suspended), 'The native supervisor must stop the suspended owned process');
  if (mode === 'startup') assert.deepEqual(workers, []);
  await fixture.stop(true); assert.deepEqual(fixture.workerPids(), []);
  page = await fixture.start();
  const tasks = sql("print(json.dumps(c.execute('SELECT id,state,attempts,deadline_at,error FROM speech_tasks ORDER BY id').fetchall()))");
  if (expected) assert.deepEqual(tasks.find(row => row[0] === expected.id).slice(1), expected.row);
  else assert.ok(tasks.every(row => row[1] === 'failed' && /startup timed out/.test(row[4])));
  assert.deepEqual(fixture.workerPids(), [], 'Restart must not silently reset a terminal task budget');
  await fixture.stop(true);
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify({ status: 'passed', mode, suspended, expected, tasks, owned_workers_after_exit: fixture.workerPids() }, null, 2));
  console.log(JSON.stringify({ status: 'passed', mode, tasks }));
} finally {
  if (helper?.exitCode === null) helper.kill();
  await fixture.finish();
}
