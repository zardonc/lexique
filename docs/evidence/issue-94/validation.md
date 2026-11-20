# Validation

All **50 groups / 200 fields** passed independent source, rights and quality
review. [Independent review](independent-review.md), [admission](admission.json)
and the canonical [field review](field-review.json) record the accepted delivery.
Native lookup/save/reopen, saved-state preservation and offline verification
results and remaining gates are recorded in the parent
[aggregate verification](../issue-90-99/README.md).

## Retained author checks

Base: `3397219e8d159ef60db3ed5456bdcb2d6ff41b4a`, matching integration branch `codex/d2-content-90-99` before edits. Checkout: `C:/Users/inthe/IdeaProjects/lexique-content-94`, branch `codex/d2-content-94`. Execution: Windows x64, PowerShell 7, uv with Python 3.14.10; pnpm 11.25.0 for the documented documentation check.

The following results describe the original candidate handoff. Its full captures
and logs are preserved under integration's
`target/implement-90-99/worker-audit/author-94/`.

| Command / check | Result | Retained evidence |
| --- | --- | --- |
| `git merge-base --is-ancestor codex/d2-content-90-99 HEAD` | Passed before edits; HEAD equalled integration tip. | Base recorded above. |
| `uv run --python 3.14.10 python -m unittest discover -s scripts -p test_core_batch_94.py` before adding content | Red: 50 exact missing-row failures in one focused test. | `target/issue-94/red.log` |
| Same focused test after adding draft shard | Green: one test passed, all 50 groups and 200 nonempty fields checked. | `target/issue-94/green.log` |
| `uv run --python 3.14.10 python target/issue-94/assemble.py` | Passed: exact fixed-capture span bindings, 50 groups, 200 fields, 88 used source pages; four shared monolithic files compared byte-for-byte with base. | `target/issue-94/assemble.log`, `author-check.json` |
| `git diff --cached --check` | Passed for the ticket-scoped handoff. | `target/issue-94/diff-check.log` |
| Existing `Test-Documentation` validator restricted to the staged Markdown files | Passed: three documents, 15 local links, zero problems. | `target/issue-94/docs-check.json` |

The focused test exercised the original additions-only draft alongside existing curation. Its structural checks did not establish semantic or rights acceptance. The final accepted replacements are included in the [combined core](../../../resources/core-pilot-79/README.md); combined resource, native and package results belong to the parent aggregate.

The content author and independent reviewer are separate. Learner acceptance and wider release gates remain open.
