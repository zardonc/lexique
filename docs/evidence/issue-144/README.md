# Issue #144: content author package

**50 exact groups / 200 proposed fields**, frozen rows **3151–3200**.
Every group has one selected contemporary sense with scoped French, applicable
IPA, English and Chinese. Exact spellings, grammatical subclasses, row order
and frequency strings are unchanged. CEFR remains unknown. No group is omitted
or annotated; no lookup alias is proposed.

[Readable proposals](review-candidates.md), [groups](groups.json),
[field and usage ledger](field-review.json), [source context](source-context.json),
[exact excerpts](source-excerpts.json), [rights](selected-rights.md),
[notices](NOTICES.md) and [admission status](admission.json) supply the reviewable
author result. [Original proposals](authored/groups.json) are preserved for
separate review. The batch uses 52 fixed first-party French article captures:
50 exact pages plus `la sienne` and `four électrique` support.

The author inspected the actual selected definitions, IPA, French/POS frames
and construction/register restrictions. `sienne/PRO:pos` retains its required
`la sienne` construction and feminine singular agreement. `ciao/ONO` is the
French farewell interjection. `business`, `vexer`, `enculer` and `pipi` retain
their informal, vulgar or childish labels. `souper/VER` retains the captured
Belgium/Canada/Switzerland/Occitania evening-meal restriction. `département`
selects the company unit, excluding an outdated territorial count. `go` selects
the board game. Pronunciation variants are selected explicitly, without
changing frozen spellings.

Raw API responses, request URLs, response headers, literal articles,
contributors, reused rights authorities and all author helpers are archived
outside the repository at
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-144/`.
[Capture metadata](source-captures.json) binds the archive to every selected
span. Two completed contributor chains retain all six main responses and one
support response, named registries, anonymous counts and history attribution.
All requests in this run succeeded; successful captures were retained.

## Checks

- Focused missing-resource check: expected failure at the approved
  resource/admission seam; retained `RED.log` under the task archive.
- `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_144.py`:
  passes exact frozen membership/frequencies, 200 complete fields, fixed raw/API
  equality and spans, grammatical frames, complete contributor continuation,
  applicable language/licence bindings and preserved pending author decisions.
- `reproduce.py` in the task archive: passes byte equality for all nine author
  JSON evidence files using retained fixed sources.
- Final documentation, diff and baseline resource checks are recorded in
  [final verification](final-verification.json).

## Remaining gates

**0 new independently accepted groups / 0 admitted fields.** All 200 fields
await independent quality and selected-span rights review. There are no
unresolved authored field values. Actual combined conversion/notices, native
all-group lookup/save/reopen and learner acceptance have not been run by this
author. Those integration/release gates remain pending; this package does not
close #144 or establish release readiness.

Canonical `core-139-v1` at author start has 2,932 admitted groups / 11,728 fields
and 18 zero-field annotations. No canonical input or saved learner data is
changed by this package. Reuse within this batch is zero. The denominator stays
**4,087 groups**; its 50 groups and 200 fields remain outside admitted coverage
until separate review and integration succeed.
