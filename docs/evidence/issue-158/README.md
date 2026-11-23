# #158 author content shard

Rows 3851–3900 contain **50 exact frozen groups / 200 proposed fields**, with
one scoped modern-use candidate per group. Independent quality, selected-expression
rights and construction/spelling review remain pending. There are zero independently
accepted or canonically admitted new fields.

[Readable proposals](review-candidates.md) retain every French definition, IPA,
English explanation and Chinese explanation. [groups.json](groups.json) preserves
the frozen identities, row order and both frequency strings; CEFR stays unknown.
[field-review.json](field-review.json) binds each field/use to its fixed sources
with null reviewers. [admission.json](admission.json) distinguishes authored
completeness from missing admitted fields.

[Selected spans](source-excerpts.json), [source context](source-context.json),
[capture pointers](source-captures.json), [modern/grammar corroboration](modern-corroboration.json),
[rights proposals](rights.json), [selected rights](selected-rights.md),
[notices](NOTICES.md) and [legal text](COPYING) accompany the shard.
All 53 fixed Wiktionnaire pages have complete contributor pagination and linked
history attribution. Contributor registries do not prove sentence authorship.

`g/NOM` selects the masculine invariable acceleration-unit noun `/ʒe/`, with
modern recreational passenger usage corroborated by Europa-Park. The gram
symbol is excluded. `cm/NOM` proposes the nominal `centimètre` reading of the
measurement symbol. Its source symbol classification is explicit; OQLF confirms
the masculine unit name, `cm` binding and symbol rules. Independent review must
resolve that construction/classification fit. `peso` retains its attested
traditional relationship to `péso`; `boeuf` retains its typographic relationship
to `bœuf`. `aimé` uses the lexical adjective; `condoléance` retains usual plural
`condoléances`. No group is dropped, substituted or exempted; no alias is added.
The CFC cartographic senses in `transfert` are excluded.

## Verification and handoff

The approved resource/admission check is
[scripts/test_core_batch_158.py](../../../scripts/test_core_batch_158.py).
It checks exact membership/frequency, four-field completeness, fixed API/raw/span
and IPA bindings, contributor pagination, notices, explicit pending reviews and
rejection of an author review by the existing admission loader.
[Author verification](author-verification.json) records actual commands/results.
[Reuse](reuse.json) identifies the integration baseline.

Reusable helpers, raw captures, requests, legal authorities and complete logs
are retained at
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-158`.
The archive's helper-origins.json and delivery-pointers.json identify preparation,
selection, reproduction and execution evidence. Temporary logs stay in the
worktree's ignored target/issue-158 directory.

## Limitations

All 200 new fields still require independent quality and selected-expression
rights review, including actual original-expression comparisons. The `g` everyday
eligibility and `cm` nominal symbol reading require independent decisions. The
fixed baseline is core-149-v1 with 3,431 admitted groups / 13,724 fields and
656 unaccepted groups against the unchanged 4,087-group denominator. This shard
changes no global admission counters or learner data. Canonical assembly,
conversion/packaging, native lookup/save/reopen, upgrade preservation and learner
acceptance are coordinator-owned and unrun here. Frontend and Rust checks are
unrun because this change contains only batch evidence and its resource test.
