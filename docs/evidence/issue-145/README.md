# Issue #145 author package: rows 3201–3250

**50 exact groups, 50 selected everyday uses and 200 authored FR/IPA/EN/ZH
fields**, pending separate independent quality and selected-span rights review.
There are zero author-unresolved fields, zero reused admitted groups and zero
independently accepted or newly admitted fields. No exception, substitution,
POS merger or runtime alias is proposed. The frozen denominator remains
4,087 groups / 3,773 headwords; CEFR remains unknown.

[Ticket #145](https://github.com/zardonc/lexique/issues/145) and
[parent #78](https://github.com/zardonc/lexique/issues/78) define the exact scope
and approved resource/admission seam. [Readable candidates](review-candidates.md)
show every field and selected/excluded use. [groups.json](groups.json) retains
exact spellings, grammatical subclasses, row order and both frequency strings.

## Source and rights evidence

The package uses 51 real fixed French Wiktionnaire revisions: the 50 exact
headword pages and supporting `tel`. [source-captures.json](source-captures.json)
identifies retained raw/API paths, revision IDs, timestamps, request evidence
and complete contributor pagination. [source-excerpts.json](source-excerpts.json)
binds all 200 fields and supporting language/POS/construction spans;
[source-context.json](source-context.json) retains contributor credits,
anonymous counts, import notices and spelling relationships.

`telles` and `tels` remain ADJ:ind, supported by their exact French inflections
and `tel`'s explicit indefinite-adjective definition. `out` remains the French
invariable adjective meaning démodé, with French /awt/. `taré` remains a noun
in its familiar derogatory everyday use, excluding the obsolete sense.

[Selected rights](selected-rights.md), [rights.json](rights.json),
[NOTICES.md](NOTICES.md) and [COPYING](COPYING) preserve CC BY-SA 4.0
contribution/adaptation obligations and the separately credited DAF8 original
collective-work basis, expired 2006-01-01. The CFC cartographic `symbole`
definition is unselected. EN/ZH explanations are new Lexique wording based on
the scoped French sense. No literary/news quotation or media supplies a field.

[Original proposals](authored/groups.json),
[original field ledger](authored/field-review.json) and
[original rights proposals](authored/rights.json) retain the author wording and
pending statuses for separate review. Historical original-work evidence is
reused only at its original boundary; it accepts no new field.

## Checks and handoff

The focused [batch tests](../../../scripts/test_core_batch_145.py) exercise the
spec-approved resource/admission records. Their initial run found no canonical
content for these 50 rows. Author checks verify exact membership/frequencies,
four-field completeness, literal fixed API/raw span equality, actual French
POS frames, full contributor continuations, explicit rights and pending gates,
the three sensitive adjective identities, legal text and retained originals.
These checks do not establish independent language quality or admission.

[Verification records](author-verification.json) contain actual commands and
results. Raw captures, helpers, RED/GREEN logs, authorities and reproduction
evidence are archived at:

`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-145/`.

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/author-145`, branch
`codex/d2-content-145`. The initial baseline was
`ff8aae2bdaf36096c6600d5d29342745b97c0eb7`, canonical `core-139-v1`:
2,932 admitted groups / 11,728 fields and 18 zero-field annotations.
The integration merge and final tested base are recorded in the verification
ledger and archived Git logs. The author patch contains only this evidence
directory and its focused batch test.

## Pending gates

All 200 fields still lack independent quality and selected-span rights
acceptance. Construction/spelling review, canonical conversion, packaged
combined obligations and all-group native lookup/save/reopen are pending with
the coordinator. Learner acceptance, whole-core completion and release gates
remain open. Frontend, Rust and native checks were not run for this author-only
content package. Existing learner data and previously admitted content are
unchanged by the owned diff. No push or tracker change is made.
