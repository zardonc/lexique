# Issue #130 separate reviewer handoff

**50 exact groups, 50 selected-use proposals and 200 sourced field proposals.**
Author checks pass; all independent content/rights gates remain pending.

- Ticket: [#130](https://github.com/zardonc/lexique/issues/130), parent
  [#78](https://github.com/zardonc/lexique/issues/78).
- Author checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-130`,
  branch `codex/d2-content-130`.
- Diff base: `e4005d868880dac12c84c412773049eea737bc67`. The delivered commit is the branch tip; resolve its full
  SHA with `git rev-parse codex/d2-content-130` before review.
- Review scope: committed `docs/evidence/issue-130/**` and
  `scripts/test_core_batch_130.py` only. Patch command:
  `git diff e4005d868880dac12c84c412773049eea737bc67..codex/d2-content-130 -- docs/evidence/issue-130 scripts/test_core_batch_130.py`.
- Retained evidence: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-130/`.
  Full patch `author-package.patch`, test-only patch `author-tests.patch`, command
  logs, raw API/wiki files, request URLs, manual and preparation helpers live there.
  A detached review checkout needs this actual root to inspect ignored captures.
- Author validation: Windows x64 / PowerShell 7 / uv Python; source-only checks,
  never native execution. [Verification](verification.json) states the tested
  working-tree scope and commit-identity files. Read [coding standards](../../../CODING_STANDARDS.md).

Read the exact issue JSON and full parent from
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/issue-130-139/` with fresh
context. [Original authored snapshots](authored/groups.json) and
[author field proposals](authored/field-review.json) retain the original wording,
source/span, usage and rights proposal. Every altered field needs a new exact
accepted-text decision; no historical review is transferable to changed wording.

Review every field for the selected sense/POS/use, spelling, pronunciation,
English/Chinese accuracy, construction/register and rights at the selected span.
Record an actual separate reviewer identity, exact proposed/accepted text,
source/version/span, reasons and replacements using the #120–#129 shape. The
author has supplied no independent reviewer identity or acceptance.

Pay particular attention to lowercase pejorative `führer`, adjective versus
language-noun `grec`, both feminine plural `lesquelles` frames, the restricted
`guerrier` sense, adjective `chic` and the exact `la plupart des` expression.
Check the dataset’s phon/syll fields and manual Tables 2/3 independently. Its
source_kind/version must remain an actual licensed dataset, not a fake wiki
revision or public-domain publication. No group may be omitted or exempted
without the project owner's actual decision.

The documented [author checks](verification.json) can be reproduced with
`uv run --no-project python scripts/test_core_batch_130.py --author-only`.
The default command intentionally retains the missing-admission failure until
the coordinator assembles independently accepted content. Assembly requires
the ROWS extension and the actual dataset-version source kind listed in
[admission](admission.json). Native lookup for every group, representative
save/reopen, preservation, combined notices and release gates are coordinator-owned.

The stable archive contains a complete copy of the original ignored captures and
helpers. Original files remain in the author checkout. Source tests resolve
missing checkout captures through admission.author_evidence_root. Reproducing
the preparation helpers requires placing them back under target/issue-130 in a
checkout, because their root is relative to that original location.
