# D2 frozen batches #120–#129

`core-129-v2` contains **2,435 accepted groups / 9,740 fields**, including **500 new groups / 2,000 fields** at frozen rows 1951–2450. The denominator remains **4,087 groups / 3,773 headwords**: 1,652 groups are unaccepted; 2,433 everyday groups are complete and 1,654 are incomplete. The two previously approved specialist senses and 15 zero-field annotations / 60 missing fields remain separately counted.

All ten batches passed separate content and selected-rights review. **1,973 fields remain as proposed; 27 exact replacements** preserve the selected meaning, grammar or construction. Original proposals and independent-review snapshots are retained in each batch.

| Batch | Frozen rows | Accepted groups / fields | Replaced fields |
| --- | --- | --- | --- |
| [#120](../issue-120/README.md) | 1951–2000 | 50 / 200 | 3 |
| [#121](../issue-121/README.md) | 2001–2050 | 50 / 200 | 3 |
| [#122](../issue-122/README.md) | 2051–2100 | 50 / 200 | 7 |
| [#123](../issue-123/README.md) | 2101–2150 | 50 / 200 | 4 |
| [#124](../issue-124/README.md) | 2151–2200 | 50 / 200 | 0 |
| [#125](../issue-125/README.md) | 2201–2250 | 50 / 200 | 4 |
| [#126](../issue-126/README.md) | 2251–2300 | 50 / 200 | 1 |
| [#127](../issue-127/README.md) | 2301–2350 | 50 / 200 | 1 |
| [#128](../issue-128/README.md) | 2351–2400 | 50 / 200 | 3 |
| [#129](../issue-129/README.md) | 2401–2450 | 50 / 200 | 1 |

## Resource admission

The [canonical resource](../../../resources/core-pilot-79/README.md) converts without network access and includes [combined notices](../../../resources/core-pilot-79/NOTICES.md), fixed source versions, contributors, selected import/original-work boundaries and [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING). The format-1 SQLite artifact is **11,350,016 bytes**. All 1,935 prior raw groups and compiled payloads, 15 spelling relationships / 11 aliases, and 15 approved zero-field annotations are preserved.

[Conversion](../../../target/implement-120-129/core-prepare-final.log), [coverage](../../../target/desktop-core-pilot/coverage.json), [preservation](../../../target/implement-120-129/preservation.json), [author archives](../../../target/implement-120-129/author-archive.json) and [reviewer archives](../../../target/implement-120-129/reviewer-archive.json) retain the measured outputs and byte-preservation evidence. Fresh author recapture for #124 encountered HTTP 429; independent retained-source validation and offline reproduction passed.

## Verification

`core-129-v2` provides the independently accepted ordinary `épée/NOM` weapon
meaning in all three languages. Fixed source spans, IPA, attribution and
ShareAlike obligations remain intact. All **2,434 other raw records and compiled
payloads**, all 2,467 source records, 15 spelling relationships and 11 aliases
are preserved, including the 1,935 groups admitted before these batches.

Windows 11 x64 checks use the matching rebuilt executable and format-1 core:
**11,350,016 bytes**, SHA-256
`2dd5c7be5d906cc09f8f8a0650652bea066a8ffbd77cb5043e2cd8548e64e5de`.
The [build identity](../../../target/fix-193/build.json) records the source base,
retained working diff, binary identity and platform. Core resources require
their matching executable because the native reader verifies its compiled hash.

| Check | Result | Evidence |
| --- | --- | --- |
| Produced-core regression | Failed on all three restricted meanings; passed with the everyday weapon definition | [RED](../../../target/fix-193/red.log), [GREEN](../../../target/fix-193/green.log) |
| Core conversion and batch admission | 17 core and 44 batch tests passed | [Core](../../../target/fix-193/core-tests.log), [batches](../../../target/fix-193/batches-tests.log) |
| Portable packaging | 16 tests passed; actual ZIP verified all 183 files | [Tests](../../../target/fix-193/package-tests.log), [matched ZIP](../../../target/fix-193/checked-package.log) |
| TypeScript production build and locked native rebuild | Passed | [Build](../../../target/fix-193/native-build.log) |
| Native batch #120 | All 50 groups / 200 fields matched; all 46 headwords saved and reopened | [Results](../../../target/issue-120/native-core-129-v2-final/results.json), [identity](../../../target/issue-120/native-core-129-v2-final/run-metadata.json) |
| Native resource upgrade | Existing digests, notes, examples, FSRS and protected audio preserved | [Results](../../../target/issue-129/native-core-129-v2-final-upgrade/results.json) |
| Other raw records, payloads and aliases | Preserved | [Read-only comparison](../../../target/fix-193/preservation.log) |

Both current native runs retained the hard **60-second** limit, recorded zero
external WebView requests and left zero owned processes. Their
[execution metadata](../../../target/fix-193/native-verification.json) records
the exact scope. Nine unchanged batches retain their baseline native evidence
below; they were not rerun with `core-129-v2`.

## Baseline verification (`core-129-v1`)

Windows 11 x64 production build: `2c14c0e7e3cf27c480fd816123bdbf097d6e5392`. Executable SHA-256: `a02a1c78b8084fa749444c778e881734f048dd1454d2d296c6cd6bb0849dd8c4`. The format-1 core is **11,350,016 bytes**, SHA-256 `06e0a500a900e63d5438769b2835fe373fe747cdaeeb2fd3d57bf6f49be9c4fc`. The portable ZIP passed **183-file** verification.

| Check | Result | Evidence |
| --- | --- | --- |
| Core conversion and content batches | 135 passed | [Core/cohort log](../../../target/implement-120-129/core-all.log) |
| Final admission metadata and evidence links | 44 focused tests, lint, documentation links and diff checks passed | [Batch tests](../../../target/implement-120-129/cohort-postnative.log), [lint](../../../target/implement-120-129/lint-final.log), [documentation links](../../../target/implement-120-129/docs-final.log) |
| Portable package | 16 passed; actual ZIP verified | [Tests](../../../target/implement-120-129/package-final-tests.log), [package metadata](../../../target/implement-120-129/final-package.json) |
| Rust formatting and locked precompilation | Passed | [Format](../../../target/implement-120-129/rust-format-final.log), [precompile](../../../target/implement-120-129/rust-precompile-final.log) |
| Backend execution | 210 passed; hard 60-second limit | [Backend log](../../../target/implement-120-129/rust-tests-final.log) |
| Frontend lint, types and production build | Passed | [Lint](../../../target/implement-120-129/lint.log), [types](../../../target/implement-120-129/typecheck.log), [native/production build](../../../target/implement-120-129/final-native-build.log) |
| Browser smoke | 1 passed | [Smoke log](../../../target/implement-120-129/browser-smoke.log) |
| Native batches #120–#129 | All ten passed: exact lookup, trusted save, restart/reopen | [Native execution metadata](../../../target/implement-120-129/native-execution-verification.json) |
| Closed `core-119-v1` upgrade | Passed; learning state and closed baseline preserved | [Upgrade results](../../../target/issue-129/native-upgrade/results.json) |
| Prior raw records, payloads and aliases | All 1,935 groups preserved | [Preservation](../../../target/implement-120-129/preservation.json) |

All eleven native runners matched the recorded executable, retained their hard 60-second execution limit, recorded zero external WebView requests and left zero owned processes. The upgrade used the closed six-headword `core-119-v1` baseline, retaining references, frozen digests, notes, examples, FSRS and protected selected audio. Speech models were absent.

[Build identity](../../../target/implement-120-129/final-build.json), [upgrade-helper scope](../../../target/implement-120-129/native-upgrade-helper-fix.patch) and [complete verification metadata](../../../target/implement-120-129/verification-final.json) retain exact inputs, outputs and evidence scope. The character-budget fixture covers its selected headword; production resource and response limits remain unchanged. Final admission metadata reports zero missing or pending fields in all ten completed batches; authored and independent-review snapshots remain preserved.

## Remaining gates

Whole-machine offline denial, other-platform native execution, learner acceptance and formal release readiness remain unverified. CEFR remains unknown. [PR #193](https://github.com/zardonc/lexique/pull/193) implements these ten batches; #78 and the complete D2 target remain open.
