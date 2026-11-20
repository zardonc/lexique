# D2 content batches #100–#109

`core-109-v1` admits **1,436 groups / 5,744 fields**, including **1,434 everyday groups** and the two existing specialist senses. Frozen rows **951–1450** add **492 groups / 1,968 independently reviewed fields**. The denominator remains **4,087 groups / 3,773 headwords**; **2,651 groups remain unaccepted** and **2,653 remain incomplete for everyday use**.

| Batch review | Frozen rows | Admitted groups | Fields | Zero-field annotations |
| --- | --- | ---: | ---: | ---: |
| [#100](../issue-100/independent-review.md) | 951–1000 | 50 | 200 | 0 |
| [#101](../issue-101/independent-review.md) | 1001–1050 | 49 | 196 | 1 |
| [#102](../issue-102/independent-review.md) | 1051–1100 | 49 | 196 | 1 |
| [#103](../issue-103/independent-review.md) | 1101–1150 | 50 | 200 | 0 |
| [#104](../issue-104/independent-review.md) | 1151–1200 | 50 | 200 | 0 |
| [#105](../issue-105/independent-review.md) | 1201–1250 | 46 | 184 | 4 |
| [#106](../issue-106/independent-review.md) | 1251–1300 | 49 | 196 | 1 |
| [#107](../issue-107/independent-review.md) | 1301–1350 | 50 | 200 | 0 |
| [#108](../issue-108/independent-review.md) | 1351–1400 | 49 | 196 | 1 |
| [#109](../issue-109/independent-review.md) | 1401–1450 | 50 | 200 | 0 |

All field text, selected source spans, spelling/POS bindings, modern use and reuse obligations passed separate independent review. The four [Mme fields](../issue-102/mme-independent-review.md) and [marché replacements](../issue-103/independent-review.md) remain ordinary content. The latter select Théodore Soulice's 1836 public-market clause with a separate fixed pronunciation source. Actual credits and selected boundaries are in [the bundled notices](../../../resources/core-pilot-79/NOTICES.md) and [source context](../../../resources/core-pilot-79/selected-source-context.json).

The [owner-amended #78](https://github.com/zardonc/lexique/issues/78) approves eight new annotations: rows 1005 `m/PRO:per`, 1051 `s/PRO:per`, 1208 `quiconque/PRO:int`, 1218 `qu/CON`, 1219 `qu/PRO:int`, 1220 `qu/PRO:rel`, 1300 `d/PRE` and 1386 `nulle/PRO:ind`. Each retains its frozen identity, frequencies, zero admitted fields and four missing fields. With the seven existing annotations, **15 annotations retain 60 missing fields**. No new alias or CEFR assignment is added.

Reconstruction is byte-identical. All **944 prior SQLite payloads and 11 aliases** are preserved byte-for-byte. Source records total **1,475**, including **449 new selected sources**. All **1,836 original evidence files**, including the separate replacement author's captures, are byte-preserved in the [archive manifest](../../../target/implement-100-109/author-evidence-relocation.json). Exact batch heads and current artifact identities are retained in [aggregate.json](aggregate.json).

## Verification

| Check | Result | Retained evidence |
| --- | --- | --- |
| Converter, all cohorts and failure paths | 63 passed after test consolidation | `fresh-worker-core-all.log` |
| Portable package and input failures | 16 passed | `final-package-tests.log` |
| Frontend lint, typecheck and build | Passed | `final-lint-v3.log`, `final-typecheck-v3.log`, `final-build-v3.log` |
| Applicable browser flows, including three locales | 13 passed | `final-browser-core-v3.log` |
| Rust formatting and locked precompilation | Passed | `final-rust-fmt.log`, `final-rust-precompile.log` |
| Backend execution | 194 passed; hard 60-second limit preserved | `final-rust-tests.log` |
| Matching Windows x64 release and package | Passed | `final-native-build-v3.log`, `final-package-v3.json` |
| Ten native batches | All passed: exact fields/provenance, trusted save and reopen state | `native-batches-summary.json` |
| Saved-state upgrade | Passed: digests, notes, examples, progress, replay and protected audio preserved; new poisson/camarade saved | `native-upgrade-v2.log` |
| Eleven native runs: binary, network and cleanup | Passed: matching binary, zero external WebView requests and zero owned processes | `native-execution-verification.json` |
| Independent integration review | Standards: one P3 resolved, zero remaining; spec: zero findings | [Final review](final-review.md) |

The final v3 app/resources and native harness were built and tested from **`090b10c2384a68ebb9a5ea0cb9346a7c1765dafa`**. The later evidence commit changes documentation only. The tests consolidated at `f0344637027cc5e5d7ffad1fa0c0d1608ee4c9a3` cover identical converter/resource inputs. Backend tests ran at `45c9e4ab841c4e92081702b7bf412828868c4e2d`; Rust sources and SQLite content are unchanged.

SQLite: **6,721,536 bytes**, SHA-256 `75548a5c0cc9ec2e7007237f9c5cfaaf357d83ffef47706273cff9633b6c8c19`. Windows x64 [development portable](../../../target/implement-100-109/final-package-v3/Lexique-0.1.0-windows-x64-portable.zip): **38,547,297 bytes**, **183 verified files**, SHA-256 `78abfbf31a6fbd193086f60aea6ce4747611ee2d6e1e4255b5c537e84e413aab`. Binary SHA-256: `2135708520e81b78b2327af431b29d8d9b174c79fc75e1accc03e6d5ccbb4f59`.

Complete logs are under [the integration evidence root](../../../target/implement-100-109/). Native batches use `target/issue-N/native-admitted-v3`; upgrade uses `target/issue-109/native-upgrade-v2`. Its closed source copy has **261 byte-identical files** before and after execution. The first #100 attempt exposed a fixture destination guard ending at #99; its failure and zero-process cleanup remain in `native-100.log` and `target/issue-100/native-admitted-v1`. Corrected v2 and final v3 runs pass without changing timeouts.

Commands use pinned Python 3.14.10, pnpm 11.25.0 and Rust 1.99.1, with MSVC initialized for native builds:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p 'test_core*.py'
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_package_portable.py
pnpm lint
pnpm typecheck
pnpm build
pnpm --filter @lexique/desktop exec playwright test --config ../../target/implement-100-109/playwright.config.ts core-pilot.spec.ts
pnpm rust:fmt
pnpm rust:test:build
pnpm test:rust
pnpm desktop:build --no-bundle
uv run --python 3.14.10 --no-project python -X utf8 scripts/package_portable.py --output-dir target/implement-100-109/final-package-v3
pwsh -NoProfile -File target/implement-100-109/run_native.ps1
uv run --python 3.14.10 --no-project python -B -X utf8 target/implement-100-109/check_native.py
```

Playwright owns its isolated port-1440 server and uses the verified installed Edge executable. Native fixtures preserve the existing 60-second hard execution limit. [PR #180](https://github.com/zardonc/lexique/pull/180) closes these ten tickets when merged; parent #78 remains open.

## Limitations

Whole-machine offline execution, other-platform native flows, learner acceptance and full D2 release readiness remain unverified. Two statute refreshes returned HTTP 403; [#107](../issue-107/rights.json) and [#109](../issue-109/rights.json) retain those failures and the established edition/source basis. Current everyday coverage remains incomplete. Final documentation checks and matching-head CI are recorded separately in the aggregate closeout.
