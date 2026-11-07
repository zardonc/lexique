# Reordering tense blocks across native restarts (#46)

2026-11-06: the Windows desktop lets the learner select any of the 23 tense
blocks, move it earlier or later, and reset the whole order. The selection
shows the block's position. Native buttons and the select work with the
keyboard; focus stays on the move button during a save. The same order applies
to other verbs, including defective verbs, with unavailable blocks retained.

This implements [#46](https://github.com/zardonc/lexique/issues/46), part of
[#38](https://github.com/zardonc/lexique/issues/38). It preserves the independently
owned coloring preference verified in [#45](../issue-45/README.md).

## Order and preferences

Reset restores the exact agreed order:

```text
P PC I F C S PQP J PA FA CP SP T SQP Y YP W WP G K KP GP GPP
```

The first seven codes identify the priority tenses even after they move.
Match highlights continue to identify their actual cells; verb coloring keeps
its own validated spans. Reordering does not change any table content.

The restricted `conjugation_block_order` command reads or saves a JSON array
under `app_metadata['conjugation-block-order']`. Rust and the frontend both
require every agreed block exactly once. Missing, duplicate, unknown and
incorrectly typed codes are rejected. Only an absent preference uses the
default; malformed or invalid saved data fails visibly.

This entry is separate from `conjugation-preferences`, whose existing
`{"mode": ...}` contract is unchanged. Saving or resetting order preserves the
coloring selection; saving a coloring mode preserves the order. Schema remains
17, with no learning-data migration. The existing database-opening boundary
continues to reject unsupported higher versions.

Failed writes retain the previous displayed order. Failed reads disable order
controls and offer reload after repair. A shared preference-table outage affects
both settings and each reports its own error. Browser preview explicitly says
that order changes are not saved. Controls, status and failure messages support
the existing three interface languages and themes.

## Actual Windows results

[Native results](native-results.json) cover keyboard movement of `PQP` across the
priority boundary, all 23 blocks on `laver`, `falloir` and `se laver`, and unchanged
match/priority/coloring cues. The custom order survived three normal application
restarts, while coloring was separately saved and restored as `main`, `verbal`
and `ending`. Reset restored the default, which survived a fourth normal restart.

Failure checks reject invalid IPC orders, report a write-rejecting trigger,
malformed JSON and invalid saved arrays, and recover from an unavailable
`app_metadata` table. Fault injection uses only copied synthetic data and restores
the table/settings. Six vocabulary records and their notes, twelve cards and
progress records, speech preferences, selected audio and seven audio files
remain unchanged. Browsing and changing display preferences do not create
learning records or enroll cards.

## Verification

Windows x64; uv Python 3.14.10, Node.js 24.22.1, pnpm 11.25.0 and Rust 1.99.1.
Tests observe the already approved desktop boundary, through browser adapter
fixtures and actual Windows IPC/UI. Backend tests are precompiled separately;
backend and native execution retain their hard 60-second limits.

| Command / boundary | Result |
| --- | --- |
| `pnpm test:e2e -- conjugation-order.spec.ts conjugation-coloring.spec.ts --workers=2` | PASS; 11 focused ordering/coloring checks |
| `pnpm test:e2e -- --workers=2` | PASS; all 118 browser tests |
| `pnpm test:e2e -- conjugation.spec.ts conjugation-coloring.spec.ts conjugation-order.spec.ts --workers=2` | PASS; all 17 conjugation checks with the shared independent order fixture |
| `pnpm lint`, `pnpm typecheck` | PASS |
| `pnpm desktop:build` | PASS; production frontend build and matching rebuilt Windows x64 release reader |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `cargo clippy --locked -p lexique-storage --all-targets -- -D warnings` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS; all 143 backend tests |
| `pnpm test:tooling` with verified speech inputs | PASS; 2 build-argument, 8 portable and 3 speech-package checks; no skips |
| Windows `native-conjugation-check.mjs` | PASS; custom-order and reset restarts, all-mode preservation, failures and learning-data preservation |
| Obscura browser preview | PASS; order moves to position 8/23 with an explicit non-persistence notice and no error |
| `pnpm test:docs` | PASS; 118 Markdown documents and 706 local links |
| `git diff --cached --check`, `git diff --check` | PASS |
| Public resource CLI suite | Not rerun; resource/rules/CLI are unchanged, with current coverage retained in #45 evidence |

Logs are in `target/issue-46-20261106/`. The executed reader, full native report
and screenshots are in `target/issue-46/native-order-20261106-2/`. Native lookup
measured 34.3 ms first invocation and 35.7 ms median over five subsequent
invocations, without an OS cache flush. This slice does not change the resource
or claim new CLI performance measurements.

From the repository root, select uv Python before Python-dependent commands:

```powershell
$issue46Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue46Python) + [IO.Path]::PathSeparator + $env:PATH
$env:LEXIQUE_SPEECH_BUNDLE = '.tooling/issue-14/supertonic-evidence-v16-revised'
$env:LEXIQUE_OPUS_ENCODER = '.tooling/issue-15-opus/speech-stage/opusenc.dll'
$env:LEXIQUE_OPUS_SOURCES = '.tooling/issue-15-opus'
pnpm desktop:build
pnpm test:tooling
```

Reproduce native acceptance from `apps/desktop`, using a new destination below
the existing `target/issue-46/` parent:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/conjugation-format6-fresh-20261106-2/Lexique-data ../../target/issue-46/native-new
```

## Review

The review compares the staged change against fixed point
`fdb0cfea716840c540a653bf248e7766cfcbb45b`, using
`git diff --cached fdb0cfea716840c540a653bf248e7766cfcbb45b --`, and the accepted
#46 issue. Standards and spec are reviewed independently.

### Standards

No documented-standard violations or Fowler smell findings. The independent
default-order fixture is shared by the browser mocks and assertions, using
the existing fixture directory and no production-code dependency. The
validated runtime and storage boundaries follow the repository conventions.

### Spec

No missing, partial or incorrectly implemented requirements, and no scope
creep. The exact reset order, keyboard movement, defective-block retention,
priority and match identity, independent coloring preference, visible failures
and native restart/data-preservation evidence satisfy #46.

Standards: 0 findings. Spec: 0 findings.

## Limitations

Fresh combined disconnected-package acceptance remains #49. This feature check
did not block networking or generate new speech; positive packaging uses the
existing admitted TTS inputs and required notices. Other OS/architecture
execution and formal release acceptance were not run.
