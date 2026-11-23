import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { native24Fixture } from '../apps/desktop/e2e/native-24-harness.mjs';
import { fixtureSql } from '../apps/desktop/e2e/native-backup-state.mjs';

process.chdir(fileURLToPath(new URL('../apps/desktop/', import.meta.url)));
const target = resolve('../../target');
const issues = Array.from({ length: 10 }, (_, index) => `issue-${150 + index}`);

async function removeOwned(path) {
  const inside = relative(target, resolve(path));
  assert.ok(inside && !inside.startsWith('..') && !isAbsolute(inside));
  await rm(path, { recursive: true, force: true });
}

test('new D2 batch fixtures accept descendants and reject roots, siblings and escapes', {
  skip: process.platform !== 'win32',
}, async () => {
  for (const issue of [...issues, 'grammar-classification']) {
    const root = join(target, issue);
    await mkdir(root, { recursive: true });
    const owned = await mkdtemp(join(root, 'guard-preflight-'));
    const missingPackage = join(owned, 'missing-package');
    const missingBinary = join(owned, 'missing.exe');
    const destination = join(owned, 'native');
    const sibling = join(target, `${issue}-sibling-${randomUUID()}`);
    const escape = join(root, '..', `guard-escape-${randomUUID()}`);
    try {
      // A missing package ends the fixture before any native process starts.
      await assert.rejects(native24Fixture(missingPackage, missingBinary, destination), { code: 'ENOENT' });
      await mkdir(join(destination, 'Lexique-data', 'data'), { recursive: true });
      assert.equal(fixtureSql(destination, 'print(json.dumps(c.execute("SELECT 7").fetchone()[0]))'), 7);
      for (const denied of [root, sibling, escape]) {
        await assert.rejects(native24Fixture(missingPackage, missingBinary, denied), { code: 'ERR_ASSERTION' });
        assert.throws(() => fixtureSql(denied, 'print(json.dumps(7))'), { code: 'ERR_ASSERTION' });
      }
    } finally {
      await removeOwned(owned);
      await removeOwned(sibling);
      await removeOwned(resolve(escape));
    }
  }
});
