# Ticket #96 validation

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-content-96`, branch
`codex/d2-content-96`, Windows x64, PowerShell 7 and uv/Python 3.14.10.
The initially clean worktree was fast-forwarded to integration base
`b2891b3a8e15f4ad5f22d54ed00e62237d6409d7` before ticket edits.

Complete command logs and capture data are ignored under
`C:/Users/inthe/IdeaProjects/lexique-content-96/target/issue-96/`.

| Command or check | Result | Retained evidence |
| --- | --- | --- |
| `git merge --ff-only codex/d2-content-90-99` | Passed; clean initial integration base | `base-merge.log` |
| `uv run --python 3.14.10 python -m unittest discover -s scripts -p test_core_batch_96.py` before content | Red: exact 50-row handoff missing | `red.log` |
| Same focused membership check after content | Green: one test passed | `green.log` |
| First-party API capture helper | Ten pages captured; subsequent HTTP 429 responses retained; final console encoding failure did not invalidate the saved captures | `fetch.log`, `source-index.json` |
| Same-origin browser latest then fixed-revision API | 39 French and 49 English page captures retained | `browser-latest.api.json`, `browser-fixed.api.json`, `en-latest.api.json`, `en-fixed.api.json` |
| `uv run --python 3.14.10 python target/issue-96/assemble.py` | 50 groups, 200 exact field bindings, 94 used sources; zero independent acceptances | `assemble.log` |
| Author check of spans, frozen records and pending review states | Passed | `author-check.json` |
| Scoped `Test-Documentation` and `git diff --cached --check` | Passed | `docs-check.json`, `diff-check.log` |

The focused test checks the ticket's JSON resource handoff against literal
ticket membership and nonempty four-field source bindings. It is not an
admitted SQLite artifact check or independent semantic review. Current admitted
resource inputs, converter, global notices/counters and shared tests are
preserved. Parent acceptance and conversion must reach the produced artifact
before any coverage is recorded as admitted.

Unrun worker checks: independent field/usage/rights and spelling acceptance;
admitted SQLite reproduction and membership; combined notices and packaging;
native lookup/save/reopen, offline execution and learner-state preservation.
No native GUI build or shared heavy Cargo test was run. Learner acceptance and
whole-core release remain open.
