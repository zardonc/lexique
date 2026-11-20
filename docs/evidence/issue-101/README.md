# Issue #101 reviewed content handoff

**49 groups / 196 FR/IPA/EN/ZH fields independently accepted.** All 50 frozen rows 1001–1050, exact spellings/POS subclasses and frequencies are retained. Row **1005 `m/PRO:per`** has four missing fields; no substitute, runtime alias or annotation exception is admitted. The denominator remains 4,087 groups / 3,773 headwords.

The seven canonical files are [groups](groups.json), [sources](sources.json), [field review](field-review.json), [source context](source-context.json), [source excerpts](source-excerpts.json), [reuse](reuse.json) and [admission](admission.json). [Independent review](independent-review.md) records exact source-quality judgments and the unresolved row; [rights](rights.json) records combined obligations. [Original candidates](review-candidates.json) remain unchanged, and [fixed captures](source-captures.json) locate the owning raw evidence. [Source research](source-research.md) retains acquisition and source boundaries.

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-101`; author head `da07f57e1c6236fa535512be3af2a700077bc359`, review diff base `d9f8e985a83512c1292255fc6d97f6605628fc6c`. Full Windows logs/captures: `target/issue-101/`.

The [focused check](../../../scripts/test_core_batch_101.py) verifies all frozen groups, the explicit gap, exact field/provenance joins and preserved candidates with only the three recorded review corrections:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_101.py -v
```

Both checks pass. Parent-owned canonical conversion, assembled notices and native lookup/trusted save/reopen remain separate gates. No application/native suite was run; four missing fields keep this batch incomplete.
