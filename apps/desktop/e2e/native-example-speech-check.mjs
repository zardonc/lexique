/* global process, window, HTMLMediaElement, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 5, 'Expected package, binary and new destination');
const [resources, binary, destination] = process.argv.slice(2).map((value) => resolve(value));
const fixture = await native24Fixture(resources, binary, destination);
await cp(join(dirname(binary), 'dictionary'), join(destination, 'dictionary'), { recursive: true });
const model = join(destination, 'speech/assets/text_encoder.int8.onnx');
let hidden = false;
function snapshot() {
  const result = spawnSync(join(destination, 'speech/runtime/python.exe'), ['-B', '-X', 'utf8', '-c',
    "import sqlite3,json,sys\nc=sqlite3.connect(sys.argv[1]);c.row_factory=sqlite3.Row\nprint(json.dumps({t:[dict(r) for r in c.execute('SELECT * FROM '+t+' ORDER BY 1')] for t in ['vocabulary','review_cards','review_progress','review_ratings','word_audio_selections']}))",
    join(destination, 'Lexique-data/data/lexique.sqlite3')], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
async function open(page) {
  await page.context().setOffline(true);
  await page.evaluate(() => {
    Reflect.set(window, 'examplePlayers', []);
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () { Reflect.get(window, 'examplePlayers').push(this); return play.call(this); };
  });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
}
async function play(page) {
  await page.getByRole('button', { name: /播放例句: Je vais à l’école/ }).click();
  await expect.poll(() => page.evaluate(() => Reflect.get(window, 'examplePlayers').at(-1)?.currentTime ?? 0)).toBeGreaterThan(0);
}
try {
  let page = await fixture.start();
  await page.context().setOffline(true);
  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const saved = await invoke('vocabulary_save', { request: { operation: 'native-example-speech', headword: 'école', notes: '',
      example_fr: 'Je vais à l’école.', example_zh: '我去学校。', example_sense: 0, more_examples: [], id: null, revision: null } });
    await invoke('review_add', { vocabularyId: saved.record.id });
    await invoke('speech_generate', { vocabularyId: saved.record.id });
  });
  const before = snapshot();
  await open(page);
  await play(page);
  assert.deepEqual(snapshot(), before);
  await fixture.stop(true);
  await rename(model, model + '.unavailable'); hidden = true;
  page = await fixture.start();
  await open(page);
  await play(page);
  assert.deepEqual(snapshot(), before);
  await page.locator('#example-fr').fill('Une nouvelle phrase sans cache.');
  await page.getByRole('button', { name: /播放例句: Une nouvelle phrase/ }).click();
  await expect(page.getByRole('alert').filter({ hasText: /Speech|speech|model|asset|onnx/i })).toBeVisible();
  assert.deepEqual(snapshot(), before);
  await writeFile(join(destination, 'example-speech-results.json'), JSON.stringify({ status: 'passed',
    network: 'WebView offline; native adapter uses local resources', missingCacheSynthesisAndPlayback: 'passed',
    cachedPlaybackWithoutModel: 'passed', missingModelError: 'visible', learningAndSelectedAudio: 'preserved' }, null, 2));
  console.log('Native example synthesis, cached playback, failure and learning preservation passed');
} finally {
  await fixture.finish();
  if (hidden) await rename(model + '.unavailable', model);
}
