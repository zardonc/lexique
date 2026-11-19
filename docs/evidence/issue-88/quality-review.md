# Issue #88: independent review pending

The candidate set contains **49 new senses / 196 proposed fields**, with
**row360 `aujourd'hui/ADV` and its four previously accepted fields reused
byte-identically**. All fifty frozen identities remain separate.

The new records have `independent_review: pending` and
`rights_eligibility_status: pending_selected_span_audit`.
`field-review.json` retains proposed text, exact fixed source spans and pending
usage decisions. `batch_88_review.reviewer` is null. No new field is independently
accepted. The actual converter rejects this draft at its review gate.

[review-candidates.json](review-candidates.json) is the reviewer input:
all four proposed texts per new sense, exclusions, source revisions/lines,
supporting source spans, eligibility state and the unchanged reused sense.
The complete owning evidence root is
`C:\Users\inthe\IdeaProjects\lexique-issue-88\target\issue-88`.
The tracked resource inputs and this evidence directory form the review diff
from `d55b6bdf497d310592dc67db88d7b2b09521c1b5`.

| Open decision | Exact scope |
| --- | --- |
| Meaning and construction accuracy | All 196 new FR/IPA/EN/ZH fields; English and Chinese are adaptations of the recorded French spans |
| Selected-span rights | All 41 selected source contexts, including 17 pages declaring DAF8 imports; current page credit alone does not resolve imported wording |
| Modern everyday POS eligibility | Row357 `ok/ADV`: fixed French source labels agreement use an interjection; an adverb use is unestablished |
| Spelling and grammatical binding | Lowercase `ok` versus source `OK`, pronoun `tous` versus its determiner, `quelques/ADJ:ind`, and `plein/PRE` constructions |
| Combined redistribution | Coordinator's complete notices, source package and admitted artifact |
| Native and learner acceptance | Coordinator's actual lookup/save/restart, learning-state preservation and subsequent learner feedback |

Other new senses are contemporary-use candidates awaiting acceptance. In
particular, row391 `lire/NOM` selects the currency of Turkey, distinct from
both the former Italian currency and `lire/VER`; row397 `putain/NOM` retains
its vulgar, pejorative prostitute sense rather than the separate interjection.
Source API/span checks and preservation tests are author verification; they
do not supply an independent semantic or rights verdict.
