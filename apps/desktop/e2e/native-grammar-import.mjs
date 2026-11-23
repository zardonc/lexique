/* global window, URL, Response */
import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { expect } from '@playwright/test';
import { fixtureSql, readState, compareLearning } from './native-backup-state.mjs';

export async function grammarImportCheck({ scenario, page, destination, invoke, picker, evidence, python }) {
  const records = () => invoke('grammar_list', { query: '', sort: 'created-asc' });
  await page.evaluate(() => {
    // Tauri's invoke property is immutable. Observe the real IPC fetch and only
    // replace delivery of one successful response, after native work finishes.
    const original = window.fetch.bind(window);
    window.grammarImportCalls = [];
    window.fetch = async (url, options) => {
      const response = await original(url, options);
      const command = new URL(String(url)).pathname.slice(1);
      if (!['note_import_preview', 'note_import_commit'].includes(command) || response.headers.get('Tauri-Response') !== 'ok') return response;
      const result = await response.clone().json(), args = JSON.parse(options.body);
      window.grammarImportCalls.push({ command, args, result });
      if (command === 'note_import_commit' && window.dropGrammarCommitResponse) {
        window.dropGrammarCommitResponse = false;
        return new Response('Committed response lost', { headers: { 'Tauri-Response': 'error', 'content-type': 'text/plain' } });
      }
      return response;
    };
  });
  await page.getByRole('banner').getByRole('combobox').first().selectOption('zh');
  await page.getByRole('button', { name: '笔记', exact: true }).click();
  await page.getByRole('tab', { name: '语法', exact: true }).click();
  const region = page.getByRole('region', { name: '导入语法 CSV', exact: true });
  async function file(name, rows) {
    const filename = join(destination, `${name}.csv`);
    python("import csv,json,sys\nwith open(sys.argv[1],'w',encoding='utf-8-sig',newline='') as f: csv.writer(f).writerows(json.loads(sys.argv[2]))", [filename, JSON.stringify(rows)]);
    return filename;
  }
  const headers = ['exchange_id', 'title', 'body', 'question', 'answer'];
  const body = '  été, "中文"\r\n`````  ', question = ' Pourquoi ?\n', answer = ' Parce que.\n';
  const confirm = async () => {
    await region.getByRole('button', { name: '确认导入', exact: true }).click();
    await expect(region.getByRole('status')).toContainText('导入已保存');
  };
  const queue = async () => (await invoke('review_queue', { timezone: 'UTC', skipped: [] })).queue.cards;
  const sqlPause = () => fixtureSql(destination, "print(json.dumps([dict(r) for r in c.execute('SELECT v.id,e.exchange_id,g.question,g.answer,p.vocabulary_id AS paused FROM vocabulary v JOIN grammar_exchange_identities e ON e.record_id=v.id LEFT JOIN review_cards r ON r.vocabulary_id=v.id LEFT JOIN grammar_cards g ON g.card_id=r.id LEFT JOIN review_paused p ON p.vocabulary_id=v.id ORDER BY v.id')],ensure_ascii=False))", {}, true);
  if (scenario === 'grammar-import') {
    const filename = await file('paired', [headers, ['external-native-214', '  Une règle  ', body, question, answer]]);
    const before = readState(destination);
    await picker('导入语法 CSV', filename);
    for (const value of ['external-native-214', 'Une règle', '中文', 'Pourquoi ?', 'Parce que.']) await expect(region).toContainText(value);
    compareLearning(readState(destination), before);
    await confirm();
    const [saved] = await records();
    assert.equal(saved.title, 'Une règle'); assert.equal(saved.body, body); assert.deepEqual(saved.category_ids, []);
    assert.equal(saved.card.question, question); assert.equal(saved.card.answer, answer);
    assert.deepEqual(await queue(), []);
    const state = sqlPause(); assert.equal(state[0].exchange_id, 'external-native-214'); assert.equal(state[0].paused, saved.id);
    evidence.imported = saved; evidence.sqlite = state;
    evidence.checks.push('T09/T13: actual picker, UI preview/confirmation, retained identity, raw strings, uncategorized card paused in real SQLite and excluded from real queue');
  } else if (scenario === 'grammar-import-restart') {
    const [saved] = await records(); assert.equal(saved.body, body); assert.equal(saved.card.question, question);
    assert.equal(sqlPause()[0].exchange_id, 'external-native-214'); assert.equal(sqlPause()[0].paused, saved.id);
    assert.deepEqual(await queue(), []);
    await invoke('review_pause', { vocabularyId: saved.id, paused: false });
    assert.equal((await queue())[0].id, saved.card.id);
    await invoke('review_rate', { request: { operation: 'import-rating', card_id: saved.card.id, expected_revision: 0, rating: 'good', timezone: 'UTC' } });
    assert.equal((await invoke('review_progress', { cardId: saved.card.id })).revision, 1);
    await page.getByRole('button', { name: '导出 CSV', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: '导出语法笔记' }), exportedFile = join(destination, 'reopened-export.csv');
    await dialog.getByLabel('导出范围').selectOption('all'); await picker('保存 CSV', exportedFile, true);
    await expect(dialog.getByRole('status').last()).toHaveText('已导出');
    const exported = python("import csv,json,sys\nwith open(sys.argv[1],encoding='utf-8-sig',newline='') as f: print(json.dumps(list(csv.DictReader(f)),ensure_ascii=False))", [exportedFile]);
    assert.deepEqual(exported, [{ exchange_id: 'external-native-214', title: saved.title, body, question, answer }]);
    await dialog.getByRole('button', { name: '关闭', exact: true }).click();
    evidence.sqlite = sqlPause();
    evidence.checks.push('T13/T16: packaged app reopened copied library, stable identity/content/card, queue exclusion until explicit resume, normal successful rating and real CSV re-export');
  } else if (scenario === 'grammar-import-identifiers') {
    const missing = await file('missing-id', [['title'], ['Same title']]);
    const blank = await file('empty-id', [headers, ['', 'Same title', '', ' \n', '\t']]);
    for (const filename of [missing, missing, blank]) {
      await picker('导入语法 CSV', filename); await expect(region).toContainText('确认导入时分配新身份'); await confirm();
    }
    const saved = await records(); assert.equal(saved.length, 3); assert.equal(new Set(saved.map((record) => record.id)).size, 3);
    for (const record of saved) { assert.equal(record.body, ''); assert.equal(record.card, null); }
    const state = sqlPause(); assert.equal(new Set(state.map((row) => row.exchange_id)).size, 3);
    assert.ok(state.every((row) => row.exchange_id && row.paused === null));
    evidence.sqlite = state;
    evidence.checks.push('T10/T12/T14: missing/blank identities receive distinct identities; repeated identifier-free file is a new note; missing/empty body and absent/blank pair create no card');
  } else if (scenario === 'grammar-import-validation') {
    const before = readState(destination);
    for (const [name, rows] of [
      ['one-side', [headers, ['bad-pair', 'Invalid pair', '', 'Question', '']]],
      ['title', [['title'], [' ']]],
      ['body-limit', [headers, ['', 'Long body', 'é'.repeat(20001), '', '']]],
      ['question-limit', [headers, ['', 'Long question', '', 'é'.repeat(20001), 'Answer']]],
    ]) {
      await picker('导入语法 CSV', await file(name, rows));
      await expect(region.getByRole('alert')).toContainText(/row 2/i);
      await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
      await region.getByRole('button', { name: '取消导入', exact: true }).click();
    }
    await picker('导入语法 CSV', ''); await expect(region.getByRole('status')).toContainText('已取消');
    compareLearning(readState(destination), before);
    const cancelled = await page.evaluate(() => window.grammarImportCalls.filter((call) => call.command === 'note_import_preview').at(-1));
    assert.equal(cancelled.result, null);
    await assert.rejects(invoke('note_import_commit', { operation: cancelled.args.operation, selections: [] }), /preview no longer available/i);
    evidence.checks.push('T11/T14/T37: located invalid pair/title/content limits, disabled confirmation, actual picker cancellation and released session, unchanged SQLite');
  } else if (scenario === 'grammar-import-failure') {
    const filename = await file('failure', [headers, ['failure-214', 'Saved despite start failure', body, question, answer]]);
    const before = readState(destination);
    await picker('导入语法 CSV', filename);
    fixtureSql(destination, "c.executescript(\"CREATE TRIGGER fail_import_pause BEFORE INSERT ON review_paused BEGIN SELECT RAISE(FAIL,'import pause write failed'); END;\"); c.commit()", {});
    await region.getByRole('button', { name: '确认导入', exact: true }).click();
    await expect(region.getByRole('alert')).toContainText('import pause write failed');
    compareLearning(readState(destination), before);
    fixtureSql(destination, "c.execute('DROP TRIGGER fail_import_pause'); c.commit()", {});
    await confirm(); const [saved] = await records();
    fixtureSql(destination, "c.executescript(\"CREATE TRIGGER fail_import_start BEFORE DELETE ON review_paused BEGIN SELECT RAISE(FAIL,'start write failed'); END;\"); c.commit()", {});
    await region.getByRole('button', { name: '开始复习', exact: true }).click();
    await expect(region.getByRole('alert')).toContainText('start write failed');
    await expect(region.getByRole('status')).toContainText('导入已保存');
    assert.deepEqual(await records(), [saved]); assert.equal(sqlPause()[0].paused, saved.id); assert.deepEqual(await queue(), []);
    await page.screenshot({ path: join(destination, 'start-failure.png') });
    fixtureSql(destination, "c.execute('DROP TRIGGER fail_import_start'); c.commit()", {});
    await region.getByRole('button', { name: '开始复习', exact: true }).click();
    await expect(region).toContainText('已开始复习'); assert.equal((await queue())[0].id, saved.card.id);
    evidence.sqlite = sqlPause();
    evidence.checks.push('T16/T39: real failed pause write rolls back complete import; same preview retries; real failed postcommit resume is visible, preserves content and pause; explicit retry enters real queue');
  } else if (scenario === 'grammar-import-replay') {
    const filename = await file('replay', [['title', 'question', 'answer'], ['Replay note', 'Question', 'Answer']]);
    await picker('导入语法 CSV', filename);
    await page.evaluate(() => { window.dropGrammarCommitResponse = true; });
    await region.getByRole('button', { name: '确认导入', exact: true }).click();
    await expect(region.getByRole('alert')).toContainText('Committed response lost');
    const commit = await page.evaluate(() => window.grammarImportCalls.find((call) => call.command === 'note_import_commit'));
    const before = readState(destination);
    const [saved] = await records();
    assert.ok(saved.card); assert.equal((await records()).length, 1);
    assert.equal(sqlPause()[0].paused, saved.id); assert.deepEqual(await queue(), []);
    await confirm();
    const retried = await page.evaluate(() => window.grammarImportCalls.filter((call) => call.command === 'note_import_commit').at(-1));
    assert.deepEqual(retried, commit);
    assert.deepEqual(await records(), [saved]); compareLearning(readState(destination), before);
    assert.deepEqual(await invoke('note_import_commit', commit.args), commit.result);
    compareLearning(readState(destination), before);
    await assert.rejects(invoke('note_import_commit', { ...commit.args, selections: [{ index: 0, action: 'skip' }] }), /decision|selection/i);
    await invoke('note_import_cancel', { operation: commit.args.operation });
    await assert.rejects(invoke('note_import_commit', commit.args), /preview no longer available/i);
    await picker('导入语法 CSV', filename); await confirm(); assert.equal((await records()).length, 2);
    evidence.sqlite = sqlPause(); evidence.calls = await page.evaluate(() => window.grammarImportCalls);
    evidence.checks.push('T10/T15/T37: first response dropped after real native IPC commit; UI retry with same submission returns identical receipt despite already committed preview, SQLite unchanged with one paused note/card; conflicting selection rejected, cancelled session unavailable, separate identifier-free submission creates another note');
  } else if (scenario === 'grammar-import-known') {
    const [saved] = await records(), before = readState(destination);
    const filename = await file('known', [headers, ['external-native-214', saved.title, saved.body, saved.card.question, saved.card.answer]]);
    await picker('导入语法 CSV', filename); await expect(region).toContainText('内容相同'); await confirm();
    await expect(region.getByRole('status')).toContainText('跳过 1'); compareLearning(readState(destination), before);
    await writeFile(filename, 'exchange_id,title\r\nexternal-native-214,Changed title\r\n');
    await picker('导入语法 CSV', filename); await expect(region).toContainText('当前导入保留原内容');
    await expect(region.getByRole('button', { name: '确认导入', exact: true })).toBeDisabled();
    await region.getByRole('button', { name: '取消导入', exact: true }).click();
    compareLearning(readState(destination), before);
    evidence.checks.push('known unchanged effective content skips; changed known identity previews saved and incoming text, cannot overwrite; existing learning data preserved');
  } else throw new Error(`Unknown grammar import scenario ${scenario}`);
}
