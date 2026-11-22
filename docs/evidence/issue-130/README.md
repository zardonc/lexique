Current result: [final admission and native verification](README-admission.md). The preparation record below retains its original scope.

# Issue #130 reviewed content shard

**50 exact frozen groups / 50 selected uses / 200 fields independently reviewed.**
There are **199 unchanged fields, one Chinese correction and zero unresolved
selected fields**. Rows 2451–2500, original POS subclasses, frequency strings and
order remain unchanged. The denominator remains 4,087 groups / 3,773 headwords.

[Independent review](README-independent-review.md) records the separate reviewer,
author commit, review base, checks and evidence archive. [Groups](groups.json),
[field/use ledger](field-review.json), [exact replacement](accepted-replacements.json),
[sources](sources.json), [source context](source-context.json),
[excerpts](source-excerpts.json), [captures](source-captures.json),
[rights](rights.json), [selected rights](selected-rights.md),
[notices](NOTICES.md) and [COPYING](COPYING) hold the reviewed shard.
[Authored snapshot](authored/snapshot.json) identifies the preserved proposals.

The fixed source set comprises 50 actual Wiktionnaire article/redirect captures
and the licensed Lexique 3.83 dataset. Exact `la plupart des/ADJ:ind` uses
[the original phrase row and documented phonemic transcription](lexique-phrase-source.json),
not component IPA inferred by the reviewer. No annotation, specialist exception
or runtime alias is admitted.

## Verification and admission

`uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_130.py`
checks the reviewed shard, exact membership, every field/use decision, capture
bindings, contributors, selected rights and the replacement ledger. The optional
`--require-canonical` additionally checks assembly into the shared resource.
[Verification](verification.json) records actual results.

Independent selected-content acceptance is **50 groups / 200 fields**. Canonical
admission remains **zero new fields / 200 pending fields**. The coordinator's
merged converter supports Lexique `3.83` / `fixed_licensed_dataset_capture` and
its fixed dataset credit link. Actual assembly, conversion, combined notices,
native lookup/save/reopen, data preservation and learner/release gates remain
coordinator-owned. See [admission](admission.json).
