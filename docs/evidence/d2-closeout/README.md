# D2 packaged candidate and learner review

The Windows x64 candidate contains `core-162-v1+gender-2`, format 2: **4,062 admitted senses / 16,248 fields**, **25 approved zero-field annotations / 100 missing fields**, and the unchanged **4,087-group / 3,773-headword scope**. Everyday coverage remains 4,059 complete groups; the three approved specialist senses are separate. Of 2,134 noun senses, **2,133 have admitted gender**. LMY reviewed all 194 residual decisions: use Lexique when nonnull, otherwise Candidate; omit when both are null. Only frozen row 253, `gens`, is omitted.

The [44-sense learner packet](learner-review-packet.md) is copied from the packaged SQLite and spans all 21 admitted POS classes, auxiliaries, function words, polysemy, spelling relationships, restricted usage and long content. Its exact sense IDs remain unchanged, with updated gender and provenance. The [194-row gender table](../noun-gender-residual-review.md) records LMY's completed decisions. Real learner feedback and acceptance of the packet and application remain pending.

## Standards

Review base `136c43fea172b73566004bb5bd9475b79fc36373`, implementation head `e1ed65d470a34b05ca8b938cd5cf41a9f1f3d3e0`. No documented-standard violations were found in the reviewed noun-capture, admission and residual-table files. One possible duplicated-code smell concerns marker recognition in the research parser and resource builder. The automatic admission path accepts explicit, agreed `m` / `f`, while research collects broader candidates and uncertainty; their distinct responsibilities remain in place. This is a judgement call rather than a confirmed bug. LMY's subsequent decisions use one shared policy function in the shipped converter and evidence generator.

## Spec

Two findings are resolved: the generated resource report states the reader's **24 MiB** limit, and the assembled four-field learner packet is available. The gender table retains all 187 original residuals and seven additional exceptions. LMY's recorded column decisions admit 193 values (92 `m`, 41 `f`, 60 `mf`); 37 differing columns follow Lexique. Lexique selections cite Lexique 3.83 rather than the Wiktionnaire header. Regeneration replays the exact reviewed columns, and admission rejects missing decisions or changed columns/values. The noun review gate is complete; learner acceptance remains pending.

## Windows x64 evidence

The [current receipt](verification.json) binds implementation source `c473ba3`, native-check source `cfd2869`, executable SHA-256, resource SHA-256, package measurements, exact native commands, network boundary and cleanup. The [prior gender-1 receipt](verification-gender-1.json) retains its original artifact and checks. Logs remain in the owning worktree under `target/lmy-*` and `target/issue-164/lmy-*`.

- Portable ZIP: **50,918,521 bytes**; installation: **281,851,359 bytes**; installed core: **211,887,450 bytes**. The packager verifies 188 files, source/notice hashes and exclusion of learning data. This candidate omits the speech model.
- Core SQLite: **21,286,912 bytes**; maximum sense JSON: **6,754 bytes**; longest required field: **566 Unicode scalars**. Reported bounds are 512 query UTF-8 bytes, 200 matching senses, 4 MiB responses, 24 MiB resources and 20,000 field scalars.
- Four native scenarios pass: all 36 admitted #162 batch records save/reopen; the exact 44-sense packet matches native lookup; synthetic TSV file-input import/restart/rejected replacement/valid replacement/removal preserves saved digests; missing and corrupt resources preserve core/legacy records, examples, history, operations and selected/protected audio. Every scenario retains the hard 60-second timeout and leaves no host, worker or helper processes.
- Core preparation: 20 tests pass. Policy/capture: four tests and 307 fixed-revision replays pass. Portable packaging: 16 tests pass. The shipped converter starts in isolation; all 4,062 original senses retain their four fields, usage and original provenance. Locked Rust precompile/tests, lint, typecheck, rustfmt and Windows x64 release build pass. Earlier UI verification remains attached to the [implementation evidence](../cefr-bug-fixes-20261127.md).

## Acceptance remaining

Issues [#163](https://github.com/zardonc/lexique/issues/163), [#164](https://github.com/zardonc/lexique/issues/164) and [#78](https://github.com/zardonc/lexique/issues/78) remain open. Noun reviewer LMY and all 194 decisions are recorded; the real learner's exact sense coverage, feedback and acceptance for the packet and application remain pending. The owner-approved annotations remain missing fields rather than everyday senses.

Native checks deny external WebView routes after startup; whole-machine network denial is unrun. File-input injection uses synthetic TSV data and does not exercise the Win32 picker or download official FLELex data. The current candidate's old-binary/resource upgrade was not rerun; prior [core-162-v1 upgrade evidence](../issues-160-162/README.md) remains separate. Formal Full/Lite speech/download delivery, other-platform native execution and M5 release gates remain unrun.

Old readers reject format-2 resources and gender-bearing saved digests. Rollback requires a matching application/resource pair with a compatible pre-change backup or separately preserved data directory.
