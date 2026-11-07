# Windows x64 conjugation acceptance

2026-11-07: the confirmed conjugation scope in
[#38](https://github.com/zardonc/lexique/issues/38) is accepted on Windows x64.
#39's six metadata criteria were checked and the issue was closed first;
all eleven children, #39–#49, are now closed. The user's fresh offline pass
applies to the preserved #49 speech package. The specification and its
two public test boundaries remain unchanged.

## Scope and evidence

The table maps every numbered story in the
[specification](../../m3-conjugation-spec.md) to feature and combined evidence.
Browser checks establish presentation; the retained Windows checks establish
native resource access, persistence, generation/playback and data preservation.

| Stories | Accepted behavior | Evidence |
| --- | --- | --- |
| 1, 20, 56 | Admitted resources without a frequency cap; pinned provenance, separate licenses, replaceable resources and complete TTS package | [Metadata](../issue-39/README.md), [resource summary](../issue-49/resource-summary.json), [package inventory](../issue-49/package-inventory.json), [source rebuild](../issue-49/README.md) |
| 2, 5, 14–19 | Native simple lookup, all selectable candidates and cell counts, stable alternatives and independent match cues | [#40](../issue-40/README.md), [combined native results](../issue-49/native-results.json) |
| 3–4 | Both dictionary navigation directions, retained query/candidate context and explicit unsaved-edit handling | [#47](../issue-47/README.md), [combined native results](../issue-49/native-results.json) |
| 6–8, 26–29, 35–39 | Ordinary and pronominal complete expressions, all 23 blocks, subjects/reflexives, independent alternatives, justified agreement and grammatical absences | [#41](../issue-41/README.md), [#43](../issue-43/README.md), [resource-wide coverage](../issue-49/resource-summary.json), [final public CLI corpus](../issue-49/cli-results.json) |
| 9–13 | Case/Unicode/apostrophe/space normalization, meaningful accents, whole-expression verification, retained input and visible no-match/resource failures | [#44](../issue-44/README.md), [public CLI corpus](../issue-49/cli-results.json), [native failures](../issue-49/native-results.json) |
| 21–25 | Traditional groups, independently supported auxiliaries/use labels, present and explicitly missing English meanings | [#39](../issue-39/README.md), [#42](../issue-42/README.md), [current metadata/English coverage](../issue-49/resource-summary.json) |
| 30–34 | Seven priority identities, keyboard reorder/reset and shared order restored across verbs and normal restarts | [#46](../issue-46/README.md), [fresh-package results](../issue-49/fresh-results.json) |
| 40–46 | Exact three-mode spans, default/persisted ending mode, entire third-group exclusion and independent matching/priority cues | [#45](../issue-45/README.md), [linguistic rules](../issue-45/linguistic-rules.md), [fresh-package results](../issue-49/fresh-results.json) |
| 47–51 | Separate complete-expression playback, voice/speed preferences, real uncached generation and visible/recoverable failures | [#48](../issue-48/README.md), [fresh-package results](../issue-49/fresh-results.json), [native speech recovery](../issue-49/native-results.json) |
| 52 | Preserved vocabulary identities, notes, selected word audio, independent cards/progress and review previews; no implicit collection or enrollment | [combined native results](../issue-49/native-results.json), [storage equivalence and preservation](../conjugation-storage/README.md) |
| 53–54 | Keyboard controls, non-color cues and the existing three themes/languages | [feature checks](../issue-49/README.md), [fresh-package results](../issue-49/fresh-results.json) |
| 55 | Fresh matching Windows x64 package, user-reported network-blocked first launch/playback/restart acceptance | [offline acceptance](../issue-49/offline-acceptance.md), [package identity](../issue-49/package-inventory.json) |

## Resource and package

The accepted ZIP is
`target/issue-49/offline-package-20261107/Lexique-0.1.0-windows-x64-portable-speech.zip`:
177,115,811 bytes and 1,119 verified files, with the reader rebuilt from
application baseline `88d628e`, format-6 conjugation, dictionary, speech
runtime/model/voices/encoder, original sources and separate notices. It and
the original delivery directory remain unchanged; tests used separate copies.
The archive identity and included dependencies are retained in the inventory.

The inventory's `offline_acceptance` and fresh native results'
`offlineAcceptance` fields are pre-acceptance snapshots. Their pending markers
precede the user's 2026-11-07 pass recorded in the offline acceptance guide;
the raw local results and accepted archive remain unchanged.

The database is 11,005,952 bytes, with 170 shared models, 24,307 stem references,
1,451,539 cells and 10,781 identities (7,826 ordinary and 2,955 pronominal).
All 23 block states are supported or explicitly inapplicable; coverage has
no missing/unconfirmed ordinary or pronominal block entries. Source-only forms
and unknown metadata remain explicit. Twenty ordinary group classifications
remain unconfirmed; no unsupported group or ending is invented. English is
measured separately: 1,493 available and 9,288 unavailable identities,
with 2,089 senses.

The [storage contract](../../conjugation-storage.md) and preserved exhaustive
format-5 comparison retain every cell and identity and the learning-data
boundary. Format 6 requires the matching rebuilt reader. Current preparation,
native builds, release copying, Tauri packaging, fixtures and backend CI use
`target/desktop-conjugation-v6-stats/`; earlier generated resources remain intact.
The optimized preparation reproduces the accepted package's database bytes.

The builder measures stem pages inside a rollback-only savepoint in its
original transaction, using standard SQLite page-size/freelist accounting.
The [portable size results](portable-page-accounting.json) retain identical
database bytes and the complete numeric report: 9,994,240 table bytes,
278,528 lemma-index bytes and 733,184 reverse-index bytes. The public CLI
rebuild test also runs with optional `dbstat` access unavailable and compares
the completed database, report and redistribution materials. Original resources
and the accepted ZIP remain intact.

The [CLI evidence](../issue-49/cli-latency.md) retains full source/database/
meaning validation. All six final public CLI processes passed the original
1,700 ms budget: 1,100.073 ms first, 936.198 ms subsequent median and
1,096.710 ms subsequent maximum. The measurement tool returns failure if any
sample exceeds the budget and retains every sample. The optimization is in
the repository and new preparation; the accepted ZIP remains unchanged.

## Checks

Existing acceptance evidence at `38ae2ae` includes 36 public conjugation CLI,
143 separately precompiled backend, 3 desktop-library, 7 dictionary-resource,
146 browser and 2+8+3 tooling tests, including positive TTS packaging with no
skips. Production frontend and the matching Windows x64 reader, locked Rust,
packaged-source rebuild, final extracted-resource corpus and actual fresh/full/
speech/navigation flows passed. Backend and native execution retain their hard
60-second limits. Detailed commands and results remain in the feature reports.

This closeout retains the native/offline acceptance and checks the builder
and preparation configuration with:

| Command or boundary | Result |
| --- | --- |
| Existing public CLI `test_traditional_groups_and_ordinary_auxiliaries_have_source_evidence` | PASS: fresh current build, eight independent lemmas, 46.340 seconds |
| Public CLI rebuild with optional `dbstat` access unavailable | PASS: completed database/report/materials preserved, 41.160 seconds |
| `pnpm conjugation:prepare`; prior resource/report comparison | PASS: byte-identical database and equal numeric report in the separate current directory |
| `pnpm test:conjugation` | PASS: all 36 public CLI tests, 293.383 seconds |
| `pnpm rust:fmt`; `pnpm rust:test:build`, then `pnpm test:rust` | PASS: formatting, separate compilation and 143 backend tests within the hard 60-second runner limit |
| `pnpm test:tooling`, admitted TTS inputs and short task TEMP | PASS: 2 build-argument, 8 portable and all 3 speech-package tests; no skips |
| `cargo clippy --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings` | PASS: 13.17 seconds |
| CI preparation path compared with `conjugation:prepare` | PASS: the current resource directory matches |
| GitHub #39 body/state read-back and native child relationships | PASS: all six criteria checked; all eleven children closed |
| `pnpm test:docs` | PASS: 124 documents, 786 local links, zero problems |
| `git diff --cached --check` | PASS: staged whitespace |

Local command logs are retained under `target/issue-38/closeout-20261107/`.
Documentation and staged whitespace checks validate the closeout text and
links; PR CI validates the published branch separately.

## Review basis

The requested review point is `38ae2ae`. Independent Standards and Spec
reviews cover the closeout documentation, portable builder page accounting,
public CLI regression and aligned preparation/build/package paths with:

```powershell
$closeoutBase = (git merge-base 38ae2ae HEAD).Trim()
git diff --cached $closeoutBase
git log 38ae2ae..HEAD --oneline
```

## Standards

PASS: zero documented violations and zero baseline smells. The documentation
keeps resource admission, native/offline acceptance and formal distribution
gates separate. Rollback-only page accounting uses the existing build
transaction, and preparation, native builds, packaging and fixtures share the
matching resource directory. The backend runner keeps its hard 60-second
execution limit.

## Spec

PASS: zero actionable findings. All 56 stories map to feature and combined
evidence; resource counts, explicit unknowns, partial English coverage and the
matching format-6 reader agree with the confirmed specification. The retained
pre-acceptance snapshots and final user-reported offline pass are distinguished.
The rebuilt database matches the prior resource and accepted ZIP byte for byte;
the complete size report is unchanged. The public CLI regression denies
optional `dbstat` access and verifies the database, report and notices.

Standards: 0 violations, 0 smells, no worst issue. Spec: 0 findings, no worst
issue. Both reviews were read-only; tests and remote CI were not rerun by the
reviewers.

## Limits

The user reported the offline pass without machine/runtime versions,
network-block details or per-step logs. Local native evidence used Windows 11
Home 10.0.26300 x64 and WebView2 157.0.4341.42 with networking available.
Timings use fresh/subsequent processes without an OS cache flush and do not
establish a cold-disk bound. Positive local speech packaging uses a short
task-owned TEMP.

This closes the agreed Windows x64 conjugation scope. Other-platform execution,
missing-WebView2 first-install offline setup, full dictionary/MDX/MDD and graded
resources, example-sentence speech, other M3 modules, M4 synchronization,
M6 browser persistence and M5 formal Full/Lite distribution retain their own
gates. CI compilation is separate from native/offline acceptance.
