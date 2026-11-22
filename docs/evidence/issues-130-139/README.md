# D2 frozen batches #130–#139

`core-139-v1` admits **497 new groups / 1,988 fields** at frozen rows 2451–2950, bringing the core to **2,932 groups / 11,728 fields**. The denominator remains **4,087 groups / 3,773 headwords**: 1,155 groups remain unaccepted; 2,930 everyday groups are complete and 1,157 remain incomplete. The two existing specialist senses remain unchanged. 18 owner-approved zero-field annotations retain 72 missing fields, including the three approvals for `oublie/NOM`, `r/ADV` and `man/NOM` on 2026-11-21.

All admitted fields passed separate semantic/use and selected-text rights review. 33 exact replacements are recorded; author proposals, source captures and reviewer decisions remain preserved.

| Batch | Frozen rows | Accepted groups / fields | Zero-field annotations | Replaced fields |
| --- | --- | --- | --- | --- |
| [#130](../issue-130/README-admission.md) | 2451–2500 | 50 / 200 | 0 | 1 |
| [#131](../issue-131/README-admission.md) | 2501–2550 | 50 / 200 | 0 | 0 |
| [#132](../issue-132/README-admission.md) | 2551–2600 | 49 / 196 | 1 | 4 |
| [#133](../issue-133/README-admission.md) | 2601–2650 | 49 / 196 | 1 | 0 |
| [#134](../issue-134/README-admission.md) | 2651–2700 | 50 / 200 | 0 | 3 |
| [#135](../issue-135/README-admission.md) | 2701–2750 | 50 / 200 | 0 | 6 |
| [#136](../issue-136/README-admission.md) | 2751–2800 | 49 / 196 | 1 | 3 |
| [#137](../issue-137/README-admission.md) | 2801–2850 | 50 / 200 | 0 | 2 |
| [#138](../issue-138/README-admission.md) | 2851–2900 | 50 / 200 | 0 | 12 |
| [#139](../issue-139/README-admission.md) | 2901–2950 | 50 / 200 | 0 | 2 |

## Resource admission and preservation

The [canonical resource](../../../resources/core-pilot-79/README.md) converts without network access. [Combined notices](../../../resources/core-pilot-79/NOTICES.md), fixed versions and selected spans, full contributor/history links, historical original-work boundaries, modification notices and [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING) accompany the resource. The exact Lexique 3.83 dataset capture used for one reviewed IPA field retains its own fixed edition, credits and transformation record.

All prior **2,435 raw groups / 9,740 fields**, source/context/review records, compiled SQLite payloads, 15 spelling relationships and 11 runtime aliases are unchanged. Annotated identities retain their order, frequency values and denominator; empty annotation placeholders and unadmitted proposals remain in the original batch archives, with four explicit missing records per annotation in the canonical review. No substitute spelling, POS, sense or alias is admitted.

## Verification

Windows 11 x64 native build: `d34efc046d3fd6e4d9e980253228830f42c6aea8`; executable SHA-256 `fd940094701928acaeeb0609e34eece1efe12a98ffb8b2970506518b026eaba3`. The format-1 core is **13,656,064 bytes**, SHA-256 `7d480491c094f89100b9f1767816d4e9e753b12a8b26ba2ccf338d0aabf04051`.

Expanded core checks ran at `6afd73ece72ccbc7b2f658d811900b8ee09c18b0`. Its only change after the build is the #130 Python test's canonical review-status expectation; [the source comparison](../../../target/implement-130-139/check-source-gap.json) confirms unchanged runtime, native harness and resource inputs. Admission summaries are recorded after execution.

| Check | Result | Evidence |
| --- | --- | --- |
| Core conversion and batch checks | 206 passed | [Log](../../../target/implement-130-139/core-all.log) |
| Portable package | 16 passed; actual ZIP contents verified | [Tests](../../../target/implement-130-139/package-tests.log), [package](../../../target/implement-130-139/final-package.json) |
| Frontend lint, types and production build | Passed | [Lint](../../../target/implement-130-139/final-lint.log), [types](../../../target/implement-130-139/final-typecheck.log), [build](../../../target/implement-130-139/final-native-build.log) |
| Rust formatting, locked compilation and precompilation | Passed | [Check](../../../target/implement-130-139/final-rust-check.log), [precompile](../../../target/implement-130-139/final-rust-build.log) |
| Backend execution | 218 passed; hard 60-second limit | [Metadata](../../../target/implement-130-139/final-rust-verification.json) |
| Native batches #130–#139 | All ten passed exact lookup, trusted save, restart/reopen | [Execution](../../../target/implement-130-139/native-execution-verification.json) |
| Closed `core-129-v2` upgrade | Passed; prior references, digests, notes, examples, FSRS, audio relationships and byte-identical closed baseline retained | [Results](../../../target/issue-139/native-upgrade/results.json) |
| Prior raw records and compiled payloads | All 2,435 groups preserved | [Preservation](../../../target/implement-130-139/preservation.json) |

All eleven native runs matched the recorded executable, retained the hard 60-second limit, recorded zero external WebView requests and left zero owned processes. [Complete verification metadata](../../../target/implement-130-139/verification-final.json) retains the measured outputs. Native storage assertions establish the save/reopen and upgrade behavior; the renderer request denial does not establish whole-machine offline execution. Resource updates require the matching executable because its reader verifies the compiled resource identity. Learner data is unchanged; application/resource rollback requires the matching old executable and core, preserving the existing data directory and backups.

## Remaining gates

Full D2 and #78 remain open. Learner acceptance, whole-machine offline denial, other-platform native execution, CEFR and formal release are unverified. [PR #209](https://github.com/zardonc/lexique/pull/209) implements only #130–#139 and remains unmerged.
