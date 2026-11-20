# Lexique FR–EN–ZH D2 core

This development resource contains **1,436 independently reviewed senses / 5,744 fields**
in 1,436 frozen headword/POS groups, content version `core-109-v1`.
Its permanent target is **4,087 groups / 3,773 headwords**; **1,436 groups are accepted
and 2,651 remain unaccepted**. Everyday coverage is **1,434 complete groups
and 2,653 incomplete groups**. All **5,744 fields** have exact independent reviews.

| Batch | Frozen rows | Admitted groups | Fields | Reused groups | Annotation exceptions |
| --- | --- | ---: | ---: | ---: | ---: |
| #84 | 151–200 | 50 | 200 | 0 | 0 |
| #85 | 201–250 | 50 | 200 | 0 | 0 |
| #86 | 251–300 | 50 | 200 | 0 | 0 |
| #87 | 301–350 | 49 | 196 | 0 | 1 |
| #88 | 351–400 | 50 | 200 | 1 | 0 |
| #89 | 401–450 | 50 | 200 | 2 | 0 |
| #90 | 451–500 | 50 | 200 | 1 | 0 |
| #91 | 501–550 | 50 | 200 | 0 | 0 |
| #92 | 551–600 | 50 | 200 | 0 | 0 |
| #93 | 601–650 | 50 | 200 | 0 | 0 |
| #94 | 651–700 | 50 | 200 | 0 | 0 |
| #95 | 701–750 | 50 | 200 | 0 | 0 |
| #96 | 751–800 | 50 | 200 | 0 | 0 |
| #97 | 801–850 | 48 | 192 | 0 | 2 |
| #98 | 851–900 | 48 | 192 | 0 | 2 |
| #99 | 901–950 | 49 | 196 | 0 | 1 |
| #100 | 951–1000 | 50 | 200 | 0 | 0 |
| #101 | 1001–1050 | 49 | 196 | 0 | 1 |
| #102 | 1051–1100 | 49 | 196 | 0 | 1 |
| #103 | 1101–1150 | 50 | 200 | 0 | 0 |
| #104 | 1151–1200 | 50 | 200 | 0 | 0 |
| #105 | 1201–1250 | 46 | 184 | 0 | 4 |
| #106 | 1251–1300 | 49 | 196 | 0 | 1 |
| #107 | 1301–1350 | 50 | 200 | 0 | 0 |
| #108 | 1351–1400 | 49 | 196 | 0 | 1 |
| #109 | 1401–1450 | 50 | 200 | 0 | 0 |

All prior **944 groups / 3,776 fields**, their raw records, source bindings,
independent reviews, contexts and compiled payloads remain exact. Row 483
`coeur/NOM` is reused unchanged in #90 and counted once globally; row 1911
remains admitted. The 15 reviewed spelling relations yield 11 query aliases.

The owner-approved specialist senses are row 34 `ça/NOM` (psychoanalytic id, #81)
and row 128 `par/NOM` (single-hole standard golf strokes, #83). Their original
restrictions, approvals and four-field records remain intact. Annotation exceptions are row 87 `quoi/ADJ`, row 305 `dont/PRO:int`, row 816 `t/PRO:per`, row 849 `c/PRO:dem`, row 859 `l/ART:def`, row 860 `l/PRO:per`, row 934 `i/ADJ:num`, row 1005 `m/PRO:per`, row 1051 `s/PRO:per`, row 1208 `quiconque/PRO:int`, row 1218 `qu/CON`, row 1219 `qu/PRO:int`, row 1220 `qu/PRO:rel`, row 1300 `d/PRE`, row 1386 `nulle/PRO:ind`. Each retains its exact frozen identity, frequencies, four missing fields and zero admitted content.
The 15 owner-approved annotations retain 60 missing fields in the fixed denominator.
The original seven approvals and two specialist senses remain intact. New approvals are recorded with their matching issue and date in field-review.json.
No new runtime alias is admitted; existing apostrophized records and relative `dont` at row 306 remain intact.

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
