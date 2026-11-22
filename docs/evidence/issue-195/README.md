# Category review acceptance (#195 / #201)

The read-failure retry correction passed 474 browser tests and the targeted
Windows native failure journey. Earlier native and backend results below retain
their original source revisions.

## Read failure retry correction (PR #208)

After integrating admitted main `7ba54109e55dcb068a23be688a0d478aa68832eb`
(#209), clean merged source `0f51ca974e448004f7823edaab51af263bb5fed2`, tree
`cb3a2bcc75f359480dfe1375a0da3f8d05837d2a`, passed lint, typecheck, build and
all six focused read/save failure regressions. Application, browser fixture/spec
and Rust source match the earlier tested correction below; resources match main
byte for byte. Both native destination whitelists and verification sections were
preserved. The matching locked native build passed in 120.04 s; its fresh SQLite
write-lock failure journey passed in 34.61 s with clean source metadata, SQLite
checks and zero owned processes. Merged logs have the `merged-` prefix in the
same evidence root; fixture metadata/results/cleanup are in integration
`target/ticket-201/read-retry/native-merged-failure`. The full local browser result
below retains its earlier SHA; final integrated CI is recorded on the PR.

Clean tested source: `17657572109e0285885a7b82239f46101ffa6b7e`, tree
`c8ab6d0e6e9a4f2593f8a27a5f31676b3318e74f`, Windows x64. The selector uses the
hook's existing save-error state to choose the failure message and retry action.
Read errors call `retryRead()` even when an applied scope already exists; save
errors continue to retry only the pending save.

| Check | Result |
| --- | --- |
| Focused read/save failure regressions | 6 passed; two new refresh-read cases first failed with no extra read after retry |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | Passed |
| `pnpm --filter @lexique/desktop test:e2e --workers=2` | 474 passed; bundled Chromium, `CI=true`, no browser override |
| `pnpm desktop:build --no-bundle` | Fresh executable passed, 130.84 s; locked Cargo 1.99.1 |
| `node e2e/native-review-scope-check.mjs <resources> <binary> <destination> failure` | Passed, 37.48 s; unchanged hard 60-second execution limit |

The new browser cases exercise `review_scope` and `learning_categories` read
failures after successful loading. They assert the read-error message, reload
button, an actual new read, error clearance and zero scope saves. The native
failure journey verifies existing rating/save failures with real SQLite locks,
preserved input, refresh, save-only retry and unchanged rating/progress/global
usage. SQLite integrity and foreign keys passed; task-owned hosts, workers and
helpers were cleaned up.

Evidence root:
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\integration\target\spec-195\read-retry`.
Native fixture: the same integration checkout's
`target\ticket-201\read-retry\native-failure`. Retained logs include red/green
checks, source patch, full browser reports, native build identity and fixture
metadata/results/cleanup. Pre-assertion failures from the old build-script path
cache and a rejected fixture destination remain recorded; source and admission
rules were unchanged. Other native UI journeys and full local Rust/speech-cache
suites were not rerun. Earlier results below retain their original identities;
exact integration CI is recorded on [PR #208](https://github.com/zardonc/lexique/pull/208).

## Frontend CI closeout (PR #208)

Clean tested commit: `03ef643bbc4cc8e116fd3a09b7f6ad24f4a9ab18`, tree
`8c4201e8c36eb006efd655e6f4fcb6cce808cf67`, after integrating
`b1c39c07fa49826c7d591ecd7efcc23437bf7b44`. This closeout changes only the
rating/skip T06 browser fixture and documentation. Runtime, backend, native
harness and dependencies are unchanged from the reviewed implementation below.

Both original b1 frontend jobs failed after scope-save retry. Their Playwright
traces show focus and review-change within the queue refresh's 50 ms debounce.
The fixture supplied one old-scope reply per notification, leaving surplus
old-scope replies after the coalesced refresh. Strict runtime validation correctly
rejected a later reply with `Queue response does not match the applied review
scope`. With a paused browser clock, both focused checks reproduced that failure
before the fix. The corrected fixture derives subsequent queue scope from saved
scope state, and the clock forces one coalesced queue refresh after two scope
reads. Error visibility, pending choice, completed-card boundary, save-only retry,
successful Voyage display, no remaining alert and no duplicate rating assertions
remain enforced. No production validation was weakened.

Windows x64 `10.0.26300`, pnpm `11.25.0`, Playwright `1.64.1`, bundled Chromium
`156.0.8078.4` / revision `1248`, `CI=true`, no executable override and **two
workers** match the failed jobs' browser/worker settings. This is local Windows
verification; an updated Ubuntu CI run has not been performed here.

| Clean tested-head check | Outcome |
| --- | --- |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | Passed |
| `pnpm --filter @lexique/desktop test:e2e review-scope.spec.ts -g "holds the completed boundary" --workers=2` | 2 passed |
| `pnpm --filter @lexique/desktop test:e2e --workers=2` | 472 passed, 261.91 s |

Evidence root: this checkout's `target/ticket-201/ci-closeout`. It retains
`verification-run.json`, required-check logs, `coalesced-red.log` and its
regression patch/results/report, focused green and full suite logs/results/reports,
both original CI artifact extractions, selected trace events, and raw/plain
logs for all four failed jobs below. Two incidental full-suite passes with eight
workers are preserved as `unintended-workers8-*`; an extra `--` stopped flag
parsing, so these were replaced by the explicitly verified two-worker runs above.
Native/backend suites were not rerun: their actual source, tests and harness are
identical to the earlier recorded revisions; their evidence is not relabeled.

| Original b1 run, attempt 1 | Frontend | Backend |
| --- | --- | --- |
| [PR 44977549743](https://github.com/zardonc/lexique/actions/runs/44977549743/attempts/1) | [137070714276](https://github.com/zardonc/lexique/actions/runs/44977549743/job/137070714276): failed rating/skip T06 | [137070714258](https://github.com/zardonc/lexique/actions/runs/44977549743/job/137070714258): failed V03 timing |
| [Push 44977547321](https://github.com/zardonc/lexique/actions/runs/44977547321) | [137070706549](https://github.com/zardonc/lexique/actions/runs/44977547321/job/137070706549): failed rating/skip T06 | [137070706296](https://github.com/zardonc/lexique/actions/runs/44977547321/job/137070706296): failed V03 timing |

The backend owner's Windows x64 Debug observations at unchanged b1 passed:
focused V03 in 0.77 s, and the unfiltered four-test preflight binary with default
threads in 0.81 s. All nine counted samples contended; transaction max 18.2263 ms
(100 ms gate), writer max 40.1496 ms (250 ms gate), hard execution timeout 60 s.
The owner verified `ensure_review_cards` and `review_cards` byte-equivalent to spec
base `e4005d868880dac12c84c412773049eea737bc67`; backend source/tests/runner are
unchanged from passing `27624ed84b9b0b3e3e4bf6568ded579fe3fb1324`.
The controlled [PR attempt 2 backend retry, job 137079648762](https://github.com/zardonc/lexique/actions/runs/44977549743/job/137079648762)
completed successfully at unchanged b1, with all limits intact. Its snapshot is
`v03-ci-rerun-jobs.json`; local observations are `v03-local*` and
`v03-parallel-local*`. This establishes recovery, not a backend code fix. The
original V03 slowdown remains unexplained, and the workflow aggregate remains
failed because its frontend is still the failed b1 result. Original failures
remain evidence; later combined-head CI results belong on the PR.

## Updated-source review fixes (PR #208)

Tested clean implementation: `ee8d3efee2ab21dc7a2fde535faf72902e0993d4`, tree
`45617fdffc28d2884fc213da0078026835bfa789`, in
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\review-fixes`.
The required start and integrated ancestor was
`27624ed84b9b0b3e3e4bf6568ded579fe3fb1324`. Environment: Windows x64
`10.0.26300`, pnpm `11.25.0`, Cargo `1.99.1`, uv Python `3.14.10`, initialized
MSVC and installed Edge/WebView2; installation used the frozen lockfile.

The Spec P2 is resolved: successful scope/category reads preserve a failed scope
save's visible error and save-only retry. Changing/cancelling the choice or saving
successfully clears that failure. Both rating and skip browser checks now refresh
through focus and review-change before retry, retain the completed boundary and
pending input, and assert no additional rating. The native failure journey uses
real SQLite write locks, focus and an actual category-change event, then verifies
unchanged ratings, progress and global usage after retry. The Standards P3 is
resolved by naming the native categories `voyageCategory` and `ecoleCategory`.

| Updated-source check | Outcome |
| --- | --- |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | Passed |
| `pnpm test:e2e -- --workers=4` | 472 passed, 186.30 s |
| `pnpm desktop:build --no-bundle` | New executable completed, 173.58 s; locked Cargo |
| All twelve tracked native scenarios | 12 passed; three failed renderer attempts retained with fresh passing retries |

All updated native attempts used this same source/tree and rebuilt executable.
Resource input and approved shared Cargo cache were integration `target\release`
and `target`; fixture destinations belong to this checkout's ignored
`target\ticket-201\native-ee8d3ef`. Single-category, multiple-category and
explicit-choice backups passed in their respective `*-retry` destinations.
Metadata, real SQLite, lifecycle and cleanup records remain in each fixture.
Passing fixtures also retain results, protected audio, backups and snapshots as
exercised by their scenarios. Every attempt retained the hard 60 s
execution limit; cleanup and final inventory report zero task-owned processes.

Updated evidence root:
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\review-fixes\target\ticket-201`.
`verification-run.json`, `native-runs.json`, `acceptance-summary.json`,
`focused-native-failure.json`, browser reports and regression logs record the
commands and results. The unchanged storage/desktop-library/speech-cache suites
were not rerun on updated source; their complete results below remain tied to
`a42f21e8664887aa899498dea6ae6ecb6d8ec822`. The failed b1 combined-head CI and subsequent backend recovery are recorded above; CI after this frontend closeout remains unrun.

## Earlier full acceptance

The following complete checks and T01–T19 table describe the original acceptance
revision. Its runtime/tests match the review's required starting head; the updated
review fixes above have their own execution evidence.

### Revision and environment

- Issue: [#195](https://github.com/zardonc/lexique/issues/195), acceptance slice
  [#201](https://github.com/zardonc/lexique/issues/201), draft
  [PR #208](https://github.com/zardonc/lexique/pull/208).
- Original execution checkout (removed): `C:\Users\inthe\IdeaProjects\lexique-category-review-195\ticket-201`.
- Required integration ancestor: `c658984141742c18bd54f6864bee6a30899f20d9`.
- Tested implementation SHA: `a42f21e8664887aa899498dea6ae6ecb6d8ec822`;
  clean tree: `9d06c344b0d3f5e986f49094f238544319d38086`.
- Windows x64 `10.0.26300`, PowerShell `7.6.9`, Node `24.22.1`, pnpm
  `11.25.0`, pinned Cargo `1.99.1`, uv Python `3.14.10`, initialized MSVC,
  installed Edge/WebView2. The frozen dependency install preceded execution.
- Evidence root:
  `C:\Users\inthe\IdeaProjects\lexique-category-review-195\evidence\ticket-201`.
  `retained-location.md` maps the original metadata prefix to this retained root.
  `verification-run.json` records commands, versions, SHA/tree, timing and logs;
  `native-runs.json` records each final native invocation and artifact directory.

### Complete checks

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

### Native T01–T19 evidence

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
and `README.txt`. Updated-source runs used the approved integration
`target\release` as resource input and the single-writer shared Cargo cache
`C:\Users\inthe\IdeaProjects\lexique-category-review-195\integration\target`.
No existing learning data is used as fixture input. From `apps/desktop`, run:

```powershell
$taskResources = 'C:\Users\inthe\IdeaProjects\lexique-category-review-195\integration\target\release'
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

Updated-source attempts for `backup-single`, `backup-multiple` and
`backup-needs-selection` failed with WebView2 access violation `0xc0000005`.
The retained dump analyses identify `RTSSHooks64.dll + 0x1490AF`, `+0x3B1D0`
and `+0x1490AF`, respectively. Each failed fixture, dump, log, metadata and
cleanup record remains under `native-ee8d3ef/<scenario>`; analyses are
`native-<scenario>-crash-analysis.json` in the updated evidence root.
Fresh retries passed from the same source and binary. No global RTSS shutdown
or owner exclusion from #192 was applied; a controlled RTSS-disabled comparison
remains unrun. These failures remain a material environment limitation.

Other platforms, clean-machine installation, whole-machine offline operation and
formal release acceptance remain unrun. The native fixture sets its WebView
offline and uses local speech/resources; this does not prove complete OS network
isolation or device-independent audible output. Playback evidence is the actual
native audio element and application rating eligibility.

Two earlier native attempts had WebView2 access violations, each with
exception `0xc0000005`:

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
