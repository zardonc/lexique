import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = new URL('../', import.meta.url);
// pnpm consumes forwarded separators; add Cargo's separator at the final process boundary.
const result = spawnSync(process.execPath, [
  fileURLToPath(new URL('apps/desktop/node_modules/@tauri-apps/cli/tauri.js', root)),
  'build', ...process.argv.slice(2), '--', '--locked',
], { cwd: fileURLToPath(root), stdio: 'inherit' });
if (result.error) console.error(result.error);
process.exitCode = result.status ?? 1;
