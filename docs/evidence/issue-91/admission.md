# Issue #91 content admission evidence

Rows 501–550 have **50 independently accepted selected uses / 200 fields** with
exact frozen identities, frequency strings, source revisions and one-based spans.
[Independent review](independent-review.md), [admission](admission.json) and the
canonical [field review](field-review.json) record the final accepted wording,
source boundaries and rights. The fixed denominator remains 4,087 groups.

The author-stage resource-boundary test is
[test_core_batch_91.py](../../../scripts/test_core_batch_91.py). It checks the
additions against the frozen manifest, exact batch membership, distinct POS groups,
four field names/languages, nonempty texts and source/excerpt bindings. It does not
claim an admitted SQLite artifact. The accepted records are now included in the
[combined core](../../../resources/core-pilot-79/README.md); combined verification
belongs to the parent [aggregate evidence](../issue-90-99/README.md).

## Retained author checks

These results describe the original candidate handoff, before independent review.
Full captures and logs are preserved under integration's
`target/implement-90-99/worker-audit/author-91/`.

| Check | Result | Retained log |
| --- | --- | --- |
| Focused test before additions | Expected failure: absent rows 501–550 | `target/issue-91/red-shard.log` |
| Focused test after additions | Passed, one test | `target/issue-91/green.log` |
| Fixed revision refetch | Passed: 44 French and 44 English pages match the first capture | `target/issue-91/fixed-audit.log` |
| Exact selected spans and author draft status | Passed; source lines match retained fixed API content | `target/issue-91/author-validation.log` |
| Prior resource preservation | Passed: shared tracked resource/pipeline files unchanged | `target/issue-91/preservation.log` |
| Diff and local documentation links | Passed for the scoped files | `target/issue-91/diff-check.log`, `target/issue-91/doc-links.log` |
| Independent quality/rights acceptance | Passed for all 50 uses / 200 fields | [independent-review.md](independent-review.md), [field-review.json](field-review.json) |
| Native lookup/save/reopen, offline and preservation checks | Pending at integration | [Aggregate verification](../issue-90-99/README.md) |

Commands use PowerShell 7 and `uv run --python 3.14.10 python -X utf8`.
The focused command is `uv run --python 3.14.10 python -X utf8 scripts/test_core_batch_91.py`.
Fixed retrieval adapts the earlier `audit84_fetch.py` batch-revision pattern;
the retargeted helper and complete captures stay under the evidence root.
The canonical records contain the independently accepted replacements.
No scope exception is recorded for this batch.

The accepted uses retain `près de` for `près/PRE`, the determiner/pronoun
distinction for `d'autres`, and the restrictions for the other homographs.
Learner acceptance and whole-core release gates remain open.
