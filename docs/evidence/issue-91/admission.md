# Issue #91 content admission evidence

Rows 501–550 have 50 candidate selected uses and 200 nonempty fields with exact
frozen identities, frequency strings, source revisions and one-based spans.
All 200 fields and all 50 usage records are **pending independent review**.
No field is admitted by this worker; the fixed denominator remains 4,087 groups.
The prior shared resource records and its global metadata are unchanged.

The resource-boundary test is
[test_core_batch_91.py](../../../scripts/test_core_batch_91.py). It checks the
additions against the frozen manifest, exact batch membership, distinct POS groups,
four field names/languages, nonempty texts and source/excerpt bindings. It does not
claim an admitted SQLite artifact. The current converter requires independently
accepted fields, global row updates and final checksums; those are parent work.

| Check | Result | Retained log |
| --- | --- | --- |
| Focused test before additions | Expected failure: absent rows 501–550 | `target/issue-91/red-shard.log` |
| Focused test after additions | Passed, one test | `target/issue-91/green.log` |
| Fixed revision refetch | Passed: 44 French and 44 English pages match the first capture | `target/issue-91/fixed-audit.log` |
| Exact selected spans and author draft status | Passed; source lines match retained fixed API content | `target/issue-91/author-validation.log` |
| Prior resource preservation | Passed: shared tracked resource/pipeline files unchanged | `target/issue-91/preservation.log` |
| Diff and local documentation links | Passed for the scoped files | `target/issue-91/diff-check.log`, `target/issue-91/doc-links.log` |
| Independent quality/rights acceptance | Pending, separate central reviewer | [field-review.json](field-review.json) |
| Converter/native/offline/package checks | Not run by worker; parent owns combined verification | [admission.json](admission.json) |

Commands use PowerShell 7 and `uv run --python 3.14.10 python -X utf8`.
The focused command is `uv run --python 3.14.10 python -X utf8 scripts/test_core_batch_91.py`.
Fixed retrieval adapts the earlier `audit84_fetch.py` batch-revision pattern;
the retargeted helper and complete captures stay under the evidence root.
The recorded authored texts are proposed replacements for separate review, not
independently accepted wording. No scope exception is requested or recorded.

The author found a sourced everyday candidate for each exact group. The reviewer
must assess the explicit prepositional construction for `près/PRE`, the
determiner/pronoun distinction for `d'autres`, and the sense restrictions recorded
for the remaining homographs. A failed field or usage review keeps its group
incomplete; automated membership cannot resolve semantic or rights objections.
