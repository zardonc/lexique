# D2 #143 content author package

All frozen rows 3101–3150 retain their exact word/POS identities, frequency
strings and order. This package proposes 49 selected senses / 196 fields.
`pars/NOM` remains one unresolved group with four missing authored fields.
No new field has independent acceptance or canonical admission.

| Measure | Author package |
| --- | ---: |
| Frozen groups | 50 |
| Groups with four proposed fields | 49 |
| Proposed fields awaiting independent quality and rights review | 196 |
| Unresolved groups / missing authored fields | 1 / 4 |
| Independently accepted new groups / fields | 0 / 0 |
| Reused admitted groups / fields | 0 / 0 |
| Missing admitted fields in this batch | 200 |
| Owner-approved annotation or specialist exceptions in this batch | 0 |
| Fixed article captures / selected or construction-support sources | 54 / 51 |
| Global frozen denominator | 4,087 |

The author base is `ff8aae2bdaf36096c6600d5d29342745b97c0eb7`,
`core-139-v1`, 2,932 previously admitted groups / 11,728 fields. Canonical
resources and learner data are outside this package. Author proposals do not
increase admitted coverage. [Reuse](reuse.json) records the exact comparison base.

[Readable proposals](review-candidates.md) show every four-field proposal,
selected/excluded use and fixed lexical/IPA source. [Groups](groups.json),
[field review](field-review.json), [source excerpts](source-excerpts.json),
[source context](source-context.json), [rights](rights.json) and
[admission](admission.json) retain machine-readable author records.
Original proposals remain in [authored groups](authored/groups.json),
[authored field review](authored/field-review.json),
[authored rights](authored/rights.json) and [authored admission](authored/admission.json).
Every reviewer/status record identifies the author-stage work; reviewer is null.

## Selected boundaries and unresolved content

`pars/NOM` retains its exact identity and zero proposed fields. The exact noun
frame gives only the plural of `par` and leaves its IPA blank. The inspected
singular nouns are a golf term and a Georgian letter name; neither establishes
the required ordinary everyday sense. The row-128 singular golf approval does
not grant an exception to row 3128. [Unresolved evidence](unresolved-evidence.json)
retains the exact fixed spans, four blockers and absence of owner approval.
No verb form of `partir`, singular substitution or annotation is proposed.

`uns/PRO:ind` is limited to `les uns … les autres`. The fixed `l’un` pronoun
frame explicitly records `mp=les uns` and `pmp=le.z‿œ̃`; the fixed `les uns`
page corroborates the masculine plural pronoun frame. IPA /œ̃/ selects the
plural pronoun segment, excluding article and liaison. The separate partitive
`l’un de` sense requires `quelques-uns` in the plural and is excluded. The noun
frame of the exact `uns` page supplies no pronoun meaning. These source bindings
retain frozen `uns` and introduce no lookup alias or standalone use.

`chevalier` selects the explicitly contemporary title in an order and preserves
optional-schwa IPA notation. `mexicain/NOM` is lowercase Mexican Spanish, not
the capitalized inhabitant. `jusque/PRE` keeps its required construction and
isolated entry pronunciation. `are/NOM` is exactly 100 square metres.

## Captures, authors and rights

[Source captures](source-captures.json) identify real API responses, revisions,
timestamps, raw article paths, request URLs and all contributor continuation
parts. Named registries and returned anonymous counts cover all 54 articles.
Linked histories retain article credits; they do not prove individual sentence
authorship. All raw captures, rights captures, helpers and logs are archived
outside the repository at:

`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-143`

Capture helpers retain successful responses and honor HTTP retry headers with
exponential backoff. The successful article/contributor capture requests in
this run returned no HTTP 429. Fixed authorities retain their actual prior
capture metadata and reuse chain. [Selected rights](selected-rights.md),
[NOTICES](NOTICES.md) and [COPYING](COPYING) retain CC BY-SA 4.0 contribution and
adaptation obligations, complete source/history credit and separate DAF8
original-work attribution. Original collective-edition evidence from #129 is
kept separate from new field review. New EN/ZH are scoped Lexique explanations,
not upstream bilingual quotations. Project source code remains MIT.

## Verification boundary

The focused resource/admission check first failed because the exact 50-row
slice had no canonical content. Subsequent focused checks cover real selected
spans, same-POS French frames, complete contributor pagination, author-stage
rights/review states, construction restrictions and all four unresolved fields.
Author reproduction uses retained fixed captures and compares exact materials
without network access. Complete commands and logs are retained in the archive.

| Command | Actual result |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_143.py` | Passed, three focused resource/admission tests |
| `uv run --python 3.14.10 python -B -X utf8 ../context/author-evidence/issue-143/reproduce.py` | Passed, 14 author materials reproduced byte-for-byte without network |
| `pnpm test:docs` | Passed, no link or formatting problems; exact counts are in verification.json |
| `git diff --cached --check` | Passed |

[Verification record](verification.json) records the merged integration base,
actual author checks, source/field measurements and pending gates.

Independent quality/rights review, canonical conversion and actual combined
notices, every-group native lookup, trusted save/reopen, learner acceptance and
release gates remain pending with the coordinator. Frontend, Rust and native
checks are not run for this scoped author package; no application code changed.
This package does not complete ticket #143 or the full #78 core.
