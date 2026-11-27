/* global process, URL */
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { isAbsolute, join, relative, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { native24Fixture } from '../apps/desktop/e2e/native-24-harness.mjs';
import { fixtureSql, inspectPackage } from '../apps/desktop/e2e/native-backup-state.mjs';

process.chdir(fileURLToPath(new URL('../apps/desktop/', import.meta.url)));
const target = resolve('../../target');
const issues = Array.from({ length: 15 }, (_, index) => `issue-${150 + index}`);

async function removeOwned(path) {
  const inside = relative(target, resolve(path));
  assert.ok(inside && !inside.startsWith('..') && !isAbsolute(inside));
  await rm(path, { recursive: true, force: true });
}

test('new D2 batch fixtures accept descendants and reject roots, siblings and escapes', {
  skip: process.platform !== 'win32',
}, async () => {
  for (const issue of ['issue-79', 'issue-149', ...issues, 'issue-75', 'grammar-classification', 'issue-189', 'review-fixes', 'ticket-201']) {
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
      for (const source of [destination, sibling, join(destination, '..', 'outside.lexique-backup')]) {
        assert.throws(() => inspectPackage(destination, source), { code: 'ERR_ASSERTION' });
      }
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

test('startup fixtures retain their additional roots without admitting backup destinations', {
  skip: process.platform !== 'win32',
}, async () => {
  for (const issue of ['issue-9', 'issue-24', 'issue-56', 'issue-57', 'issue-58', 'issue-59', 'issue-65', 'issue-66']) {
    const root = join(target, issue);
    await mkdir(root, { recursive: true });
    const owned = await mkdtemp(join(root, 'guard-preflight-'));
    const destination = join(owned, 'native');
    try {
      await assert.rejects(native24Fixture(join(owned, 'missing-package'), join(owned, 'missing.exe'), destination), { code: 'ENOENT' });
      assert.throws(() => fixtureSql(destination, 'print(json.dumps(7))'), { code: 'ERR_ASSERTION' });
      await assert.rejects(native24Fixture('missing-package', 'missing.exe', root), { code: 'ERR_ASSERTION' });
    } finally {
      await removeOwned(owned);
    }
  }
});
