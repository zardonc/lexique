# D2 content batches #84–#89

`core-89-v1` provides **450 accepted groups / 1,800 fields**, including 448
everyday groups and the two previously approved specialist senses. The frozen
denominator remains **4,087 groups / 3,773 headwords**: 3,637 groups have no
admitted content and 3,639 lack everyday completeness. These batches extend the
existing partial resource; the complete D2 and CEFR gates remain open in
[#78](https://github.com/zardonc/lexique/issues/78).

| Batch | Frozen targets | Accepted groups | Fields | Unchanged reused groups | Annotation exceptions |
| --- | ---: | ---: | ---: | ---: | ---: |
| [#84](../issue-84/admission.md) | 50 | 50 | 200 | 0 | 0 |
| [#85](../issue-85/admission.md) | 50 | 50 | 200 | 0 | 0 |
| [#86](../issue-86/admission.md) | 50 | 50 | 200 | 0 | 0 |
| [#87](../issue-87/admission.md) | 50 | 49 | 196 | 0 | 1 |
| [#88](../issue-88/admission.md) | 50 | 50 | 200 | 1 | 0 |
| [#89](../issue-89/admission.md) | 50 | 50 | 200 | 2 | 0 |

The batch totals include reused rows 360, 422 and 423, counted once globally.
All prior **154 groups / 616 fields** retain identical compiled payloads, source
bindings, reviews and contexts. The owner approved row 305 `dont/PRO:int` as a
zero-field annotation exception, retained in the denominator; modern relative
`dont` remains available through row 306. The existing `quoi/ADJ` annotation,
`ça/NOM` specialist sense and `par/NOM` golf sense retain their approved scope.

The resource carries 377 sources and 11 spelling relations. Its SQLite file is
**2,117,632 bytes**; the reviewed source materials are **7,017,410 bytes**.
Exact reconstruction, fixed-version source spans, independent field reviews,
attribution and compiled preservation passed. Accepted bindings and source
evidence are recorded in each batch's `admission.json` and the
[resource notices](../../../resources/core-pilot-79/NOTICES.md).

## Verification

Windows x64 build and native execution use source commit
`571d221f7fbe9e28be58bd22a24ced3578b9a9b2`; packaging and browser verification use
`291b254c70246ecaa8ccb8448230ecd3d9d0968e`, which only updates the packager and
its version fixture. Evidence was collected on 2026-11-18–19 UTC and retained
under [the integration evidence root](../../../target/implement-84-89/).

| Check | Result | Retained evidence |
| --- | --- | --- |
| Rust formatting, locked native compilation | Passed | `final-build.log`, `final-rust-test-build.log` |
| Separately precompiled backend tests | 194 passed; existing hard 60-second execution limit retained | `final-rust-tests.log` |
| Converter and focused batch checks | 13 converter tests and 19 focused tests passed | `finalize/core-tests.log`, `finalize/focused-tests.log` |
| Reconstruction, source notices and previous-content preservation | Passed | `finalize/checks.json`, `finalize/compiled-preservation.json` |
| Frontend lint, typecheck and production build | Passed | `final-frontend-lint.log`, `final-frontend-typecheck.log`, `final-build.log` |
| Coverage hint in Chinese, English and French | Three browser tests passed | `coverage-ui-green.log` |
| Portable package checks | 16 passed | `package-version-green.log` |
| Full browser suite | 393 passed in 2.9 minutes | `final-browser.log` |
| Six native content batches | All passed; exact batch membership and saved/reopened fields verified | `native-batches-summary.json` |
| Native resource upgrade | Passed; saved digests, records, sources, review history and protected audio preserved; `quel/ADJ:int` and `leur/PRO:pos` saved | `native-upgrade.log` |

Native #84 and #85 evidence is retained under `target/issue-N/native-admitted`;
#86–#89 uses `target/issue-N/native-admitted-v2`. The upgrade profile is at
`target/issue-89/native-upgrade`, with the closed baseline at
`target/issue-84/native-baseline-upgrade-source`. Every completed native run
reports zero external WebView requests and empty owned-process cleanup. Each
run retains the existing **60-second** hard timeout. Representative screenshots
show readable Chinese content, French IPA, grammatical labels and restored
learner text; lookup coverage is checked against the frozen CSV.

The freshly built portable ZIP is **35,857,718 bytes**, with **183 verified
files**, retained at `target/implement-84-89/final-package/`. Packaging verifies
the admitted resource and complete source/notice set. This development package
requires an existing WebView2 runtime and contains no TTS model.

## Rollback and remaining gates

Resource rollback requires a matching application/resource pair because the
desktop binary validates the resource against its build-fixed checksum. Saved
digests remain frozen when resource content changes. Older application readers
reject saved records containing
`ADJ:int` or `PRO:pos`; rejection can fail an entire notebook/card/import
response. Application downgrade therefore requires a compatible backup or
continued use of the new reader. Resources and learner data remain separate.

Whole-machine offline testing, other-platform hands-on execution, human learner
acceptance, complete D2/CEFR coverage and formal release gates were not verified
by these runs. The browser suite does not establish native persistence.
