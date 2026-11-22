# Issue #133 independent field and rights review

**49 ordinary groups / 49 uses / 196 fields accepted unchanged.** There are no
wording replacements or unresolved ordinary content/rights blockers. Rows
2601–2650 retain all 50 exact identities, subclasses, frequencies and order.
Only row 2626 `r/ADV` is a zero-field annotation, with four missing fields and
the [owner's exact approval](owner-approval.md). No additional exception or runtime
alias is admitted; the denominator remains 4,087 groups.

Reviewer: **Independent field and selected-rights reviewer #133; separate from
author and integration owner**. This is an independent agent review; learner
acceptance remains separate. The [child](https://github.com/zardonc/lexique/issues/133)
and [parent](https://github.com/zardonc/lexique/issues/78) were read from retained
context JSON, including the 2026-11-21 owner amendment.

Checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/review-133`, branch
`codex/d2-review-133`. Author commit: `afca8fa7a4a09121ed77fb99e72cbbc67b4ceae0`.
Review base: `fdd2f3b1f057a9a5b7af05641502651253beb439`. Current integration was
merged before delivery; [delivery identity](delivery.json) records its exact tip.
Only `docs/evidence/issue-133/**` and `scripts/test_core_batch_133.py` are reviewer changes.
The retained `accepted-shard.patch` covers those paths from the integration tip.

[Field review](field-review.json) keeps every exact proposed/accepted text, source
span, specific semantic/language reason, separate rights reason and reviewer.
[Use decisions](groups-independent-review.json) and source-binding decisions have
their own reasons. [Authored snapshots](authored/snapshot.json) preserve original
records, including unresolved `r`. [Replacements](accepted-replacements.json) is empty.
Historical helpers and structural checks do not supply linguistic verdicts.

## Content and rights decisions

All French definition, IPA, English and Chinese fields were read against their
selected French/POS/use frames. `essentiel` selects the singular noun; `mercredi`
the weekday noun; `fermé` its lexical adjective. `fan` uses French admirer `/fan/`,
not a device or the `/fɑ̃/` interjection. `magazine` does not become `magasin`.
Gender counterparts and applicable source variants remain evidence, without aliases.

`salopard` and `garce` retain offensive register and referents. `pourvoir X de Y`
keeps transitive equipping and its evidenced passive. Food `cuire` keeps both
argument frames. Aircraft `décoller` and noise-producing `craquer` are intransitive
at their selected uses despite broader verb-head labels. `serment` retains a sacred
witness; `célibataire/ADJ` retains never-married status and legal marriageable age.
These are restricted contemporary uses, not new specialist exceptions.

`for/NOM` survives only in the expressly named `for intérieur` constructions;
the phrase entry supplies conscience semantics and generally-singular evidence.
Bare `/fɔʁ/` remains separate from connected phrase pronunciation. Literal source
`/i(m).mɑ̃s/` for `immense` and `/ʁə.lɑ.ʃe/` for `relâcher` remain applicable variants.
Every accepted group's four fields describe the same selected use.

The independent audit passes for **51 fixed articles**, **196 field spans** and
**399 total selected/supporting/annotation span occurrences**. Raw UTF-8 article
bytes match first-party API revision content and identity. **22 contributor responses**
form two completed chains with **3,148 named article memberships** and **541 anonymous
article counts**. These are credits, not selected-sentence authorship.

All 196 fields pass selected-span rights review. **39 source records** retain the
DAF8 original-work boundary: Académie française, eighth edition 1932–1935, with
the established L123-3 expiry **2006-01-01**. Dates, licences, source bytes and
wording need no correction. Modern contributions and Lexique adaptations/new EN/ZH
explanations retain CC BY-SA 4.0. Copyright-policy/template text, Terms section 7,
complete licence, full-history credits and modification notices were inspected.

`réserve` selects reserved things at line 26; the CFC cartographic import at line 88
is unselected and its rights are not claimed. `assiette`'s image/caption prefix is
excluded from accepted plate semantics. Examples, media, etymology, other languages
and linked external dictionary prose do not enter learner fields.
[Selected rights](selected-rights.md) records those boundaries.

## Evidence, verification and limitations

Original capture root:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-133/`.
Independent source/cache, owner-amendment captures, exact decisions and check archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-evidence/issue-133/issue-133-review/`.
Copies are byte-verified. Article bytes are compared literally; policy/template
text uses declared CRLF-to-LF normalization. Retained CNRTL/Robert letter/nominal
frames establish no eligible adverb or adverb IPA and supply no redistributed prose.

```powershell
uv run --no-project python -B -X utf8 scripts/test_core_batch_133.py
uv run --no-project python -B -X utf8 target/issue-133-review/audit.py
pnpm test:docs
git diff --cached --check
```

The focused annotation regression failed on the authored unresolved status and
passes on the exact owner-approved annotation. Five shard checks and the independent
source audit pass. Documentation/link and diff results are recorded in
[delivery identity](delivery.json) and the external archive.

The shard has 196 accepted fields, zero ordinary missing fields and four annotation
missing fields. Canonical assembly has zero newly installed fields and 196 pending
fields. Global annotation/converter changes, combined packaging, native lookup/save/
restart, offline execution, learner acceptance and release gates remain coordinator-owned
and unrun. Frontend/Rust checks are unrun; no application code changed. No tracker
operation was performed.
