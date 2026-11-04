// Independent IANA learning-day reference using Node's bundled ICU, not the Rust scheduler.
import { readFileSync } from 'node:fs';

const { now, timezone } = JSON.parse(readFileSync(0, 'utf8'));
if (!Number.isSafeInteger(now)) throw new Error('Expected integer Unix seconds');
const format = new Intl.DateTimeFormat('en-CA', {
  timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit',
  hour: '2-digit', hourCycle: 'h23',
});
function day(at) {
  const parts = Object.fromEntries(format.formatToParts(new Date(at * 1000)).map(p => [p.type, p.value]));
  const date = new Date(`${parts.year}-${parts.month}-${parts.day}T00:00:00Z`);
  if (Number(parts.hour) < 4) date.setUTCDate(date.getUTCDate() - 1);
  return date.toISOString().slice(0, 10);
}
const current = day(now);
let left = now, right = now + 172800;
if (day(right) === current) throw new Error('Learning-day boundary unavailable');
while (right - left > 1) {
  const middle = Math.floor((left + right) / 2);
  if (day(middle) === current) left = middle;
  else right = middle;
}
process.stdout.write(JSON.stringify({ day: current, next: right, icu: process.versions.icu, tz: process.versions.tz }));
