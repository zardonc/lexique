import { expect, test } from '@playwright/test';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

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
