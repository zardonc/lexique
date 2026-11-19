# Issue #103 author handoff

Frozen rows **1101–1150** have **50 candidate groups / 50 selected senses / 200
nonempty FR/IPA/EN/ZH fields**, with exact word, grammatical subclass, frequency
strings and order preserved. The denominator remains **4,087 groups**.
Independent selected-content and rights admission is **pending**: **zero groups
and zero fields are admitted by this handoff**. There are no missing candidate
fields or proposed annotation exceptions.

[Candidates](review-candidates.json), [groups](groups.json), [field review](field-review.json)
and [admission](admission.json) retain that status explicitly. The seven-file
reviewer/merger input uses an array for `groups.json` and `sources.json`, and
objects for field review, source context, source excerpts, reuse and admission.
The separate reviewer must supply accepted text, same-use decisions, actual
selected-source rights and their own identity before normalization or admission.

[Source research](source-research.md), [sources](sources.json), [source excerpts](source-excerpts.json),
[source context](source-context.json) and [capture paths](source-captures.json)
identify the fixed originals, exact spans, contributors, spelling/IPA bindings,
modifications and external/import boundaries. The existing
[accepted #99 example](../issue-99/README.md) and
[core materials](../../../resources/core-pilot-79/README.md) supply the established
integration route. [Reuse](reuse.json) is empty; previously accepted canonical
content was not edited.

## Author checks and retained evidence

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-103`, branch
`codex/d2-content-103`, initial base
`aae5273af846e57668a8d267fa6a6566c4fd6e91`. Integration tip
`87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5` was merged before the author
commit; `target/issue-103/integration-merge.log` records the merge. The author diff covers only this
directory and [the focused resource check](../../../scripts/test_core_batch_103.py).
Execution environment: Windows x64, PowerShell 7, uv Python 3.14.10.

| Check | Result | Evidence below `target/issue-103/` |
| --- | --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_103.py -v` | Passed: exact frozen membership, four-field source bindings, original API/body equality and pending review consistency | `author-green.log` |
| Initial focused resource check before content | Expected RED at the first missing candidate file | `author-red.log` |
| Fixed original acquisition and candidate preparation | Passed: 48 lexical articles, paginated contributors, 16 reference templates and fixed copyright policy | `capture.log`, `capture-support.log`, `capture-authorities.log`, `prepare.log` |
| DAF8 original endpoint inspection | 40 responses retained: 37 entries and three no-entry responses | `capture-originals.log`, `original-entry-check.log`, `original-index.json` |
| Scoped diff and Markdown links | Passed | `diff-check.log`, `doc-links.log` |

Complete original bodies and working preparation files remain in the ignored
`C:/Users/inthe/IdeaProjects/lexique-batch-103/target/issue-103/` directory.
`handoff.json` records the authored commit, reviewed diff commands and retained
patch paths; `full-author.patch` and `author-test.patch` separate the complete
handoff from its executable check.
`sources/` contains complete unmodified API responses and blank-line-preserving
wikitext, including local quoted context. `originals/` contains complete DAF8
endpoint HTML responses and separate readable text. Tracked excerpts select
lexical definitions, grammatical frames, pronunciation, restrictions and
bibliographic markers. Archive these owning paths before retiring the worktree.

## Limitations

All 200 fields await independent quality and selected-text rights review.
Page-level imports and references do not establish selected-sentence authorship;
modern proprietary sources and historical originals retain separate boundaries.
DAF8 endpoints for `plusieurs`, `excellent` and `arrière` returned no entry;
their Wiki candidates remain sourced to their own fixed revisions. DAF8
`bosser` and `récupérer` do not contain the selected modern working and
strength-recovery uses. The L123-3 primary statute retrieval returned HTTP 403;
the source record retains the proposed historical-edition basis and owning URL,
without claiming a successful legal refresh.

Independent admission, canonical conversion/notices, real native lookup,
trusted save/reopen and final integration verification belong to integration.
No frontend, Rust, native, package, learner or release checks were run here.
