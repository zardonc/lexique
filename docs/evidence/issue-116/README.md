# Issue #116 independently reviewed content

All **50 exact frozen rows 1751–1800 / 50 senses / 200 FR/IPA/EN/ZH fields**
are independently accepted, with **zero unresolved fields**. The review accepts
197 fields unchanged and three restricted replacements for `nombre/NOM`.
The denominator remains **4,087 groups**. Canonical admission remains pending,
with **zero newly admitted fields** in this reviewer package.

The [ticket](https://github.com/zardonc/lexique/issues/116) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define this batch.
The parent’s Testing Decisions preapprove the existing resource/admission seam.

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact identities, order, frequencies, selected uses and four fields |
| [sources.json](sources.json) | Fixed revisions, attribution, licences and modifications |
| [source-excerpts.json](source-excerpts.json) | 200 exact field bindings and grammatical support |
| [source-context.json](source-context.json) | Full returned contributor registries, import/reference boundaries and grammar |
| [source-captures.json](source-captures.json) | Raw API/wikitext paths and complete contributor pagination |
| [field-review.json](field-review.json) | Original proposals, accepted texts and independent field/use/spelling decisions |
| [review-candidates.json](review-candidates.json) | Complete unchanged author groups and rationale |
| [rights.json](rights.json) | Every field’s selected boundary, author rights basis and obligations |
| [reuse.json](reuse.json) | Zero reuse against the existing admitted input |
| [admission.json](admission.json) | Counts and separate pending admission/conversion/native gates |
| [NOTICES.md](NOTICES.md) | Contribution, original-work and frozen-membership notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [source-research.md](source-research.md) | Selected-use and construction/spelling/flexion evidence |
| [independent-review.md](independent-review.md) | Independent decisions, rights and verification |
| [rights-review.md](rights-review.md) | Selected boundaries and redistribution obligations |

## Author verification

The author's [focused check](../../../scripts/test_core_batch_116.py) recorded RED on
the absent package and GREEN on the authored handoff. Preparation compares the
complete retained API revisions with their wikitext and validates the French/POS
frames, all 200 field spans and isolated IPA variants. The current test checks
exact membership, frequencies, complete provenance, original proposals,
accepted restrictions, rights and construction/flexion decisions while
retaining pending canonical admission.
The focused test, byte-for-byte reproduction, owned-document links, full
repository documentation check and staged diff check pass.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-116/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-116/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_116.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-116/reproduce.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

Full evidence remains in this worktree’s ignored `target/issue-116/`:
[RED](../../../target/implement-110-119/author-evidence/issue-116/red.log),
[preparation](../../../target/implement-110-119/author-evidence/issue-116/prepare.log),
[author rights check](../../../target/implement-110-119/author-evidence/issue-116/rights-author-check.log),
[GREEN](../../../target/implement-110-119/author-evidence/issue-116/green.log),
[reproduction](../../../target/implement-110-119/author-evidence/issue-116/reproduction.log) and
[final verification](../../../target/implement-110-119/author-evidence/issue-116/final-verification.json).
Source-capture requests/responses, preparation/selection scripts, full patches
and documentation results are retained there too.

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-116`, branch
`codex/d2-content-116`. Initial integration `codex/d2-content-110-119` was
confirmed and merged at `32b5d2b`; its latest tip is merged before handoff.
The final verification record identifies the tested commit and integration tip.
Preparation reuses the #114 author scripts retained at
`C:/Users/inthe/IdeaProjects/lexique-batch-114/target/issue-114/`, with the #109
and #110–114 evidence layouts.

## Remaining gates

All rows **1751–1800** pass separate semantic, selected-span rights and
construction/spelling/flexion review. There are zero unresolved selected fields,
owner exceptions, substitutions or admitted runtime aliases. Source-binding
relations retain `alias_admission=not_admitted`.

Canonical admission, combined conversion/notices, actual native lookup/save/
reopen, learner acceptance and release/platform checks remain parent-owned and
unrun for this author handoff. Author verification supplies no independent
quality or admission decision. The subsequent independent decision is recorded
in [independent-review.md](independent-review.md); full reviewer logs remain in
`C:/Users/inthe/IdeaProjects/lexique-review-116/target/issue-116-review/`.
