# CI recovery for PRs #174–#176

All three PR heads passed 22 checks before merging on 2026-11-18.

| PR | Verified head | Merge commit | Checks |
| --- | --- | --- | ---: |
| [#174](https://github.com/zardonc/lexique/pull/174) | [bfab389](https://github.com/zardonc/lexique/commit/bfab389407d736af8ab501646d5ee8c2bfdc6553) | [04e3ec6](https://github.com/zardonc/lexique/commit/04e3ec60a2ba4b88ca4f7cf233833032e37ee88b) | 22/22 |
| [#175](https://github.com/zardonc/lexique/pull/175) | [60af1f4](https://github.com/zardonc/lexique/commit/60af1f4bf1de7637eebe89f132efa7676adff291) | [19db064](https://github.com/zardonc/lexique/commit/19db0645f6bfc1f92d1aaed59ea9a33bf2e841bd) | 22/22 |
| [#176](https://github.com/zardonc/lexique/pull/176) | [15df003](https://github.com/zardonc/lexique/commit/15df003e3b5408af36f69acf01936e03343f1bdf) | [42c7205](https://github.com/zardonc/lexique/commit/42c720556fd1195cca39ba704761877d1c0df48a) | 22/22 |

The combined merge tree has its own [main CI run](https://github.com/zardonc/lexique/actions/runs/44377907700).

## Queue E2E races

The [original frontend failure](https://github.com/zardonc/lexique/actions/runs/44249142896/job/134615259934) exposed two assertions that ran before their queue requests completed:

- `queue-window.spec.ts` emitted a change before the initial 21-card queue was ready. The test now waits for that queue before emitting the event.
- `review-queue.spec.ts` inspected IPC calls when preloaded listening audio appeared. The test now polls for the `review_queue` request containing `skipped: [1]`, then checks that skipping produced no rating.

With queue dispatch delayed by 500 ms, the two tests failed in all four controlled attempts before the fix and passed in all four afterward. The temporary dispatch delay was removed. `pnpm lint`, `pnpm typecheck`, `pnpm build`, the focused E2E check and all 391 browser tests passed on Windows x64 with Microsoft Edge at head `15df003e3b5408af36f69acf01936e03343f1bdf`. [PR #176](https://github.com/zardonc/lexique/pull/176) changes these two tests.

## Intel macOS timing

The [original native failure](https://github.com/zardonc/lexique/actions/runs/44249142896/job/134615260024) measured a 50-word V03 transaction at **135.163 ms** and its competing writer at **189.102 ms**. The transaction exceeded its 100 ms budget; the writer remained within its 250 ms budget.

The temporary probe invoked `node scripts/test-rust.mjs` against only `v03_candidate_transaction_sizes_and_competing_notes_and_ratings`. It used `macos-15-intel`, `x86_64-apple-darwin`, test optimization level 1 and one test thread. Compilation stayed separate from the runner's hard 60-second execution limit. Fixture setup and writer readiness preceded the timed transaction. Every executed timing sample retained the 100/250 ms assertions, including samples without observed contention.

| Probe run and fixed head | Main-connection control | Test invocations | Result |
| --- | --- | ---: | --- |
| [CPU probe](https://github.com/zardonc/lexique/actions/runs/44358388535), `6989f31206fe6476ab871e28f2b11a9582223175` | Normal synchronization | 100 | 99 passed; one competing writer took 370.716 ms |
| Same CPU probe and head | `synchronous=OFF` | 100 | 99 passed; a 20-word transaction took 190.122 ms, including 185.796 ms in COMMIT with 1.084 ms of thread CPU |
| [Lock probe](https://github.com/zardonc/lexique/actions/runs/44363441630), `647060721dba8b4543a631bf02d131c72395fcd8` | Normal busy timeout and synchronization | 100 | 100 passed; all 902 timing samples retained; maximum transaction/writer times 96.339/213.731 ms |
| Same lock probe and head | Busy timeout zero; normal synchronization | 100 | 90 passed; ten COMMIT errors were `SQLITE_BUSY`; all 833 completed timing samples and ten failed commits retained |

With busy waiting disabled, a successful COMMIT still took **82.506 ms elapsed and 0.954 ms of thread CPU**. These controls establish COMMIT lock contention and substantial waiting beyond the busy handler. Disabling synchronization also retained a budget violation. Elapsed and thread CPU measurements cannot identify the original timeout's exact lock, filesystem operation or scheduling interruption.

The probe's competing note writer uses an autocommit UPDATE; production [vocabulary saving](../../../crates/storage/src/vocabulary.rs) begins an Immediate transaction before writing notes. The timing effect of that fixture difference remains unmeasured. Production durability and the 100/250 ms operation budgets remain as configured. All failed invocations remain in the linked workflow artifacts.

## ARM64 runner acquisition

Three ARM64 macOS jobs in [PR #176's CI](https://github.com/zardonc/lexique/actions/runs/44351931557) were cancelled before execution. GitHub annotations reported: “The job was not acquired by Runner of type hosted even after multiple attempts” and macOS ARM64 capacity constraints. The targeted retries passed at the same head, completing its 22-check matrix before merge.

## Verification boundary

The PR checks establish recovery at the fixed heads above. The native timing gate remains intermittent across the retained diagnostic runs. Learner acceptance, full D2/CEFR coverage, whole-machine offline operation, other-platform GUI acceptance and release readiness retain their existing boundaries.
