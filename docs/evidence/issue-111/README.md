# Issue #111 authored content

All **50 exact frozen groups at rows 1501–1550** have one authored everyday
selected sense and **200 nonempty FR/IPA/EN/ZH fields**. Headwords, POS
subclasses, frequencies and order match the fixed manifest. There are **zero
unresolved exact groups**, **zero reused groups**, and no added annotations,
specialist exceptions or runtime aliases.

**Independent semantic, spelling/construction and selected-span rights review
is pending for all 200 fields.** The author records zero independently accepted
groups or fields. Candidate completeness adds no admitted coverage.

The [ticket](https://github.com/zardonc/lexique/issues/111) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the scope.
[Source research](source-research.md) records selected-use boundaries;
[notices](NOTICES.md) and [rights proposals](rights.json) identify the applicable
source layers and redistribution obligations.

## Author package

| File | Content |
| --- | --- |
| [groups.json](groups.json) | 50 frozen identities, authored senses and four fields per sense |
| [sources.json](sources.json) | 43 fixed source revisions, attribution and modification records |
| [source-excerpts.json](source-excerpts.json) | All 200 field bindings, exact raw lines, IPA variant positions and support |
| [source-context.json](source-context.json) | French/POS/construction frames, complete contributors and original-work boundaries |
| [source-captures.json](source-captures.json) | Retained API/wikitext paths and complete contributor pagination |
| [field-review.json](field-review.json) | Proposed text and pending separate review for 200 fields, 50 uses and 50 spelling relations |
| [rights.json](rights.json) | Per-field author rights proposals and pending independent decisions |
| [reuse.json](reuse.json) | Explicit empty reuse; existing canonical content remains unchanged |
| [coverage.json](coverage.json) | Fixed 4,087-group / 16,348-field denominator and pending admission counts |
| [measurements.json](measurements.json) | Actual authored-field and source-capture sizes; runtime measurements pending |
| [admission.json](admission.json) | Complete author package with independent and parent-owned gates pending |
| [COPYING](COPYING) | Actual complete CC BY-SA 4.0 licence text |

The baseline canonical input at `fa4ea3e` has 1,436 admitted groups / 5,744
fields, with 2,651 groups / 10,604 fields without admitted content. This batch's
50 groups / 200 fields remain pending and are included in those missing totals.
The parent recomputes combined coverage after independent review.

## Verification and evidence

The [focused resource check](../../../scripts/test_core_batch_111.py) uses the
parent-authorized resource/admission seam. It checks exact frozen membership,
frequencies, four-field bindings, captured raw spans, language labels, attribution,
actual licence text and honest pending states. Preparation also verifies that
retained wikitext equals the captured API revision and that all seven contributor
capture parts terminate their pagination. These are author checks.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-111/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_111.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

[Missing-batch RED](../../../target/issue-111/red.log),
[rights RED](../../../target/issue-111/rights-red.log),
[preparation](../../../target/issue-111/prepare.log),
[GREEN](../../../target/issue-111/green.log),
[documentation](../../../target/issue-111/docs.log) and
[diff](../../../target/issue-111/diff-check.log) evidence remains in
`C:/Users/inthe/IdeaProjects/lexique-batch-111/target/issue-111/`, together with
the author scripts, exact source selection and full API/wikitext captures.

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-111`, branch
`codex/d2-content-111`. Initial HEAD, integration tip and merge base were all
`fa4ea3eab1558fbede04b50cf0cd15f05bdf26dc`. The integration tip is merged before
handoff; the final tested Git state is retained in
[verification state](../../../target/issue-111/verification-state.json).

## Remaining gates

The parent arranges independent review before canonical admission. Combined
conversion, actual notices/source bundling, native lookup for every audited
group, representative trusted save/reopen, resource-update preservation and
native size/performance checks remain parent-owned and unrun for this author
package. Frontend/Rust/UI checks are unrun because this changes only owned
content evidence and its resource test. Whole-core, learner acceptance and
release/platform gates retain their separate states.
