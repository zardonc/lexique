# Issue #135 independent field and selected-rights review

**50 exact groups / 50 selected uses / 200 fields accepted.** Frozen rows
2701–2750, spellings, POS subclasses, frequencies and ordering are preserved.
There are 194 unchanged fields, six FR/EN/ZH replacements and zero unresolved
field blockers. No additional specialist or zero-field annotation approval is
assumed. The denominator remains 4,087 groups.

Reviewer: **Independent field and selected-rights reviewer #135; separate from
author and coordinator**. This is agent review; learner acceptance remains
separate. The reviewer did not author these proposals.
Author commit: `5a2275d8a313b298edee51455f3e228fccd4e52e`.
Review base: `7e13502`; checkout:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/review-135`, branch
`codex/d2-review-135`. Integration `2a3e70a` is merged before the scoped review
commit. Reviewer changes own this directory and
`scripts/test_core_batch_135.py` only.

The reviewer read the exact #135/#78 archive under
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/issue-130-139`, the
reviewer guide/pipeline notes, project rules, product/roadmap, admission gates,
glossary/resource ADRs and coding standards. The established #129 evidence
shape guides recording, without supplying acceptance for this batch.

## Accepted fields and uses

[Field review](field-review.json) records each exact proposal and accepted text,
fixed source/span, actual semantic reason, selected-rights reason and reviewer.
All 50 use and spelling records retain their exact constructions, exclusions
and source evidence. [Final author snapshots](authored/commit-5a2275d/) equal
their Git blobs at the author commit; earlier authored proposals remain intact.
[Replacement records](accepted-replacements.json) retain all six exact
proposed/accepted texts, spans and reasons.

`douer/VER` selects **être doué de + a natural/providential quality or ability**.
Larousse explicitly classifies the construction as passive; the fixed `doué`
verb-flexion frame establishes its relation to the frozen infinitive `douer`.
All three meaning fields state the selected passive construction. IPA remains
the isolated infinitive `/dwe/`. The absolute talented adjective, `doué pour`
and unrestricted modern active granting are excluded.

`cieux/NOM` selects **sous d’autres cieux**, meaning elsewhere/in other countries
or places. Le Robert’s current locution and dated contemporary examples support
the use. The fixed `ciel` country/climate definition supplies the semantics;
`cieux` supplies exact plural spelling and `/sjø/`. All three meaning fields
state the idiom. [Modern-use evidence](modern-use-evidence.json) records actual
external captures and their corroboration-only rights boundary.

The two `auquel` records preserve separate interrogative/relative masculine
singular à + lequel constructions. `abuser` retains informal intransitivity,
`dégueulasse` vulgar register, `hauteur` aspirated-h grammar without a pronounced
`/h/`, `disparu` the selected missing-person subset, and `âgé` absolute elderly
use. `étonnant` takes IPA from its adjective agreement template. Noun, adjective,
verb and other-language frames were checked at every selected boundary.

## Source and rights audit

All **51 fixed articles** match retained first-party API revision bytes.
All **200 selected field spans** and **208 supporting-span occurrences** match.
Contributor chains finish across **six primary and one support responses**,
retaining **3,037 named article memberships and 494 anonymous article counts**.
These are article credits, not sentence authorship. Fixed URLs and complete
linked histories remain retained.

All 30 declared DAF8 original-work boundaries retain their separate collective
edition credit and conservative **2006-01-01** expiry. Three fixed policy/import
raw pages match API bytes; four authority materials and the complete licence
are retained. The unselected CFC coating in `couche` supplies no diaper field.
[Selected rights](selected-rights.md), [rights](rights.json),
[source context](source-context.json), [sources](sources.json),
[excerpts](source-excerpts.json), [captures](source-captures.json),
[reuse](reuse.json), [notices](NOTICES.md) and [COPYING](COPYING) keep source,
licence and modification boundaries distinct from quality acceptance.

## Checks and archives

The focused resource/admission seam checks exact membership/frequencies,
four-field/use alignment, fixed bytes/POS/IPA, complete contributor pagination,
rights/reviewer consistency, original proposals, exact replacement membership,
modern corroboration and the distinction from canonical admission.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_135.py
uv run --python 3.14.10 python -B -X utf8 target/issue-135-review/audit.py
pnpm test:docs
git diff --check
```

Focused tests pass: six tests. The source audit passes. Documentation/link and
diff checks pass, with exact commands/results in [verification](review-verification.json).
The focused check first detected the missing explicit French passive construction
and unaccepted review records; retained red/green logs identify that boundary.

Stable source/author archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-135`.
Independent source audit, modern raw captures, authority responses, helper scripts,
source copies and complete logs:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-evidence/issue-135/issue-135-review`.
Working audit root: `target/issue-135-review` in this review checkout.
Retained review patches describe the owned staged changes against merged
integration; verification records identify base, platform and evidence root.

## Remaining gates and evidence limits

Direct Légifrance retrieval was denied with HTTP 403; indexed first-party L123-3
text is retained separately. No successful raw statute capture is claimed.
Other original-edition and contemporary-use pages were captured successfully.

[Admission](admission.json) records **200 independently accepted fields and zero
new canonical fields**, with 200 pending coordinator admission. Shared registry,
canonical conversion/notices, matching native all-group lookup/trusted save and
restart/preservation are coordinator-owned and unrun here. Learner acceptance,
whole-machine offline, other-platform and release gates remain pending.
