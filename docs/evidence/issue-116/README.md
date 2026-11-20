# Issue #116 authored content

All **50 exact frozen rows 1751–1800** have one modern selected use and four
authored fields: **50 senses / 200 FR/IPA/EN/ZH fields**. No assigned group or
authored field is missing. The denominator remains **4,087 groups**. There are
**zero independently accepted or newly admitted fields**; all 50 rows await
the parent’s separate reviewer.

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
| [field-review.json](field-review.json) | Actual proposed text and pending field/use/spelling decisions |
| [rights.json](rights.json) | Every field’s selected boundary, author rights basis and obligations |
| [reuse.json](reuse.json) | Zero reuse against the existing admitted input |
| [admission.json](admission.json) | Counts and separate pending admission/conversion/native gates |
| [NOTICES.md](NOTICES.md) | Contribution, original-work and frozen-membership notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [source-research.md](source-research.md) | Selected-use and construction/spelling/flexion evidence |
| [independent-review.md](independent-review.md) | Separate reviewer handoff |

## Author verification

The [focused check](../../../scripts/test_core_batch_116.py) recorded RED on
the absent package and GREEN on the authored handoff. Preparation compares the
complete retained API revisions with their wikitext and validates the French/POS
frames, all 200 field spans and isolated IPA variants. The test checks exact
membership, frequencies, complete content/provenance, selected rights records,
construction/flexion evidence and pending independent/admission states.
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
[RED](../../../target/issue-116/red.log),
[preparation](../../../target/issue-116/prepare.log),
[author rights check](../../../target/issue-116/rights-author-check.log),
[GREEN](../../../target/issue-116/green.log),
[reproduction](../../../target/issue-116/reproduction.log) and
[final verification](../../../target/issue-116/final-verification.json).
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

All rows **1751–1800** await fresh separate semantic, selected-span rights,
construction/spelling/flexion review. There are no author-identified unresolved
content rows, owner exceptions, substitutions or admitted runtime aliases.
Source-binding relations retain `alias_admission=not_admitted`.

Canonical admission, combined conversion/notices, actual native lookup/save/
reopen, learner acceptance and release/platform checks remain parent-owned and
unrun for this author handoff. Author verification supplies no independent
quality or admission decision.
