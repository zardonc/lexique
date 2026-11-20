# Ticket #96 validation

All **50 groups / 200 fields** passed independent source, rights and quality
review, including the scoped spelling/form relationships.
[Independent review](independent-review.md), [admission](admission.json) and the
canonical [field review](field-review.json) record the final accepted content.
Native lookup/save/reopen, saved-state preservation and offline verification
results and remaining gates are recorded in the parent
[aggregate verification](../issue-90-99/README.md).

## Retained author checks

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-content-96`, branch
`codex/d2-content-96`, Windows x64, PowerShell 7 and uv/Python 3.14.10.
The initially clean worktree was fast-forwarded to integration base
`b2891b3a8e15f4ad5f22d54ed00e62237d6409d7` before ticket edits.

Complete command logs and capture data are ignored under
`C:/Users/inthe/IdeaProjects/lexique-content-96/target/issue-96/`.
The following results describe the original candidate handoff. Full captures
and logs are preserved under integration's
`target/implement-90-99/worker-audit/author-96/`.

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
admitted SQLite artifact check or independent semantic review. The final accepted
replacements are included in the [combined core](../../../resources/core-pilot-79/README.md);
combined resource, native and package results belong to the parent aggregate.

The content author and independent reviewer are separate. Learner acceptance and
whole-core release remain open.
