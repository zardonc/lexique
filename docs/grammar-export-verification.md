# Grammar CSV export verification (#213)

Ticket #213 delivers grammar CSV export and persistent exchange identity only.
Schema 21 assigns identities to existing and new grammar records; snapshot v5
preserves them through full restore and continues reading supported v1–v4 snapshots.
Import and grammar Markdown/ZIP remain subsequent slices of #212.

## Source and evidence

Verified on Windows x64 in
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/author-213`, branch
`codex/grammar-exchange-213`, based on
`ff8aae2bdaf36096c6600d5d29342745b97c0eb7`.
Execution used the uncommitted ticket implementation. Native `run-metadata.json`
records that base SHA, dirty tree, executable identity and platform. The evidence
root's `handoff.json` maps the final committed source to these runs; retained
`tested-source.patch` captures the final change from the base.

Complete local evidence remains at
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/author-213/target/ticket-213/`.
Keep this directory when copying evidence before removing the worktree.

## Checks

| Command | Result and retained log |
| --- | --- |
| `pnpm install --frozen-lockfile` | Passed; `install.log`, no lockfile changes |
| `pnpm lint` / `pnpm typecheck` | Passed; `lint.log`, `typecheck.log` |
| `pnpm desktop:build --no-bundle` | Passed, including production frontend build; `native-build.log` |
| Portable packaging | Passed, 186 verified files; `package.log` |
| Playwright `grammar-export.spec.ts note-export.spec.ts grammar-notes.spec.ts` | 16 passed; `browser-regression.log` |
| Playwright `notebook.spec.ts -g "browser notebook clearly reports unavailable native persistence"` | 1 passed; `browser-capability.log` |
| `pnpm rust:test:build` | Passed, compilation separate; `rust-test-build-final3.log` |
| `pnpm test:rust` with `RUST_TEST_NOCAPTURE=1` | 246 passed; `rust-tests-final3.log`, unchanged 60-second runner |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` | Passed; `desktop-compile-final3.log` |
| `pnpm test:desktop:lib` | 148 passed; `desktop-tests-final2.log`, eight existing bounded partitions |
| `pnpm rust:fmt` | Passed; `rust-fmt-final.log` |
| `pnpm rust:check` | Passed, locked workspace compilation; `rust-check-final.log` |
| `pnpm test:docs` / `git diff --check` | Passed; `docs-final3.log`, final diff check |

The storage seam covers migration, independent same-title identities, rename,
saved scopes/order/content, identity validation and old snapshot restore. The
desktop exchange seam writes actual CSV files from real SQLite, checks exact
Unicode/whitespace/multiline pairs and five headers, verifies cancellation, induces
an actual write error by selecting a directory, retries successfully and compares
the full learning snapshot. The UI seam covers shared scope selection, saved
content and visible failure/cancellation results.

TDD evidence: `red-identity.log` fails at the missing capture API and
`green-identity.log` passes; `red-ui.log` reaches the grammar UI and fails on the
missing export action, then `green-ui.log` passes. `red-csv.log` is an environment
failure, not a behavioral red. Earlier failed logs remain retained separately;
legacy fixture corrections remove schema 21 objects before resetting to the
historical schema version. Production migration strictness is unchanged.

## Native acceptance

Each command ran from `apps/desktop`, using the existing
`node e2e/native-interchange-check.mjs <portable-dir> <fresh-exe> <evidence-dir> <scenario> [<source-dir>]`
driver and its unchanged 60-second scenario budget. Portable resources are under
`target/ticket-213/package/Lexique-0.1.0-windows-x64-portable`; the executable is
`target/release/lexique-desktop.exe`.

| Final scenario / evidence directory under `native/` | Passed acceptance |
| --- | --- |
| `grammar-seed` / `seed-01` | Three independent equal-content records, rename identity, categories, real cards/rating/history and pause state (T01–T02) |
| `grammar-csv` / `csv-02` | Real schema 20 upgrade; Win32 save picker; all/search/single CSV, counts/order, accents/Chinese/quotes/multiline/whitespace, independent pairs, no-card blanks, saved content despite draft; cancel then save; learning unchanged (T01–T05, T08, T37) |
| `grammar-backup` / `backup-03` | Real v5 archive preview, full restore and controlled restart preserve exchange/local identities and learning data (T06, T38) |
| `grammar-legacy` / `legacy-01` | Actual v4 archive without identity collection previews/restores/restarts, retains learning data and assigns persistent identities (T07) |
| `grammar-vocabulary` / `vocabulary-02` | Reopen legacy-restored library, retain assigned identities, export existing four-field vocabulary CSV without changing learning (T07, T38) |

Each directory retains `results.json`, `run-metadata.json`, real files/SQLite,
screenshots, lifecycle evidence and `cleanup.json`; all five final cleanup records
report zero owned hosts, workers and helpers. Matching top-level native logs retain
the successful scenario outcome. Later changes only corrected test fixtures and
documentation; the application tested natively is unchanged.

## Limits

The WebView network context was offline; whole-machine offline acceptance was not
run. Other-platform GUI, clean-machine installation and release readiness were not
verified. Real filesystem failure is proven at the desktop Rust seam; GUI failure
presentation uses IPC fixtures rather than an induced native picker write failure.
Browser fixtures prove UI behavior and honest unavailable persistence (T40), not
native storage. T39 concerns post-import enrollment and does not apply to #213.
Old applications cannot open schema 21 or read snapshot v5: rollback requires a
compatible pre-upgrade backup and separate old data directory, retaining new data.
