# Issue #102 independent reviewer handoff

All **50 frozen groups at rows 1051–1100** are retained. **48 groups / 192 exact FR/IPA/EN/ZH fields** are independently accepted and unchanged. Row 1057 has **four new author fields pending separate independent review**; row 1051 still lacks four fields. Eight fields remain unadmitted; batch admission is pending. There is no approved exception or runtime alias. The denominator remains **4,087 groups**.

[Groups](groups.json) contains accepted content and the four pending Mme fields; [review candidates](review-candidates.json) preserves the author's 50-group draft, frequencies and pending statuses. The handoff comprises groups, [sources](sources.json), [field review](field-review.json), [source context](source-context.json), [source excerpts](source-excerpts.json), [reuse](reuse.json) and [admission](admission.json). [Independent review](independent-review.md) records the unchanged 192-field acceptance decision; [rights](rights.json) identifies the required notices. [Mme author handoff](mme-author.md) and [exact new fields and provenance](mme-author.json) record the four pending fields, selected layers and construction evidence. [Source research](source-research.md) records author selections.

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

The focused resource test compares field content and source bindings across the author/reviewer handoffs, checks the 192 accepted texts against the independent records, and requires all four Mme fields to remain pending. Original-source verification compares complete wikitext to the original API response, then checks selected French-language spans, revisions, URLs, lines and columns. Current logs are `mme-author/focused-green.log`, `mme-author/source-bindings.log`, `mme-author/preservation.log` and `mme-author/diff-links.log` under the evidence root. These checks establish authored artifact consistency; new translation and rights review remains pending.

## Blockers and remaining gates

Row **1051 `s/PRO:per`** has an independently confirmed bare-spelling pronoun source gap and no candidate fields. Row **1057 `mme/NOM`** has the supported lowercase dictionary-key relationship to French `Mme` and four sourced author fields awaiting a separate independent check. Each retains zero admitted fields, four missing admitted fields and no approved exception. [Independent review](independent-review.md), [source excerpts](source-excerpts.json) and [admission](admission.json) record the proof and remaining gates. `ben/NOM` passes for modern slang trousers `/bɛn/`, with the botanical sense excluded.

The 192 candidate fields passed independent field/usage/selected-text rights review. Remaining source/author work, any owner scope decision, shared conversion/actual notices, native lookup/trusted save/reopen and final integration verification remain pending. Shared canonical resource files and learning data are outside this review change.
