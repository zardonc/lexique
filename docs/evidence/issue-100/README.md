# Issue #100 reviewed content handoff

**50 exact groups / 200 FR/IPA/EN/ZH fields accepted by a separate content/rights reviewer.** All frozen rows 951–1000, POS subclasses and frequencies remain present, with no missing fields, runtime aliases or annotation exceptions. The global denominator remains 4,087 groups / 3,773 headwords.

The seven canonical files are [groups](groups.json), [sources](sources.json), [field review](field-review.json), [source context](source-context.json), [source excerpts](source-excerpts.json), [reuse](reuse.json) and [admission](admission.json). [Independent review](independent-review.md) records exact judgments and limitations; [rights](rights.json) records combined obligations. [Original candidates](review-candidates.json) are unchanged. [Fixed captures](source-captures.json) locate the owning API/wikitext evidence. [Source research](source-research.md) records acquisition and original-layer boundaries.

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-100`; reviewed author head `c37db48246e3710c11891f5d9738f3122c234e17`, diff base `87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5`. Windows logs and captures remain under `target/issue-100/`.

The [focused check](../../../scripts/test_core_batch_100.py) verifies frozen membership, exact accepted text/provenance against the original candidates, independent bindings and original captures:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_100.py -v
```

All three checks pass in the owning worktree. Content/rights acceptance is separate from parent-owned canonical conversion, assembled notices and native lookup/trusted save/reopen. Application/native checks were not run.
