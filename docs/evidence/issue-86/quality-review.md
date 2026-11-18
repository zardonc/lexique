# Issue #86 independent quality review: pending

**Reviewer: unassigned. Accepted groups: 0. Accepted fields: 0.**
The batch supplies 50 sourced selected-use candidates and 200 draft fields for a
fresh reviewer allocated after coordinator integration. No author check is
labelled independent and no pending record is marked passed.

## Exact candidates

The [curation](../../../resources/core-pilot-79/curated-inputs.json) contains the
exact candidate fields and usage restrictions for frozen rows 251–300.
Their stable sense IDs are `b86-251-ver` through `b86-300-nom`, with the exact
Lexique subclass encoded in each ID. Each group has one sense and four fields.

The matching [field-review records](../../../resources/core-pilot-79/field-review.json)
retain each exact `proposed_text`, source/version/span, headword and POS, with
`accepted_text: null`, pending quality/rights status and `reviewer: null`.
Usage records similarly retain `proposed_usage`, null accepted usage, exclusions
and supporting source evidence. The `batch_86_review` section reports the actual
pending status separately from the pre-existing independent reviews.

The [fixed source table](source-table.md) and
[research notes](source-research.md) provide the versioned evidence for these
specific selections. Automated span checks establish that the proposed citations
match the captures; they do not establish semantic accuracy, everyday eligibility,
licensing acceptance or learner quality.

## Required decisions

All 200 fields require an independent check of French meaning, applicable IPA,
English and Chinese alignment, construction/register restrictions, source
authorship and rights, followed by exact accepted text or a concrete unresolved
field reason.

Two groups carry explicit unresolved source-binding records:

| Row | Frozen group | Decision needed |
| ---: | --- | --- |
| 270 | `ouais/ADV` | Establish an eligible exact adverb use/source or resolve the interjection-to-frozen-ADV taxonomy correspondence. Both captured dictionaries label this selected use an interjection. |
| 274 | `vivre/NOM` | Decide whether the explicitly evidenced modern plural-only `vivres` food-supplies use satisfies frozen noun `vivre`; the singular food use is dated. |

These eight fields remain incomplete for admission. The original headword/POS
groups are retained, and no scope exception or ownership approval is assumed.

The review must also verify the article-bound possessive `le leur`, the
`quelle`/`quel` interrogative flexion, `cet`/`ce` vowel-sound condition, plural-only
`gens`, ONO interjections, noun/adjective distinctions and `/fis/` versus
`/fil/`. Those have positive fixed source evidence and remain pending review,
rather than being silently treated as accepted.

## Application and release boundary

Pending-source rejection by the existing converter is expected. There is no
admitted resource or per-ticket native result to audit yet. The coordinator owns
combined metadata and ROWS, shared ADJ:int/PRO:pos validators and localized
labels, native lookup of all accepted batch groups, representative trusted
save/reopen, preservation checks, final source/quality review and release gates.
