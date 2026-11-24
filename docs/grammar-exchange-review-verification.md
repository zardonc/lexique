# Grammar exchange review verification (#212)

## Corrected behavior and source

Review base: `ff7d1bdfbe9c534a0bfdbb2961dbe34a469bf9e0`.
The application and native regression driver were tested from clean commit
`6600e66c86f5f9d8be270ebaf3f899ea28d4afd6`, tree
`b8b8c20da1948e40380076b5da929428bd3b1cce`.

Single grammar Markdown saves use the existing identity-aware filename rule,
shared with grammar ZIP export. Vocabulary and grammar ZIP writers share their
Stored archive lifecycle, entry errors, finish and sync handling. CSV, Markdown
and ZIP grammar previews share identity matching and unchanged/different/create
classification, retaining each parser's location diagnostics and grouping.
The Markdown export button has no constant availability prop. Atomic import,
receipt, revision, card and learning-state semantics are unchanged.

## Commands and results

| Check | Result |
| --- | --- |
| `pnpm install --frozen-lockfile` | Passed; pnpm 11.25.0, no dependency or lockfile changes. |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | Passed. |
| Focused Playwright grammar export/Markdown checks | 5 passed with the verified installed Edge executable. |
| `pnpm test:e2e --workers 4` | Complete browser suite: 488 passed. |
| `pnpm rust:fmt`, `pnpm rust:check` | Passed with pinned Cargo 1.99.1 and the MSVC x64 environment. |
| `pnpm rust:test:build`, then `pnpm test:rust` | Separate precompilation passed; 256 core/storage/dictionary checks passed. |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` | Separate desktop precompilation passed. |
| Desktop export and grammar-import filters through `scripts/test-rust.mjs` | 5 export and 11 import checks passed. |
| `pnpm test:desktop:lib` | All eight partitions passed: 167 passed, two subprocess entry tests ignored as intended. |
| `pnpm desktop:build --no-bundle` and portable packaging | Fresh Windows x64 release passed; package readback verified 186 files. |

Storage and desktop execution retain the unchanged hard 60-second runner limit.
Native fixtures retain their own hard 60-second limit. Browser checks establish
UI behavior; native fixtures use the packaged executable, real Win32 dialogs,
real Tauri IPC and isolated SQLite libraries.

## Packaged native evidence

The release executable and packaged `Lexique.exe` have the same SHA-256:
`A3EC1C37A2216217A8277C72B778078E500C9CACCA75DF0E0F5CDB8AEED78967`.
Portable ZIP SHA-256:
`E029E760203CAB0471641F0A021A915BCB9E2BE45194C3B1ADFE5CFC7BBA4BEF`.
`binary-lineage.json` records their paths, source and tree. Each native fixture
independently records the actual command, driver source, clean state and binary.

`review-filenames` created two independent records titled `CON.same/title`.
The actual Win32 filename edit control suggested:

- `_CON.same_title-0ca3bbe49cb0dcc77497baffc7136863-2.md`
- `_CON.same_title-7435d70a0a96b3b5cd48fadd9cf5c69d-1.md`

Both cancelled picker readbacks and subsequent saves are retained. Both files
coexist, contain the correct identity and body, and leave learning state intact.

`review-zip-source` imported three grammar identities, including same-title
records and two paused cards, and exported search/all ZIPs. A fresh independent
`review-zip-destination` imported the all ZIP and exported CSV. All three records'
five content fields matched the incoming CSV (`native-zip-compare.log`).

`review-seed` established grammar learning history, category and pause state.
Vocabulary CSV and Markdown smoke checks ran on separate copies of that library
and preserved its learning state. `review-vocabulary-zip-retry` verified native
vocabulary Markdown defaults, search/all ZIPs, distinct Windows filenames,
extracted exact text and identical reimport without changing saved records.

## Evidence and limits

Complete logs, command scripts, source patches, portable package, native files,
screenshots, SQLite libraries, per-run metadata, lifecycle, failures and cleanup
are retained under `target/review-fixes` and `target/ticket-217/review-*` in the
review-fixes checkout, and archived under
`C:/Users/inthe/IdeaProjects/lexique-grammar-exchange-212/evidence/review-fixes`.
All ten fixture cleanup reports show zero owned hosts, workers and helpers.
Earlier ticket evidence remains untouched.

The retained filename regression reports both generic defaults. The first
vocabulary ZIP attempt encountered
WebView2 `ProcessFailed`, exception `0xC0000005`; its minidump maps the exception
address to `RTSSHooks64.dll + 0x1490AF`. The same packaged binary passed in a fresh
fixture. The failed dump and lifecycle remain archived; no RTSS-disabled control
or full root-cause analysis was performed.

Native WebView contexts were offline; this is not whole-machine offline proof.
Other platforms, clean-machine installation and release acceptance remain
unverified. Previously completed feature acceptance remains in its ticket records.
