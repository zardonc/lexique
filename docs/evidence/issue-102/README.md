# Issue #102 author handoff

All **50 frozen groups at rows 1051–1100** are retained. **48 groups / 192 FR/IPA/EN/ZH fields** have complete source-bound candidates. Two exact spellings remain blocked, with eight fields without eligible candidates. Independent admission is pending for every field; this handoff admits **zero groups / zero fields**. The denominator remains **4,087 groups**.

[Groups](groups.json) and [review candidates](review-candidates.json) preserve the exact memberships and frequency strings. The seven-file handoff comprises groups, [sources](sources.json), [field review](field-review.json), [source context](source-context.json), [source excerpts](source-excerpts.json), [reuse](reuse.json) and [admission](admission.json). Reviewer identities and accepted texts are unset. [Independent review](independent-review.md) records the pending boundary; [source research](source-research.md) explains selections, rights and spelling restrictions.

## Fixed original evidence

[Source captures](source-captures.json) identify original first-party API response bytes, complete revision wikitext, actual revision timestamps and linked contributor histories. Full evidence and author scripts remain at:

`C:/Users/inthe/IdeaProjects/lexique-batch-102/target/issue-102/`

The `sources/` directory uses unique page-ID filenames for case-distinct titles. These are actual current API-selected revisions, not an October cutoff. [Rights](rights.json) records fixed authorities and imported original layers. Source excerpts retain one-based lines and columns; unrelated example prose stays in the local captures.

## Author verification

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_102.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-102/verify_sources.py
git diff --check
```

The focused resource test had a retained missing-candidate RED before the handoff was written. Membership and field checks passed after preparation. Original-source verification compares complete wikitext to the original API response, then checks selected French-language spans, revisions, URLs, lines and columns. Logs are `membership-red.log`, `fields-red.log`, `green.log`, `source-validation.log` and the final merge/diff/link logs under the evidence root. These checks establish authored artifact consistency, not independent translation or rights acceptance.

## Blockers and remaining gates

Row **1051 `s/PRO:per`** has no established bare-spelling pronoun source. Row **1057 `mme/NOM`** has no established lowercase French noun binding. Each retains zero fields, four missing fields, no approved exception and pending independent confirmation. Their fixed-source evidence is in [source excerpts](source-excerpts.json) and [admission](admission.json).

Independent field/usage/rights review, any owner scope decision, shared conversion/notices, native lookup/trusted save/reopen and final integration verification remain pending. Shared canonical resource files and learning data are outside this authored change.
