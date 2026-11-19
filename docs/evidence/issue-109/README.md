# Issue #109 authored content

All **50 frozen rows 1401–1450** retain their exact headwords, POS subclasses,
frequencies and order. Each has one authored selected sense with **FR/IPA/EN/ZH**:
**50 candidate senses / 200 candidate fields**. Independent acceptance is
**0 groups / 0 fields**. The global denominator remains **4,087 groups**.

The [ticket](https://github.com/zardonc/lexique/issues/109) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the scope.
[Source research](source-research.md) explains the selected-use boundaries and
retained evidence. This handoff is for a separate content/rights reviewer.

## Handoff files

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities and complete proposed senses |
| [sources.json](sources.json) | Fixed source records, attribution, original-work rights proposals and modifications |
| [field-review.json](field-review.json) | Proposed field text, exact sources, use and spelling evidence; independent statuses pending |
| [source-context.json](source-context.json) | French/POS frames, construction evidence, contributors, bibliography and import boundaries |
| [source-excerpts.json](source-excerpts.json) | Exact one-based raw lines and selected IPA variant positions |
| [reuse.json](reuse.json) | Empty reuse list; no prior content is rewritten |
| [admission.json](admission.json) | Candidate counts and pending independent/admission outcomes |
| [source-captures.json](source-captures.json) | Fixed API/wikitext paths and complete contributor pagination |

## Author verification

The existing resource boundary is used by
[the focused check](../../../scripts/test_core_batch_109.py). The parent’s
Testing Decisions already authorize this seam. Author preparation compares
the complete captured API revision with the retained wikitext, verifies French
and POS frames, and validates all 200 selected spans. The test checks frozen
membership, frequencies, nonempty four-field candidates, bindings and pending
independence/admission states. Neither check is an independent semantic or rights
review.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-109/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_109.py
git diff --cached --check
```

The focused test recorded an expected RED on the absent handoff, then GREEN
with the authored package. [RED](../../../target/issue-109/red.log),
[preparation](../../../target/issue-109/prepare.log) and
[GREEN](../../../target/issue-109/green.log) logs are retained in the owning
worktree. Diff/link results and the final tested Git state are retained there.

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-109`, branch
`codex/d2-content-109`, based on integration `codex/d2-content-100-109` at
`aae5273af846e57668a8d267fa6a6566c4fd6e91`. Integration’s current tip is merged
before handoff. Source/capture/preparation evidence remains at
`C:/Users/inthe/IdeaProjects/lexique-batch-109/target/issue-109/`.

## Pending outcomes

No unsupported-field blocker was identified in author selection. Separate review
must decide all 200 fields, all 50 uses, the construction/component bindings and
the selected-text rights boundaries, including the 27 DAF8-marked pages. All
independent statuses remain pending. No new specialist or annotation exception
is proposed. The seven existing exceptions elsewhere remain unchanged.

Canonical resource admission, conversion, notices packaging and actual native
lookup/save/reopen are integration-owned. They were not run for this candidate
handoff. Whole-core completion, learner acceptance and release/platform gates
remain separate.
