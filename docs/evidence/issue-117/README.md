# Issue #117 author package

All **50 frozen groups, rows 1801–1850**, have one modern selected sense with
**200 authored FR/IPA/EN/ZH fields**, zero missing authored fields and exact
headword/POS/frequency/order preservation. Independent acceptance is
**0 new groups / 0 new fields**. The denominator remains **4,087 groups**;
all 200 batch fields remain missing from newly admitted coverage pending the
parent's separate reviewer and integration gates. No annotation exception or
runtime alias is proposed.

Scope: [#117](https://github.com/zardonc/lexique/issues/117),
[parent #78](https://github.com/zardonc/lexique/issues/78),
and the [fixed membership](../../../resources/core-pilot-79/scope-manifest.csv).

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities and complete selected fields |
| [review-candidates.json](review-candidates.json) | Unchanged author content for separate review |
| [field-review.json](field-review.json) | 200 pending field reviews, 50 selected uses and 50 binding-only spelling relations |
| [sources.json](sources.json) | 50 immutable revisions, authors, rights layers and modification records |
| [source-excerpts.json](source-excerpts.json) | Exact definition/IPA lines, IPA columns and construction/flexion spans |
| [source-context.json](source-context.json) | Full contributor names, original-work credits and source frames |
| [source-captures.json](source-captures.json) | Full raw API/source, contributor, authority and preparation paths |
| [rights.json](rights.json) | 200 selected-field rights proposals, legal text and redistribution obligations |
| [reuse.json](reuse.json) | No assigned group already admitted at the baseline |
| [admission.json](admission.json) | Author counts and pending parent gates |

[Source research](source-research.md) records the selected-use boundaries.
[Independent review status](independent-review.md) reserves judgment for the
parent's fresh separate reviewer.

## Author verification

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-117`, branch
`codex/d2-content-117`; initial integration baseline
`83c7381a3e70699790caccb5aea81858e5b47380` was confirmed and merged before work.
The owned changes are this directory and
[scripts/test_core_batch_117.py](../../../scripts/test_core_batch_117.py).
The complete evidence root is `C:/Users/inthe/IdeaProjects/lexique-batch-117/target/issue-117/`.

The preapproved resource boundary recorded missing-package
[RED](../../../target/issue-117/red.log), then complete-fields
[GREEN](../../../target/issue-117/fields-green.log).
The rights/construction slice recorded missing-rights
[RED](../../../target/issue-117/rights-red.log), followed by final
[GREEN](../../../target/issue-117/green.log): **2 tests passed**.
Membership, field/source/version/span binding, pending review, exact flexions,
rights records and retained raw construction spans are checked.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-117/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-117/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_117.py
pnpm test:docs
git diff --cached --check
```

Preparation reproduces from retained fixed captures without a live dictionary
request. The full [preparation](../../../target/issue-117/prepare.log),
[completion](../../../target/issue-117/finish.log),
[documentation](../../../target/issue-117/docs-check.log),
[diff](../../../target/issue-117/diff-check.log),
[initial integration](../../../target/issue-117/integration-initial.log),
[final integration](../../../target/issue-117/integration-final.log) and
[tested state](../../../target/issue-117/tested-state.json) logs remain local.
The full authored patch and separate tests patch are retained as
`target/issue-117/authored-package.patch` and `target/issue-117/tests.patch`.

Selected text totals **15,250 characters / 19,009 UTF-8 bytes**, with a longest
field of **200 characters**. These are authored-text measurements.

The documentation check passed for all 295 tracked documents with no problems;
the staged diff check passed. Final post-merge results and the exact tested
head/integration SHAs are retained in the tested-state record.

## Pending parent gates

All 50 groups / 200 fields await separate semantic, selected-rights,
construction and spelling review. The existing DAF8 collective-work legal
basis is reused from #99; it has not received an independent refresh here.
Page import markers do not establish the authorship of a selected sentence.

Admission, reproducible canonical conversion, combined packaged obligations,
actual native lookup/save/reopen and learner acceptance are parent-owned and
unrun here. Frontend, Rust, platform and release checks were not run for this
author package. Saved learner data and the shared canonical resource retain
their existing owners.
