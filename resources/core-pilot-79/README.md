# Lexique FR–EN–ZH D2 core

This development resource contains **450 independently reviewed senses / 1,800 fields**
in 450 frozen headword/POS groups, content version `core-89-v1`.
Its permanent target is **4,087 groups / 3,773 headwords**; **450 groups are accepted
and 3,637 remain unaccepted**. Literal everyday coverage is **448 complete groups
and 3,639 incomplete groups**. All **1,800 fields** have exact independent reviews.

| Batch | Frozen rows | Admitted groups | Fields | Reused groups | Annotation exceptions |
| --- | --- | ---: | ---: | ---: | ---: |
| #84 | 151–200 | 50 | 200 | 0 | 0 |
| #85 | 201–250 | 50 | 200 | 0 | 0 |
| #86 | 251–300 | 50 | 200 | 0 | 0 |
| #87 | 301–350 | 49 | 196 | 0 | 1 |
| #88 | 351–400 | 50 | 200 | 1 | 0 |
| #89 | 401–450 | 50 | 200 | 2 | 0 |

The prior **154 groups / 616 fields**, their sources, independent reviews, selected
contexts and eight spelling relations remain exact. Reused rows 360, 422 and 423
are counted once. The eleven reviewed spelling relations yield eight query aliases,
including the source forms `œil` for frozen `oeil` and `OK` for frozen `ok`.

The owner-approved specialist senses are row 34 `ça/NOM` (psychoanalytic id, #81)
and row 128 `par/NOM` (single-hole standard golf strokes, #83). Their original
restrictions, approvals and four-field records remain intact. Annotation exceptions
are row 87 `quoi/ADJ` (#82) and row 305 `dont/PRO:int` (#87, 2026-11-18). Both retain
their original frozen identities, four missing fields and zero admitted content.
The denominator remains unchanged. Batch #87 passes its amended scope with
49 admitted groups / 196 fields plus one annotation exception; relative `dont`
remains the independent everyday use at row 306.

[curated-inputs.json](curated-inputs.json) contains the complete FR/IPA/EN/ZH fields,
use restrictions, fixed versions and actual adaptations. [field-review.json](field-review.json)
records independent exact-text, rights, usage and owner decisions. Selected excerpts
are in [selected-source-excerpts.json](selected-source-excerpts.json), and applicable
import/bibliography notices, full histories and original author/version boundaries
are retained in [selected-source-context.json](selected-source-context.json).
Whole-page captures, unselected quotations and media are excluded.

Selected and adapted contributions use **CC BY-SA 4.0**; see [NOTICES.md](NOTICES.md)
and [COPYING](COPYING). The beau/NOM prose adapts Théodore Soulice's public-domain
1836 original, printed p. 56 / scan p. 62; the fixed Wikisource transcription is
revision 14772408, lines 142–143, ending before the separate beau idéal clause.
Wikisource transcription and Lexique adaptations retain their own CC BY-SA 4.0
credit and obligations. Original Soulice prose is not relabelled as CC-licensed.
Lexique membership credit is retained in [MEMBERSHIP-NOTICES.txt](MEMBERSHIP-NOTICES.txt),
[LEXIQUE-UPSTREAM-README.txt](LEXIQUE-UPSTREAM-README.txt) and
[lexique-provenance.json](lexique-provenance.json). Converter/application code is MIT.

Run `pnpm core:prepare` and `pnpm test:core` from the checkout. From a prepared
standalone directory, run `python core_pilot.py prepare rebuilt --source-dir .
--project-license LICENSE-MIT`. The prepared materials include complete source
inputs, selected excerpts, notices, licences, converter, checksums and coverage.
`coverage.json` reports missing content and annotations separately; `report.json`
reports each batch's actual content and scope acceptance.

The whole D2 release, learner acceptance and remaining platform/release gates are
incomplete. No CEFR labels are inferred. Native package acceptance is recorded
separately from content admission.
