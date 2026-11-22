# Issue #130 author package

**50 frozen groups / 50 selected-use proposals / 200 sourced fields** for rows
2451–2500. All exact headwords, POS subclasses, frequency strings and order match
[ticket #130](https://github.com/zardonc/lexique/issues/130). The denominator remains
**4,087 groups / 3,773 headwords**. There are no reused groups, new annotations,
specialist exceptions, runtime aliases or learner-data changes.

[Groups](groups.json) are proposals pending separate independent review.
[Research](source-research.md), [sources](sources.json), [excerpts](source-excerpts.json),
[context](source-context.json), [field proposals](field-review.json),
[selected rights](selected-rights.md), [rights records](rights.json),
[notices](NOTICES.md) and [COPYING](COPYING) form the author package.
[Original proposals](authored/groups.json) remain available for accepted replacements.

The source set comprises **50 actual fixed French article/redirect captures and
one fixed Lexique dataset source**. Contributor pagination completed in seven
responses with 3,234 named article-contributor memberships. Raw evidence, request
URLs, fixed manual and reproducing helpers remain at
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-130/`.
[Capture records](source-captures.json) give precise locations.

## Verification

`uv run --no-project python scripts/test_core_batch_130.py --author-only` passes
four focused checks: exact frozen proposals/honest gates, actual capture/source
bindings, complete contributor pagination, and phrase/ambiguous-form bindings.
The initial resource-boundary check failed because all 50 rows lack admitted
content. The default command still fails that admission check as expected.
[Verification metadata](verification.json) and ignored logs record actual results.
`git diff --check` and focused local-link checks are recorded separately.

Independent accepted new groups/fields: **0 / 0**. All **200 fields** await
independent quality and selected-span rights review and remain missing from
admitted-content coverage. Author content blockers: **none**; selected-use and
rights eligibility are author proposals, subject to that review.

## Review and integration

Use the [separate reviewer handoff](README-independent-review.md). After review,
merge `codex/d2-content-130` into `codex/d2-content-130-139` using
`git merge --no-ff codex/d2-content-130`. The coordinator then assembles exact
accepted rows and updates resource coverage against all 4,087 groups.

Two concrete converter requirements are retained in [admission](admission.json):
the current row limit ends at 2450; the phrase IPA uses actual Lexique version
`3.83` with `fixed_licensed_dataset_capture`, which the current wiki/public-domain
source validator cannot accept. Preserve its CC BY-SA licence and original
dataset credits when adding support. No temporary acceptance flag or simulated
independent-review state is provided.

Combined conversion/package checks, actual native lookup for all 50 groups,
representative save/reopen and persisted-data preservation remain coordinator-owned.
Learner acceptance, other-platform execution, whole-machine offline checks and
formal release completion remain pending.
