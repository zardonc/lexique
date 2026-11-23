# Single grammar CSV import verification (#214)

Ticket #214 adds one canonical comma-separated grammar CSV record through the
existing native exchange session and notebook controls. Optional complete question
and answer content creates a paused card in the same SQLite transaction. Starting
review is a separate explicit action after commit. Known unchanged input skips;
changed known input is explained and cannot overwrite saved content. Column
mapping, multi-record input and selected updates remain #215.

## Checked source and environment

Verified on November 23, 2026, Windows x64, in
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/author-214`, branch
`codex/grammar-exchange-214`. Implementation and fresh native binary source:
`ac17b766414ed598ee02d0eef9b312d4c8b646c2`, tree
`bc91b358cc8d07ee809bce5576bacafa4e86252a`. Frontend and Rust checks ran against
the working tree committed as that source. Native acceptance ran from clean
harness commit `431bd518f5360247085ff2961034a4f267c9c542`, tree
`6993f15ae0a7acaa7f39e46659fd581d9320295d`; only native test code changed after
the binary build. The executable SHA-256 is
`416a5b13067c55b1b772dadc6544a116c6b87880f640c7e60d66c727e1c2cd09`.

Initial integration `d80a604597f10ccf110a5070cd97cdd98865ba46`, #213
`ee5a0aed9c972fbd809da37a2db4becd378a6696`, and latest integration
`bacf7b7306fcbb5f57a9e4825c333d134fd6b4f4` are ancestors. Final integration merge
reported already up to date. No core, schema, snapshot format or dependencies
changed in #214. The parent's WASM verification for the schema assertion fix was
not duplicated.

Pinned pnpm 11.25.0, Rust 1.99.1 and MSVC were used with two Cargo build jobs.
Dependencies, prepared resources, build target and portable package were local to
this worktree. Prepared dictionary/conjugation/frequency candidates were copied
and validated; core resources were prepared locally. No mutable node_modules or
target directory was shared with integration.

## Evidence archive

Durable evidence is at
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/evidence/ticket-214/`.
Original complete local runs remain under this worktree's `target/ticket-214/`.
Paths below are relative to either evidence root. `handoff.json` records final
commits and evidence identities, `lineage.json` records implementation ancestry,
and `tested-source.patch` captures the final delta from integration. The archive
retains one portable package ZIP and executable, command logs, real exchange
files, SQLite data, screenshots, lifecycle/cleanup reports and backup artifacts.
Duplicate fixture executables/static resources and disposable WebView profiles
are omitted; `archive-manifest.json` records retained file hashes.

## Commands and results

