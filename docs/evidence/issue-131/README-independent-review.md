# Issue #131 independent field and rights review

**50 exact groups / 50 current everyday uses / 200 fields accepted.** All 200
field texts are unchanged. Four field-rights bases for `chérir` use the
independently verified Larousse original-work boundary. There are zero unresolved
quality or selected-rights fields and no new exceptions or runtime aliases.

Reviewer: **Independent field and selected-rights reviewer #131; separate from
original author and coordinator**. This records separate agent field/rights
review; learner acceptance remains a separate gate.

- Author commit: `f81207941dc340430f3a99a6a3d0825a4215ff16`.
- Review base: `7e13502`; branch `codex/d2-review-131`.
- Checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/review-131`.
- Author archive: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-131`.
- Independent audit/archive: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-evidence/issue-131/independent-review-7e13502`.
- Original work and proposals: [authored snapshot](authored/snapshot.json),
  [exact field decisions](field-review.json) and [rights replacements](accepted-replacements.json).

The reviewer guide at `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-guide.md`,
project rules, [coding standards](../../../CODING_STANDARDS.md), exact ticket and
parent specification, glossary, resource ADRs, pipeline notes and the #129
independently reviewed shape supplied the review boundaries.

## Accepted meaning and construction boundaries

All 200 actual FR/IPA/EN/ZH texts and all 50 selected uses were read against the
retained fixed source bytes. [Per-field reasons](field-review.json) distinguish
French semantics, applicable headword pronunciation, English and Chinese alignment,
everyday use, construction and spelling evidence. Scripts verify the ledger
and source bindings; their output does not supply the linguistic decisions.

`arabe` is the lowercase language noun. `critique`, `objectif` and `pratique`
retain noun frames; `pourri` retains its physical-decay adjective frame, distinct
from intransitive `pourrir`. The ordinary wrestling, cancer, ownership,
cultivation, pump and monarchy senses require no specialist exception.

`bye` and `bah` use French interjection frames and recorded `/baj/` and
`/baː/`. `amen` uses the French interjection frame in English Wiktionary,
its linked prayer-ending equivalent and the same French pronunciation; French
Wiktionnaire's different POS classification remains in the raw evidence.
`nôtres/PRO:pos` requires `les nôtres`, with plural masculine/feminine agreement;
`/notʁ/` is the isolated headword component of the attested construction.

Person-object `déshabiller` and `chérir`, physical-insertion `enfoncer`, rejection
`rejeter` and completion `achever` retain transitive restrictions.
`parvenir à` retains an intended endpoint, effort/difficulty and indirect
complement. `divorce` retains judicial dissolution; `pizza` retains the selected
tomato-sauce preparation. `sous-titre` includes same-language accessibility text.
`neveu` retains both sibling branches as `侄子、外甥`; `danseur` is not
restricted to professional dancers.

## Source and selected-rights audit

All **53** retained source revisions have literal UTF-8 equality with their API
revision content. **50** language/POS frames, **2,106** selected/supporting span
occurrences and **53** complete contributor pagination chains pass the audit.
Registries contain **3,484** named article-contributor memberships and **610**
anonymous article counts; these are article credits rather than unique people
or sentence-level authorship. Four fixed policy/import-template revisions match
their retained API text, with CRLF normalization recorded separately.

DAF8 original collective-work rights use final publication in 1935 and
**2006-01-01** expiry. `chérir` declares both DAF8 and Larousse imports.
The [BnF catalogue](https://catalogue.bnf.fr/ark:/12148/cb33995829b) records
**1866–1890**, seventeen volumes including supplements. The conservative
whole-work L123-3 calculation uses **1961-01-01**, covering the later endpoint
without assigning the sentence to a particular volume or author. The publisher's
main-volume/supplement chronology is independently retained in
[original-work review](original-work-review.json).

Only lexical definitions, phonetic notation and necessary language/grammar facts
are selected. `achever`'s Godefroy import lies in unselected Old French.
`chérir`'s Rivarol commentary and thing-object extension are unselected.
`parviendrez à` retains a short common grammatical combination, with no expressive
GEOguide Corse text selected. Contemporary bibliographic references,
literary/news quotations, externally attributed translations and media are
unselected. CC BY-SA 4.0, article/history and original-work credits, legal text,
modification notices, ShareAlike and no additional restrictions apply.

## Verification

Windows x64, PowerShell 7, Python through uv; approved resource/admission seam.

```powershell
uv run --no-project python -B -X utf8 scripts/test_core_batch_131.py
uv run --no-project python -B -X utf8 target/issue-131-review/audit.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --check
```

Five focused batch tests passed: frozen membership, exact accepted/proposed
field/use/rights agreement, literal fixed bytes and complete contributors,
original-work basis and separate decisions. The source audit passed. Retained
evidence includes `review-red.log`, `green.log`, `source-audit.json`, exact source
inspection captures, authority bytes, `decisions.json` and final verification
metadata. Archive fallback, integration and documentation results accompany
the final verification record.

## Limitations

Canonical conversion, combined package notices, all-group native execution,
save/reopen, learning-data preservation, offline/platform checks, learner
acceptance and release readiness remain coordinator-owned. The live statute
request returned HTTP 403; the fixed copyright-policy L123-3 quotation and
established original-work record supply the stated basis.
