# D2 packaged candidate and learner review

The Windows x64 candidate contains `core-162-v1+gender-1`, format 2: **4,062 admitted senses / 16,248 fields**, **25 approved zero-field annotations / 100 missing fields**, and the unchanged **4,087-group / 3,773-headword scope**. Everyday coverage remains 4,059 complete groups; the three approved specialist senses are separate. Of 2,134 noun senses, 1,940 have admitted gender and 194 await independent human review.

The [44-sense learner packet](learner-review-packet.md) is copied from the packaged SQLite and spans all 21 admitted POS classes, auxiliaries, function words, polysemy, spelling relationships, restricted usage and long content. Its exact sense IDs and provenance are retained. The [194-row gender table](../noun-gender-residual-review.md) remains pending; packet preparation and code review do not provide human semantic or learner acceptance.

## Standards

Review base `136c43fea172b73566004bb5bd9475b79fc36373`, implementation head `e1ed65d470a34b05ca8b938cd5cf41a9f1f3d3e0`. No documented-standard violations were found in the reviewed noun-capture, admission and residual-table files. One possible duplicated-code smell concerns marker recognition in the research parser and resource builder. The builder deliberately accepts only explicit, agreed `m` / `f`, while research collects broader candidates and uncertainty; their distinct admission responsibilities remain in place. This is a judgement call rather than a confirmed bug.

## Spec

Two findings are resolved: the generated resource report now states the reader's **24 MiB** limit rather than 20,000,000 bytes, and the assembled four-field learner packet is available. The inspected gender table meets its structural requirements and withholds all 194 pending values. The independent human and real learner decisions remain external acceptance gates.

## Windows x64 evidence

The [receipt](verification.json) binds application source `e1ed65d`, packaging/native-check source `ffe7d74`, executable SHA-256, resource SHA-256, package measurements, exact native commands, network boundary and cleanup. Application Rust/React code is unchanged between these two commits. Logs remain in the receipt's owning worktree under `target/`.

- Portable ZIP: **50,890,522 bytes**; installation: **281,708,041 bytes**; installed core: **211,744,132 bytes**. The packager verifies 188 files, source/notice hashes and exclusion of learning data. This candidate omits the speech model.
- Core SQLite: **21,143,552 bytes**; maximum sense JSON: **6,754 bytes**; longest required field: **566 Unicode scalars**. Reported bounds are 512 query UTF-8 bytes, 200 matching senses, 4 MiB responses, 24 MiB resources and 20,000 field scalars.
- Four native scenarios pass: all 36 admitted #162 batch records save/reopen; the exact 44-sense packet matches native lookup; synthetic TSV file-input import/restart/rejected replacement/valid replacement/removal preserves saved digests; missing and corrupt resources preserve core/legacy records, examples, history, operations and selected/protected audio. Every scenario retains the hard 60-second timeout and leaves no host, worker or helper processes.
- Final core preparation: 19 tests pass. Portable packaging: 16 tests pass. Native fixture boundaries: two tests pass, including #163/#164 descendants and rejection of roots/escapes. Lint and focused resource-budget regression pass. Earlier Rust and UI verification remains attached to the [implementation evidence](../cefr-bug-fixes-20261127.md).

## Acceptance remaining

Issues [#163](https://github.com/zardonc/lexique/issues/163), [#164](https://github.com/zardonc/lexique/issues/164) and [#78](https://github.com/zardonc/lexique/issues/78) remain open. Record the independent reviewer and row decisions for the noun table, and the real learner's exact sense coverage, feedback and acceptance for the packet and application. The owner-approved annotations remain missing fields rather than everyday senses.

Native checks deny external WebView routes after startup; whole-machine network denial is unrun. File-input injection uses synthetic TSV data and does not exercise the Win32 picker or download official FLELex data. The current candidate's old-binary/resource upgrade was not rerun; prior [core-162-v1 upgrade evidence](../issues-160-162/README.md) remains separate. Formal Full/Lite speech/download delivery, other-platform native execution and M5 release gates remain unrun.

Old readers reject format-2 resources and gender-bearing saved digests. Rollback requires a matching application/resource pair with a compatible pre-change backup or separately preserved data directory.
