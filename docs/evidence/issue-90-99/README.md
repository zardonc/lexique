# D2 content batches #90–#99

`core-99-v1` provides **944 accepted groups / 3,776 fields**, including 942
everyday groups and the two previously approved specialist senses. The frozen
denominator remains **4,087 groups / 3,773 headwords**: 3,143 groups have no
admitted content and 3,145 lack everyday completeness. Whole-core D2 coverage and
learner acceptance remain open in [#78](https://github.com/zardonc/lexique/issues/78).
Per-word CEFR grading is deferred outside this frozen target.

| Batch | Frozen rows | Accepted groups | Fields | Unchanged reused groups | Annotation exceptions |
| --- | --- | ---: | ---: | ---: | ---: |
| [#90](../issue-90/README.md) | 451–500 | 50 | 200 | 1 | 0 |
| [#91](../issue-91/README.md) | 501–550 | 50 | 200 | 0 | 0 |
| [#92](../issue-92/README.md) | 551–600 | 50 | 200 | 0 | 0 |
| [#93](../issue-93/README.md) | 601–650 | 50 | 200 | 0 | 0 |
| [#94](../issue-94/README.md) | 651–700 | 50 | 200 | 0 | 0 |
| [#95](../issue-95/README.md) | 701–750 | 50 | 200 | 0 | 0 |
| [#96](../issue-96/README.md) | 751–800 | 50 | 200 | 0 | 0 |
| [#97](../issue-97/README.md) | 801–850 | 48 | 192 | 0 | 2 |
| [#98](../issue-98/README.md) | 851–900 | 48 | 192 | 0 | 2 |
| [#99](../issue-99/README.md) | 901–950 | 49 | 196 | 0 | 1 |

These batches add **494 groups / 1,976 fields**. Row 483 `coeur/NOM` reuses
the existing sense unchanged and counts once globally. All 450 previous
compiled payloads and eight previous query aliases remain exact. Seven approved
annotations retain zero fields and 28 missing fields: rows 87, 305, 816, 849,
859, 860 and 934. Their frozen identities and frequencies remain unchanged.

The resource contains 1,026 source records and 15 spelling relations yielding
11 query aliases. The #92 `nouveau`/`nouvelle` and `sœur`/`soeur` relations and
#98 identity bindings retain their source-only decisions. Exact accepted
text, source versions, selected spans, rights and modifications are in each
batch's independent review and [combined notices](../../../resources/core-pilot-79/NOTICES.md).
The SQLite resource is **4,435,968 bytes**; reconstruction is byte-identical.

## Verification

Source and frontend checks use commit
`fa649fc202a007ca27c35909d9901c7db16cfb3e` on Windows x64. Backend checks use
`8b08a1adc45f05809dc16027c1533e25b6d88d7e`, which enables optimization for
the existing SHA-256 dependency in the test profile. Every checksum validation
and assertion remains active; precompilation is separate from execution.
Complete logs are retained in [the integration evidence root](../../../target/implement-90-99/).

| Check | Result | Retained evidence |
| --- | --- | --- |
| Converter, frozen batch membership and input failures | 43 passed | `final-core-tests.log` |
| Package version, materials and failure checks | 16 passed | `final-package-tests.log` |
| Reconstruction and previously admitted payloads/aliases | Passed | `compiled-preservation.json`, `verify-final.log` |
| Frontend lint, typecheck and production build | Passed | `final-lint.log`, `final-typecheck.log`, `final-build.log` |
| Browser core preflight and full suite | 13 and 393 passed | `final-browser-preflight.log`, `final-browser-full.log` |
| Rust formatting and separately precompiled backend execution | 194 passed within the existing hard 60-second limit | `final-rust-fmt.log`, `final-rust-precompile.log`, `final-rust-tests.log` |
| Locked Windows x64 release build and verified portable package | Passed | `final-native-build.log`, `final-package.json` |
| Ten native content batches | All passed; exact fields, provenance, spelling queries and save/reopen state compared | `native-batches-summary.json` |
| Native saved-state resource upgrade | Passed; references, frozen digests, notes, examples, source associations, review history and protected audio preserved | `native-upgrade.log` |
| All eleven native runs: matching binary, network boundary and process cleanup | Passed; zero external WebView requests and zero owned processes remaining | `native-execution-verification.json` |

The release binary and package were built from
`8b08a1adc45f05809dc16027c1533e25b6d88d7e`. Native execution uses harness commit
`bc711b6802f5bfbd0e8f58979c2d4d993d11450a`, which extends the existing
fixture ownership check to the ten new batch directories. The portable ZIP is
**37,008,688 bytes** with **183 verified files**. It requires an existing
WebView2 runtime and contains no TTS model.

Native #90–#95 evidence is at `target/issue-N/native-admitted-v2`; #96–#99
uses `target/issue-N/native-admitted-v3`. The upgrade is at
`target/issue-99/native-upgrade-v2`. Its closed source profile is copied
byte-for-byte into `target/issue-99/native-baseline-upgrade-source` (261 files).
All native runs retain the existing 60-second hard timeout and verify normal
shutdown. The first #96 run timed out waiting for shutdown; its cleanup was
empty and the fresh run passed with unchanged source and timeouts. That failure
is retained in `native-96-close-failure.log` and the original fixture results.
The earlier backend budget failures are retained in `backend-concurrent-failure.log`
and `backend-isolated-failure.log`; the optimized test profile passes all checks.

Commands, run from the repository root except the native harness:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p 'test_core*.py'
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_package_portable.py
pnpm lint
pnpm typecheck
pnpm build
pnpm rust:fmt
pnpm rust:test:build
pnpm test:rust
pnpm desktop:build --no-bundle
uv run --python 3.14.10 --no-project python -X utf8 scripts/package_portable.py --output-dir target/implement-90-99/final-package
```

MSVC is initialized and the pinned Cargo/Python directories are selected before
native compilation. Browser commands use the isolated port-1439 configuration
at `target/implement-90-99/playwright.config.ts`: the core preflight and full
suite run through `pnpm exec playwright test --config` from `apps/desktop`.
The native harness runs `batch` for each issue and `upgrade` for the closed
profile through `node e2e/native-d2-pilot-check.mjs`. Each run records its exact
arguments, binary checksum and hard timeout in `run-metadata.json`.

## Rollback and remaining gates

Resource rollback requires a matching application/resource pair because the
desktop binary validates its build-fixed resource checksum. Saved digests
remain frozen when resource content changes. The existing reader supports
the grammatical subclasses in this resource. Downgrading to a reader that
rejects saved `ADJ:int` or `PRO:pos` records can fail a whole notebook/card/import
response; use a compatible backup or continue with the current reader.
Resources and learner data remain separate.

Whole-machine offline testing, other-platform hands-on execution, human learner
acceptance, whole-core D2 coverage and formal release gates remain unverified.
Per-word CEFR grading is deferred outside this frozen target.
The browser suite does not establish native persistence.
