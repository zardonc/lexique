/* global window */
import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { expect } from '@playwright/test';
import { fixtureSql, readState, compareLearning } from './native-backup-state.mjs';

export async function grammarBatchCheck(ctx) {
  const { scenario, page, destination, invoke, picker, evidence, region, file, headers, confirm, records, queue, sqlPause } = ctx;
  const decision = (title) => region.getByRole('combobox', { name: `导入决定：${title}`, exact: true });
  const exclude = (title) => region.getByRole('checkbox', { name: `排除无效组：${title}`, exact: true });
  const sql = (code, parameters = {}, readOnly = false) => fixtureSql(destination, code, parameters, readOnly);
  async function seed(title, pair = true) {
    let record = (await invoke('grammar_save', { request: { operation: `seed-${title}`, id: null, revision: null, title, body: 'Saved body', category_ids: [] } })).record;
    if (pair) record = (await invoke('grammar_card_save', { request: { operation: `card-${title}`, id: record.id, revision: record.revision, question: 'Saved question', answer: 'Saved answer' } })).record;
    return record;
  }
  const identity = (id) => sqlPause().find((row) => row.id === id).exchange_id;
  if (scenario === 'grammar-batch-mapping') {
    const filename = join(destination, 'external.csv');
    await writeFile(filename, '\ufeffID;Heading;Text;Q;A\r\ncoalesce;Été 中文;"line 1\r\nline 2";Question;Answer\r\ncoalesce;Été 中文;"line 1\r\nline 2";Question;Answer\r\nconflict;Conflict;One;;\r\nconflict;Conflict;Two;;\r\n;Independent;;;\r\n;Independent;;;\r\nbad;One sided;;Question;\r\nskipped;Excluded valid;;;\r\n');
    await region.getByRole('combobox', { name: '分隔符', exact: true }).selectOption('semicolon');
    const before = readState(destination);
    await picker('导入语法 CSV', filename); await expect(region.getByRole('alert')).toContainText('title');
    for (const [index, field] of ['exchange_id', 'title', 'body', 'question', 'answer'].entries()) await region.getByRole('combobox', { name: field, exact: true }).selectOption(String(index));
    await region.getByRole('button', { name: '重新预览', exact: true }).click();
    await expect(region).toContainText('line 2'); await expect(region).toContainText('conflicting');
    await exclude('Conflict').check(); await exclude('One sided').check(); await decision('Excluded valid').selectOption('skip');
    compareLearning(readState(destination), before); await confirm();
    const saved = await records(); assert.equal(saved.length, 3); assert.equal(saved.filter((r) => r.title === 'Independent').length, 2);
    const paired = saved.find((r) => r.title === 'Été 中文'); assert.equal(paired.body, 'line 1\r\nline 2'); assert.equal(paired.card.question, 'Question');
    assert.equal(identity(paired.id), 'coalesce'); assert.equal(new Set(sqlPause().map((r) => r.exchange_id)).size, 3);
    assert.equal(sqlPause().find((r) => r.id === paired.id).paused, paired.id); assert.deepEqual(await queue(), []);
    await picker('导入语法 CSV', ''); await expect(region.getByRole('status')).toContainText('已取消');
    evidence.checks.push('T17/T20/T22–T24/T37: real picker BOM/semicolon/external mapping, multiline locations, identical identity coalescing, conflicting group and one-sided pair exclusion, selected valid skip, independent id-free rows, cancellation');
  } else if (scenario === 'grammar-batch-learning') {
    const category = await invoke('category_create', { name: 'Local category' });
    const active = await seed('Active'), paused = await seed('Paused'), noCard = await seed('NoCard', false);
    await invoke('grammar_save', { request: { operation: 'classify-active', id: active.id, revision: active.revision, title: active.title, body: active.body, category_ids: [category.id] } });
    await invoke('review_pause', { vocabularyId: paused.id, paused: true });
    await queue(); await invoke('review_rate', { request: { operation: 'learn-active', card_id: active.card.id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    const before = readState(destination), progress = await invoke('review_progress', { cardId: active.card.id });
    await picker('导入语法 CSV', await file('updates', [headers, [identity(active.id), 'Renamed active', '', 'New question', 'New answer'], [identity(paused.id), 'Renamed paused', '', ' ', ''], [identity(noCard.id), 'New card on saved note', '', 'New Q', 'New A']]));
    for (const title of ['Renamed active', 'Renamed paused', 'New card on saved note']) await decision(title).selectOption('update');
    await confirm(); const saved = await records();
    assert.deepEqual(saved[0].category_ids, [category.id]); assert.equal(saved[0].body, ''); assert.equal(saved[0].card.id, active.card.id);
    assert.equal(saved[0].card.question, 'New question'); assert.equal(saved[0].card.answer, 'New answer');
    assert.deepEqual(await invoke('review_progress', { cardId: active.card.id }), progress);
    assert.deepEqual(saved[1].card, paused.card); assert.equal(sqlPause()[0].paused, null); assert.equal(sqlPause()[1].paused, paused.id); assert.equal(sqlPause()[2].paused, noCard.id);
    const after = readState(destination); assert.deepEqual(after.ratings, before.ratings); assert.deepEqual(after.progress, before.progress);
    assert.equal(await region.getByRole('button', { name: '开始复习', exact: true }).count(), 1);
    await picker('导入语法 CSV', await file('omitted', [['exchange_id', 'title'], [identity(active.id), 'Title only update']]));
    await decision('Title only update').selectOption('update'); await confirm(); assert.equal((await records())[0].body, '');
    await picker('导入语法 CSV', await file('omitted-preserve', [['exchange_id', 'title'], [identity(await seed('BodyPreserved', false).then((r) => r.id)), 'Body preserved rename']]));
    await decision('Body preserved rename').selectOption('update'); await confirm(); assert.equal((await records()).find((r) => r.title === 'Body preserved rename').body, 'Saved body');
    evidence.checks.push('T18/T19/T21/T27: selected real updates preserve omitted body/pair, clear explicit empty body, replace complete pairs together, retain active/paused card identities, rating/progress/due and local categories; new card on existing note atomically paused');
  } else if (scenario === 'grammar-batch-stale' || scenario === 'grammar-batch-occupied') {
    const saved = await seed('Original', false), id = identity(saved.id), occupied = scenario.endsWith('occupied');
    const rows = [headers, ['', 'Must roll back', '', 'Q', 'A'], [occupied ? 'newly-occupied' : id, 'Imported update', '', '', '']];
    await picker('导入语法 CSV', await file('stale', rows)); if (!occupied) await decision('Imported update').selectOption('update');
    if (occupied) {
      const other = await seed('Occupant', false);
      sql("c.execute('UPDATE grammar_exchange_identities SET exchange_id=? WHERE record_id=?',('newly-occupied',p['id'])); c.commit()", { id: other.id });
    } else await invoke('grammar_save', { request: { operation: 'local-edit', id: saved.id, revision: saved.revision, title: 'Local edit', body: 'Local body', category_ids: [] } });
    const before = readState(destination);
    await region.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(region.getByRole('alert')).toContainText('changed after preview');
    compareLearning(readState(destination), before); assert.ok(!(await records()).some((r) => r.title === 'Must roll back'));
    await region.getByRole('button', { name: '取消导入', exact: true }).click();
    evidence.checks.push(`T25: ${occupied ? 'newly occupied identity' : 'local revision change'} rejects whole real selected batch including earlier new note/card; SQLite unchanged`);
  } else if (scenario === 'grammar-batch-failure') {
    await queue(); const before = readState(destination);
    await picker('导入语法 CSV', await file('second-write-failure', [headers, ['first', 'First', '', 'Q1', 'A1'], ['second', 'Second', '', 'Q2', 'A2']]));
    sql("c.executescript(\"CREATE TRIGGER fail_second BEFORE INSERT ON review_paused WHEN (SELECT count(*) FROM review_paused)=1 BEGIN SELECT RAISE(FAIL,'second pause write failed'); END;\"); c.commit()");
    await region.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(region.getByRole('alert')).toContainText('second pause write failed');
    compareLearning(readState(destination), before); assert.deepEqual(await records(), []);
    sql("c.execute('DROP TRIGGER fail_second'); c.commit()"); await confirm(); const saved = await records(); assert.equal(saved.length, 2); assert.deepEqual(await queue(), []);
    sql("c.executescript(\"CREATE TRIGGER fail_first_start BEFORE DELETE ON review_paused WHEN OLD.vocabulary_id=\"+str(p['id'])+\" BEGIN SELECT RAISE(FAIL,'first card start failed'); END;\"); c.commit()", { id: saved[0].id });
    const first = region.getByRole('region', { name: 'First', exact: true }), second = region.getByRole('region', { name: 'Second', exact: true });
    await first.getByRole('button', { name: '开始复习', exact: true }).click(); await expect(first.getByRole('alert')).toContainText('first card start failed');
    await second.getByRole('button', { name: '开始复习', exact: true }).click(); await expect(second).toContainText('已开始复习'); await expect(first.getByRole('alert')).toBeVisible();
    assert.equal(sqlPause()[0].paused, saved[0].id); assert.equal(sqlPause()[1].paused, null); assert.deepEqual(await records(), saved);
    assert.deepEqual((await queue()).map((r) => r.id), [saved[1].card.id]);
    await page.screenshot({ path: join(destination, 'per-card-start-failure.png') });
    sql("c.execute('DROP TRIGGER fail_first_start'); c.commit()");
    await first.getByRole('button', { name: '开始复习', exact: true }).click(); await expect(first).toContainText('已开始复习');
    evidence.checks.push('T26/T39: real second pause write failure rolls back first and second notes, identities, cards, pause and receipt; same submission retries; first card resume fails visibly while second succeeds; imported content retained, first stays paused until explicit successful retry');
  } else if (scenario === 'grammar-batch-replay') {
    await picker('导入语法 CSV', await file('lost-response', [['title', 'question', 'answer'], ['Independent', 'Q', 'A'], ['Independent', 'Q', 'A']]));
    await page.evaluate(() => { window.dropGrammarCommitResponse = true; });
    await region.getByRole('button', { name: '确认导入', exact: true }).click(); await expect(region.getByRole('alert')).toContainText('Committed response lost');
    await queue(); const before = readState(destination), saved = await records(); assert.equal(saved.length, 2); assert.ok(sqlPause().every((r) => r.paused === r.id));
    await confirm(); const calls = await page.evaluate(() => window.grammarImportCalls.filter((call) => call.command === 'note_import_commit'));
    assert.equal(calls.length, 2); assert.deepEqual(calls[0], calls[1]); compareLearning(readState(destination), before); assert.deepEqual(await records(), saved);
    await assert.rejects(invoke('note_import_commit', { ...calls[0].args, selections: [{ index: 0, action: 'skip' }, { index: 1, action: 'create' }] }), /selection|content/i);
    evidence.calls = calls; evidence.checks.push('T24/T26: committed native batch response dropped; same UI submission returns identical receipt and two independent id-free paused cards without replaying writes; changed request reuse rejected');
  } else if (scenario === 'grammar-batch-restart') {
    const saved = await records(); assert.equal(saved.length, 2); assert.ok(sqlPause().every((r) => r.paused === r.id)); assert.deepEqual(await queue(), []);
    assert.ok(saved.every((r) => r.card.question === 'Q' && r.card.answer === 'A'));
    evidence.checks.push('packaged app reopened persisted batch library; identities, independent notes and paused queue exclusion survive');
  } else throw new Error(`Unknown batch scenario ${scenario}`);
  evidence.sqlite = sqlPause(); evidence.grammarRecords = await records();
}
