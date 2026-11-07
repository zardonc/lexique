import { spawnSync } from 'node:child_process';
import { cpSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = new URL('../', import.meta.url);
// pnpm consumes forwarded separators; add Cargo's separator at the final process boundary.
const result = spawnSync(process.execPath, [
  fileURLToPath(new URL('apps/desktop/node_modules/@tauri-apps/cli/tauri.js', root)),
  'build', ...process.argv.slice(2), '--', '--locked',
], { cwd: fileURLToPath(root), stdio: 'inherit' });
if (result.error) console.error(result.error);
if (result.status === 0) {
  const args = process.argv.slice(2);
  const targetIndex = args.indexOf('--target');
  const target = targetIndex < 0 ? args.find((arg) => arg.startsWith('--target='))?.slice(9) : args[targetIndex + 1];
  const profile = args.includes('--debug') ? 'debug' : 'release';
  const output = new URL(`target/${target ? `${target}/` : ''}${profile}/`, root);
  for (const [prepared, destination] of [['desktop-dictionary-sample', 'dictionary'], ['desktop-conjugation-v6-color', 'conjugation']]) {
    cpSync(new URL(`target/${prepared}/`, root), new URL(`${destination}/`, output), { recursive: true });
  }
}
process.exitCode = result.status ?? 1;
