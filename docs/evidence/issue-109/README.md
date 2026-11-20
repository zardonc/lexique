# Issue #109 independently reviewed content

All **50 frozen rows 1401–1450** retain their exact headwords, POS subclasses,
frequencies and order. Each has one authored selected sense with **FR/IPA/EN/ZH**:
**50 accepted senses / 200 accepted fields**, with **no missing fields**.
The global denominator remains **4,087 groups**. Final package admission awaits
the parent's combined conversion, notices and native gates.

The [ticket](https://github.com/zardonc/lexique/issues/109) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the scope.
[Source research](source-research.md) explains the selected-use boundaries and
retained evidence. [Independent review](independent-review.md) records exact
same-use quality and [selected-span rights](rights.json). Original draft groups
and pending author statuses are preserved in [review candidates](review-candidates.json).

## Handoff files

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities and complete independently accepted senses |
| [sources.json](sources.json) | Fixed source records, attribution, original-work rights proposals and modifications |
| [field-review.json](field-review.json) | Proposed and unchanged accepted text, exact sources, independent use/rights/spelling judgments |
| [source-context.json](source-context.json) | French/POS frames, construction evidence, contributors, bibliography and import boundaries |
| [source-excerpts.json](source-excerpts.json) | Exact one-based raw lines and selected IPA variant positions |
| [reuse.json](reuse.json) | Empty reuse list; no prior content is rewritten |
| [admission.json](admission.json) | 50 groups / 200 fields accepted; final package gates pending |
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

## Review outcome and remaining gates

All 200 fields and 50 uses pass independent quality/selected-rights review.
Explicit construction/component evidence supports `quels`, `mien`, `mienne`
and `enceinte`. Exact `to/NOM` selects the ordinary regional food, `cent/NOM`
the number 100, and both `enfoiré` POS rows preserve their insulting register.
No exact-row gap, runtime alias or owner exception is added.

Canonical resource admission, conversion, notices packaging and actual native
lookup/save/reopen are parent-owned and were not run for this review.
The established DAF8 edition-level basis is reused; the independent direct
statute endpoint returned HTTP 403. Whole-core completion, learner acceptance
and release/platform gates remain separate.
