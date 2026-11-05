/* global process, window, document, performance, MutationObserver, setTimeout, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { native24Fixture } from './native-24-harness.mjs';

const fixture = await native24Fixture(...process.argv.slice(2));
let page;
try {
  await fixture.start(); await fixture.stop(true);
  const database = join(fixture.destination, 'Lexique-data', 'data', 'lexique.sqlite3');
  const seeded = spawnSync('python', ['-X', 'utf8', '-c', `import sqlite3,sys,time,json
sys.path.insert(0,'../../scripts')
from s0_baseline import seed_rows
c=sqlite3.connect(sys.argv[1]);seed_rows(c,104,'new',int(time.time()))
for identity,snapshot in c.execute('SELECT id,snapshot FROM vocabulary').fetchall():
 value=json.loads(snapshot);value['digest']=[{'pronunciations':[],'grammar':[],'translation':'fixture'}];c.execute('UPDATE vocabulary SET snapshot=? WHERE id=?',(json.dumps(value),identity))
c.execute("DELETE FROM review_cards WHERE kind='listening'")
c.execute('UPDATE review_settings SET new_limit=1000,review_limit=1000');c.commit()`, database], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(seeded.status, 0, seeded.stderr);
  page = await fixture.start();
  await page.getByRole('button', { name: '闪卡复习', exact: true }).click();
  const samples = [];
  for (let index = 0; index < 100; index++) {
    await page.getByRole('button', { name: '显示答案', exact: true }).click();
    await page.waitForFunction(() => [...document.querySelectorAll('.queue-review-card button')].some(button => button.textContent === '记得' && !button.disabled), undefined, { timeout: 3000 });
    await page.evaluate(() => {
      const heading = document.querySelector('.queue-review-card h3');
      if (!heading) throw new Error('Review heading unavailable');
      const before = heading.textContent;
      const started = performance.now();
      window.nextCardTiming = new Promise((done) => {
        const observer = new MutationObserver(() => {
          const next = document.querySelector('.queue-review-card h3');
          if (next && next.textContent !== before) {
            observer.disconnect(); done(performance.now() - started);
          }
        });
        observer.observe(document.body, { childList: true, subtree: true, characterData: true });
      });
      const button = [...document.querySelectorAll('.queue-review-card button')].find(button => button.textContent === '记得');
      if (!button || button.disabled) throw new Error('Rating unavailable');
      button.click();
    });
    samples.push(await page.evaluate(() => Promise.race([window.nextCardTiming, new Promise((_, reject) => setTimeout(() => reject(new Error('Next card timing did not settle')), 3000))])));
  }
  const sorted = [...samples].sort((a, b) => a - b);
  const report = { attempts: 100, failures: 0, method: 'nearest rank', boundary: 'rating DOM click through native commit to next heading DOM commit; recognition cards; includes IPC, not isolated post-return UI latency', p50: sorted[49], p95: sorted[94], max: sorted.at(-1), samples };
  await writeFile(join(fixture.destination, 'results.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify({ ...report, samples: undefined }));
  await fixture.stop(true);
} catch (error) {
  await writeFile(join(fixture.destination, 'failure.json'), JSON.stringify({ error: String(error), text: await page?.locator('body').innerText().catch(() => null) }, null, 2));
  throw error;
} finally { await fixture.finish(); }
