# Issue #115 author package

All **50 frozen groups, rows 1701–1750**, retain their exact spellings, POS
subclasses, frequencies and order. Each has one authored modern selected sense:
**50 senses / 200 FR/IPA/EN/ZH fields**, with **zero missing authored fields**.
There are **zero independently accepted new groups or fields**: all 200 fields
await the parent's fresh separate quality, spelling/construction and rights
review. No group is dropped, substituted or given a new exception. The fixed
denominator remains **4,087 groups**.

Scope: [#115](https://github.com/zardonc/lexique/issues/115),
[parent #78](https://github.com/zardonc/lexique/issues/78), and the unchanged
[scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Records

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact groups, complete authored fields, selected uses and exclusions |
| [review-candidates.json](review-candidates.json) | Same authored content for the separate reviewer |
| [field-review.json](field-review.json) | 200 texts/source bindings, 50 uses and spelling relations; reviewer null |
| [sources.json](sources.json) | 50 fixed Wiki revisions and one fixed DAF8 edition/page; contributor/history and original-work credits |
| [source-excerpts.json](source-excerpts.json) | Exact definition/IPA lines, IPA columns and grammar/construction support |
| [source-context.json](source-context.json) | Full contributor registries, original imports/references and nominal/spelling evidence |
| [source-captures.json](source-captures.json) | Real raw-source, authority and preparation paths |
| [rights.json](rights.json) | Author rights proposal, legal text, attribution/modifications and redistribution obligations |
| [reuse.json](reuse.json) | Zero assigned groups already admitted at the initial base |
| [admission.json](admission.json) | Author counts and pending review/conversion/native gates |

[Source research](source-research.md) explains the selected-source boundaries.
[Independent review status](independent-review.md) reserves judgments for the
parent's separate reviewer.

## Verification and handoff

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-115`, branch
`codex/d2-content-115`. Integration `codex/d2-content-110-119` was confirmed at
`e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451` and fast-forwarded before authoring.
Only this evidence directory and
[the focused test](../../../scripts/test_core_batch_115.py) are author-owned
tracked changes. The two captured APIs, full contributor responses, publication
pages, rights authorities and preparation scripts remain under
`target/issue-115/` in this checkout.

The parent's Testing Decisions approve the resource boundary. The check first
recorded the absent-batch [RED](../../../target/issue-115/red.log), then
[GREEN for all fields](../../../target/issue-115/fields-green.log).
The rights/construction slice recorded missing-rights
[RED](../../../target/issue-115/rights-red.log), followed by final
[GREEN](../../../target/issue-115/green.log): **2 tests passed**.
The checks verify membership, source-span bindings, contributor context,
complete rights records, pending independence, nominal `tien`, typographic
`oeuvre`, familiar feminine `com` and singular/plural `os` restrictions.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-115/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-115/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_115.py
git diff --cached --check
```

[Preparation](../../../target/issue-115/prepare.log),
[completion](../../../target/issue-115/finish.log),
[diff check](../../../target/issue-115/diff-check.log),
[local link check](../../../target/issue-115/docs-check.log),
[initial integration](../../../target/issue-115/integration-start.log),
[final integration](../../../target/issue-115/integration-end.log) and
[tested Git state](../../../target/issue-115/tested-state.json) are retained.
Authored fields total **15,709 characters / 19,523 UTF-8 bytes**; the longest is
**216 characters**. These measure selected text, not converted or release size.

## Pending parent gates

Separate semantic/quality and selected-rights review remains pending for all
50 groups / 200 fields. Modern construction corroboration is factual evidence,
not independent reviewer acceptance. The historical DAF8 edition-level legal
basis is reused from existing evidence; it was not refreshed independently here.

Conversion, combined packaged notices, native lookup/save/reopen, learner
acceptance and formal admission remain parent-owned and unrun here. All
200 fields remain missing from newly admitted batch coverage until those gates
pass. The shared canonical core and saved learning/audio data are unchanged.
