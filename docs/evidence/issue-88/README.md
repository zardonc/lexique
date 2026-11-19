# Issue #88: sourced content draft

All **50 exact frozen groups in rows 351–400** are represented. The draft adds
**49 candidate senses / 196 proposed fields** and reuses
**row360 `aujourd'hui/ADV` / four accepted fields** byte-identically.
All new field, usage and spelling reviews are pending.

| Result | Count |
| --- | ---: |
| Exact frozen identities | 50 |
| New candidate senses / fields | 49 / 196 |
| Unchanged reused groups / fields | 1 / 4 |
| Newly independently accepted fields | 0 |
| New fields awaiting semantic and rights review | 196 |
| Fixed API captures / selected source contexts | 43 / 41 |
| Byte-identical research copies / first-party API captures | 38 / 5 |
| Selected source locations | 105 |
| Selected pages declaring DAF8 imports | 17 |
| Concrete POS/use eligibility gaps | 1 |
| Frozen global denominator | 4,087 |

**Row357 `ok/ADV` remains unresolved.** Fixed French `OK` revision 39863637
labels the agreement marker an interjection, not an adverb. The four proposed
fields describe that agreement use, and the usage record preserves the
classification mismatch. Its attested lowercase spelling does not resolve
the adverb requirement. No scope exception is assumed.

[review-candidates.json](review-candidates.json) contains exact proposed text,
sense identities, fixed definition/IPA locations and construction support.
[source-research.md](source-research.md) records all fifty selections and
source boundaries. [source-captures.json](source-captures.json) identifies
retained API/raw evidence and copied-source origins.
[quality-review.md](quality-review.md) records the remaining independent audit.

The resource changes add only ticket-owned groups, sources, selected excerpts,
contexts, pending reviews and two `ok` spelling records. All inherited content
and metadata, including #89's draft, are retained. The permitted one-line `ROWS`
extension includes rows351–400 and counts row360 once; existing input checksums
are refreshed. Coordinator integration owns counters, content version,
validators, final admission and native validation. Learning data remains untouched.

## Checks

Owning checkout: `C:\Users\inthe\IdeaProjects\lexique-issue-88`, branch
`codex/d2-content-88`. Retained execution evidence: `target/issue-88/`.
Python commands use uv Python **3.14.10**, Windows x64.
Draft diff base: `d55b6bdf497d310592dc67db88d7b2b09521c1b5`, the integration
tip containing #89 before this draft. The preservation check covers every
inherited source, review, sense, context and spelling record, plus byte-identical row360.

| Command | Actual result / retained log |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_88.py -v` | Passed: three focused resource-boundary checks; `focused-tests.log` |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-88/validate_sources.py` | Passed: 43 fixed API bindings, 38 byte-identical copies, 196 exact field spans, 41 contexts and 105 selected locations; `source-validation.log`, `source-validation.json` |
| `uv run --python 3.14.10 python -B -X utf8 scripts/core_pilot.py prepare target/issue-88/prepared` | Blocked as expected at the independent-review gate: `Unreviewed field source or incompatible licence`; `conversion.log` |
| Focused test development | Membership failed for the 49 missing rows before drafting; membership and preservation passed after drafting. The pending-review gate passed with the permitted row-set extension; `tdd-membership-red.log`, `tdd-membership-green.log`, `focused-tests.log` |
| `pnpm test:docs` | Passed: 184 documents / 1,450 local links; `docs-check.log` |
| `git diff --cached --check` | Passed; `diff-check.log` |

## Gaps

Independent semantic/rights review remains pending for all 196 new fields,
including the selected boundaries on seventeen DAF8-declaring pages and
`ok/ADV` eligibility. The source checks establish capture/span binding, not
translation accuracy or underlying import rights. Converter admission,
coverage/size, native lookup/save/restart, packaged notices and combined
upgrade preservation are coordinator checks. Frontend, Rust, browser, speech
and release checks were not run for this pending content draft.
