# Queue snapshot and position regression fixes

2026-11-04 (America/Vancouver). This work preserves the pre-existing workspace changes and uses synthetic databases only. No commit, push, issue closure, model switch, or RTSS inspection/settings change was performed.

## Reproductions and causes

- The saved S0 native executable still failed `native-s0-baseline.mjs ... gap-red`: all 21 requests omitted the due intraday listening card with both limits zero. The workspace already contained its discovery fix. The rebuilt candidate passes the same 21 assertions; this work adds an explicit storage regression rather than claiming a new quota fix.
- After test precompilation, `target/debug/deps/review-438041f43c99dd8b.exe queue_audio_checks_do_not_hold_a_database_snapshot --exact --nocapture` failed in 0.02 seconds with `database is locked`. A second real SQLite connection could not commit during the availability callback: the queue retained its read snapshot and, on first admission, its day-reset writes across that callback.
- `target/debug/deps/review-b4c143ff393e835e.exe queue_reindexes_positions_near_the_integer_limit --exact --nocapture` failed in 0.01 seconds with `UNIQUE constraint failed: review_queue_entries.position`. Saturating append arithmetic reused the maximum position instead of renumbering.

The queue now reads progress, settings, usage and positions in a short snapshot, releases it before availability/file work, and acquires a writer only for required day/position changes. SQLite `data_version` detects intervening external commits; at most three attempts reread authority before returning an explicit `QueueChanged` error. This is connection-local concurrency detection, not the planned durable business queue revision or window protocol.

Position exhaustion and insufficient gaps share the existing negative-position renumbering strategy inside the short transaction. Arithmetic is checked. Regression coverage verifies commit failure rolls back the whole renumbering, retries succeed, and stable refresh preserves order. The concurrent-writer regression verifies both successful re-reading after a change and bounded failure during sustained changes.

The desktop audio inspector now reads progress and existing audio candidates in batches before starting file workers. Workers perform no SQL. Candidate ranking, the existing twenty-key cutoff, expiry, GC exclusion, shared ownership, full safe file reads, checksums and legacy fallback remain in effect. A differential storage test compares batch results against the original key/asset APIs, including expired and shared assets. This still scans eligible audio files and is not the future bounded playback window.

## Validation

- `pnpm lint`, `pnpm typecheck`, `pnpm build`: passed.
- `pnpm rust:fmt`, `pnpm rust:check`: passed; existing incremental-cache hard-link warning retained.
- `pnpm rust:test:build`, then `pnpm test:rust`: passed, 101 tests, original hard 60-second execution limit retained.
- Release desktop lib tests precompiled separately and executed through `measure_s0.execute(timeout=60)`: 2 passed. Four existing test-only dead-code warnings remain.
- `pnpm test:e2e`: initially unable to launch because the expected Playwright executable was missing. After `pnpm --filter @lexique/desktop exec playwright install chromium`, all 87 passed in 21.2 seconds. These use simulated IPC, separate from native evidence.
- `pnpm desktop:build --no-bundle`: passed. Fixed candidate: `target/issue-24/diagnosis-20261105/candidate.exe`, SHA-256 `7e12ce4b6f7ecce18b1c32a5c6a774a3d56bc24484f3d2b03172390b784b2bcc`.
- Native Q04: `target/issue-24/diagnosis-gap-green-20261105/s0-results.json`, 21/21 passed, owned workers after exit zero.
- Native 10,000-word / 20,000-card new-card fixture: `target/issue-24/diagnosis-scale-20261105/s0-results.json`, 21/21 passed. First request 1,110.3 ms; later requests are separate warm observations. Header-only Ogg fixtures do not prove decoding/playback, physical cold disk, or the final p95 target. Twenty warm samples are insufficient for a reported p95.
- Native product path: `target/issue-24/diagnosis-product-20261105/results.json`, passed on the same candidate. Six real CPU-generated outputs fully decoded as finite, audible mono audio; playback reached ended, concurrent lookup/note edit/rating committed, and audio/progress survived restart. Actual play start was 38.2 ms for a cached output, not an uncached synthesis promise. One resident worker was observed during work and zero owned workers after shutdown.

Run native scripts from `apps/desktop`, after adding the directory returned by `uv python find 3.14` to PATH. Inputs are the resource-only reusable fixture `../../target/issue-24/followup-final-product-20261103`, the fixed candidate above, and a new destination. Each native script retains its existing hard 60-second execution limit.

The same native scale script was also run against saved S0 executable `target/issue-25/fixed-native/Lexique.exe` (hash `57334356d84addf966e3a639d0fdb7cea38d936d96171f6f0f15fb46de8ed77b`). Both runs used 10,000 synthetic words, two cards each, healthy header-only files, one first request and twenty warm requests. Compilation and other test workloads were stopped for these runs. No OS cache reset or RTSS inspection was performed. Results are local observations, not a controlled hardware/environment attribution or acceptance of the final performance target.

| Executable | First ms | Warm median ms | Warm range ms | Failed requests |
| --- | ---: | ---: | --- | ---: |
| Saved S0 | 1,354.6 | 603.0 | 468.3–645.8 | 0/21 |
| Current workspace | 1,110.3 | 170.0 | 133.6–267.6 | 0/21 |

Comparison: `target/issue-24/diagnosis-20261105/comparison.json`; old raw results: `target/issue-24/diagnosis-scale-baseline-20261105/s0-results.json`. The current executable includes the pre-existing workspace fixes as well as this work, so the difference cannot all be attributed to this turn's changes. Twenty warm samples are not used to claim p95.

## Remaining scope

This is partial progress on #26, not acceptance of #26 or #24. A single snapshot across all desktop phases, persistent queue revisions, legacy registration outside the hot path, selected audio versions, publication/recovery and retention protocols, bounded byte/playback windows, incremental updates, speech fairness, and the full #34 performance/lifecycle matrix remain to be implemented or verified in their owning tickets. No new claim is made for clean-system offline execution, other architectures/platforms, physical power failure, or formal release readiness.
