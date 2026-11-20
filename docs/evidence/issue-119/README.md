# Issue #119 independently reviewed content

All **50 exact frozen rows 1901–1950** have complete selected content:
**49 new groups / 49 selected senses / 196 authored FR/IPA/EN/ZH fields**, plus
the **unchanged accepted côte/NOM group / four fields at row 1911**. The fixed
denominator remains **4,087 groups**. No assigned group or authored field is
missing. All **49 new uses / 196 fields are independently accepted**, with
five exact wording replacements and no unresolved fields. The full batch is
**50 accepted groups / 200 fields**, counting reused côte once globally.
Canonical admission and actual combined packaging remain parent-owned.

The [independent review](independent-review.md) records individual field,
use, spelling and selected-rights decisions; [rights review](rights-review.md)
records the corrected DAF8 original-work basis. Original proposals are retained
in [review-candidates.json](review-candidates.json) and `field-review.json`.

The [ticket](https://github.com/zardonc/lexique/issues/119) and
[parent](https://github.com/zardonc/lexique/issues/78) define this batch. The
parent’s Testing Decisions authorize the existing resource/admission seam.

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities, order, frequencies, selected uses and four fields |
| [reuse.json](reuse.json) | `groups: [{frozen_row: 1911, ...}]`, fixed record/field identifiers and unchanged accepted côte |
| [sources.json](sources.json) | 52 fixed source revisions, attribution, licences and modifications |
| [source-excerpts.json](source-excerpts.json) | 196 exact new field bindings and construction evidence |
| [source-context.json](source-context.json) | Selected boundaries, import/reference markers, complete named registries and anonymous counts |
| [source-captures.json](source-captures.json) | Complete retained API/wikitext paths, contributor pagination and preparation scripts |
| [field-review.json](field-review.json) | Proposed/accepted texts and independent field/use/spelling reasons; no côte review copies |
| [rights.json](rights.json) | Original proposals, independent selected-rights decisions, exact spans and obligations |
| [admission.json](admission.json) | 49 new accepted groups / 196 fields, one reused group / four fields and separate parent gates |
| [NOTICES.md](NOTICES.md) | Attribution, modification and original-work notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [source-research.md](source-research.md) | Selected-use table and construction/spelling evidence |
| [independent-review.md](independent-review.md) | Independent reviewer result and verification |
| [review-candidates.json](review-candidates.json) | Unchanged original author group objects and proposals |
| [rights-review.md](rights-review.md) | Independent original-work and selected-boundary reasoning |

`côte/NOM` retains `cote-noun-rib`, source `fr_cote` and all four accepted
field objects exactly as recorded in the existing
[curated input](../../../resources/core-pilot-79/curated-inputs.json).
The [existing review](../../../resources/core-pilot-79/field-review.json)
continues to supply its quality evidence. It contributes no new pending fields.

## Author verification

The authored [focused check](../../../scripts/test_core_batch_119.py) passed
**2 tests** before review; the independent review now passes **3 tests**.
The TDD record covers absent membership/reuse evidence and incomplete source
metadata before their corresponding passing packages. Complete fixed API
revisions reproduce the authored source bindings; byte-for-byte reproduction
passes for all **9 JSON evidence files**. Full retained contributor pagination
terminates for both French request sets and the English `heu` request.
Owned-document links, the repository documentation check and staged diff check
are recorded in the final verification log.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-119/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-119/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-119/finish_spelling.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_119.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-119/reproduce.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-119`, branch
`codex/d2-content-119`. The clean initial base was an ancestor of integration
`codex/d2-content-110-119`; integration was merged at
`6b29f8be699ef3b489a4e5285db18ed120fe7501` before authoring. The latest
integration tip is merged before handoff. The tested commits, commands and
results are in [final verification](../../../target/issue-119/final-verification.json).

All raw responses, requests, source wikitext, scripts, reproduction and check
logs remain in this checkout’s ignored `target/issue-119/`, including
[membership RED](../../../target/issue-119/red.log),
[source RED](../../../target/issue-119/rights-red.log),
[GREEN](../../../target/issue-119/green.log),
[reproduction](../../../target/issue-119/reproduction.log) and
[documentation](../../../target/issue-119/full-docs-check.log).
Preparation follows reviewed #110/#111/#113/#114 evidence layouts and the
authored #116 pattern. Fixed rights authorities are reused unchanged from
`C:/Users/inthe/IdeaProjects/lexique-batch-116/target/issue-116/`.

## Remaining gates

All **49 new groups / 196 new fields** pass independent semantic,
selected-span rights and construction/spelling review. There are **zero
unresolved content fields**, new owner exceptions, drops or substitutions.
All spelling/inflection relations are source bindings;
`alias_admission=not_admitted` throughout.

Canonical admission, combined conversion/notices, actual native
lookup/save/reopen, learner acceptance and platform/offline/release checks
remain parent-owned and unrun by this reviewer. Existing learner data
and canonical resources are unchanged.
