# #149 author content package

Rows 3401–3450 contain 50 exact frozen word/POS groups, 50 selected modern uses
and 200 proposed French-definition, IPA, English and Chinese fields. The
membership, row order and both frequency strings match the unchanged
4,087-group manifest. All 53 fixed French sources, including `le sien`,
`massacrer` and `lesbianisme` support, have complete contributor captures.

[Readable proposals](review-candidates.md) and [groups.json](groups.json) retain
the precise meanings, constructions, registers, pronunciations and excluded
uses. [Selected source spans](source-excerpts.json),
[source context](source-context.json), [capture paths](source-captures.json),
[rights proposals](rights.json), [selected rights](selected-rights.md) and
[notices](NOTICES.md) provide the source and modification evidence. Original
author proposals are preserved in [authored/groups.json](authored/groups.json)
and the adjoining original ledgers for separate review.

The scoped resource test is [test_core_batch_149.py](../../../scripts/test_core_batch_149.py).
The initial check failed because canonical content lacked all 50 groups;
subsequent focused checks cover complete membership, actual source bytes and
POS spans, complete contributor continuations, proposed rights and preserved
original proposals. Commands, results and archive locations are recorded in
[author-verification.json](author-verification.json).

## Admission and remaining gates

[admission.json](admission.json) records zero independently accepted new groups
and fields, 200 fields pending independent quality review, 200 pending
selected-span rights review and 200 missing admitted fields. There are no
author-unresolved content rows; the proposed meanings and restrictions still
require the separate review. The inherited canonical baseline is `core-139-v1`,
2,932 groups and 11,728 fields. No #149 canonical content or learner data is
changed by this package.

Independent language, modern-use, spelling/construction and selected-rights
review; combined conversion and redistribution materials; actual native lookup,
save and reopen; learner acceptance and wider release gates remain pending.
Frontend/Rust/browser checks are unrun because this package changes only the
owned evidence and Python resource test. The global denominator remains 4,087;
candidate completeness is separate from admitted coverage.
