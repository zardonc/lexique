# Category review acceptance (#195 / #201)

The integrated category-review implementation passed the complete browser suite,
storage and desktop checks, and twelve Windows x64 native scenarios. The native
scenarios use the rebuilt application, real SQLite, fresh synthetic learning data,
the existing CDP fixture, native backup dialogs, and the approved local resources.

## Revision and environment

- Issue: [#195](https://github.com/zardonc/lexique/issues/195), acceptance slice
  [#201](https://github.com/zardonc/lexique/issues/201), draft
  [PR #208](https://github.com/zardonc/lexique/pull/208).
- Checkout: `C:\Users\inthe\IdeaProjects\lexique-category-review-195\ticket-201`.
- Required integration ancestor: `c658984141742c18bd54f6864bee6a30899f20d9`.
- Tested implementation SHA: `a42f21e8664887aa899498dea6ae6ecb6d8ec822`;
  clean tree: `9d06c344b0d3f5e986f49094f238544319d38086`.
- Windows x64 `10.0.26300`, PowerShell `7.6.9`, Node `24.22.1`, pnpm
  `11.25.0`, pinned Cargo `1.99.1`, uv Python `3.14.10`, initialized MSVC,
  installed Edge/WebView2. The frozen dependency install preceded execution.
- Evidence root:
  `C:\Users\inthe\IdeaProjects\lexique-category-review-195\ticket-201\target\ticket-201`.
  `verification-run.json` records commands, versions, SHA/tree, timing and logs;
  `native-runs.json` records each final native invocation and artifact directory.

## Complete checks

| Command | Outcome |
| --- | --- |
| `pnpm lint` | Passed |
| `pnpm typecheck` | Passed |
| `pnpm build` | Passed |
| `pnpm rust:fmt` | Passed |
| `pnpm rust:check` | Passed; locked workspace compilation |
| `pnpm rust:test:build` | Passed; separate storage precompilation |
| `pnpm test:rust` | 242 passed, 44.06 s, hard 60 s runner |
| `cargo test --locked --release -p lexique-desktop --lib --test speech_cache --no-run` | Passed; separate desktop precompilation |
| `pnpm test:desktop:lib` | 147 passed in eight bounded groups; includes window and backup checks |
| `pnpm test:speech:cache` | 11 passed |
| `pnpm test:e2e -- review-scope.spec.ts -g 'after rating holds the completed boundary' --workers=1` | Relevant preflight: 1 passed |
| `pnpm test:e2e -- --workers=4` | Complete browser suite: 472 passed, 191.36 s |
| `pnpm desktop:build --no-bundle` | Passed; final native executable rebuilt, 101.96 s |
| Native scenarios below | 12 passed; each preserves the hard 60 s execution limit |

Three Rust subprocess helper entry points are intentionally ignored as standalone
tests; their parent tests exercise them. Desktop groups and speech-cache execution
retain the existing 60 s runner. Initial compilation is separate.
Documentation `git diff --cached --check`, introduced local links and retained
acceptance pointers passed after execution; results are in `doc-checks.json`.

The dedicated browser regression checks two eligible old-scope cards, a pending
different scope, failed scope save after rating or skip, and save-only retry.
The completed-card boundary preserves pending input and prevents the next answer
or a duplicate rating/quota charge. Public storage export/read/import regressions
cover stale-current cleanup to remaining category IDs, Uncategorized, and
`needs_selection`. Incoming malformed or dangling references still reject before
replacement. The combined full suites include these regressions.

## Native T01–T19 evidence

Every scenario below lives under `native-a42f21e/<scenario>` in the evidence root;
the passing All backup run is `backup-all-retry`, and `backup-all` retains the
failed renderer attempt described below.
Its `run-metadata.json` identifies the clean tested source, binary and command;
`results.json` retains queue/window/audio observations and final learning state;
`lifecycle.jsonl` records restarts; `cleanup.json` confirms zero owned hosts,
speech workers or helper processes. Each fixture keeps its `Lexique-data` SQLite
and protected audio. Backup fixtures also keep actual ZIPs, previews and extracted
`snapshot.json`. Resource model duplicates can be removed after cleanup; learning
data and synthesized audio remain.

| Scenario | Acceptance exercised |
| --- | --- |
| `selection` | T01/T02/T04/T05/T07/T08/T09/T10: mixed All vocabulary/grammar; selected recognition/listening; real synthesis, HTML audio playback and rating; pending skip; applied-only restart; stale native cursor; union/overlap and Uncategorized |
| `live` | T11/T12/T13: identity-preserving rename, failed membership save, committed warm-cache edits, finishable current card, partial/final category deletion and restart |
| `undo` | T03/T15/T16: admission before quota, shared daily usage, latest global undo outside scope, original quota refund/replay, pending choice preservation, invalid latest target and restart |
| `diagnostics` | T04/T10/T14: unenrolled, paused, future due, unavailable listening audio and explicit-choice diagnostics retain scope |
| `failure` | T05/T06: real SQLite write locks fail rating/save; answer and pending scope survive; cancel/reselect works; committed rating then failed save after skip retries only scope save; exactly one persisted rating |
| `backup-all` | T17/T18: All roundtrip with pending choice excluded |
| `backup-single` | T17: single category, real synthesized audio and rated progress roundtrip |
| `backup-multiple` | T17: category union plus Uncategorized, identities/memberships restored |
| `backup-uncategorized` | T17: Uncategorized alone restored |
| `backup-needs-selection` | T12/T17: explicit-choice state restored |
| `backup-old` | T18: missing scope defaults to All, replacing destination preference |
| `backup-invalid` | T18: dangling incoming category rejected before replacement; destination learning data/scope preserved |

T08's late responses/obsolete deltas and T16's retry/conflict cases also have
browser/storage coverage in the complete suites. Native evidence establishes
the real IPC/SQLite/runtime behavior listed above. The retained synthetic
`selection/native-category-review.png` shows the applied category/Uncategorized
selector and completed audio status; assertions and persisted records establish
the execution result.

## Repeat the checks

Use PowerShell 7 and the pinned tools from the repository and
`LOCAL-DEV-ENV.md`. Initialize MSVC, put `rustup which cargo`'s directory first
on the process PATH, and put uv's Python 3.14.10 directory on the process PATH.
Install with `pnpm install --frozen-lockfile`. Verify the installed Edge path
before setting `PLAYWRIGHT_EXECUTABLE_PATH`; Playwright owns its server.
Run the commands in the table in order, precompiling before the bounded Rust
execution commands. Rebuild the native application from the same clean committed
implementation used for the full suites.

For native acceptance, prepare a distribution-resource directory containing the
current `dictionary`, `conjugation`, `frequency`, approved `speech`, `LICENSE`
and `README.txt`. This run used this checkout's `target\release` as resource input
and the single-writer shared Cargo cache
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\integration\target`.
No existing learning data is used as fixture input. From `apps/desktop`, run:

```powershell
$taskResources = (Resolve-Path ../../target/release).Path
$taskBinary = (Resolve-Path "$env:CARGO_TARGET_DIR/release/lexique-desktop.exe").Path
New-Item -ItemType Directory -Force -Path ../../target/ticket-201 | Out-Null
$taskRunRoot = Join-Path (Resolve-Path ../../target/ticket-201).Path 'native-fresh'
New-Item -ItemType Directory -Path $taskRunRoot | Out-Null
foreach ($taskScenario in @('selection','live','undo','diagnostics','failure',
    'backup-all','backup-single','backup-multiple','backup-uncategorized',
    'backup-needs-selection','backup-old','backup-invalid')) {
  $taskDestination = Join-Path $taskRunRoot $taskScenario
  node e2e/native-review-scope-check.mjs $taskResources $taskBinary $taskDestination $taskScenario
  if ($LASTEXITCODE -ne 0) { throw "$taskScenario failed" }
}
```

Use a fresh absent destination for each run; the fixture never reuses old results
or data. Retain stdout/stderr in bounded-display logs. Check metadata for the
expected SHA/tree and clean source, the execution limit, and cleanup. The harness
stops only its own hosts, speech workers and helpers, including across restarts.

## Compatibility and retained slice evidence

SQLite schema remains 20. Supported v1–v4 snapshots without `review_scope` default
to All. A pre-scope strict v4 reader rejects the entire new scope-bearing backup.
Rollback requires a supported pre-scope backup and separately preserved current
data. Pending scope edits are transient until applied; they are absent from restart
and backup persistence.

Prior slice evidence remains at
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\evidence\ticket-196`
through `ticket-200`; removed implementer worktrees are not evidence locations.
Those results retain their own tested revisions. #200's retained handoff/evidence
records old-reader compatibility; this acceptance does not relabel earlier runs.

## Limitations and failed checks

Other platforms, clean-machine installation, whole-machine offline operation and
formal release acceptance remain unrun. The native fixture sets its WebView
offline and uses local speech/resources; this does not prove complete OS network
isolation or device-independent audible output. Playback evidence is the actual
native audio element and application rating eligibility.

Two attempts had WebView2 access violations, each with exception `0xc0000005`:

- Exploratory undo: `native-undo-crash-analysis.json` identifies
  `RTSSHooks64.dll + 0x1490AF`; dump remains under
  `exploratory-undo-1/Lexique-data`.
- Final All backup's first attempt: `native-backup-all-crash-analysis.json`
  identifies `RTSSHooks64.dll + 0x3B1D0`; dump, lifecycle, failed log and cleanup
  remain under `native-a42f21e/backup-all` and `native-backup-all.log`.

The complete final undo and fresh All backup retry passed. A controlled
RTSS-disabled comparison remains unrun, so these failures remain a material
environment limitation. No owner exclusion from #192 is applied here. Final
native evidence records all attempts, including the failed All backup run.

The baseline combined CI jobs failed the stale-current export assertion; raw/plain
logs remain in integration `target/spec-195/ci-logs/137036769792.log`,
`137036760205.log`, and `137036769758.log`. The public normalization roundtrip and
incoming-validation checks pass in this Windows suite; CI at the acceptance
revision and execution on other platforms remain unrun.
