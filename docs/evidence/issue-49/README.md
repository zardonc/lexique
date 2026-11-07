# Fresh offline Windows conjugation acceptance (#49)

2026-11-07: the user reported that fresh offline acceptance passed for the
prepared Windows x64 speech package. Local checks cover fresh launch, complete
conjugation lookup, expression generation/playback, display restoration,
dictionary navigation and learning-data preservation for
[#49](https://github.com/zardonc/lexique/issues/49).
[#42](https://github.com/zardonc/lexique/issues/42) and
[#43](https://github.com/zardonc/lexique/issues/43) were checked against their
committed feature evidence, updated and closed before this preparation.

The accepted [ZIP](../../../target/issue-49/offline-package-20261107/Lexique-0.1.0-windows-x64-portable-speech.zip)
is described in the [offline acceptance guide](offline-acceptance.md). The ZIP
and original delivery directory remain preserved and contain no `Lexique-data/`.
Local testing used separate copies extracted from the verified ZIP.

## Package and resources

The [inventory](package-inventory.json) identifies the matching rebuilt reader,
format-6 database, dictionary and TTS assets. The reader was rebuilt from
application baseline `88d628e5c9ba09444315cac786b0a1e6934339d8`.
The 177,115,811-byte ZIP contains 1,119 verified files; extracted logical files
total 278,690,690 bytes, including 241,571,209 speech bytes. Model, voices,
Python/runtime libraries, Opus encoder, original sources and separate notices
are included. The unsigned development package requires an existing Microsoft
Edge WebView2 runtime; its README describes the official standalone offline
installer. Runtime data and WebView2 are outside these package sizes.

Eight MIT converters and the LGPL-LR ending rules accompany the compact
resource. Following the packaged rebuild instructions in a separate workspace
produced a byte-identical database in 21,826.3 ms. Resource replacement stays
independent of learning data. The [resource summary](resource-summary.json)
records the actual package's complete coverage and size accounting.

| Measure | Bytes |
| --- | ---: |
| Original Lefff morphology, uncompressed / gzip | 15,677,049 / 2,843,158 |
| Format-6 database / gzip | 11,005,952 / 6,143,296 |
| Shared model JSON / SQLite pages | 271,970 / 335,872 |
| Stem references / identity index pages | 733,184 / 278,528 |
| Source-line anchors / compressed verb information | 1,593,616 / 5,073,116 |
| Conjugation with sources, licenses and converters | 15,796,883 |
| Matching Windows reader | 15,602,176 |

The database meets the existing 16 MiB compact-resource check. It contains
170 models, 24,307 stems, 1,451,539 cells and 10,781 identities: 7,826 ordinary
and 2,955 pronominal identities for 2,954 pronominal base lemmas. All 23 block
states are accounted for as supported or explicitly inapplicable; pronominal
coverage has no incomplete entries. Regular ending coverage has no unconfirmed
rules, and all 95,658 third-group cells have no ending color. Source-only forms
and unconfirmed metadata remain explicit. English coverage is separate:
1,493 available and 9,288 unavailable identities, with 2,089 senses.

The preserved [format-5 equivalence evidence](../conjugation-storage/README.md)
covers every cell and identity, native-runtime comparisons and learning-data
preservation. Its disk/index budgets remain historical. Current query and
ending refinements retain their admitted [normalization](../issue-44/README.md)
and [linguistic rules](../issue-45/linguistic-rules.md).

## Executed checks

Commands ran with the pinned local tools and uv Python 3.14.10. Raw command
output remains in [the task logs](../../../target/issue-49/logs).

| Command or boundary | Result |
| --- | --- |
| `pnpm lint`; `pnpm typecheck`; `pnpm desktop:build --no-bundle` | PASS: production frontend and matching Windows reader |
| `pnpm rust:fmt`; `pnpm rust:check` | PASS: formatting and locked workspace compilation |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS: 143 backend tests; separate compilation and hard 60-second execution limit |
| `cargo test --locked --release -p lexique-desktop --lib --no-run`, then `pnpm test:desktop:lib` | PASS: 3 desktop-library tests |
| `pnpm test:conjugation`; `pnpm test:resources` | PASS: 36 conjugation CLI and 7 dictionary resource tests |
| `pnpm test:e2e` | PASS: 146 browser tests |
| `pnpm test:tooling` with the admitted TTS inputs | PASS: 2 build-argument, 8 portable and all 3 speech-package tests; no skips |
| Packaged source rebuild and ZIP read-back | PASS: byte-identical database and all 1,119 archive files verified |
| Final extracted-resource CLI corpus | PASS: 7 existing independent public CLI cases using only the packaged converters and inputs |
| Native fresh, full, speech and navigation flows | PASS: actual extracted reader/resource, each retaining the hard 60-second limit |
| `pnpm test:docs`; `node --check apps/desktop/e2e/native-conjugation-check.mjs`; `git diff --cached --check` | PASS: 122 documents, links and syntax/whitespace checks |
| Fresh network-blocked package acceptance | PASS: user reported "offline test passed"; see the recorded result in the acceptance guide |

The [final package CLI results](cli-results.json) cover every ordinary block,
ambiguity, original query preservation, normalization, present/missing English
meanings, complete pronominal expressions, agreement, ending spans and
accented-negative/unmatched queries. Expected values come from the existing
public tests, independent of the generated package.

[Fresh results](fresh-results.json) establish empty initial learning data,
uncached generation/playback of eight expression alternatives, male
slow/normal playback of the same audio, both navigation directions, keyboard
controls, three languages/themes and preference/order restoration across
normal exits and different verbs. Mode 3 stays chosen while the entire
third-group table has no ending colors; resetting order survives restart.
Browsing and playback create no learning records or cards.

[Combined native results](native-results.json) cover all interpretations,
23 blocks, justified absences/agreement, resource and preference failures,
three color-mode restarts and independent order reset. Speech recovery after
restoring a missing bundle succeeds. Recognition/listening previews use the
existing selected word audio, with identical bytes and unchanged progress.
Six existing synthetic records, twelve cards/progress states and seven audio
files survive; navigation explicitly saves one additional non-review verb
and changes only its intended notes/revision. Save failure, discard, save and
leave, duplicate/conflict handling and resource restoration are exercised.

## Costs

[Measurements](measurements.json) use the actual extracted database and reader.
Fresh launch to native IPC availability took 1,068.3 ms. Native IPC lookup of
`elles se sont abstenues` took 52.4 ms first and 53.2 ms warm median, with a
65.0 ms warm maximum. Six fresh CLI processes, including source validation
and response serialization, took 1,790.1 ms first, 1,377.7 ms warm median and
2,811.9 ms warm maximum. The first and maximum CLI samples miss the existing
1,700 ms evidence budget. These are first/subsequent process measurements;
OS caches were not flushed.

The [CLI latency follow-up](cli-latency.md) reuses sorted syntax slots in the
existing per-load cache and parses each verb's meanings once. All source,
database and meaning checks remain active. The new preparation directory is
`target/desktop-conjugation-v6-cli/`; its database is byte-identical to this
accepted package. Six fresh CLI processes subsequently measured 965.0 ms first,
947.5 ms warm median and 980.1 ms maximum, all within the original 1,700 ms
budget. The timing command now returns failure on a budget miss while retaining
the complete measurement report.

The final check against the newly prepared resource also passed all six
processes: 1,100.1 ms first, 936.2 ms warm median and 1,096.7 ms warm maximum.

## Reproduction

From the repository root, select uv Python and task-owned temporary paths:

```powershell
$issue49Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue49Python) + [IO.Path]::PathSeparator + $env:PATH
$env:TEMP = (Resolve-Path target/issue-49/tmp).Path
$env:TMP = $env:TEMP
$env:LEXIQUE_SPEECH_BUNDLE = '.tooling/issue-14/supertonic-evidence-v16-revised'
$env:LEXIQUE_OPUS_ENCODER = '.tooling/issue-15-opus/speech-stage/opusenc.dll'
$env:LEXIQUE_OPUS_SOURCES = '.tooling/issue-15-opus'
pnpm desktop:build --no-bundle
pnpm test:tooling
python -X utf8 scripts/package_portable.py --speech-bundle $env:LEXIQUE_SPEECH_BUNDLE --speech-encoder $env:LEXIQUE_OPUS_ENCODER --speech-sources $env:LEXIQUE_OPUS_SOURCES --output-dir target/issue-49/offline-package-new
python -X utf8 docs/evidence/conjugation-storage/measure_cli.py target/issue-49/package-extracted-20261107/Lexique-0.1.0-windows-x64-portable-speech/conjugation/conjugation.sqlite3 target/issue-49/cli-costs-new.json
```

Run native checks from `apps/desktop`, with a separate new destination per flow:

```powershell
$issue49Package = '../../target/issue-49/package-extracted-20261107/Lexique-0.1.0-windows-x64-portable-speech'
$issue49Seed = '../../target/issue-49/seed-20261107/Lexique-data'
node e2e/native-conjugation-check.mjs $issue49Package "$issue49Package/Lexique.exe" $issue49Seed ../../target/issue-49/native-new --fresh-package
```

Use separate new destinations with no final flag, `--speech` or `--navigation`
for the remaining flows. The seed contains only the known synthetic learning
fixture; speech assets come from the extracted package. The original delivery
is preserved for offline acceptance.

## Review basis

The review covers the staged preparation against fixed point
`88d628e5c9ba09444315cac786b0a1e6934339d8`, using
`git diff --cached 88d628e --`, and the accepted #49 issue. Standards and spec
are reviewed independently before committing.

## Standards

PASS: zero documented violations and zero actionable baseline smells in the
12-file preparation. The native harness reuses existing helpers, keeps copies
task-owned and preserves the hard execution limit. Selected-audio byte checks
and unchanged progress follow AGENTS.md and ADR 0008. Packaging, source/license
assertions and evidence preserve existing validation and data boundaries.

## Spec

PASS for the requested preparation: zero actionable findings, with no missing
preparation requirement, scope creep or incorrect implementation identified.
The harness uses packaged TTS, native playback, empty initial learning data and
existing preference/navigation/preservation checks. Evidence separately covers
the full admitted resource, independent CLI expectations, reproducible sources
and retained format-5 equivalence. The user has now reported that the package's
offline acceptance passed.

Standards: 0 findings, no worst issue. Spec: 0 findings, no worst issue.

## Limitations

The user supplied an offline pass without machine/runtime versions, network
block details or per-step logs. Local native checks ran on
Windows 11 Home 10.0.26300 x64 with WebView2 157.0.4341.42 and networking
available. Two local WebView2 process access violations were located by their
owned minidumps at `RTSSHooks64.dll + 0x1490af` (`0xC0000005`); RTSS remained
running, and its existing compatibility limitation is unresolved. The passing
isolated flows do not establish RTSS compatibility. Original CLI budget misses
and current passing measurements are retained separately. Other-platform
compilation/execution, missing-WebView2
clean-machine setup and M5 formal distribution were not run.
