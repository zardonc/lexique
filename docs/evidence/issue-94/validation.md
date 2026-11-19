# Validation

Base: `3397219e8d159ef60db3ed5456bdcb2d6ff41b4a`, matching integration branch `codex/d2-content-90-99` before edits. Checkout: `C:/Users/inthe/IdeaProjects/lexique-content-94`, branch `codex/d2-content-94`. Execution: Windows x64, PowerShell 7, uv with Python 3.14.10; pnpm 11.25.0 for the documented documentation check.

| Command / check | Result | Retained evidence |
| --- | --- | --- |
| `git merge-base --is-ancestor codex/d2-content-90-99 HEAD` | Passed before edits; HEAD equalled integration tip. | Base recorded above. |
| `uv run --python 3.14.10 python -m unittest discover -s scripts -p test_core_batch_94.py` before adding content | Red: 50 exact missing-row failures in one focused test. | `target/issue-94/red.log` |
| Same focused test after adding draft shard | Green: one test passed, all 50 groups and 200 nonempty fields checked. | `target/issue-94/green.log` |
| `uv run --python 3.14.10 python target/issue-94/assemble.py` | Passed: exact fixed-capture span bindings, 50 groups, 200 fields, 88 used source pages; four shared monolithic files compared byte-for-byte with base. | `target/issue-94/assemble.log`, `author-check.json` |
| `git diff --cached --check` | Passed for the ticket-scoped handoff. | `target/issue-94/diff-check.log` |
| Existing `Test-Documentation` validator restricted to the staged Markdown files | Passed: three documents, 15 local links, zero problems. | `target/issue-94/docs-check.json` |

The focused test exercises the additions-only draft alongside existing curation. The shared converter intentionally still accepts only the independently admitted baseline. No converted-resource, native storage, UI, save/reopen, offline, preservation-through-upgrade, packaging or release result is claimed for this draft. The parent owns those checks after independent acceptance and final shared integration.

All field and usage acceptance is pending. The author did not act as an independent reviewer. Documentation and final Git checks are recorded in the outside-repository handoff with their actual result and tested commit.
