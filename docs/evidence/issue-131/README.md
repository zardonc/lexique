# D2 issue #131: sourced author package

Frozen rows **2501–2550** contain **50 proposed everyday groups / 200 authored fields**, matching the exact [ticket](https://github.com/zardonc/lexique/issues/131) under [parent #78](https://github.com/zardonc/lexique/issues/78). There are no reused groups, new exceptions, missing authored fields or author-identified unresolved content rows. **Zero new fields are independently accepted or admitted.** The global denominator remains 4,087 groups / 3,773 headwords.

The [readable proposals](source-research.md), [exact groups](groups.json), [fixed sources](sources.json), [capture registry](source-captures.json), [source and contributor context](source-context.json), [field excerpts](source-excerpts.json), [per-field review proposals](field-review.json), [rights proposals](rights.json), [selected-rights explanation](selected-rights.md), [notices](NOTICES.md) and [licence](COPYING) form the author package. The [reviewer handoff](README-independent-review.md) records the independent review and integration boundary.

## Verification

- `uv run --no-project python -B -X utf8 scripts/test_core_batch_131.py`: passed, three focused author resource/admission checks. The checks compare all frozen memberships, 200 source bindings, 53 actual revision captures, complete contributor continuation chains, rights materials and pending review states.
- `uv run --no-project python -B -X utf8 scripts/test_core_batch_131.py --admitted`: expected failure against the baseline core; all 50 groups lack admitted content. The initial failure is retained in the author evidence archive. This gate remains active for coordinator assembly after independent acceptance.
- Stable-archive fallback: passed all three author checks in a retained fixture with no local source captures.
- `git diff --cached --check`: passed. `pwsh -NoProfile -File scripts/check-docs.ps1`: passed, including documentation format and repository links. Command logs are retained in the archive.

Source, audit and helper captures are preserved at `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-131`; originals remain at `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-131/target/issue-131`. The admission and capture fallback use the stable archive. No pre-existing resources or learning data are modified.

## Remaining gates

Every proposed field, selected use and spelling/construction relationship awaits an actual separate independent reviewer. Source-specific rights and original-work assessment, including the Larousse layer, also await that reviewer. Conversion, combined notices, native lookup of all groups, trusted save/reopen, learning-state preservation, offline execution and learner acceptance require coordinator verification. This package does not establish native or release completion.
