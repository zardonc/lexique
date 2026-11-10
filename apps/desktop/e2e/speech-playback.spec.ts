import { expect, test } from '@playwright/test';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { readFileSync } from 'node:fs';

test('slow playback uses the same normal-speed audio at 0.6x', async ({ page }) => {
  const file = resolve(process.cwd(), '../../docs/evidence/issue-14/supertonic-slower/listen.html');
  await page.goto(pathToFileURL(file).href);
  for (const voice of ['女声', '男声']) {
    const normal = page.getByLabel(`${voice}法语句子常速`);
    const slow = page.getByLabel(`${voice}法语句子慢速`);
    expect(await normal.getAttribute('src')).toBe(await slow.getAttribute('src'));
    expect(await normal.evaluate((audio: HTMLAudioElement) => audio.playbackRate)).toBe(1);
    expect(await slow.evaluate((audio: HTMLAudioElement) => audio.playbackRate)).toBe(0.6);
    await slow.evaluate(async (audio: HTMLAudioElement) => { await audio.play(); });
    await expect.poll(() => slow.evaluate((audio: HTMLAudioElement) => audio.currentTime)).toBeGreaterThan(0);
    expect(await slow.evaluate((audio: HTMLAudioElement) => audio.playbackRate)).toBe(0.6);
    await slow.evaluate((audio: HTMLAudioElement) => audio.pause());
  }
});

test('calibrated Opus samples keep full sentence durations and play at each WPM endpoint', async ({ page }) => {
  const folder = resolve(process.cwd(), '../../docs/evidence/issue-56');
  const report = JSON.parse(readFileSync(resolve(folder, 'calibration.json'), 'utf8')) as {
    samples: { file: string; duration_seconds: number }[];
  };
  await page.goto(pathToFileURL(resolve(folder, 'listen.html')).href);
  const durations = await page.evaluate(async (samples) => Promise.all(samples.map((sample) => new Promise<number>((done, reject) => {
    const audio = new Audio();
    audio.onloadedmetadata = () => done(audio.duration);
    audio.onerror = () => reject(new Error(`Cannot decode ${sample.file}`));
    audio.src = new URL(sample.file, location.href).href;
  }))), report.samples);
  for (const [index, duration] of durations.entries()) expect(Math.abs(duration - report.samples[index].duration_seconds)).toBeLessThan(0.03);
  for (const [voice, wpm, rate] of [
    ['女声', 80, 0.535989389], ['女声', 180, 1.205976126], ['女声', 240, 1.607968168],
    ['男声', 80, 0.536376731], ['男声', 180, 1.206847645], ['男声', 240, 1.609130193],
  ] as const) {
    const audio = page.getByLabel(`${voice} ${wpm} WPM`, { exact: true });
    expect(await audio.evaluate((audio: HTMLAudioElement) => audio.playbackRate)).toBeCloseTo(rate, 7);
    expect(await audio.evaluate((audio: HTMLAudioElement) => audio.preservesPitch)).toBe(true);
    await audio.evaluate(async (audio: HTMLAudioElement) => audio.play());
    await expect.poll(() => audio.evaluate((audio: HTMLAudioElement) => audio.currentTime)).toBeGreaterThan(0);
    await audio.evaluate((audio: HTMLAudioElement) => audio.pause());
  }
});
