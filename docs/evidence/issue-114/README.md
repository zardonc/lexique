# Issue #114 authored content

All **50 exact frozen rows 1651–1700** have one authored selected sense with
**FR/IPA/EN/ZH: 200 proposed fields**, with no missing authored fields or row
substitutions. The fixed denominator remains **4,087 groups**. **Zero new fields
are independently accepted or admitted**; separate semantic, selected-span
rights and construction/spelling review remains pending for all 50 rows.

The [ticket](https://github.com/zardonc/lexique/issues/114) and
[parent](https://github.com/zardonc/lexique/issues/78) define this batch.
The parent’s Testing Decisions approve the existing resource/admission seam.

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact identities, frequencies, order, selected uses and four fields |
| [sources.json](sources.json) | Fixed revisions, article/history attribution, licences and modifications |
| [source-excerpts.json](source-excerpts.json) | 200 exact field bindings and language/POS/construction support |
| [source-context.json](source-context.json) | Full returned contributor registries, import/reference boundaries and grammar |
| [source-captures.json](source-captures.json) | Raw API/wikitext paths and terminal contributor pagination |
| [field-review.json](field-review.json) | Actual proposed text and separate pending decisions for every field/use |
| [rights.json](rights.json) | Per-source author rights proposals, original-work layers and obligations |
| [reuse.json](reuse.json) | Zero reuse against the base admitted input; existing content unchanged |
| [admission.json](admission.json) | Counts and pending review, resource, conversion and native gates |
| [source-research.md](source-research.md) | Selected-use restrictions and rights/source boundaries |
| [independent-review.md](independent-review.md) | Separate reviewer handoff, with no acceptance claimed |

## Author checks

The [focused test](../../../scripts/test_core_batch_114.py) first recorded RED
on the absent package, then GREEN on the complete authored package. Preparation
compares complete retained API revisions and wikitext, verifies French/POS
frames, and validates every selected source span. These checks verify author
bindings and handoff completeness; they do not supply independent quality review.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-114/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-114/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_114.py
git diff --cached --check
```

Local evidence: [RED](../../../target/issue-114/red.log),
[source preparation](../../../target/issue-114/prepare.log),
[author rights check](../../../target/issue-114/rights-author-check.log),
[GREEN](../../../target/issue-114/green.log),
[final checks](../../../target/issue-114/final-verification.json).
The [capture manifest](source-captures.json) gives exact raw paths.
Preparation reuses the previous #109 author scripts retained under
`C:/Users/inthe/IdeaProjects/lexique-spec-100-109/target/implement-100-109/worker-audit/author-109-final/`.

Owning worktree: `C:/Users/inthe/IdeaProjects/lexique-batch-114`, branch
`codex/d2-content-114`, based on integration `codex/d2-content-110-119` at
`fa4ea3e`. Full captures, selection/preparation scripts and logs remain under
`target/issue-114/` in that worktree. The integration tip is merged before delivery.

## Remaining gates

All rows **1651–1700**, each with four authored fields, await fresh separate
semantic, rights and construction/spelling review arranged by the parent.
There are no author-identified missing-content rows, owner exceptions or runtime
aliases. No assigned group was previously admitted at the base.

Canonical resource admission, combined conversion/notices, actual native
lookup/save/reopen, learner acceptance and release checks remain parent-owned
and unrun for this author handoff. Existing admitted resources and learner data
are unchanged.
