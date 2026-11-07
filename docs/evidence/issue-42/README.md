# Offline English verb meanings (#42)

2026-11-06: verb headers and ambiguous candidate choices display licensed offline English senses from FreeDict French-English 0.4.1. Coverage is **1,377 of 7,826 identities (17.6%), with 1,953 senses**. The other 6,449 identities explicitly show that meanings are unavailable; their conjugation tables remain usable. English coverage is independent of the unchanged 23-block conjugation coverage.

## Resource admission and mapping

The [official source release](https://download.freedict.org/dictionaries/fra-eng/0.4.1/) has 8,505 TEI entries. Its archive, published SHA-512, original AUTHORS and full COPYING are retained with [pinned provenance and mapping policy](../../../resources/freedict-fra-eng-0.4.1/README.md). Content remains GPL-2.0-or-later, attributed to Horst Eyermann, Michael Bunk, John Darrington and the FreeDict project. Application/converter code remains MIT. The maintained [FreeDict dictionaries project](https://github.com/freedict/fd-dictionaries) supplies the reference representation; no upstream executable code runs.

Only explicit verb parts of speech and English translation citations qualify. Exact ordinary headwords require admitted ordinary use; explicit `se / s' / s’` headwords retain the original headword and a separate pronominal use label. Source entry/sense ordinals, usage labels and translation alternatives remain separate from morphology provenance. Nouns, French example citations and unmatched multiword phrases cannot supply a meaning. There is no fuzzy matching, French-Chinese substitution, online translation or credential requirement.

Public CLI expectations include `aimer → love; appreciate, like`, `être → be`, `suivre → act on, act upon, follow, observe`, and `sommer → admonish, scold`. `abstenir` retains the source `s'abstenir · abstain` label. `abéliser`, `falloir` and historical `avenir` remain unavailable; the noun `avenir → future` is excluded. Bare `taire` reflexive translations and the misplaced `retenir → thunder` sense are excluded using the references documented in the source policy; other legitimate `retenir` senses remain available.

Format 4 stores English meanings separately in the replaceable resource, with validated structured IPC and text rendering. Preparation uses `target/desktop-conjugation-v4`; incompatible existing directories are preserved and rejected visibly. Five MIT converter modules, all original source inputs and notices accompany the generated database. Learning storage, vocabulary identities and review enrollment are unchanged.

## Resource and costs

[Resource report](resource-report.json), [timings and budgets](measurements.json), and [rebuild from staged Git bytes](staged-byte-rebuild.json).

| Measure | Result |
| --- | --- |
| Lefff rows / expanded source slots / identities | 342,771 / 397,413 / 7,826 |
| Total source and generated slots | 780,818 |
| Database / gzip bytes | 124,735,488 / 26,542,947 |
| Database increase over format 3 | 860,160 bytes |
| Lemma index / reverse index bytes | 14,376,960 / 17,252,352 |
| Original English source archive | 214,980 bytes |
| Resource build wall time | 13,939 ms |
| First CLI process / warm median / warm maximum | 1,642 / 1,645 / 1,661 ms |

The database remains below the enforced 128 MiB native snapshot limit; the reverse index remains below the existing 18 MiB evidence budget. All six measured queries meet the 1,700 ms evidence budget. Query measurements start fresh public CLI processes against a warm filesystem and include source integrity checks and complete table serialization. They do not measure OS-cold disk access or native IPC latency.

## Verification

Commands use pinned pnpm, managed Python and task-owned temporary directories. Backend compilation precedes execution under the unchanged hard 60-second runner.

| Check | Result |
| --- | --- |
| `pnpm test:conjugation` | PASS, 24 public CLI tests in 295.716 s: mapped/missing meanings, ordinary/pronominal distinctions, malformed source inputs and stored payloads, coverage, reproducibility and existing conjugation behavior |
| `pnpm lint`, `pnpm typecheck` | PASS |
| `pnpm desktop:build --no-bundle` | PASS, Windows x64 release and production frontend build |
| `pnpm test:e2e --workers=2` | PASS, all 105 browser tests in 50.6 s, with resource-derived fixtures and existing languages/themes/navigation |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS, 138 precompiled backend tests |
| `pnpm test:resources` | PASS, 7 dictionary resource tests |
| `pnpm test:tooling` | PASS: 2 build-argument, 8 portable and 2 speech failure checks; 1 speech-package integration check skipped without separately verified inputs |
| Windows `native-conjugation-check.mjs` | PASS, actual resource access, candidate/header meanings, pronominal label, unavailable state, 23 blocks, counts/highlights, visible resource errors and normal restart |
| Export staged Git bytes, then public CLI build | PASS, 40 files exported; pinned English payload hashes and final database SHA-256 match |
| `pnpm test:docs`, staged whitespace check | PASS, 110 documents and 662 local links; no whitespace errors |

[Native results](native-results.json) and [rendered candidate/header meanings](desktop.png). Native lookup and restart preserved one synthetic vocabulary record, two cards/progress records, speech preferences and all three audio files, including selected audio bytes. The harness uses copies and stops its owned app. Reproduce from `apps/desktop`, supplying a fresh destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/issue-42/native-new
```

Independent standards and specification reviews compared the implementation with baseline `e4a3ac1f5d15fefe6aea1b53bc1e0162419430ae` and the accepted #42 spec. Both axes have no remaining confirmed findings in scope.

## Limitations

The small legacy English resource provides partial coverage and does not certify every lexical use. Fresh offline packaging and other OS/architecture execution are deferred at the user's request; Windows native execution does not establish network-blocked fresh-package acceptance. Combined package acceptance remains #49, and formal release readiness is not claimed.
