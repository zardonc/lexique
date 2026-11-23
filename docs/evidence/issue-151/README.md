# Issue #151 author package

Frozen rows **3501–3550** contain all **50 exact word/POS groups**, one selected
sense each and **200 authored fields**. The original frequency strings and
4,087-group / 3,773-headword denominator are preserved. CEFR remains unknown.
Independent quality, usage/spelling and selected-expression rights review is
pending: **zero newly admitted groups or fields**, **200 missing admitted fields**.

[Readable candidates](review-candidates.md) and [groups](groups.json) retain the
exact FR/IPA/EN/ZH proposals. [Source captures](source-captures.json) bind 52 fixed
French articles and nine complete contributor API parts to the external archive.
[Sources](sources.json), [context](source-context.json),
[excerpts](source-excerpts.json), [field review](field-review.json),
[rights](rights.json), [selected rights](selected-rights.md),
[original expression](original-expression-proposals.json) and
[modern-use corroboration](modern-usage-corroboration.json) support separate review.
[Admission](admission.json) and [reuse](reuse.json) retain the baseline and gates.
[Notices](NOTICES.md) and [legal text](COPYING) accompany the proposed content.

The exact author proposals are preserved in [authored](authored/groups.json).
Reusable helpers, API/raw capture bytes, authority origins and complete logs are
outside the repository at
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-151`.
The package reuses the established #149 capture/preparation/rights/notice helpers
and `scripts/core_batch_test_helpers.py`; it adds no resource pipeline or dependency.

The boundary test is
`uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_151.py`.
Actual author checks and commands are in [author verification](author-verification.json).
Canonical assembly, converter registration, package reproduction, native lookup,
save/reopen, learner acceptance and whole-core release gates remain with integration.
