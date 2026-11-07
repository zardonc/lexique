# Compact conjugation storage verification

Format 6 stores 170 shared simple-form models and 24,307 compatible stem references. The Python and native runtimes reproduce all **1,451,539 cells and 10,781 identities** from the preserved format-5 database, including metadata, provenance, duplicate interpretations, cell order and expression order. The database shrank by **95.6%**, from 239.04 MiB to 10.46 MiB.

The unchanged conjugation lookup interface returns complete tables. Existing admitted rules construct compounds, subjects, reflexives, gerunds and agreement alternatives at lookup time. Source participle variants, lexical restrictions and explicit corrections remain available. Storage suffixes describe compression; #45 supplies independently validated linguistic endings.

## Sizes and timings

[Storage measurements](measurements.json) and [fresh review results](review-2026-11-06.json).

| Measure | Format 5 | Format 6 |
| --- | --- | --- |
| Database bytes | 250,650,624 | 10,969,088 |
| Database gzip bytes | 49,581,488 | 6,139,062 |
| Lemma index bytes | 28,651,520 | 278,528 |
| Reverse lookup bytes | 35,909,632 | 733,184 |

The model JSON occupies 271,970 bytes, with 1,593,616 bytes of source-line anchors. Compressed verb information occupies 5,072,692 bytes; this retains source evidence and use metadata. Physical SQLite page accounting is included in the measurements. The conjugation resource with all redistribution sources, licenses and seven MIT converters occupies 15,746,014 bytes.

The verified Windows x64 speech development ZIP is **168.84 MiB** (177,043,746 bytes), with 1,117 verified files. It includes the rebuilt Format 6 reader, dictionary sample, compact conjugation resource and pinned offline speech runtime/model. Extracted logical files total 278,409,926 bytes, including 241,571,209 speech bytes; system WebView2 and runtime data are outside this count. The package inventory identifies the executed binary as `611446e4a3efb2294c989b1ece8491eb97f758c7ed9f801aa40745b4f36b4139` and the resource as `598d88df83952c071ca69c7c2bcb2a89ed0fe926f4a831866a2d3b6e33a89147`.

For `elles se sont abstenues`, six fresh CLI processes measured 1730.9 ms first, 1457.5 ms warm median and 1484.1 ms warm maximum. The first process exceeds the existing 1,700 ms budget by 30.9 ms; all five subsequent processes meet it. CLI costs include original-source validation and response serialization. Actual native IPC measured 33.8 ms first and 33.3 ms warm median over five subsequent calls.

## Verification

- The fresh public build's `--equivalence-baseline` comparison checks every format-5 table and every metadata field, then all cell fields and ordered expression arrays. It passed for 1,451,539 cells and 10,781 identities.
- [Native comparison driver](check_native.py) exports complete expected public lookup responses directly from format 5 and uses [the native checker](../../../crates/dictionary/examples/compare_conjugation.rs). **10,824 public lookups** cover every identity and all cells. All 28 batches passed, each with a hard 60-second subprocess timeout.
- `pnpm test:conjugation`: 31 public CLI tests passed in 436.196 seconds, covering ordinary/pronominal constructions, complete-expression verification, source variants, resource/metadata failures, size accounting, reproducibility and redistribution materials. The updated compact-resource public-report/query check also passed separately in 15.119 seconds. Individual CLI calls retain their 60-second limit.
- `pnpm rust:test:build`, then `pnpm test:rust`: all 142 backend tests passed; compilation remains separate from the unchanged hard 60-second execution limit. The precompile command and backend CI prepare both dictionary and Format 6 conjugation resources. Dictionary Clippy, all-workspace formatting and locked workspace compilation passed.
- Lint, typecheck, production frontend build and the real Windows release build passed.
- The Windows native harness passed actual resource IPC, all 23 blocks, candidate selection, variants, ordinary/pronominal agreement, failure presentation and normal restart. Six fresh synthetic vocabulary records, twelve cards with progress, speech preferences and all seven audio files remained identical.
- Two build argument checks, eight portable packaging checks and all three speech-package checks passed, including positive TTS packaging with verified inputs. Actual portable packaging verified 1,117 files, the matching reader/resource identity and absence of learning data.
- The fresh native speech check passed six real generated/decoded word audios, completed playback, one resident model process, normal shutdown and progress/audio restoration. No owned model process remained after normal shutdown.
- [#38](https://github.com/zardonc/lexique/issues/38), [#44](https://github.com/zardonc/lexique/issues/44), [#45](https://github.com/zardonc/lexique/issues/45) and [#49](https://github.com/zardonc/lexique/issues/49) were updated and read back exactly. Titles, labels and comments are preserved; body-only changes retain existing dependencies.

## Reproduction

Use the pinned tools from `LOCAL-DEV-ENV.md` and `rust-toolchain.toml`. Keep the original database and its sibling report at `target/conjugation-refactor/baseline-v5/`. A build destination must be new.

```powershell
pnpm conjugation:build target/conjugation-recheck --equivalence-baseline target/conjugation-refactor/baseline-v5/conjugation.sqlite3
cargo build --locked --release -p lexique-dictionary --example compare_conjugation
python -X utf8 docs/evidence/conjugation-storage/check_native.py target/conjugation-refactor/baseline-v5/conjugation.sqlite3 target/desktop-conjugation-v6/conjugation.sqlite3 target/release/examples/compare_conjugation.exe target/conjugation-native-recheck
python -X utf8 docs/evidence/conjugation-storage/measure_cli.py target/desktop-conjugation-v6/conjugation.sqlite3 target/conjugation-cli-costs.json
```

The native harness runs from `apps/desktop` with an unused owned destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/conjugation-refactor/native-new
```

Positive TTS packaging uses the retained verified inputs and a new output directory, after rebuilding the reader:

```powershell
pnpm desktop:build --no-bundle
python -X utf8 scripts/package_portable.py --speech-bundle .tooling/issue-14/supertonic-evidence-v16-revised --speech-encoder .tooling/issue-15-opus/speech-stage/opusenc.dll --speech-sources .tooling/issue-15-opus --output-dir target/conjugation-speech-new
```

The prepared [offline checklist](../../../target/conjugation-review-20261106/offline-checklist.md) identifies the delivered ZIP and currently implemented flows. The user reported that fresh disconnected acceptance of this package passed.

The [storage contract](../../conjugation-storage.md) records the representation and integration requirements. Original format-5 resources remain intact; format 6 requires its corresponding rebuilt reader and resource package.

## Limitations

Equivalence preserves the admitted source data and its known linguistic limitations. These measurements use first/subsequent processes or native invocations without flushing OS caches. The first fresh CLI process missed the latency budget. One native speech run hit a WebView2 access violation (`ProcessFailed kind=0`, exit `-1073741819`); a separate fresh run completed successfully. RTSS was running, but this run's fault module was not established; its injection compatibility remains excluded by the existing #24 scope. These native harness runs had network access. Disconnected acceptance is user-reported, without supplied test-machine/version or per-step observations. Combined #49 acceptance, the browser suite for this review, other platforms and formal release acceptance remain unrun.
