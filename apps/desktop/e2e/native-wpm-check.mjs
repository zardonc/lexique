/* global process, window, HTMLMediaElement, performance, console */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cp, rename, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { expect } from '@playwright/test';
import { native24Fixture } from './native-24-harness.mjs';

assert.equal(process.argv.length, 6, 'Expected package, binary, conjugation resource and new destination');
const [resources, binary, conjugation, destination] = process.argv.slice(2).map((path) => resolve(path));
const fixture = await native24Fixture(resources, binary, destination);
await cp(conjugation, join(destination, 'conjugation'), { recursive: true });
const database = join(destination, 'Lexique-data/data/lexique.sqlite3');
const python = join(destination, 'speech/runtime/python.exe');
function sql(code, ...args) {
  const result = spawnSync(python, ['-B', '-X', 'utf8', '-c',
    'import sqlite3,json,sys\nc=sqlite3.connect(sys.argv[1])\nc.row_factory=sqlite3.Row\n' + code,
    database, ...args], { encoding: 'utf8', windowsHide: true, timeout: 10_000 });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
function rawPreference() {
  return JSON.parse(sql("row=c.execute(\"SELECT value FROM app_metadata WHERE key='speech-preferences'\").fetchone()\nprint(json.dumps(row[0] if row else None))"));
}
function learningSnapshot() {
  return JSON.parse(sql(`tables=['vocabulary','review_cards','review_progress','review_ratings','word_audio_selections']
result={name:[dict(row) for row in c.execute('SELECT * FROM '+name+' ORDER BY 1')] for name in tables}
result['protected']=[dict(row) for row in c.execute('SELECT locator,owned,expires_at,gc_state FROM audio_version_protection WHERE owned=1 ORDER BY locator')]
result['schema']=c.execute('PRAGMA user_version').fetchone()[0]
print(json.dumps(result))`));
}
async function preferences(page, value) {
  return page.evaluate((preferences) => window.__TAURI_INTERNALS__.invoke('speech_preferences', { preferences }), value ?? null);
}
async function observePlayback(page) {
  await page.evaluate(() => {
    const players = [];
    Reflect.set(window, 'wpmPlayers', players);
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () { this.loop = true; players.push(this); return play.call(this); };
  });
}
async function play(page, button, expectedRate) {
  const index = await page.evaluate(() => Reflect.get(window, 'wpmPlayers').length);
  await button.click();
  await page.waitForFunction((index) => Reflect.get(window, 'wpmPlayers')[index]?.currentTime > 0.01, index);
  const before = await page.evaluate((index) => {
    const audio = Reflect.get(window, 'wpmPlayers')[index];
    return { currentTime: audio.currentTime, time: performance.now(), rate: audio.playbackRate, pitch: audio.preservesPitch };
  }, index);
  assert.ok(Math.abs(before.rate - expectedRate) < 0.000001);
  assert.equal(before.pitch, true);
  await page.waitForTimeout(350);
  const after = await page.evaluate((index) => ({ currentTime: Reflect.get(window, 'wpmPlayers')[index].currentTime, time: performance.now() }), index);
  const observedRate = (after.currentTime - before.currentTime) / ((after.time - before.time) / 1000);
  assert.ok(Math.abs(observedRate / expectedRate - 1) < 0.25, `Observed playback rate ${observedRate}`);
  return { index, rate: before.rate, observedRate, preservesPitch: before.pitch };
}
const results = [];
let blockedAsset;
try {
  let page = await fixture.start();
  assert.equal(await page.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('runtime_status')).schemaVersion), 17);
  assert.deepEqual(await preferences(page), { voice: 'female', wpm: 180 });
  assert.equal(rawPreference(), null);
  const seeded = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const saved = await invoke('vocabulary_save', { request: { operation: 'native-wpm-word', headword: 'école',
      notes: 'Note conservée pour la vérification WPM.', example_fr: 'Je vais à l’école.', example_zh: '我去学校。', example_sense: 0, more_examples: [], id: null, revision: null } });
    const cards = await invoke('review_add', { vocabularyId: saved.record.id });
    const audio = await invoke('speech_generate', { vocabularyId: saved.record.id });
    const recognition = cards.find((card) => card.kind === 'recognition');
    const rated = await invoke('review_rate', { request: { operation: 'native-wpm-rating', card_id: recognition.id,
      expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    if (rated.status !== 'committed') throw new Error('Fixture rating failed');
    return { id: saved.record.id, voice: audio.voice, bytes: audio.bytes.length };
  });
  assert.equal(seeded.voice, 'female'); assert.ok(seeded.bytes > 128);
  let queue;
  await expect.poll(async () => {
    queue = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('review_queue', { timezone: 'UTC', skipped: [] }));
    return queue.window.some((entry) => entry.card.kind === 'listening');
  }).toBe(true);
  const queued = queue.window.find((entry) => entry.card.kind === 'listening');
  assert.equal(queued.audio.voice, 'female');
  assert.ok(queued.audio.bytes.length > 128 && queue.metrics.strict_probes <= 8 && queue.metrics.payload_bytes <= 33_554_432);
  const before = learningSnapshot();
  for (const [voice, gender] of [['siwis', 'female'], ['tom', 'male']]) {
    for (const [speed, wpm] of [['normal', 180], ['slow', 100]]) {
      const legacy = JSON.stringify({ voice, speed });
      sql("c.execute(\"INSERT INTO app_metadata VALUES('speech-preferences',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value\",[sys.argv[2]])\nc.commit()", legacy);
      assert.deepEqual(await preferences(page), { voice: gender, wpm });
      assert.equal(rawPreference(), legacy);
    }
  }
  await fixture.stop(true); page = await fixture.start();
  assert.deepEqual(await preferences(page), { voice: 'male', wpm: 100 });
  await observePlayback(page);
  await page.getByRole('button', { name: '设置', exact: true }).click();
  for (const [voice, wpm, rate] of [
    ['female', 80, 0.535989389], ['female', 180, 1.205976126], ['female', 240, 1.607968168],
    ['male', 80, 0.536376731], ['male', 180, 1.206847645], ['male', 240, 1.609130193],
  ]) {
    await page.getByLabel('法语声线').selectOption(voice);
    const slider = page.getByRole('slider', { name: '朗读语速' });
    if (wpm === 80) { await slider.focus(); await slider.press('Home'); }
    else await page.getByRole('button', { name: wpm === 180 ? '正常 180 WPM' : '快速 240 WPM', exact: true }).click();
    await page.getByRole('button', { name: '保存语音偏好' }).click();
    await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
    results.push({ voice, wpm, ...await play(page, page.getByRole('button', { name: /^试听短句:/ }), rate) });
    await page.evaluate(() => Reflect.get(window, 'wpmPlayers').forEach((audio) => audio.pause()));
  }
  await page.screenshot({ path: join(destination, 'wpm-settings.png'), fullPage: true });
  for (const wpm of [60, 90, 260, 180.5, '180']) {
    await assert.rejects(preferences(page, { voice: 'female', wpm }));
  }
  assert.deepEqual(await preferences(page), { voice: 'male', wpm: 240 });
  sql("c.executescript(\"CREATE TRIGGER wpm_save_failure BEFORE INSERT ON app_metadata WHEN NEW.key='speech-preferences' BEGIN SELECT RAISE(ABORT,'native WPM save blocked'); END;\")");
  await page.getByRole('button', { name: '慢速 100 WPM', exact: true }).click();
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByRole('alert')).toContainText('native WPM save blocked');
  await expect(page.getByRole('slider', { name: '朗读语速' })).toHaveValue('100');
  assert.deepEqual(await preferences(page), { voice: 'male', wpm: 240 });
  assert.deepEqual(JSON.parse(rawPreference()), { voice: 'male', wpm: 240 });
  sql('c.execute("DROP TRIGGER wpm_save_failure")\nc.commit()');
  await page.getByRole('button', { name: '保存语音偏好' }).click();
  await expect(page.getByText('语音偏好已保存。', { exact: true })).toBeVisible();
  await preferences(page, { voice: 'male', wpm: 240 });
  await page.getByRole('button', { name: '词典', exact: true }).click();
  await page.getByLabel('法语词或词组').fill('école');
  await page.getByRole('button', { name: '查词', exact: true }).click();
  results.push({ flow: 'word', ...await play(page, page.getByRole('button', { name: /播放单词: école/ }).first(), 1.609130193) });
  await page.getByRole('button', { name: '生词本', exact: true }).click();
  await page.locator('.vocabulary-list button').click();
  const listening = await play(page, page.getByRole('button', { name: '查看听音卡', exact: true }), 1.607968168);
  results.push({ flow: 'listening-selected-female', ...listening });
  const card = page.getByRole('dialog', { name: '听音卡', exact: true });
  await card.getByRole('button', { name: '显示答案', exact: true }).click();
  await preferences(page, { voice: 'male', wpm: 80 });
  assert.ok(Math.abs(await page.evaluate((index) => Reflect.get(window, 'wpmPlayers')[index].playbackRate, listening.index) - 1.607968168) < 0.000001);
  results.push({ flow: 'listening-next-play', ...await play(page, card.getByRole('button', { name: '重播', exact: true }), 0.535989389) });
  await expect(card.getByRole('heading', { name: 'école', exact: true })).toBeVisible();
  await card.getByRole('button', { name: '跳过', exact: true }).click();
  await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('nous avions lavé');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  results.push({ flow: 'conjugation', ...await play(page, page.getByRole('button', { name: /播放表达: nous avions lavé/ }).first(), 0.536376731) });
  assert.deepEqual(learningSnapshot(), before);
  await fixture.stop(true); page = await fixture.start();
  assert.deepEqual(await preferences(page), { voice: 'male', wpm: 80 });
  assert.deepEqual(learningSnapshot(), before);
  const valid = rawPreference();
  sql("c.execute(\"UPDATE app_metadata SET value='broken' WHERE key='speech-preferences'\")\nc.commit()");
  await assert.rejects(preferences(page), /Invalid saved speech preferences/);
  sql("c.execute(\"UPDATE app_metadata SET value=? WHERE key='speech-preferences'\",[sys.argv[2]])\nc.commit()", valid);
  await fixture.stop(true);
  blockedAsset = join(destination, 'speech/assets/text_encoder.int8.onnx');
  await rename(blockedAsset, blockedAsset + '.unavailable');
  page = await fixture.start();
  await page.getByRole('button', { name: '动词变位', exact: true }).click();
  await page.getByLabel('法语动词或简单变位词形').fill('nous avons lavé');
  await page.getByRole('button', { name: '查询变位', exact: true }).click();
  await page.getByRole('button', { name: /播放表达: nous avons lavé/ }).first().click();
  await expect(page.locator('.conjugation').getByRole('alert')).toBeVisible();
  assert.deepEqual(learningSnapshot(), before);
  await writeFile(join(destination, 'wpm-results.json'), JSON.stringify({ status: 'passed', playback: results,
    legacyRead: 'passed', saveFailure: 'passed', invalidPreferences: 'passed', restart: 'passed',
    learningAndSelectedAudio: 'preserved', resourceFailure: 'visible', listeningQuality: 'pending human listening' }, null, 2));
  console.log(JSON.stringify({ status: 'passed', playbackCases: results.length, learningAndSelectedAudio: 'preserved' }));
} finally {
  await fixture.finish();
  if (blockedAsset) await rename(blockedAsset + '.unavailable', blockedAsset);
}
