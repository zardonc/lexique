# D2 pilot #79 native evidence

The twelve-group [pilot #79](https://github.com/zardonc/lexique/issues/79)
provides French definitions, IPA, English and Chinese meanings through the
dedicated `builtin-core` dictionary, native saving, notebook details, review
backs and versioned learning backups. It contains twelve reviewed senses and
48 fields. Coverage is **12 / 4,087 groups**, with **4,075 incomplete groups**
against **3,773 headwords**. The complete release remains
[parent #78](https://github.com/zardonc/lexique/issues/78).

## Resource and build identity

The Windows x64 production executable was built from
`5e50e01d9b01fcb4aa2daefe74ec5c2d549f44e2` with
`pnpm desktop:build --no-bundle`. Its SHA-256 is
`4d755c54e80a65de3061e6df964b4ac783c9100fedb11fb8d74adcf397c16c33`.
The production database is `dictionary/core-pilot-79/core.sqlite3`, format 1,
content version `pilot-79-v1`: **77,824 bytes**, SHA-256
`2d4c2c7bd982146832f353f9ad4353d936c8ebf39dd4b0f718a0c863e6c34db3`.

[Curated inputs](../../../resources/core-pilot-79/curated-inputs.json),
[independent field review](../../../resources/core-pilot-79/field-review.json),
[selected source excerpts](../../../resources/core-pilot-79/selected-source-excerpts.json)
and [notices](../../../resources/core-pilot-79/NOTICES.md) bind every field to
its source revision, contributor history, selected span, licence and recorded
adaptation. Selected content and membership use CC BY-SA 4.0; application and
converter code retain MIT. The complete CC BY-SA legal text is 20,138 bytes.
Preparation verifies the produced database against the separate accepted
review. Standalone reproduction yields the same database bytes. The prepared
`coverage.json` lists each incomplete group and its four missing fields.

Both actual portable packages passed independent checks of their admitted
fields/use, payload/provenance, legal text, source materials, notice links and
coverage denominator:

| Development package | ZIP bytes | Verified files | SHA-256 |
| --- | ---: | ---: | --- |
| Default | 35,133,135 | 183 | `2ee7b44b3b80f94e3f7ca4fdaaa56502b91b5a01b624d663592ed02511778e5e` |
| With speech | 196,231,723 | 1,245 | `01f80b619aadb6bd0511c59a205ffd032cc7cf8c6358019d012f231abc7e07c2` |

## Native execution

Runs use the actual executable, native IPC, SQLite, system file pickers and
isolated package/data directories. The
[native runner](../../../apps/desktop/e2e/native-d2-pilot-check.mjs) retains the
existing **60-second hard execution/cleanup limit**. Each destination records
its command, binary identity, calls, assertions, process/restart events and
owned-process cleanup.

| Scenario | Result | Native run under `target/issue-79/native/` |
| --- | --- | --- |
| Twelve groups / 48 fields, explicit provider, forged digest/reference rejection, NFC and evidenced spelling bindings, identical retry, duplicate, notes, examples, rating and normal restart | Passed | `pilot-03` |
| Both review backs, selected example meaning and real synthesized/cached audio after restart | Passed | `review-02` |
| Actual default package without speech; dictionary content remains readable and generation fails visibly | Passed | `no-speech-01` |
| Fifteen native MDX imports, 18 total entries, capacity rejection and upgrade order/file binding preservation | Passed | `catalogue-02` |
| Missing, corrupt and unsupported replacement resources; other provider, committed retry, note edit and saved content remain usable | Passed | `missing-01`, `corrupt-01`, `future-01` |
| Export all twelve senses and provenance through the native picker; ZIP v1 / logical snapshot v2 | Passed | `export-03` |
| Native v2 preview, whole restore, restart and complete saved-record comparison | Passed | `restore-01` |
| Genuine existing v1 backup, old translations/examples/operations/cards/ratings/audio, edit and new core save | Passed | `legacy-01` |
| Future, mismatched and malformed core backup rejection preserves current learning and archive bytes | Passed | `reject-future-01`, `reject-mismatch-01`, `reject-malformed-01` |
| Newer resource version preserves the previous digest, progress, selected audio and committed retry | Passed | `newer-02` |
| Actual 201-result native core limit; source-dependent new save fails and committed content remains readable | Passed | `limit-02` |

All fifteen successful runs recorded zero remaining owned hosts, workers and
helpers. The complete run index, including retained failed attempts, is
`target/issue-79/native-run-index.json`.

Lookup fields and provenance are compared with the independently accepted
curation, then saved digests are compared with actual native lookup. French,
English and Chinese interface labels, POS labels, language attributes and full
field-source text pass. The saved noun example remains associated with the
noun sense of `être`. Native screenshots cover the three lookup interfaces,
notebook details and both review backs.

New-install catalogues put the core first. Upgrades append it while preserving
existing relative order and both FreeDict identities. New saves re-read the
explicit trusted provider. Note edits and committed retries retain their
saved snapshot. The logical snapshot writer uses version 2; strict readers
accept versions 1 and 2, while the ZIP format remains version 1. Existing
SQLite snapshot JSON carries the extension without a SQL migration.

The newer and result-limit cases use separately compiled fixture executables
whose normal expected resource binding matches the copied fixture database.
Their archived `build.rs` overlays change only that binding. The production
executable and validation remain separate. The limit fixture has 201 matching
`bien` senses; its copied closed profile already contains `être`.
The future-format copy fails at the established resource digest binding;
reader format validation is also covered by the backend tests.

Both fixture executables were built from `5e50e01` with
`cargo build --locked --release -p lexique-desktop --target x86_64-pc-windows-msvc --features tauri/custom-protocol`.
Their identities and binding overlays are retained in `build-fixtures/`:

| Fixture executable | Bytes | SHA-256 |
| --- | ---: | --- |
| `newer-production.exe` | 29,780,992 | `ed0f07a0f11f037b40934c201c91b6f15eb44099b6edaf1df619dab74da9f5ce` |
| `limit-production.exe` | 29,780,992 | `4ef3f43df0bea0ccdac1ed9b749c95f44ddfcdb3bb0b1cf4ecfec218658ce798` |

## Query sizes and latency

The first seven `pilot-03` lookups produced the following samples. Time is the
measured **CDP + IPC round trip**, including client/native communication, rather
than an isolated database benchmark.

| Query | JSON response bytes | Round trip ms |
| --- | ---: | ---: |
| `être` | 11,173 | 10.06 |
| `avoir` | 7,980 | 7.79 |
| `bien` | 6,511 | 7.50 |
| `aujourd'hui` | 3,727 | 11.58 |
| `fort` | 6,643 | 7.05 |
| `coeur` | 3,712 | 6.70 |
| `côte` | 3,602 | 7.85 |

## Repository checks and local evidence

Lint, typecheck, build, Rust formatting and strict core/storage/dictionary
Clippy passed. Backend tests passed after separate precompilation, with the
unchanged hard timeout. Native library source-link, catalogue and backup
partitions passed **2, 12 and 21 tests** respectively. Preparation passed four
checks, including unchanged timestamps and damaged-output repair; portable
packaging passed 16 regressions and the build copier passed two. The integrated
browser subset passed 15 checks and the full Edge suite passed **387 / 387**.
The production native release build and both real packager deliveries passed.

Complete local evidence is retained in the integration worktree at
`C:/Users/inthe/IdeaProjects/lexique/target/d2-pilot-79/integration/target/issue-79/`:
`automated/implementation/validation.md` and its 82 logs, `automated/ui/`,
`logs/browser-full.log`, `production-build.json`, `packages/`,
`resource-checks/`, `review/`, `build-fixtures/`, `native/` and
`native-run-index.json`. These are
machine-local artifacts; the source resource and runner are tracked above.

## Limitations

Network evidence denies external WebView routes after startup. Whole-machine
network denial, clean-machine installation, other-platform GUI execution,
learner acceptance, CEFR coverage and M5 release readiness remain unrun/open.
The native release build retains two pre-existing dead-code warnings in
`backup_preview.rs`. One native picker run recorded a WebView2 renderer access
violation; the fresh export run passed, and the original lifecycle/error log
is retained in `native/export-02/`. Older applications cannot read the newly
written v2 logical backups.
