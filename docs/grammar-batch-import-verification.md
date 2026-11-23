# Grammar CSV batch import verification (#215)

Ticket #215 extends #214 through the existing file session, mapping, preview,
selection, transaction, receipt and card lifecycle. Markdown (#216) and ZIP
(#217) are excluded. No dependencies, schema or snapshot format changed.

## Source and environment

Verified November 23, 2026, Windows x64, PowerShell 7, MSVC, pinned pnpm 11.25.0
and Rust 1.99.1. Worktree:
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/author-215`, branch
`codex/grammar-exchange-215`, created from integration
`29dd1fa1abf4a170ce75dc6f23ba7490a14c28e3` (completed #213/#214).

Initial implementation: `e473df12c402bec41773cae071bd2ec3b0f88ef7`.
Final application, Rust, native harness and packaged binary source:
`6cc059e2b88bea8acfbb4a6f240810bfcd89ff25`. The latter binds receipts to selected
source groups, including independent equal identifier-free rows. Subsequent
documentation-only commits do not change the tested program. Final clean handoff,
tree, integration ancestry, executable hash and exact commands are recorded in
`handoff.json` and `lineage.json` in the evidence archive.

The tested source tree is `0f0394e5440fb1025eea78d834e7baabbda20cd6`.
Final executable SHA-256:
`99218f46b3be407414cf8b03dd814fb227da585f1ebeee0c9abc5fe5f20e2f65`.
Final portable ZIP SHA-256:
`7e313e6c355c3df72828265739b2d02b025a045b8fcded071872c2e5b9ffd3cc`.
Merging the latest local integration before handoff reported already up to date;
the specified integration head is an ancestor.

Dependencies, node_modules, Cargo target and package outputs are local to this
worktree. Prepared dictionary/conjugation/frequency resources were copied from
the #214 donor and validated; core resources were prepared locally. Builds and
mutable dependencies were not shared. Fixtures use disposable learning data.

## Commands and results

| Command | Result and retained log |
| --- | --- |
| `pnpm install --frozen-lockfile` | Passed, `install.log`; lockfile unchanged |
| `pnpm lint` | Passed; `lint-final.log`, `lint-binding-final.log` (tree committed as final source) |
| `pnpm typecheck` / `pnpm build` | Passed at e473; `typecheck.log`, `build.log`; existing Vite size warning. Frontend unchanged by final receipt fix |
| `pnpm test:e2e -- grammar-import.spec.ts grammar-import-batch.spec.ts grammar-export.spec.ts note-import.spec.ts note-import-batch.spec.ts note-import-dictionary.spec.ts --workers=2` | 26 passed, `ui-regression.log`; browser fixtures prove UI behavior, not native storage |
| `pnpm rust:fmt` / `pnpm rust:check` | Passed at 6cc; `final-rust-fmt.log`, `final-rust-check.log`; locked workspace check |
| `pnpm rust:test:build` then `pnpm test:rust` | Passed at 6cc; 256 tests; `final-rust-test-build.log`, `final-rust-tests.log` |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` then `pnpm test:desktop:lib` | Passed at 6cc; 159 tests across eight bounded partitions; two intentionally ignored child entrypoints exercised by parents; `final-desktop-test-build.log`, `final-desktop-tests.log` |
| `pnpm desktop:build --no-bundle` | Passed at clean 6cc; `native-build-final.log`; existing dead-code warnings retained |
| `python -B -X utf8 scripts/package_portable.py --output-dir target/ticket-215/package-final` | Passed, 186 verified files; `package-final.log` |
| `pnpm test:docs` / `git diff --check` | Passed; `docs-final.log`, `diff-check-final.log` |

Rust compilation was separate from execution. The existing hard 60-second Rust
runner and per-scenario native timer remain unchanged. `checks.jsonl` records
commands, source SHAs, timestamps and exit status. Successful unchanged checks
were not repeated after documentation changes.

## Packaged native acceptance

All **13 scenarios passed** against the final 6cc executable and harness, with
real Win32 pickers, Tauri IPC, SQLite readback, fixture data and unchanged
60-second per-scenario limits. `native-runs.jsonl` stores the exact invocations:
from `apps/desktop`, `node e2e/native-interchange-check.mjs <package-directory>
<fresh-executable> <fixture-directory> <scenario> [source-fixture-directory]`.
`run-native.ps1` retains the worktree-local package and executable arguments.

| Scenario | Passing evidence directory |
| --- | --- |
| `grammar-batch-mapping` | `native-mapping` |
| `grammar-batch-learning` | `native-learning` |
| `grammar-batch-stale` | `native-stale` |
| `grammar-batch-occupied` | `native-occupied` |
| `grammar-batch-failure` | `native-failure` |
| `grammar-batch-replay` | `native-replay` |
| `grammar-batch-restart` | `native-restart` (copied closed replay fixture) |
| `grammar-batch-selection` | `native-selection-retry` |
| `grammar-import` | `native-single-retry` |
| `grammar-import-known` | `native-known` (copied closed single fixture) |
| `grammar-seed` | `native-seed` |
| `grammar-vocabulary` | `native-vocabulary-retry` (copied seed fixture) |
| `grammar-backup` | `native-backup` (backup, restore and controlled restart) |

Each directory retains `results.json`, clean source metadata, actual picker files,
SQLite state, screenshots, lifecycle and `cleanup.json`. All owned host, worker
and helper counts are zero. The first `native-selection`, `native-single` and
`native-vocabulary` attempts failed when WebView2 exited with access violation
`0xc0000005`; their dumps locate the exception address in
`RTSSHooks64.dll + 0x1490af`. `crash-analysis.json` and `dump-module.py` document
the minidump exception/module matching, not a full stack or root-cause analysis.
Each scenario passed in a new fixture with the same binary, source and timeout.
All three failed attempts and dumps are retained; RTSS settings were unchanged,
and a disabled-RTSS comparison was not run.

## Acceptance coverage

| Criteria | Focused evidence |
| --- | --- |
| T17 | Desktop real-file BOM variants, comma/semicolon and external mapping checks; browser mapping errors and remap; native external semicolon picker |
| T18–T21 | SQLite absent/empty body, blank/absent paired fields, complete replacement, single-sided located errors, effective skip and explicit update/skip; native learning updates |
| T22–T24 | Same-identity identical parsed values and presence coalesce; conflicts in both orders invalidate whole group; different presence stays conflicting; identifier-free equal rows stay independent |
| T25 | SQLite second connection and native session revision/identity-occupancy changes reject the entire selected batch; fresh preview succeeds in storage/session tests |
| T26 | Real SQLite trigger fails the second pause write after earlier writes; notes, identities, cards, pause and receipt roll back; exact retry succeeds; lost native commit response replays the original receipt |
| T27 | Existing card IDs, active/paused state, category memberships, progress, due and rating history remain; native SQLite readback; cards added to existing notes are paused atomically |
| T37 | Preview/exclusion and cancellation leave saved state intact and release the session; missing/excluded records are not deleted |
| T38 | Relevant vocabulary CSV/export browser checks plus native vocabulary and full backup/restore compatibility |
| T39 | Real SQLite start failure affects only the chosen card; another card starts, imported content stays saved, failed card remains paused until explicit retry |
| T40 | Browser capability checks remain explicit; packaged Win32 picker/IPC/SQLite evidence is recorded separately |

Red/green evidence includes `storage-red-1.log`, `desktop-red-1.log`,
`ui-red-1.log`, `storage-learning-red.log`, `selection-receipt-red.log` and their
green successors. `native-selection-red` demonstrates that changing the selected
equal identifier-free source row incorrectly replayed before the final binding
fix. That expected failure is retained separately from successful acceptance.

## Evidence and limits

Durable archive:
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/evidence/ticket-215`.
Local originals remain in this worktree's `target/ticket-215`. The archive retains
complete command logs, UI reports/traces, native results/screenshots, real picker
CSV files, fixture SQLite/backup/audio data, lifecycle cleanup reports, both tested
package ZIPs, source delta and handoff metadata. Duplicate fixture executables,
static resources and disposable WebView profiles are omitted; the final package
contains the tested executable and resources. `archive-manifest.json` records the
retained file count and bytes.

Windows x64 only. Focused browser checks were run; the entire browser suite,
other platforms, clean-machine installation, whole-machine offline operation and
release acceptance were not run. Native WebView request interception only proves
the checked WebView context has no external requests. WASM was not repeated:
core and its portable API are unchanged. No push or tracker mutations were made.
