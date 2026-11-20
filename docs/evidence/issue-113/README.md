# Issue #113 author package

All **50 frozen groups, rows 1601–1650**, retain their exact spelling, POS,
frequencies and order. Each has one complete authored selected sense:
**50 senses / 200 FR/IPA/EN/ZH fields**, with **zero missing authored fields**.
There are **zero independently accepted new fields**; all 200 fields await the
parent's separate quality and selected-rights review. No exact group remains
unresolved at the author boundary. The denominator stays **4,087 groups**.

Scope: [#113](https://github.com/zardonc/lexique/issues/113),
[parent #78](https://github.com/zardonc/lexique/issues/78), and the fixed
[scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Records

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact groups, complete authored fields, uses and exclusions |
| [review-candidates.json](review-candidates.json) | Author text supplied to the separate reviewer |
| [field-review.json](field-review.json) | 200 proposed texts, exact sources, 50 uses and spelling relations; reviewer null |
| [sources.json](sources.json) | 49 fixed French articles, contributor/history credit and original-work layers |
| [source-excerpts.json](source-excerpts.json) | Exact definition/IPA lines, selected IPA columns and grammar/construction support |
| [source-context.json](source-context.json) | Complete contributor registries, imports, bibliography and title-key corroboration |
| [source-captures.json](source-captures.json) | Raw API/wikitext, seven contributor parts, rights authorities and preparation paths |
| [rights.json](rights.json) | Author rights proposal, legal text, attribution/modifications and redistribution obligations |
| [reuse.json](reuse.json) | No assigned group already admitted at the base; existing records unchanged |
| [admission.json](admission.json) | Author counts, pending independent review and parent-owned conversion/native gates |

[Source research](source-research.md) explains source and construction boundaries.
[Review status](independent-review.md) explicitly reserves independent judgments
for the separate reviewer.

## Verification and handoff

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-113`, branch
`codex/d2-content-113`. Confirmed initial base and merged integration tip:
`fa4ea3eab1558fbede04b50cf0cd15f05bdf26dc`, integration branch
`codex/d2-content-110-119`. The integration merge reported already up to date.
Only this evidence directory and
[the focused test](../../../scripts/test_core_batch_113.py) are author-owned
tracked changes. Complete raw sources, preparation scripts and execution logs
remain under `target/issue-113/` in this checkout.

The parent Testing Decisions authorize the existing resource boundary. The
focused check recorded the missing-batch [RED](../../../target/issue-113/red.log),
then checked all exact groups, four-field bindings, contributor/source context,
pending independence, title spelling/construction and rights records.
Final [GREEN](../../../target/issue-113/green.log): **2 tests passed**.
The missing-rights [RED](../../../target/issue-113/rights-red.log) is retained too.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-113/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-113/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_113.py
git diff --cached --check
```

[Preparation](../../../target/issue-113/prepare.log),
[completion](../../../target/issue-113/finish.log),
[diff check](../../../target/issue-113/diff-check.log),
[local link check](../../../target/issue-113/docs-check.log),
[integration merge](../../../target/issue-113/merge-integration.log) and
[tested Git state](../../../target/issue-113/tested-state.json) are retained.
The source/field checks are author verification, not independent semantics or
rights review. Native lookup/save/reopen, conversion, combined notices, learner
acceptance and admission are parent-owned and unrun here. No shared core,
tracker, learning data or previously admitted record is changed.
