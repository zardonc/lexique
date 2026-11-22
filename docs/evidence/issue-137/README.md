# Issue #137 author package

**50 exact frozen groups / 50 selected-use proposals / 200 sourced fields** for
rows **2801–2850**, matching [ticket #137](https://github.com/zardonc/lexique/issues/137)
and the unchanged [scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).
The denominator remains **4,087 groups / 3,773 headwords**. No group is omitted,
substituted or exempted. Reused groups/fields, new annotations and runtime aliases: **0**.

[Groups](groups.json), [field proposals](field-review.json), [sources](sources.json),
[research](source-research.md), [context](source-context.json),
[excerpts](source-excerpts.json), [rights](rights.json),
[selected rights](selected-rights.md), [notices](NOTICES.md) and [COPYING](COPYING)
form the author package. [Original proposals](authored/groups.json) and companion
authored ledgers preserve the exact wording for a separate reviewer.

There are **53 fixed French article captures**: 50 exact headwords and the lexical
targets `surveiller`, `trahir` and `automobile`. Complete paginated article credits
and source/version/span records accompany every field. English and Chinese are
new Lexique explanations of the selected French semantics. They are not upstream
bilingual quotations. `évanouir` retains the modern `s’évanouir` construction;
`cingler` selects weather lashing, `boucle` a belt buckle, `as` the card noun with
pronounced final /s/, and `hâte` retains h aspiré. Register and exclusions are explicit.

Raw captures, requests, helpers and logs are preserved at
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-137/`.
[Capture metadata](source-captures.json) and [admission metadata](admission.json)
resolve missing `target/issue-137` files through that stable archive.

## Verification

Run `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_137.py --author-only`
for the four focused author checks. The default command also checks canonical
admission and reports the missing 50 rows until coordinator integration.
[Verification](verification.json) records actual commands and results, including
source-cache fallback, reproduction, document links and diff checks.

Use the [independent-review handoff](README-independent-review.md) for separate
quality and selected-span rights review. No independent reviewer is assigned by
the author. All original proposals remain intact for exact accepted replacements.

## Limitations

Independent accepted new groups/fields: **0 / 0**. All **200 fields** remain pending
quality and selected-span rights review and missing from admitted coverage.
Author content blockers: **none**. The current canonical base is `core-129-v2`
with 2,435 admitted groups; this package makes no change to the core or learner data.
Shared registration includes rows through 2950 at integration `7e13502`.
This batch still requires independently accepted assembly. Combined conversion, notices,
native lookup of every group, trusted save/reopen and persisted-data preservation
remain coordinator-owned. Learner acceptance, other-platform execution,
whole-machine offline checks and release completion remain pending.
