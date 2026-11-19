# Issue #108 author handoff

Frozen rows **1351–1400** retain all **50 exact groups / 50 candidate senses / 200
nonempty FR/IPA/EN/ZH fields**, including both frozen frequency strings. The
denominator remains **4,087 groups**. This handoff admits **zero groups and zero
fields**; independent content and rights review is pending.

The seven-file reviewer/merger contract is [groups](groups.json),
[sources](sources.json), [field review](field-review.json),
[source context](source-context.json), [source excerpts](source-excerpts.json),
[reuse](reuse.json) and [admission](admission.json).
[Candidates](review-candidates.json) provide the resource-boundary test input.
Source IDs, revisions, timestamps, exact one-based lines/columns, contributor
credit, original-source markers, modifications, constructions and inflection
bindings are retained. [Source research](source-research.md) describes the
selected uses and unresolved boundary. [Capture paths](source-captures.json)
identify the original API responses and wikitext.

## Author verification

The approved #78 resource boundary is exercised by
[test_core_batch_108.py](../../../scripts/test_core_batch_108.py). It checks the
exact frozen membership/frequencies, all four candidate fields and their fixed
source bindings, while requiring independent review to remain pending.
The preparation check additionally reconciles each selected excerpt and column
with the original API revision and its byte-preserved wikitext, French/POS
frames, applicable IPA and complete contributor pagination.

On Windows x64, Python 3.14.10:

- Focused RED: the candidate test failed because the handoff was absent.
- Candidate preparation: passed, 50 groups / 200 fields / 46 fixed articles,
  with eight complete contributor response parts.
- Focused GREEN: the candidate test passed.
- `pnpm test:docs`: passed, 255 documents / 1,852 local links, no problems.
- `git diff --cached --check`: passed.

Command:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_108.py
```

Original captures, preparation/selection helpers and logs remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-108/target/issue-108/`:
`sources/`, `source-index.json`, `contributor-index.json`, `notices-audit.json`,
`blockers.json`, `selection.py`, `prepare_candidates.py`, `red.log`, `green.log`,
`green-final.log`, `prepare.log`, `contributors.log`, `docs-check.log` and
`diff-check.log`. The full authored patch (`authored.patch`) and the separate
test patch (`tests.patch`) are retained there for reviewer handoff. Source and
candidate checks covered staged authored changes based on integration commit
`d9f8e98`; [#78](https://github.com/zardonc/lexique/issues/78) and
[#108](https://github.com/zardonc/lexique/issues/108) define the reviewed scope.

## Limitations

**Row 1386, `nulle/PRO:ind`, remains blocked on modern everyday eligibility.**
The exact feminine-pronoun source frame has only an 1855 usage example; the
linked `nul` frame calls its literary/legal pronoun invariable. Its four sourced
candidate fields remain pending, with the conflict recorded in field/usage and
inflection evidence. No annotation exception, spelling substitution or runtime
alias is proposed. Row 1385 retains its separately evidenced feminine indefinite
adjective and literary register.

Author checks establish source correspondence, not independent quality or rights
acceptance. The separate reviewer must decide every selected field, same-use
restriction and original/transcription/adaptation rights boundary. Shared core
resources, converter and verification documents remain integration-owned.
Conversion, packaged notices, real native lookup/save/reopen, learning-state
preservation, other-platform execution and release checks were not run here.
