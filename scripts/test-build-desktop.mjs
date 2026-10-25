import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, copyFileSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));

function fixture(run) {
  const directory = mkdtempSync(join(tmpdir(), 'lexique-build-'));
  try {
    mkdirSync(join(directory, 'scripts'));
    copyFileSync(join(root, 'scripts/build-desktop.mjs'), join(directory, 'scripts/build-desktop.mjs'));
    const config = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
    writeFileSync(join(directory, 'package.json'), JSON.stringify(config));
    const cli = join(directory, 'apps/desktop/node_modules/@tauri-apps/cli/tauri.js');
    mkdirSync(dirname(cli), { recursive: true });
    return run(directory, cli);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test('public pnpm build preserves spaced arguments and sends locked to Cargo', () => fixture((directory, cli) => {
  writeFileSync(cli, `require('node:fs').writeFileSync('received.json', JSON.stringify(process.argv.slice(2)));`);
  assert.ok(process.env.npm_execpath, 'Run through pnpm test:tooling');
  const result = spawnSync(process.execPath, [process.env.npm_execpath, 'desktop:build', '--debug', '--no-bundle',
    '--config', 'config with spaces.json'], { cwd: directory, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.deepEqual(JSON.parse(readFileSync(join(directory, 'received.json'), 'utf8')),
    ['build', '--debug', '--no-bundle', '--config', 'config with spaces.json', '--', '--locked']);
}));

test('CLI failure remains a failed build', () => fixture((directory, cli) => {
  writeFileSync(cli, `console.error('native build failed'); process.exit(19);`);
  const result = spawnSync(process.execPath, ['scripts/build-desktop.mjs'], { cwd: directory, encoding: 'utf8' });
  assert.equal(result.status, 19);
  assert.match(result.stderr, /native build failed/);
}));
