# D2 frozen batches #140–#149

`core-149-v1` admits **499 new groups / 1,996 fields** at frozen rows 2951–3450, bringing the offline core to **3,431 groups / 13,724 fields**. The fixed denominator remains 4,087 groups / 3,773 headwords: 656 groups remain unaccepted; 3,428 everyday groups are complete and 659 remain incomplete. There are three separately counted specialist groups and 19 zero-field annotations / 76 missing annotation fields.

The owner approved exact row 3128 `pars/NOM` as the masculine specialist golf plural, IPA `/paʁ/`, and row 3098 `damoiseau/NOM` as a zero-field annotation. Frozen identity, order and both frequencies remain unchanged. All admitted fields passed independent quality and selected-expression rights reviews; no unresolved admitted fields remain.

| Batch | Frozen rows | Admitted groups / fields | Annotations | Specialist groups |
| --- | --- | --- | --- | --- |
| [#140](../issue-140/README.md) | 2951–3000 | 50 / 200 | 0 | 0 |
| [#141](../issue-141/README.md) | 3001–3050 | 50 / 200 | 0 | 0 |
| [#142](../issue-142/README.md) | 3051–3100 | 49 / 196 | 1 | 0 |
| [#143](../issue-143/README.md) | 3101–3150 | 50 / 200 | 0 | 1 |
| [#144](../issue-144/README.md) | 3151–3200 | 50 / 200 | 0 | 0 |
| [#145](../issue-145/README.md) | 3201–3250 | 50 / 200 | 0 | 0 |
| [#146](../issue-146/README.md) | 3251–3300 | 50 / 200 | 0 | 0 |
| [#147](../issue-147/README.md) | 3301–3350 | 50 / 200 | 0 | 0 |
| [#148](../issue-148/README.md) | 3351–3400 | 50 / 200 | 0 | 0 |
| [#149](../issue-149/README.md) | 3401–3450 | 50 / 200 | 0 | 0 |

## Resource and preservation

[Canonical inputs](../../../resources/core-pilot-79/README.md), [notices](../../../resources/core-pilot-79/NOTICES.md), fixed source versions/spans, contributor/history links, selected original-expression assessments and [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING) accompany the package. Conversion requires no network access. The 15,966,208-byte SQLite resource remains within the existing 16,000,000-byte native limit.

All prior 2,932 groups / 11,728 fields, source arrays and compiled sense payloads remain unchanged. Frozen membership, 15 spelling relationships and 11 runtime aliases are retained. The fresh-Git LF inputs reproduce the same SQLite bytes as the prepared resource.

## Verification

The Windows x64 executable was built from `906acc0a477e5d035bc2227e54245b7ae4255416`. SHA-256: `41a5c1dd3dd3e5229bf43cf988d8257e540a5e35501304b46640be2647e46e2b`. Core SHA-256: `b292e46abba5494970a95fed65714eddc9333473d5aae7875c1bb0e082a5066f`. The upgrade fixture ran at `25489e6c59ffd8b1bcbe4d6ec2dc9105eee59e6e`; application runtime and resource inputs are unchanged from the build. [Final metadata](verification-final.json) records identities, native checks and coverage.

| Check | Result | Evidence |
| --- | --- | --- |
| Core conversion | 18 passed | [Log](../../../target/spec-140-149/checks/core-final.log) |
| Ten content/resource batches | 88 passed; two admission-only checks run separately | [Batch log](../../../target/spec-140-149/checks/batches-final.log), [#142: 11 passed](../../../target/spec-140-149/checks/batch-142-admitted-green.log), [#146: 10 passed](../../../target/spec-140-149/checks/batch-146-admitted-green.log) |
| Portable package | 16 passed; ZIP verified, 186 files | [Tests](../../../target/spec-140-149/checks/package-final.log), [Package](../../../target/spec-140-149/package-notices-green.json) |
| Lint, types, frontend build | Passed | [Lint](../../../target/spec-140-149/checks/lint-upgrade-fixture.log), [Types](../../../target/spec-140-149/checks/typecheck.log), [Build](../../../target/spec-140-149/checks/build.log) |
| Native fixture directory admission | Passed | [Log](../../../target/spec-140-149/checks/native-fixtures-final.log) |
| Rust formatting, precompilation and execution | Passed; 242 tests, unchanged hard 60-second runner | [Format](../../../target/spec-140-149/checks/rust-format-final.log), [Precompile](../../../target/spec-140-149/checks/rust-precompile-fixture-final.log), [Tests](../../../target/spec-140-149/checks/rust-final-green.log) |
| Windows release build | Passed | [Build](../../../target/spec-140-149/checks/native-build-final.log) |
| Exact native lookup/save/reopen | All ten batches passed | Per-batch `final-integration.json` records linked above |
| Closed `core-139-v1` upgrade | Passed | [Results](../../../target/issue-149/upgrade-from-core139-final/results.json) |
| Prior content and Git material reproduction | Passed | [Preservation](../../../target/spec-140-149/preservation-final.log), [Git conversion](../../../target/spec-140-149/checks/git-eol-green.log) |

The upgrade retains previous references, digests, notes, examples, FSRS state and current/protected audio relationships. Its fixture explicitly selects English while retaining the previous theme. All eleven successful native runs verify database integrity, deny external WebView routes after startup, record zero external requests and leave zero owned processes. Complete ignored logs, screenshots and profile snapshots remain under the integration checkout's `target/`; source/reviewer archives remain under sibling `context/`.

The final package rebases batch-document links in the combined notices to their fixed source-review revision. Compared with the native-tested package, only `NOTICES.md`, its input checksum and resource report changed; all executable, SQLite, renderer and other material bytes are identical. The packager verified the complete final ZIP and its material checksums. [Documentation/link checks](../../../target/spec-140-149/checks/docs-links-green.log) passed 512 documents / 3,988 local links.

## Rollback and remaining gates

Resource updates and rollback require a matching executable/core pair because the native reader verifies the compiled resource identity. Preserve `Lexique-data` and backups; existing saved digests retain their original content.

[PR #211](https://github.com/zardonc/lexique/pull/211) implements #140–#149 and awaits review. #78 remains open. Learner acceptance, whole-machine offline denial, other-platform native execution, full D2 / CEFR and formal release acceptance are unverified.
