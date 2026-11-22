# Issue #132 independent field and selected-rights review

**49 groups / 49 everyday selected uses / 196 fields independently accepted:
192 unchanged and four replacements.** No field/source/rights blocker remains.
The exact approved `oublie/NOM` annotation retains zero fields and four missing
fields. All 50 frozen rows 2551–2600 and the 4,087-group denominator are retained.

Reviewer: **Independent semantic and selected-rights reviewer #132; separate
from author and integration owner**. This is agent content/rights review;
learner acceptance is separate. Author commits are `a157c6c10b7da6f5cb485246e1016e92566009c9`
and `0c9d1762290856575f77f580154f651f26cfa610`. Review base is
`7e135023bff4ed3ad5c681be4200ec9b0c79398d`, checkout
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/review-132`, branch
`codex/d2-review-132`. The complete updated #132/#78 specs, reviewer guide,
pipeline notes, stable author archive and fresh oublie captures were read.
Changes are confined to this evidence directory and `scripts/test_core_batch_132.py`.

## Content decisions

The [field ledger](field-review.json) records each exact proposal, accepted text,
fixed source/span, quality reason, rights reason and reviewer. Original files
remain under [authored](authored/snapshot.json). [Replacement records](accepted-replacements.json)
retain exact text and spans for three vocal-rock restrictions and the Chinese
sentence explanation, allowing one or more clauses. Rock noun-1 describes a
common vocal configuration; noun-2's mythical bird and non-French homographs
are excluded.

`se soucier de` retains its pronoun and preposition; `jouir de` keeps its
indirect-transitive benefit use. Their IPA fields remain isolated infinitives.
`instructions` is usually plural and `fonctions` often plural. `quinze` and
`douze` retain invariant, preposed cardinal ADJ:num frames, excluding postposed
ordinals and numeral nouns. `cassé` is reviewed in its adjective section with
recorded /kɑ.se/; `tasse` selects recorded /tas/ while retaining /tɑs/ context.
`saurer` retains smoke-drying and prior salting, with no old-use label on its
selected culinary verb. `nazi/NOM` has a historical referent in ordinary current
discussion; the source does not label that word use obsolete. No new exception
is inferred from a domain label or an old referent.

Frozen 2560 `oublie/NOM` is excluded from accepted content only under the exact
[project-owner decision](owner-approval.md), dated **2026-11-21**, matching #132:
`批准仅 oublie/NOM 为零字段注记`. Fixed revision 39964053 labels the noun `vieilli`;
retained Le Robert and Académie ninth-edition captures corroborate `anciennement`
and `Vieilli`. No external dictionary text, verb substitute or query alias is admitted.

## Sources and selected rights

All **50 fixed articles**, **196 selected field spans** and **2,289 supporting
span occurrences** passed retained-capture consistency checks. Raw article
UTF-8 bytes equal first-party API revision content. Exact French POS/head/IPA
frames were checked separately from the language judgments. Seven contributor
responses complete one pagination chain, covering **3,113 named article
contributor memberships** and **483 anonymous counts**; these are article
credits, not sentence authorship or unique people.

DAF8 originals retain Académie française eighth-edition credit, with the
conservative **2006-01-01** economic expiry from final 1935 publication and
current L123-3's seventy-year term. Referenced original Littré layers retain
Émile Littré (1801–1881), BnF edition evidence and **1952-01-01** expiry under
L123-1. Current Wiktionnaire transcription/adaptation and Lexique explanations
remain CC BY-SA 4.0. Source references do not license modern external prose.
The unselected Import:CFC definitions at facture:47 and douze:184 supply no
accepted field or construction. Rock's image directive/caption remains raw
article-text evidence; no image file is selected. Rights-authority bytes were
retained with their original capture dates; policy API/raw comparison records
CRLF normalization separately from literal article byte equality.

Author source/authority archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-132/`.
Independent audit, selected projections, checks, retained source/authority copies,
specs and approval captures:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-evidence/issue-132/issue-132-review/`.

## Verification and remaining gates

The focused resource/admission check verifies exact membership, four-field/use
alignment, proposals/replacements, accepted source/rights consistency, exact
owner approval and real retained sources. No retained-source skip is permitted.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_132.py
uv run --python 3.14.10 python -B -X utf8 target/issue-132-review/audit.py
pnpm test:docs
git diff --check
```

Execution statuses and the integration merge are recorded in
[review checks](review-checks.json). Canonical assembly/conversion, global
coverage/size, combined packaged obligations, native lookup/save/reopen,
learning-data preservation, learner acceptance and offline/platform/release
verification remain coordinator-owned and unrun by this reviewer.
