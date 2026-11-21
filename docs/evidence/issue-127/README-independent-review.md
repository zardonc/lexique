# Issue #127 independent content and rights review

**50 exact frozen groups / 200 fields accepted by the independent reviewer.**
Rows 2301–2350 retain their exact headwords, Lexique POS subclasses, frequency
strings and order. **199 fields are unchanged; one English field is replaced.**
There are zero missing fields, pending independent decisions, reused groups,
new exceptions or runtime aliases. The global denominator remains 4,087 groups.

Reviewer: Independent semantic and selected-rights reviewer #127, separate
from the author and parent integration owner. This is an agent review; human
learner acceptance remains a separate gate. Author commit:
`1a2825fbe070384d45e39b91f8675c8a112c9053`, [PR #193](https://github.com/zardonc/lexique/pull/193).
Issue [#127](https://github.com/zardonc/lexique/issues/127) and parent
[#78](https://github.com/zardonc/lexique/issues/78) were read from integration’s
retained specification and issue capture.

| Evidence | Result |
| --- | --- |
| [groups.json](groups.json) | Exact 50 reviewed uses and four fields each |
| [field-review.json](field-review.json) | Separate independent field, use, spelling and selected-rights decisions with exact proposals and accepted texts |
| [rights.json](rights.json) | 200 passed selected-text rights decisions, historical layers and attribution obligations |
| [sources.json](sources.json), [source-context.json](source-context.json) | 49 passed selected source frames, complete registries, anonymous counts and both historical import layers for copie |
| [source-excerpts.json](source-excerpts.json), [source-captures.json](source-captures.json) | Unchanged exact field/frame spans and raw capture inventory |
| [admission.json](admission.json) | Live 50/200 acceptance, zero pending/missing independent fields; parent artifact/native gates remain pending |
| [review-candidates.json](review-candidates.json) | Original authored group objects |
| [admission-author-proposal.json](admission-author-proposal.json), [authored-review-proposals.json](authored-review-proposals.json) | Original admission and complete source/context/field-review/rights proposals |
| [selected-rights.md](selected-rights.md), [NOTICES.md](NOTICES.md), [COPYING](COPYING) | Selected-boundary rights, credits/modifications and complete CC BY-SA 4.0 text |
| [source-research.md](source-research.md) | All selected uses and fixed definition/IPA locations |
| [independent-review.md](independent-review.md) | Review result, exact replacement and reproducible verification handoff |

The English field for row 2322 `monseigneur/NOM` is now:
“An honorific used when addressing certain people of high birth or dignity,
especially bishops.” The apparent equivalent “Monsignor” is removed because
the selected French span establishes an address form, without establishing
that distinct specific ecclesiastical title. The original proposal remains in
the snapshots and per-field decisions.

Both `technique` POS groups retain their separate sections. `pub` is the
informal feminine advertisement /pyb/, `bite` retains vulgar register,
`au-delà` retains its place-adverb use and `associé` retains its masculine
noun and evidenced feminine form. `hop`’s aspirated-h restriction does not
require a pronounced /h/. No specialist exception is needed for the common
sand noun’s precisely scoped material definition.

## Verification

The [focused check](../../../scripts/test_core_batch_127.py) passes four tests
for exact membership, accepted content/rights, preserved proposals and identical
canonical overlap. The independent byte audit passes 49 fixed source/API
revisions, all 200 field and 200 rights spans, selected construction spans,
3,039 named per-article credits and nine responses across five terminated
contributor chains. No live source request or retry was made.

Reviewer logs and control scripts:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-127/issue-127-review/`.
Author captures:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-127/`.
Integration captures:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/issue-127/`.
The [independent review](independent-review.md) gives commands and log names.

## Remaining gates

Canonical assembly, combined package obligations, native lookup/save/reopen,
learner acceptance and offline/platform/release checks remain parent-owned and
unrun here. The established legal authorities were reused and read for this
selected-boundary review; no live statute refresh is claimed.
