# Issue #155 author content

**50 exact frozen groups and 200 sourced fields are prepared for independent
review.** Rows 3701–3750 retain their word/POS identities, order and both original
frequency strings. Every group has one proposed modern selected sense and
French definition, applicable IPA, English explanation and Chinese explanation.
No group is dropped, substituted, merged or exempted. CEFR remains unknown.

[Readable proposals](review-candidates.md), [groups](groups.json),
[field review](field-review.json), [source context](source-context.json),
[exact excerpts](source-excerpts.json), [rights](rights.json),
[selected rights](selected-rights.md) and [notices](NOTICES.md) retain the author
text and evidence for a separate reviewer.

## Content and source boundaries

The package uses 52 fixed French-Wiktionnaire captures: the 50 frozen spellings
plus `naan` and `cache`. The latter supplies the explicit lexical meaning behind
`cachette`. French `naan` lists `nan` as a spelling variant, supporting the
required bread noun with /nan/. The negative interjection remains excluded.

Selected uses keep their construction and register: informal `clope`, potentially
vulgar `chiant`, compelled adjective `obligé`, English-derived opportunity sense
of `opportunité`, singular-only language noun `portugais`, and plural restraint
sense `menottes` under the frozen headword `menotte`. Noun/adjective/verb-form
alternatives are kept distinct. `talon` uses the anatomical heel; the CFC
cartographic wording is excluded.

[Source captures](source-captures.json) point to retained raw/API revisions,
request URLs, complete paginated contributor records, authority captures and
the adapted author helpers outside the repository:
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-155`.
The records credit linked complete histories, named contributors, returned
anonymous counts and Lexique adaptations. A current registry does not establish
selected-sentence authorship. DAF8/Littré original-work evidence is separated
from contributor and adaptation rights.

## Verification

The approved resource/admission seam is tested by
[test_core_batch_155.py](../../../scripts/test_core_batch_155.py), reusing
`scripts/core_batch_test_helpers.py` for capture verification. The archived
checks record red before content and rights, then validate the complete author
package, exact source bytes and selected spans. Full commands and actual final
results are recorded in [verification.json](verification.json).

The baseline is `03a27e42c261afda7ffd4a927a4d2810f8221e8a`, `core-149-v1`, with
3,431 admitted groups and none of these 50 groups already admitted. The fixed
global denominator remains 4,087. [Reuse](reuse.json) preserves that comparison.

## Pending acceptance

Author checks accept **zero** new groups or fields. All 200 fields remain pending
independent quality and selected-span rights review; learner acceptance is not
run. [Admission](admission.json) records these counts separately from complete
authored fields. Canonical resource assembly, combined notices, conversion,
matching Windows native lookup/save/reopen, and remaining release acceptance
are owned by integration/review and have not been run for this author package.
