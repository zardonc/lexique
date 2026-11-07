# Conjugation coloring and restart persistence (#45)

2026-11-06: the Windows desktop offers three direct coloring choices, initially
mode 3: complete verbal part, main verb, and main inflection. The choice follows
different verbs and survives normal native restarts. Pronouns remain plain;
auxiliaries remain plain in mode 3. Italics, match underlining and the seven
priority markers provide independent non-color cues in all three languages and
themes.

[#41](https://github.com/zardonc/lexique/issues/41) was checked and updated first:
its seven acceptance criteria and retained CLI/native evidence are complete,
and its tracker status is closed. This slice implements
[#45](https://github.com/zardonc/lexique/issues/45), part of
[#38](https://github.com/zardonc/lexique/issues/38).

## Rules and resource

The public CLI and native reader return `coloring` for every original form and
expression. Ranges count Unicode characters with exclusive ends. For
`nous avions lavé`, the three spans are `[5,16)`, `[12,16)` and `[15,16)`;
`lavais` has ending `[3,6)`. Reflexive prefixes, imperative pronoun suffixes,
subjects, conjunctions and gerund `en` remain outside the spans.

[Independent linguistic expectations and source citations](linguistic-rules.md)
validate the separate LGPL-LR ending table over complete admitted forms. The
table never reads a storage model's cut or suffix. Checks include first-/second-
group families, spelling/stem changes, `haïr`, `rassir`, gender/number alternatives,
compounds and pronominal constructions. Cited metadata corrections identify
`mûrir` as group 2 and `messeoir`/`reparcourir` as group 3 while preserving raw
flags, forms and original sources.

The [resource report](resource-report.json) accounts for all 1,451,539 logical
cells and 10,781 identities. It reports no unresolved regular endings and zero
ending spans across all 95,658 third-group cells. Third-group and unconfirmed-
group tables remain plain in mode 3; modes 1 and 2 work normally. The retained
source-only forms under `courbaturer`, `précéder` and `égaler` remain queryable
and visibly unconfirmed for mode-3 endings: 62 cells, 59 distinct report entries.

Format 6 is prepared in `target/desktop-conjugation-v6-color/`, preserving older
directories. The database is 11,005,952 bytes (10.50 MiB); its 170 models and
24,307 stem references retain the existing storage schema. The reverse stem
table remains 733,184 bytes. Packages contain the matching rebuilt reader,
original separately licensed sources, eight MIT converters and the LGPL-LR
ending table.

## Preferences and preservation

The restricted `conjugation_preferences` command uses existing SQLite
`app_metadata` preferences, key `conjugation-preferences`, with JSON
`{"mode":"verbal"}`, `{"mode":"main"}` or `{"mode":"ending"}`. Only absent
settings use the default. Reads, writes, invalid IPC, corrupt JSON/settings and
unavailable preference storage fail visibly. A failed write retains the prior
selection; a failed read disables selection and offers reload. Browser preview
reports that changes are not saved. No schema increment or learning-data
migration is needed.

[Actual Windows results](native-results.json) verify all three modes through
normal application restarts. The harness changes modes with keyboard Space,
checks exact rendered spans, spelling/gender alternatives, irregular exclusion,
all match cues and priority markers. Fault injection uses only a copied
synthetic database: a write-rejecting trigger, invalid saved settings, and a
temporarily unavailable `app_metadata` table. Storage and settings are restored
before restart. Six vocabulary records and their notes, twelve cards and
progress records, speech preferences and seven audio files remain unchanged.
Lookup and browsing do not implicitly save records or enroll cards.

## Verification

Windows x64; uv Python 3.14.10, Node.js 24.22.1, pnpm 11.25.0 and Rust 1.99.1.
Tests use the approved public resource CLI and desktop flow. Backend tests are
precompiled separately; backend and native execution retain their hard
60-second limits.

| Command / boundary | Result |
| --- | --- |
| `pnpm conjugation:prepare` | PASS; complete resource and rule validation |
| `pnpm test:conjugation` | PASS; 36 CLI tests in 455.338 s, including reproducibility, preservation and failures |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | PASS |
| `pnpm test:e2e -- --workers=2` | PASS; all 114 browser tests, including seven coloring/control/error checks |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `cargo clippy --locked -p lexique-dictionary -p lexique-storage --all-targets -- -D warnings` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS; all 143 backend tests |
| `pnpm desktop:build` | PASS; production frontend and rebuilt Windows x64 release reader |
| `cargo test --locked --release -p lexique-desktop --lib --no-run`, then `pnpm test:desktop:lib` | PASS; all three desktop library tests |
| `pnpm test:tooling` with verified speech inputs | PASS; 2 build-argument, 8 portable and 3 speech-package checks, no skips |
| `pnpm test:speech:portable` after the final reader rebuild | PASS; all three checks, including positive TTS packaging |
| Windows `native-conjugation-check.mjs` | PASS; exact spans, all-mode restarts, preference/resource failures and data preservation |
| `pnpm test:docs` | PASS; 117 documents and 701 local links |
| Staged provenance hashes and evidence JSON checks | PASS; all six Grammalecte inputs match their recorded hashes; reports parse |
| `git diff --cached --check` | PASS |

Logs are retained in `target/issue-45-20261106/`; the full native report and
screenshots are in `target/issue-45/native-color-20261106-2/`.
[Measurements](measurements.json) record six fresh CLI processes, including
resource verification: first 1,436.5 ms, subsequent median 1,381.9 ms, all below
the 1,700 ms evidence budget. Native IPC measured 26.5 ms first invocation and
28.4 ms median over five subsequent invocations. Neither measurement flushes
the OS cache; earlier cost reports remain historical.

From the repository root, select uv Python before Python-dependent commands:

```powershell
$issue45Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue45Python) + [IO.Path]::PathSeparator + $env:PATH
$env:LEXIQUE_SPEECH_BUNDLE = '.tooling/issue-14/supertonic-evidence-v16-revised'
$env:LEXIQUE_OPUS_ENCODER = '.tooling/issue-15-opus/speech-stage/opusenc.dll'
$env:LEXIQUE_OPUS_SOURCES = '.tooling/issue-15-opus'
pnpm desktop:build
pnpm test:tooling
```

Reproduce native acceptance from `apps/desktop`, using a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/conjugation-format6-fresh-20261106-2/Lexique-data ../../target/issue-45/native-new
```

## Review basis

The review covers the working change against fixed point
`0f16f2ed88e1e8214719fee3fae36a52ea5b138c`, using
`git diff --cached 0f16f2ed88e1e8214719fee3fae36a52ea5b138c --`, and the accepted
#45 issue. Standards and spec are reviewed independently.

## Standards

Zero findings: no documented-standard violations or actionable baseline smells
in the full 48-file staged diff. The change follows existing runtime/storage
boundaries, strict IPC validation, preference persistence, resource licensing,
and preservation conventions. Resource preparation, packaging, fixtures, and
verification documentation are updated together.

## Spec

Zero findings: the change and retained evidence cover all seven #45 criteria,
with no missing requirements, scope creep, or incorrect implementation found.
The evidence covers independent linguistic spans, irregular exclusion,
accessible rendering, validated preferences, and data preservation. Actual
Windows results confirm all three modes across native restarts and meaningful
preference failures.

Standards: 0 findings, no worst issue. Spec: 0 findings, no worst issue.

## Limitations

Fresh combined disconnected-package acceptance remains #49. This native check
did not block networking or generate new speech; positive packaging validates
the existing admitted TTS resources and their notices. Other OS/architecture
execution and formal release acceptance were not run. Partial English coverage,
context-dependent agreement and source-only forms retain their documented limits.
