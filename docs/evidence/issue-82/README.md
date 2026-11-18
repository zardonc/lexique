# D2 content batch 002 (#82)

[Issue #82](https://github.com/zardonc/lexique/issues/82) retains all fifty frozen
targets in rows 0051–0100. **49 groups / 196 fields are accepted**: 47 new groups
and the two unchanged `bien` pilot groups. Frozen row 87, `quoi/ADJ`, is the
approved annotation exception with zero admitted fields. The project owner
approved the #78/#82 scope amendment on 2026-11-18 UTC. The batch is complete
within that scope: 49 groups with content and one annotation exception;
content coverage remains 49/50.

The [investigation of row 87](quoi-adj-research.md) confirms the
original Lexique `ADJ` record but finds no supported modern everyday adjective
sense. Its [approved amendment](quoi-adj-research.md#approved-scope-amendment)
preserves the exact frozen row and denominator and reports the annotation
exception separately from admitted content.

The development resource contains **104 groups / 416 reviewed fields**, content
version `core-82-v1`, resource format 1. All 57 prior groups, their 228 fields and
provenance, and all 45 prior source records are preserved exactly. Reused groups
count once. The fixed denominator remains 4,087 groups / 3,773 headwords: 104
accepted and 3,983 unaccepted groups. Literal everyday coverage is 103 complete
and 3,984 incomplete groups; the existing #81 `ça/NOM` specialist exception
remains separate. Row 87 is included in the unaccepted and everyday-incomplete
totals, with one separately reported annotation exception.

## Content and redistribution

The [source evidence](source-research.md) fixes the selected French definitions,
IPA, English and Chinese meanings to their exact POS/use and source spans.
The [independent review](quality-review.md) accepts all 188 new fields, 47 usage
records and three spelling relationships. It separately reviews the eight
declared Académie imports and preserves their original edition credit.
The [curation](../../../resources/core-pilot-79/curated-inputs.json),
[field review](../../../resources/core-pilot-79/field-review.json) and
[source context](../../../resources/core-pilot-79/selected-source-context.json)
retain these bindings and actual modifications.

`où` keeps both exact frozen pronoun subclasses with explicit source adverb
terminology. `mon/ADJ:pos` keeps its possessive construction restrictions and
has native/frontend validation and labels in English, French and Chinese.
The two `s'` groups and `t'` keep separate, evidenced curly-apostrophe bindings.
`sur/ADJ` retains its unaccented sour-taste sense and feminine `sure`.

The converter and portable package retain the actual CC BY-SA 4.0 legal text,
selected excerpts, complete contributor-history and fixed-version links,
original import credits, adaptation notices, frozen Lexique membership and
upstream notices. The converter code keeps its separate MIT licence. The
[resource notices](../../../resources/core-pilot-79/NOTICES.md) own the combined
attribution and selected-content obligations.

## Verification

The verified artifact predates the scope amendment and retains its original
`scope_acceptance: incomplete` and `everyday_sense_review: incomplete` batch
metadata. It reports row 87's four absent fields as unresolved content. The
approved exception resolves the ticket's content requirement for that row;
the artifact's content and missing-field counts retain their original values.

Checks ran on 2026-11-17 UTC, Windows x64, in
`C:\Users\inthe\IdeaProjects\lexique-82`, branch `codex/d2-content-002-82`,
based on `39e79a11614d1f8e1bb7c573c837ce440fda2a7f` plus the batch changes.
Complete execution logs and artifacts are retained under `target/issue-82/`;
the reviewed source diff is `review-final.diff`. PowerShell 7, pnpm 11.25.0,
uv-resolved Python 3.14.10 and the pinned Rust 1.99.1/MSVC toolchain were used.

| Check | Result and retained evidence |
| --- | --- |
| `pnpm core:prepare` | Passed; 50 exact batch targets, 49 accepted groups, 196 batch fields and four unresolved row-87 fields; `core-prepare.log` |
| `pnpm test:core` | 10 passed; exact membership, field/review and rights binding, unresolved-field rejection, fixed coverage and reproducible preparation; `core-tests.log` |
| Prior-content comparison | Passed; 57 groups, 228 fields/provenance and 45 source records unchanged; `preservation.json` |
| `pnpm test:tooling` | Passed, including portable content-version validation; `tooling-tests.log` |
| `pnpm lint`, `pnpm typecheck` | Passed; `lint-resume-final.log`, `typecheck-pos.log` |
| `pnpm test:e2e` | 391 passed, including possessive-subclass lookup and rendering; `e2e-final.log` |
| `pnpm rust:fmt` | Passed; `rust-fmt-resume.log` |
| `pnpm rust:test:build`, `pnpm test:rust` | Passed; 194 backend tests after separate precompilation, with the existing hard 60-second runner limit; `rust-test-build-final.log`, `rust-tests-final.log` |
| `pnpm desktop:build --no-bundle` | Passed, including the frontend production build and native compilation; `native-build-final.log` |
| `python -X utf8 scripts/package_portable.py --output-dir target/issue-82/packages-verified` | Passed; 183 verified files; `package-verified.log` |
| `pnpm test:docs`, `git diff --check` | Passed; 174 documents, 1,395 local links and no reported problems; `docs-final.log` |

The prepared SQLite is **512,000 bytes**, SHA-256
`0c969da0f5da701fa693ee098398bbbff63a4743fca54d2ac3f9d678488cc39b`.
Its `report.json` and `coverage.json` retain the fixed global denominator,
separate batch and everyday acceptance, missing/unreviewed fields and size.
The portable ZIP is **35,285,129 bytes**, SHA-256
`868f1dae437bd2263c2728de97f9da9bea00bda444ebca63c13e55102a143d7e`.

## Merge and CI

[PR #171](https://github.com/zardonc/lexique/pull/171) merged as
`7864cde9b32a2ad8d3e296a847b0539206e9162a` on 2026-11-17 UTC.
The [pull-request run](https://github.com/zardonc/lexique/actions/runs/44195770556)
and [push run](https://github.com/zardonc/lexique/actions/runs/44195759779)
each passed all eleven jobs on head `a5a33c766dde409dc18ee33d4d6462eb345a7840`.
The [main run](https://github.com/zardonc/lexique/actions/runs/44199943502)
passed all eleven jobs on the merge commit, including the four native targets.

Both Windows ARM64 head runs passed the V03 storage contention test. Its
transaction and competing-writer limits remain 100 ms and 250 ms, and the
backend runner retains its hard 60-second limit.

## Native lookup, saving and upgrade

Both native checks use the built Windows x64 executable with SHA-256
`18b31be89cc6a5ce814489ce1e092f1907ba6747a340d08e22ab7fc9d91e5c81`
and isolated copies of the verified portable package. From `apps/desktop`,
with the pinned Python on `PATH`:

```powershell
node e2e/native-d2-pilot-check.mjs no-speech ../../target/issue-82/packages-verified/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../resources/core-pilot-79/curated-inputs.json ../../target/issue-82/native/no-speech-03
node e2e/native-d2-pilot-check.mjs upgrade ../../target/issue-82/packages-verified/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../resources/core-pilot-79/curated-inputs.json ../../target/issue-82/native/upgrade-02 ../../target/issue-82/upgrade-seed
```

The `no-speech` run passes with the speech model absent in its isolated package.
It compares actual native lookup and trusted saves with all audited fields and
provenance, including 49 admitted batch groups. `pour` reopens with separate noun
and preposition fields after normal restart. `mon` renders its audited fields and
possessive subclass labels in all three locales and reopens as a saved record.
Spelling aliases, forged-input rejection, retries, duplicates, notes, examples,
card identities and FSRS state pass the existing preservation assertions.
Unavailable speech reports an error while the saved digest remains intact.

The `upgrade` run starts from a closed copy of the retained `core-81-v1` test
profile: 32 saved records covering all 57 prior groups, progress and two cached
fixture audio selections. The original profile is preserved. Its pre-start state
is the independent comparison baseline. Saved references, digests, notes,
examples, save operations, cards, progress and audio relationships remain intact
after `core-82-v1` becomes available. The original creation request replays,
cached audio is readable and a new `pour` record saves against the updated core.

Both runs retain `results.json`, `run-metadata.json`, `lifecycle.jsonl` and
`cleanup.json` in their native result directories. External WebView requests
are empty and remaining owned hosts/workers/helpers are zero. The native runner
retains its hard 60-second execution limit.

## Limits

Row 87 `quoi/ADJ` is accounted for by the approved annotation exception with
zero admitted fields. Learner acceptance, CEFR labels, the remaining 3,983
groups without admitted content, whole-machine network denial,
hands-on application checks on other platforms and formal release gates remain
open. These native runs do not execute the broader backup/resource-failure
matrix recorded in [#80](../issue-80/README.md).
