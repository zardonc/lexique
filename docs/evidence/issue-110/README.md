# Issue #110 authored content

All **50 frozen rows 1451–1500** have one complete authored selected sense with
French, IPA, English and Chinese content: **50 senses / 200 fields**, with
**no unresolved exact groups or unauthored fields**. Headwords, POS subclasses,
frequencies and order match the fixed manifest. Independent acceptance is
**0 groups / 0 fields**; the 200 authored fields await the separate reviewer.
The global denominator remains **4,087 groups**.

The [ticket](https://github.com/zardonc/lexique/issues/110) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the scope.
[Source research](source-research.md) records the exact selected uses and
construction boundaries. [Rights](rights.json) records the author assessment,
attribution and redistribution obligations. [Independent review](independent-review.md)
has no reviewer or acceptance evidence yet.

## Handoff files

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen groups and 200 complete authored fields |
| [sources.json](sources.json) | 51 fixed French sources, history credit, licences and modifications |
| [field-review.json](field-review.json) | Proposed exact texts, 200 source bindings and pending independent judgments |
| [source-context.json](source-context.json) | French/POS/form frames, construction evidence, full contributor registries and original-work boundaries |
| [source-excerpts.json](source-excerpts.json) | Exact raw one-based lines and selected IPA positions |
| [source-captures.json](source-captures.json) | API/wikitext capture paths and complete contributor pagination |
| [rights.json](rights.json) | Selected-source author assessments, captured authorities and material obligations |
| [reuse.json](reuse.json) | No previously admitted exact group reused; earlier canonical content remains unchanged |
| [admission.json](admission.json) | Author counts, baseline counts and pending review/conversion/native gates |

## Author verification

The [focused resource check](../../../scripts/test_core_batch_110.py) uses the
parent's approved resource boundary. It checks exact frozen membership and
frequencies, four nonempty fields per sense, raw source bindings, contributor
and rights records, and honest pending independent states. Raw captures are
checked when available in the author checkout; tracked bindings remain usable
in a detached checkout. Automated binding checks are not semantic or rights
acceptance.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-110/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-110/complete_handoff.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_110.py
pnpm test:docs
git diff --cached --check
```

The focused check recorded RED on the absent group package, then GREEN with the
authored fields. The rights slice recorded RED on the absent rights record, then
GREEN with complete author assessments. Both tests pass. Retained logs:
[RED groups](../../../target/issue-110/red.log),
[RED rights](../../../target/issue-110/red-rights.log),
[source capture](../../../target/issue-110/fetch.log),
[contributors](../../../target/issue-110/contributors.log),
[preparation](../../../target/issue-110/prepare.log),
[rights completion](../../../target/issue-110/handoff.log) and
[GREEN](../../../target/issue-110/green.log). Documentation and diff results are
retained in the same evidence directory.

`pnpm test:docs` passed for 277 tracked documents. The batch-only check passed
for its three Markdown files and 29 repository links; staged diff checks passed.

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-110`, branch
`codex/d2-content-110`. Its clean initial HEAD and integration merge base were
`fa4ea3eab1558fbede04b50cf0cd15f05bdf26dc`. The integration branch is
`codex/d2-content-110-119`; its current tip is merged before handoff. Preparation,
complete raw captures and execution logs remain in
`C:/Users/inthe/IdeaProjects/lexique-batch-110/target/issue-110/`.

## Remaining gates

Independent semantic, selected-span rights and spelling/construction review
remain pending for every proposed field. Combined conversion, notices packaging,
actual native lookup/save/reopen and final admission are parent-owned and unrun
here. The author reused the established DAF8 original-edition basis; the direct
statute endpoint returned HTTP 403. No live statute refresh is claimed.
Learner acceptance, whole-core completion and release/platform gates remain
separate. The unchanged baseline has **1,436 accepted groups / 5,744 fields**
and **2,651 groups without accepted content**, including the approved annotation
rows; this author package adds no accepted content to those totals.
