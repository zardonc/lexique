# Issue #145 independent review

**50 exact groups / 50 modern everyday uses / 200 fields accepted.**
There are 189 unchanged field texts, 11 corrected texts, six narrowed source
spans, one corrected use and zero unresolved fields. Frozen rows 3201–3250,
spellings, POS subclasses, order and frequency strings are preserved.
The denominator remains 4,087 groups. No exception or runtime alias is admitted.

Reviewer: Independent semantic and selected-rights reviewer #145; separate
from author and parent integration owner. Checkout:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/review-145`, branch
`codex/d2-review-145`. Author:
`3a0dea3ce4292f82d3713a9516af36d8675a96e1`; fixed review base:
`521ce08421c0b90b458f550eb427eeabb1df4d39`.
[Issue #145](https://github.com/zardonc/lexique/issues/145),
[parent #78](https://github.com/zardonc/lexique/issues/78), project guidance and
the reviewer guide were read from the supplied checkout/context.

[Field review](field-review.json) contains actual separate per-field quality
and rights reasons, all 50 use decisions and all 50 spelling decisions.
[Accepted groups](groups.json), [rights](rights.json),
[source context](source-context.json) and [exact excerpts](source-excerpts.json)
agree on accepted text and sources. [Original proposals](authored/groups.json)
and [replacements](accepted-replacements.json) preserve exact before/after.

| Row | Accepted correction |
| ---: | --- |
| 3210 `terreur/NOM` | Three explanations use extreme fear; source narrowed to the conventional synonyms épouvante/crainte violente. |
| 3211 `légume/NOM` | Three explanations and use retain substantial food quantity, distinguishing spices/herbs used as seasoning. |
| 3228 `plaie/NOM` | Three explanations use clear bodily-tissue injury wording; source narrowed to Lésion du corps. |
| 3250 `autoroute/NOM` | French/English specify intersections at road level; Chinese already states 平面交叉. |

`mouton` retains generic sheep, corroborated by the modern Robert entry.
`telles` and `tels` retain exact inflections and indefinite-adjective semantics
with exact-form IPA. `fonds` retains its lexical final s. French `out` remains
the invariable borrowed adjective démodé, with recorded /awt/. `taré/NOM`
retains its familiar derogatory person sense supported by noun line 39 and
adjective line 21. `commissariat` retains the police-station ellipse, `certes`
concession, `conquérir` a person object and `déguiser` transitive/reflexive uses.

[Selected rights](selected-rights.md) assesses actual imported spans separately
from wiki CC BY-SA 4.0. France DAF8 expiry and article markers supply no worldwide
original-expression clearance. The full expressive `terreur` proposal is not
admitted. Conventional factual definitions, grammar and IPA have individual
assessments; attribution, modifications and ShareAlike remain applicable.

## Validation and evidence

Windows, PowerShell 7, Python 3.14.10 via uv. The approved boundary is this
reviewed resource/admission shard. Focused checks reproduce four defects before
correction and pass afterward. Checks cover 200 field spans, supporting records,
French POS/IPA frames and complete contributor continuations. Author JSON
snapshots remain byte-identical to the fixed base.

- `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_145.py`: passed, 10 tests.
- Source/API/authority and proposal-preservation audit: recorded in [reviewer verification](reviewer-verification.json).
- `pnpm test:docs` and `git diff --check`: recorded in [reviewer verification](reviewer-verification.json).

Source evidence:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-145/`.
Independent decisions, source audit, authority captures and RED/GREEN logs:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/reviewer-evidence/issue-145/`.
Integration merge identity and exact commands are in the verification record.

## Limitations

This is separate agent content/rights review. Learner acceptance, canonical
assembly, combined packaged obligations, conversion, native lookup/save/restart
and release validation remain coordinator-owned and unrun here. Frontend/Rust
checks are unrun because no application code changes. These accepted fields do
not close parent #78 or establish release completeness.
