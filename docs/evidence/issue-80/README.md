# D2 preservation through resource faults and learning backup restore

[Issue #80](https://github.com/zardonc/lexique/issues/80) passes at the Windows
x64 native boundary: lookup, save, details, review, export, preview, restore and
restart retain the integrated [#79 core](../issue-79/README.md) and existing
learning content. The change adds regression coverage and native evidence.

Independent SQLite reads compare record IDs, frozen references and digest
arrays, example indices and sources, notes, save receipts, cards, FSRS progress
and history. Resource failures and replacement also compare selected/previous
audio, ownership, protection and physical-version relationships. Fault runs
compare the retained audio file bytes. FSRS memory and model parameters compare
at their exact declared `f32` values; nullable example sources use the portable
snapshot representation.

## Application and resource identity

The application code is the merged #79 base
`2c2d0ca510f03135073a4a873fe305b855d45d36`. The Windows x64 executable was built
with `cargo build --locked --release -p lexique-desktop --features tauri/custom-protocol`.
It is **29,782,528 bytes**, SHA-256
`56ad4ff2e82ece52512c154e583dcdc7a7b55ee3b9f76e1140bc285defe91cac`.

The production core remains format 1, version `pilot-79-v1`, **77,824 bytes**,
SHA-256 `2d4c2c7bd982146832f353f9ad4353d936c8ebf39dd4b0f718a0c863e6c34db3`.
Synthetic variants have separate correctly bound binaries. Their temporary
`build.rs` overlays change only `LEXIQUE_CORE_SHA256`; the original file was
restored byte for byte. Exact build commands and artifact identities are in the
[build manifest](../../../target/issue-80/build-manifest.json).

| Resource | SQLite bytes | Tested condition |
| --- | ---: | --- |
| Production | 77,824 | Existing twelve-group pilot |
| Newer | 81,920 | Changed source version and noun definition, meanings and use |
| Count limit | 1,007,616 | 201 matching senses for `bien` |
| Byte limit | 5,824,512 | 41 individually valid large senses exceeding 4 MiB |
| Unsupported | 77,824 | Correctly bound resource format 2 |
| Long content | 241,664 | 20,000-character definition, EN/ZH meanings, use and source span |

## Native matrix

The [runner](../../../apps/desktop/e2e/native-d2-pilot-check.mjs) uses real
WebView2, native IPC, SQLite, system file pickers and fresh owned package/data
directories. All fourteen cases passed. Each retained the **60-second hard
execution limit**; the longest observed run was **29.89 seconds**. Every run
ended with zero owned hosts, workers and helpers, and zero external WebView
requests.

Accepted runs are under [target/issue-80/native](../../../target/issue-80/native).
Each contains its exact command and binary identity in `run-metadata.json`,
before/final state and assertions in `results.json`, lifecycle events and
`cleanup.json`.

| Case | Passed result | Accepted run |
| --- | --- | --- |
| Missing resource | Distinct failure, alternate tab, duplicate/retry and resource-absent edit; core and legacy data/audio retained | `missing-07` |
| Corrupt resource | Unreadable SQLite rejected; the same preservation comparisons pass | `corrupt-01` |
| Changed future bytes | Format alteration violates the compiled binding and reports `corrupt` | `future-01` |
| Unsupported resource | Correctly bound format 2 reports `unsupported`; alternate provider and frozen content remain usable | `unsupported-01` |
| Newer resource | Current version difference is visible; old meanings, associations, history, receipts and audio stay frozen | `newer-01` |
| Count budget | 201 results report `limit` and refuse a new save while retaining existing learning | `count-limit-01` |
| Response budget | Oversized content reports `limit` with no partial result or new record | `byte-limit-01` |
| Long content | Complete native results, resource-absent details and both review backs; selected audio survives normal restart | `long-review-01` |
| V2 export | Seven records, twelve complete senses/provenance, sourced examples, four cards, history, receipts and two selected/previous audio relationships | `export-long-04` |
| V2 restore | Preview, confirmation cancellation, restore/restart and committed replay preserve the extended contract and learning/audio | `restore-long-05` |
| Genuine V1 restore | Existing single-translation content, operations, progress/history, audio and catalogue identities restore; replay, edit and new core save pass | `legacy-01` |
| Future backup | Snapshot version 3 fails preview before replacing current learning, audio or the original archive | `reject-future-01` |
| Inconsistent backup | Container/snapshot version mismatch fails with current data retained | `reject-mismatch-01` |
| Malformed extended field | Empty core Chinese meaning fails before replacement; current records/files remain | `reject-malformed-01` |

The exported ZIP format 1 / logical snapshot 2 archive is **235,810 bytes**.
The genuine v1 archive is the existing #75 native `export-A-03/A.lexique-backup`,
**100,595 bytes**; its original path is recorded in the
[legacy command](../../../target/issue-80/native/legacy-01/run-metadata.json).

## Actual lengths and budgets

Lengths count Unicode scalars. The selected fields stay complete; the reader
retains its 20,000-character field bound, 200-sense / **4,194,304-byte** response
budget and **16,000,000-byte** resource budget. Rust checks reject a
20,001-character definition and an oversized response without modifying the
resource. The largest successful production response observed here is
**11,173 bytes**; the long fixture response is **171,039 bytes**.

| Field | Production maximum characters | Long noun characters |
| --- | ---: | ---: |
| French definition | 195 | 20,000 |
| IPA | 12 | 5 |
| English meaning | 169 | 20,000 |
| Chinese meaning | 52 | 20,000 |
| Selected use | 168 | 20,000 |
| Source span | 80 | 20,000 |

Full measurements, per-case counts, response sizes and native round-trip timings
are in [measurements.json](../../../target/issue-80/measurements.json).
The [browser regressions](../../../apps/desktop/e2e/core-preservation.spec.ts)
also check full long text and wrapping, a late newer response beside the frozen
digest, and navigation while a lookup is pending without replacing unsaved notes.

## Automated checks

PowerShell 7, pnpm 11.25.0, Rust 1.99.1 and uv-selected Python 3.14.10 were used.
Backend compilation ran separately from execution. Full logs are retained under
[target/issue-80](../../../target/issue-80).

The count and byte-budget fixtures insert their 240 rows in two committed
transactions, avoiding per-row durable writes during backend execution.

| Command | Result | Log |
| --- | --- | --- |
| `pnpm install --frozen-lockfile` | Passed | `install-short-root.log` |
| `pnpm lint` | Passed | `lint-final.log` |
| `pnpm typecheck` | Passed | `typecheck.log` |
| `pnpm build` | Passed | `frontend-build.log` |
| `pnpm test:e2e` | 390 passed | `ui-full.log` |
| `pnpm rust:fmt` | Passed | `ci-166/fixed-fmt.log` |
| `pnpm rust:check` | Passed | `ci-166/fixed-check.log` |
| `pnpm rust:test:build` | Passed | `ci-166/fixed-build.log` |
| `cargo test --locked -p lexique-dictionary --test core_pilot` | 3 passed; 0.68 seconds execution | `ci-166/fixed-core-tests.log` |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` | Passed | `desktop-tests-build-final.log` |
| `pnpm test:rust` | 194 passed in 32.52 seconds; hard 60 seconds | `ci-166/fixed-tests.log` |
| `pnpm test:desktop:lib` | Eight partitions; 96 passed, two existing ignored entries; each hard 60 seconds | `desktop-lib.log` |
| `pnpm test:speech:cache` | 11 passed, one existing ignored entry; hard 60 seconds | `speech-cache.log` |
| `pnpm test:docs` and `git diff --check` | Passed; 168 documents and 1,357 local links checked | `ci-166/docs.log`, `ci-166/diff-check.log` |

## Verification boundary

The primary evidence is native Windows x64 execution and independent persisted
state. Browser coverage uses the approved adapter seam. External WebView routes
were denied after startup and after restore restart; whole-machine network
denial remains deferred. Other-platform hands-on checks remain unrun. Current CI
results are tracked in [PR #166](https://github.com/zardonc/lexique/pull/166).

Synthetic variants remain verification fixtures. Resource coverage remains
**12 / 4,087 groups**, with **4,075 incomplete groups**. Complete resource
admission, learner acceptance and M5 release gates remain open in
[parent #78](https://github.com/zardonc/lexique/issues/78).
