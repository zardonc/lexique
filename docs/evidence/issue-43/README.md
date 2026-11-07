# Pronominal expressions and agreement (#43)

2026-11-06: exact desktop and CLI lookup supports source-admitted pronominal infinitives, simple expressions and compounds across the applicable 23 blocks. **2,955 pronominal identities** cover 2,954 original lemmas alongside all 7,826 ordinary identities. Every original lemma has an admission or exclusion outcome; the remaining 4,872 have no explicit pronominal use. No frequency cap applies. The pronominal block report has no missing or unconfirmed entries; explicit grammatical restrictions remain separate.

## Sources and behavior

The [pinned policy](../../../resources/lefff-syntax/pronominal-rules.md) describes independent projection rules from the admitted Lefff and Grammalecte sources, with RosaeNLG lexical h/y elision. Original notices, source entry references and separate licenses remain attached. Six MIT converter modules accompany format 5 in `target/desktop-conjugation-v5`; older generated resources remain separate. No new dependency or upstream executable code is required.

Ordinary and pronominal identities remain distinct: `laver` and `se laver`, `se vouloir` and `s’en vouloir`. Positive source frames admit `se découdre`; the ordinary locution `en découdre` does not create `s’en découdre`. Whole-expression lookup retains all valid interpretations and highlights individual alternatives. It does not match a compound by just one component.

Independent CLI expectations include `je me lave`, `lave-toi`, `en se lavant`, `elles se sont abstenues`, `elles se sont parlé`, `elle s’est arrogé`, `nous nous sommes écrites`, `on s’est abstenus`, `va-t’en` and `je m’en suis allé`. Subject agreement preserves masculine/feminine alternatives, singular/plural `vous` and collective `on`; plural participles with `on` do not license `il/elle` plural agreement. Indirect reflexives with a competing direct object retain contextual alternatives. Contextual tables explicitly explain the reference participle and preceding-object dependence without supplying an absent object. Impersonal and reciprocal person restrictions are preserved; invalid pronouns and unsupported constructions return no match.

English remains independent of conjugation coverage: 1,493 of 10,781 identities have 2,089 source senses; 9,288 explicitly show unavailable meanings. New pronominal identities use explicit pronominal source headwords, including `s’en aller`, and retain their source spelling. Ordinary metadata and meanings remain intact. Lookup does not save vocabulary or enroll review cards.

## Resource and costs

[Complete coverage and exclusions](resource-report.json), [timings and budgets](measurements.json), and [rebuild from staged Git bytes](staged-byte-rebuild.json).

| Measure | Result |
| --- | --- |
| Ordinary / pronominal / total identities | 7,826 / 2,955 / 10,781 |
| Total source and generated slots | 1,451,539 |
| Database / gzip bytes | 250,650,624 / 49,581,488 |
| Database increase over format 4 | 125,915,136 bytes |
| Lemma index / reverse index bytes | 28,651,520 / 35,909,632 |
| Resource build wall time | 20,563 ms |
| Native snapshot bound / reverse index budget | 256 MiB / 36 MiB |
| First measured CLI process / warm median / warm maximum | 10,333 / 2,485 / 2,546 ms |
| Existing 1,700 ms CLI budget | MISS, all six samples exceed it |

The larger resource materializes complete, separately matchable pronominal alternatives. Its database fits the explicit 256 MiB native read-only snapshot bound, and its reverse index fits the 36 MiB evidence budget. The existing 1,700 ms CLI latency budget is evaluated separately in the measurements. The snapshot limit is a file-size bound, not a claim about peak process memory.

## Verification

All checks use Windows x64, managed Python 3.14.10, Node.js 24.22.1, pnpm 11.25.0 and the locked Rust toolchain. Backend tests are precompiled separately; the runner retains its hard 60-second execution limit.

| Command / boundary | Result |
| --- | --- |
| `pnpm test:conjugation` | PASS, 30 public CLI tests in 612.479 s; source admission, agreement, whole matches, failures, ordinary preservation and reproducibility |
| `pnpm lint`, `pnpm typecheck` | PASS |
| `pnpm desktop:build --no-bundle` | PASS, production frontend and real Windows x64 release |
| `pnpm test:e2e --workers=2` | PASS, 106 tests in 43.9 s; exact matches, alternatives, keyboard, three languages/themes and existing flows |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS, 138 precompiled backend tests |
| `pnpm test:resources` | PASS, 7 tests |
| `pnpm test:tooling` | PASS, 2 build-argument, 8 portable and 2 speech failure checks; 1 speech-package integration check skipped without separately verified inputs |
| Windows `native-conjugation-check.mjs` | PASS, actual resource lookup, ordinary/pronominal identity, agreement, 23 blocks, candidate/highlight behavior, visible resource failures and normal restart |
| Export Git index bytes, then public CLI build | PASS, 42 files exported, pinned source validation and identical database |
| `pnpm test:docs`, staged whitespace check | PASS, 112 documents and 673 local links; no whitespace errors |

[Native results](native-results.json) and [rendered pronominal metadata](desktop.png). The native harness uses an owned copy and stops its app. Restart preserves one synthetic vocabulary record, two cards with progress, speech preferences and all three audio files, including selected audio bytes. Reproduce from `apps/desktop` with a fresh destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/issue-43/native-new
```

Independent standards and specification reviews against baseline `992704079139681ceb43dd067e1e2581d79e7dcb` and the accepted #43 spec have no remaining confirmed findings in scope. Public CLI regressions cover indirect-object/contextual agreement, collective `on`, explicit `en` meanings and separate admitted constructions.

## Limitations

These are source-backed reference constructions and labelled contextual alternatives; they do not analyze arbitrary sentences or certify every lexical sense. English coverage is partial. The larger resource exceeds the existing CLI latency budget: first measured process 10,333 ms, warm median 2,485 ms. These timings include CLI resource validation and are not native IPC measurements. Fresh offline packaging and other OS/architecture execution are deferred at the user's request. Windows native checks do not establish fresh-package network-blocked acceptance, combined #49 acceptance or formal release readiness.
