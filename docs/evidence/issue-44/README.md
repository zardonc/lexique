# Typographic conjugation queries (#44)

2026-11-06: the public resource CLI and Windows desktop accept case differences, canonical Unicode equivalents, straight/curly apostrophes and trimmed/repeated Unicode whitespace. NFC, lowercase, apostrophe and whitespace normalization apply before compact suffix/stem candidate lookup and complete generated-form/expression comparison. Frontend response validation, candidate counts and expression highlights use the same comparison. Accents, original input, source spelling, all interpretations and ordinary/pronominal identities remain intact.

## Behavior and representation

Independent positive expectations include `SOMMES` → `sommer` / `être`, decomposed `ÊTRE` → `être`, ` J'AI  AIMÉ ` → `aimer`, ` QUE  J'AIME ` → `aimer`, ` ELLES SE  SONT ABSTENUES ` → `s’abstenir`, and ` VA-T'EN ` → `s’en aller`. `ABSTENUES` retains separate `abstenir` / `s’abstenir` interpretations. The admitted source-only `v'là` matches `V’LÀ` while retaining its original spelling and labels. Negative cases include `ETE`, `J'AI AIME`, `JE AVONS AIMÉ`, `JE SE LAVE`, `ELLES SE SONT ABSTENUS` and `AIMÉ AVONS`; a matching token does not establish a whole-expression match.

CLI and native input validation reject blank queries, Unicode control characters and inputs exceeding 512 UTF-8 bytes before resource access. Invalid input, no match and resource failures remain distinct. Lookup and browsing do not save vocabulary or enroll cards.

The active format-6 resource is prepared in `target/desktop-conjugation-v6-query/` with the corresponding converters. The preserved `target/desktop-conjugation-v6/` remains the storage baseline. The new database is byte-identical to that baseline: SHA-256 `598d88df83952c071ca69c7c2bcb2a89ed0fe926f4a831866a2d3b6e33a89147`, 10,969,088 bytes, 170 models, 24,307 stem references, 10,781 identities and 1,451,539 logical cells. The stem table remains 733,184 bytes. All 24,106 distinct stored stems already use the comparison spelling; suffix comparison also normalizes the admitted straight-apostrophe suffix. No expression tables, identity migrations or new source resources are introduced. Rust reuses the existing locked `unicode-normalization` 0.1.26 dependency.

## Verification

Windows x64, uv-managed Python 3.14.10, Node.js 24.22.1, pnpm 11.25.0 and Rust 1.99.1. Public resource CLI tests and the existing desktop flow are the approved test boundaries. Backend compilation is separate from the runner's unchanged hard 60-second execution limit; the native harness also retains its 60-second limit.

| Command / boundary | Result |
| --- | --- |
| Focused public CLI normalization / invalid-input checks | PASS |
| Focused Rust query and browser highlight regressions | PASS |
| `pnpm test:conjugation` | PASS, 34 public CLI tests in 381.966 s |
| `pnpm typecheck`, `pnpm lint` | PASS |
| `pnpm test:e2e --workers=2` | PASS, 107 browser tests |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `cargo clippy --locked -p lexique-dictionary --all-targets -- -D warnings` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS, 143 backend tests, no failures or skips |
| `pnpm desktop:build --no-bundle` | PASS, production frontend and Windows x64 release reader |
| `pnpm test:tooling` with verified speech inputs | PASS, 2 build-argument, 8 portable and all 3 speech-package tests; no skips |
| Windows `native-conjugation-check.mjs` | PASS, normalized query/highlight behavior, original input, failures, normal restart and learning/audio preservation |
| `pnpm test:docs`, native evidence JSON and `git diff --check` | PASS, 115 documents, 690 local links and complete positive/negative query summaries |

[Native query results](native-results.json) summarize the retained raw report at `target/issue-44/native-reviewed/report.json`; logs are in `target/issue-44-20261106/`. The native check uses an owned copy of six synthetic vocabulary records, twelve cards with progress and seven existing audio files. Records, notes, progress, selected audio bytes and speech preferences are unchanged after lookup, resource failures and normal restart. Native IPC measured 23.1 ms first invocation and 21.3 ms warm median over five subsequent invocations, without an OS cache flush.

Reproduce the native check from `apps/desktop`, creating a new destination beneath `target/issue-44/`:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/conjugation-format6-fresh-20261106-2/Lexique-data ../../target/issue-44/native-new
```

The positive speech-package check uses the rebuilt format-6 reader and these verified inputs from the repository root:

```powershell
$issue44Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue44Python) + [IO.Path]::PathSeparator + $env:PATH
$env:LEXIQUE_SPEECH_BUNDLE = '.tooling/issue-14/supertonic-evidence-v16-revised'
$env:LEXIQUE_OPUS_ENCODER = '.tooling/issue-15-opus/speech-stage/opusenc.dll'
$env:LEXIQUE_OPUS_SOURCES = '.tooling/issue-15-opus'
pnpm test:tooling
```

[#40](https://github.com/zardonc/lexique/issues/40) was checked first: its seven acceptance criteria and retained Windows evidence were complete, and its tracker status is closed. [#44](https://github.com/zardonc/lexique/issues/44) implements the typographic-query slice of [#38](https://github.com/zardonc/lexique/issues/38).

## Review

Two fresh independent reviews covered the working diff against fixed point `90b9f7420a852a87635ecc133ddce1ae6979e783`, using `git diff 90b9f7420a852a87635ecc133ddce1ae6979e783 --`, and the accepted #44 issue. Reviewers inspected source and recorded evidence; execution checks are recorded above.

### Standards

No findings: 0 documented-standard violations and 0 actionable baseline smells. Runtime boundaries, validation, visible errors, source spelling and learning identities are preserved. Preparation, release copying, portable validation and fixtures are updated together. Regression checks use the established public boundaries.

### Spec

No findings: no missing requirements, scope creep or incorrect implementation for supported project inputs. Candidate lookup and complete-expression verification normalize consistently; frontend validation, counts and highlights use equivalent comparisons. Queries, accents, source forms, variants, identities and ordering remain intact, and the compact representation is preserved.

Review totals: Standards 0; Spec 0. Neither axis has a blocking issue.

## Limitations

The native check did not block networking or generate new speech; the packaging check verifies the speech resources and licenses. Fresh combined offline acceptance remains #49. Other OS/architecture execution and formal release acceptance were not run. Existing source admission, contextual agreement, partial English coverage and the recorded CLI latency budget remain as described in the [storage report](../conjugation-storage/README.md) and [pronominal report](../issue-43/README.md).
