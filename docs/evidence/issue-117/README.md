# Issue #117 independently reviewed content

**50 exact groups / 50 everyday selected senses / 200 fields accepted unchanged.**
Corrections: **0**. Unresolved selected fields: **0**. No group, spelling, subclass,
frequency or row order changes; no owner exception or runtime alias is added.
The denominator remains **4,087 groups**. Canonical admission remains **0 new
fields**, with all **200** batch fields pending parent conversion and notices.

Scope: [#117](https://github.com/zardonc/lexique/issues/117),
[parent #78](https://github.com/zardonc/lexique/issues/78), and the
[fixed manifest](../../../resources/core-pilot-79/scope-manifest.csv).

[Accepted groups](groups.json) retain all 200 texts unchanged.
[Original candidates](review-candidates.json) retain the full author proposals
and pending draft statuses. [Field review](field-review.json) records 200 exact
accepted texts, language-specific reasons, selected-rights reasons, 50 modern
uses and 50 binding-only spelling decisions. [Sources](sources.json),
[context](source-context.json), [excerpts](source-excerpts.json) and
[captures](source-captures.json) retain fixed revisions, full contributors,
original-work credits and exact field/support bindings.
[Rights](rights.json), [reuse](reuse.json) and [admission](admission.json)
separate selected acceptance from canonical admission. The
[independent review](independent-review.md) and
[selected-rights report](selected-rights-review.md) explain the decisions.

## Identity and evidence

Author commit: `352d26695550607a0ab6b2d81a75dfa05ac25780`.
Reviewed integration/diff base: `7b9510481dde923b368700a7323c6ef19bc40b03`.
Fresh reviewer checkout: `C:/Users/inthe/IdeaProjects/lexique-review-117`,
branch `codex/d2-review-117`, separate from author and parent integration owner.
This is an agent content review; human learner acceptance remains separate.

Full ticket/parent captures were read in
`C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/`.
Author archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/author-evidence/issue-117/`.
Original author evidence remains at
`C:/Users/inthe/IdeaProjects/lexique-batch-117/target/issue-117/`.
Complete reviewer projections, audits and command outputs remain at
`C:/Users/inthe/IdeaProjects/lexique-review-117/target/review-117/`.

All fields were read in ten five-group projections against fixed definitions,
IPA, French grammatical/form frames, neighboring senses and import/reference
markers. The audit matches 50 complete raw/API revision pairs, 200 field and
238 supporting spans, three authority captures and eight terminating contributor
parts with 3,335 names summed per article. Selected text remains **15,250
characters / 19,009 UTF-8 bytes**, with a longest field of **200 characters**.

## Verification

Windows 11 x64, PowerShell 7, uv CPython **3.14.10**, pnpm **11.25.0**.

| Check | Result | Complete output |
| --- | --- | --- |
| Source/revision/span/contributor/authority audit | Passed | [source-audit.log](../../../target/implement-110-119/reviewer-evidence/issue-117/source-audit.log), [source-audit.json](../../../target/implement-110-119/reviewer-evidence/issue-117/source-audit.json) |
| Focused resource check | Passed, 2 tests | [focused.log](../../../target/implement-110-119/reviewer-evidence/issue-117/focused.log) |
| Documentation and repository links | Passed | [docs-check.log](../../../target/implement-110-119/reviewer-evidence/issue-117/docs-check.log) |
| Owned diff whitespace | Passed | [diff-check.log](../../../target/implement-110-119/reviewer-evidence/issue-117/diff-check.log) |
| Committed head and merged integration tip | Recorded | [final-verification.json](../../../target/implement-110-119/reviewer-evidence/issue-117/final-verification.json) |

Commands: `uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_117.py`,
`pnpm test:docs`, `git diff --check` and `git diff --cached --check`.
The scoped committed patch and test-only patch are `reviewer.patch` and
`reviewer-test.patch` in the reviewer evidence root.

## Pending parent gates

All 200 batch fields remain unadmitted in canonical coverage. The parent owns
binding normalization, canonical inclusion, reproducible conversion, coverage
and size reports, actual packaged notices, native lookup/save/reopen, learner
acceptance and release verification. App, Rust, native, offline and release
checks were not run here. No live refetch or statute refresh is claimed.
Existing learning data and original source captures are preserved.