| Command | Actual result / log |
| --- | --- |
| `pnpm install --frozen-lockfile` | Passed; `install.log`, unchanged lockfile |
| Dictionary, core, conjugation and frequency prepare commands | Passed; `resources-prepare.log`, `core-prepare.log`, `conjugation-prepare.log`, `frequency-prepare.log` |
| `pnpm lint` | Passed; `lint-final.log`, then `lint-native-final.log` after harness changes |
| `pnpm typecheck` / `pnpm build` | Passed; `typecheck-final.log`, `build-final.log`; existing Vite chunk-size warning retained |
| `pnpm rust:fmt` / `pnpm rust:check` | Passed; `rust-fmt-final.log`, `rust-check.log` |
| `pnpm rust:test:build` then `pnpm test:rust` | Passed; separate compilation, 252 tests; `rust-test-build.log`, `rust-tests.log`, `checks.jsonl`; unchanged hard 60-second execution runner |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` then `pnpm test:desktop:lib` | Passed; 154 tests across eight bounded runs, two deliberately ignored child entrypoints exercised by parent tests; `desktop-test-build.log`, `desktop-tests.log` |
| `pnpm test:e2e -- grammar-import.spec.ts --workers=2` | Five passed; `ui-grammar-final.log` |
| `pnpm test:e2e -- grammar-import.spec.ts grammar-export.spec.ts note-import.spec.ts note-import-batch.spec.ts --workers=2` | Initial combined run: 16 passed, three grammar fixture assertions failed; all 14 existing regression cases passed. The three grammar assertions were corrected and all five grammar cases passed in the focused final run; `ui-regression.log` retained |
| `pnpm desktop:build --no-bundle` | Passed; `native-build.log`, exact binary source above |
| `python -B -X utf8 scripts/package_portable.py --output-dir target/ticket-214/package` | Passed, 186 verified files; `package.log` |
| `node --check apps/desktop/e2e/native-grammar-import.mjs` and interchange driver syntax check | Passed |
| `pnpm test:docs` / `git diff --check` | Passed; `docs-final.log` and final handoff metadata |

TDD evidence: `red-storage.log` fails because the public import API does not yet
exist, followed by `green-storage.log` proving real SQLite persistence, pause,
reopening and explicit start. `red-desktop-build.log` shows the missing CSV seam;
`green-desktop.log` then exposes a real CRLF location defect (row 1 instead of 2),
fixed and covered by the final six desktop grammar tests. `red-ui.log` fails at
the missing UI action, followed by `green-ui.log` and the five final UI passes.
Six new storage tests cover request replay, anonymous separate submissions,
identity/revision conflicts, validation, atomic rollback and preserved active
classified learned-card state. No unchanged backend or WASM checks were rerun
after native-harness-only changes.

## Packaged native acceptance

From `apps/desktop`, every case used this existing driver, a fresh destination,
actual packaged WebView2, real Win32 picker controls, files and SQLite:

```text
node e2e/native-interchange-check.mjs ../../target/ticket-214/package/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/ticket-214/<destination> <scenario> [../../target/ticket-214/<source>]
```

The unchanged hard 60-second limit applies separately to every scenario.
`native-runs.jsonl` records all ten successful exits. Each destination retains
`run-metadata.json`, `results.json`, screenshot, `lifecycle.jsonl`, `cleanup.json`
and real exchanged files/data.

| Scenario / destination | Evidence and acceptance |
| --- | --- |
| `grammar-import` / `native-import` | T09/T13: read-only preview, unseen identity retained, raw Unicode/quotes/CRLF/backticks, uncategorized note and complete pair committed paused, real queue empty |
| `grammar-import-restart` / `native-restart`, source `native-import` | T13/T16: new native process reopens copied persisted library, identity/text/card/pause preserved, real queue excluded until explicit resume, successful rating, real CSV re-export |
| `grammar-import-identifiers` / `native-identifiers` | T10/T12/T14: missing/blank identity, repeated identifier-free file creates independent notes/identities, missing/empty body, absent/blank pair creates no card |
| `grammar-import-validation` / `native-validation` | T11/T14/T37: located invalid title/pair/content limits, disabled save, actual picker cancellation releases native operation, unchanged SQLite |
| `grammar-import-failure` / `native-failure` | T16/T39: real SQLite trigger rejects pause insertion, complete rollback and same-preview retry; postcommit resume deletion failure remains visible, saved content and pause preserved, explicit retry enters queue |
| `grammar-import-replay` / `native-replay-03` | T15: first successful response discarded after real native invocation, UI retries exact operation/selections, same receipt and exactly one paused note/card, SQLite unchanged; conflicting decision rejected, cancelled session unavailable, independent new submission creates another note |
| `grammar-import-known` / `native-known`, source `native-import` | Identical skip preserves all learning state; changed known identity displays saved/incoming content, cannot confirm overwrite |
| `grammar-seed` / `native-seed` | Preservation fixture: vocabulary, three grammar notes, category, active and paused learned cards/history |
| `grammar-vocabulary` / `native-vocabulary`, source `native-seed` | T38: actual vocabulary CSV export/import, existing vocabulary/grammar/learning preserved, provenance and no-card import behavior retained |
| `grammar-backup` / `native-backup`, source `native-seed` | T38: full v5 backup/restore and controlled process restart preserve identities, vocabulary, categories, cards, progress, history and pause |

T15 observes real IPC fetch responses without replacing parsing or persistence.
Only delivery of the first successful commit is changed to a visible transport
error. The already committed preview remains owned by the native session until
cancellation/replacement; retry reaches the request-bound SQLite receipt. It does
not attempt to reuse a cancelled session or reconstruct a submission after app
restart. Storage tests independently reopen the file database and replay receipts.

Two preliminary replay attempts are preserved: `native-replay` revealed that
Tauri's immutable invoke property prevented the initial observer from installing;
`native-replay-02` reached the real lost-response/retry path but compared its
snapshot before the first queue call created queue context. The final observer
uses actual IPC response delivery and snapshots after that initial queue read.
Both were harness defects, with failed-run artifacts and zero-process cleanup
retained. No product change or binary rebuild was needed.

## Boundaries

The five browser cases include T40's actual unavailable-native capability and UI
error/retry behavior; their mocked IPC does not establish native storage. Native
contexts were offline; whole-machine offline, other platforms, clean-machine
installation, full browser suite and release readiness were not verified.
Native import reopening uses a stopped source library copied to a fresh fixture;
same-file close/reopen is also tested by storage tests, and backup restore proves
a controlled actual native restart. Multi-record/update/mapping and subsequent
Markdown/ZIP scenarios remain later slices. All owned native hosts, workers and
picker helpers exited; unrelated processes and worktrees were preserved.
