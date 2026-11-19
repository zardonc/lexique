# Issue #102 independent reviewer handoff

All **50 frozen groups at rows 1051–1100** are retained. **48 groups / 192 exact FR/IPA/EN/ZH fields** are independently accepted. Eight fields remain missing; batch admission is pending. There is no approved exception or runtime alias. The denominator remains **4,087 groups**.

[Groups](groups.json) contains accepted content; [review candidates](review-candidates.json) preserves the author's 50-group draft, frequencies and original pending statuses. The seven-file handoff comprises groups, [sources](sources.json), [field review](field-review.json), [source context](source-context.json), [source excerpts](source-excerpts.json), [reuse](reuse.json) and [admission](admission.json). [Independent review](independent-review.md) records every acceptance boundary, blocker proof and validation; [rights](rights.json) identifies the required notices. The original [source research](source-research.md) records author selections; the independent decision supersedes its pending-review and lowercase-spelling conclusions.

## Fixed original evidence

[Source captures](source-captures.json) identify original first-party API response bytes, complete revision wikitext, actual revision timestamps and linked contributor histories. Full evidence and author scripts remain at:

`C:/Users/inthe/IdeaProjects/lexique-batch-102/target/issue-102/`

The `sources/` directory uses unique page-ID filenames for case-distinct titles. The independent reviewer refetched all 56 fixed revisions and compared complete original API/wikitext, IDs, titles, timestamps and last editors. [Rights](rights.json) records checked authorities and selected original layers. Source excerpts retain one-based lines and columns; unrelated example prose stays in the local captures. New primary `mme/Mme` corroboration and exact capture paths are in the spelling-relation records.

## Author-stage verification

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_102.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-102/verify_sources.py
git diff --check
```

The focused resource test had a retained missing-candidate RED before the handoff was written. Membership and field checks passed after preparation. Original-source verification compares complete wikitext to the original API response, then checks selected French-language spans, revisions, URLs, lines and columns. Logs are `membership-red.log`, `fields-red.log`, `green.log`, `source-validation.log` and the final merge/diff/link logs under the evidence root. These checks establish authored artifact consistency, not independent translation or rights acceptance.

## Blockers and remaining gates

Row **1051 `s/PRO:per`** has an independently confirmed bare-spelling pronoun source gap. Row **1057 `mme/NOM`** has a supported lowercase dictionary-key relationship to French `Mme`, but its four fields need an author draft and a separate independent check. Each retains zero fields, four missing fields and no approved exception. [Independent review](independent-review.md), [source excerpts](source-excerpts.json) and [admission](admission.json) record the proof and replacement needs. `ben/NOM` passes for modern slang trousers `/bɛn/`, with the botanical sense excluded.

The 192 candidate fields passed independent field/usage/selected-text rights review. Remaining source/author work, any owner scope decision, shared conversion/actual notices, native lookup/trusted save/reopen and final integration verification remain pending. Shared canonical resource files and learning data are outside this review change.
