# Issue #108 reviewed content handoff

All **50 frozen groups at rows 1351–1400** remain represented. **49 groups / 196
unchanged FR/IPA/EN/ZH fields** pass independent content and selected-text rights
review. Row **1386 `nulle/PRO:ind`** retains zero admitted fields and four missing
fields. The owner approved the zero-field annotation on 2026-11-19; no runtime alias is approved; the denominator
remains **4,087 groups**.

The seven-file handoff comprises [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). [Independent review](independent-review.md) records
each accepted meaning/grammar boundary, the unresolved exact group and the
selected-text rights decision. [Rights](rights.json) records legal authorities
and actual notice obligations. Original 200-field proposals remain in
[review-candidates](review-candidates.json).

All 46 fixed original revisions were independently refetched. Complete original
wikitext, revision IDs, titles, timestamps and editors match the original
[captures](source-captures.json); all 200 exact candidate spans match those
originals. Full captures and verification logs remain under:

`C:/Users/inthe/IdeaProjects/lexique-batch-108/target/issue-108/`

The original comparison also passed for all 173 grammar/construction spans and
the French-language frames of all 196 admitted fields; see
`independent-context-check.log` under the evidence root.

The distinct row 1385 indefinite adjective passes for the current written use:
both source pages establish feminine agreement, singular-only, before-noun
construction and literary register. The pronoun's invariable-frame conflict
does not apply to that adjective. `champ` selects the ordinary agricultural
field; the unselected CFC cartography import contributes no field.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-108/independent_original_check.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-108/independent_context_check.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_108.py
git diff --check
```

Original-source/span verification, both focused checks and diff checking passed.
Logs are `independent-original-check.log` and `independent-tests.log` under the
evidence root. Shared conversion, actual combined notices, native trusted
save/reopen, final integration review and learner acceptance remain pending.

Owner decision: the exact frozen group retains zero admitted fields and four missing fields. The 2026-11-19 approval changes scope acceptance only; the 4,087-group denominator and frequencies remain fixed.
