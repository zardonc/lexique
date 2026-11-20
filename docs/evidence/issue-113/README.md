# Issue #113 independent content and selected-rights review

**50 groups / 200 FR/IPA/EN/ZH fields accepted; zero unresolved fields.** Frozen
rows 1601–1650 retain exact spelling, POS, frequency strings and order. The
4,087-group denominator remains. There are 195 unchanged texts and five exact
reviewed replacements. No runtime alias or owner exception is added.

Scope: [#113](https://github.com/zardonc/lexique/issues/113),
[parent #78](https://github.com/zardonc/lexique/issues/78), and the
[scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Records

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact reviewed groups, fields, uses and exclusions |
| [review-candidates.json](review-candidates.json) | Unchanged original author proposals and pending draft statuses |
| [field-review.json](field-review.json) | 200 proposed/accepted texts with reasons; 50 uses and 50 source-binding relations |
| [sources.json](sources.json) | 49 fixed articles and attribution/original-work layers |
| [source-excerpts.json](source-excerpts.json) | Exact field spans and construction/component support |
| [source-context.json](source-context.json) | Full contributor registries, selected context, original rights and spelling bindings |
| [source-captures.json](source-captures.json) | Author captures and independent fixed-revision comparison reference |
| [source-revision-review.json](source-revision-review.json) | All 49 refetched revisions and complete metadata/text comparison |
| [rights.json](rights.json) | Per-source selected-rights decisions, complete contributors and redistribution obligations |
| [reuse.json](reuse.json) | No reused assigned group |
| [admission.json](admission.json) | Independent accepted counts; separate parent conversion/notices/native gates |

[Independent report](independent-review.md) records decisions and validation.
[Source research](source-research.md) records source and construction boundaries.

## Review and handoff

Pinned author commit: `e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451`.
Author captures and commands remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-113/target/issue-113/`.
Reviewer checkout: `C:/Users/inthe/IdeaProjects/lexique-review-113`, branch
`codex/d2-review-113`. Complete projections, independent API refetches, regression
and verification logs remain under its `target/issue-113-review/` directory.
The reviewer is separate from the author and parent integrator; no external
human review is claimed. The parent-approved focused resource seam passes
**3 tests** on Windows.

```powershell
uv run --python 3.14.10 python -X utf8 scripts/test_core_batch_113.py
git diff e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451 -- docs/evidence/issue-113 scripts/test_core_batch_113.py
```

The review accepts selected content and rights with recorded obligations.
Canonical conversion, actual combined notices, native lookup/save/reopen,
learner acceptance and release validation remain parent-owned and unrun here.
