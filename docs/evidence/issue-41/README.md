# Ordinary active expressions and reference tables

2026-11-06 status: all seven #41 acceptance criteria were checked against the retained coverage, CLI/browser/native, reproducibility and current Format 6 equivalence evidence. The completed Windows x64 slice is closed in GitHub; the first-process CLI latency limitation and combined #49 acceptance remain separately recorded.

[Issue #41](https://github.com/zardonc/lexique/issues/41) supplies all 23 ordinary active block identities through the public resource CLI and desktop. Exact expressions resolve with or without an appropriate subject, including subjunctive clauses. Every candidate and matching cell remains accessible; an unmatched expression never falls back to its component words. Surcomposés are excluded; the historical second conditional past appears only as subjunctive pluperfect.

The follow-up closes the previous 1,890 unresolved and 13 missing passé-composé admissions. All 7,826 source identities are accounted for in each block, with **zero missing or unconfirmed block statuses**. There are 7,687 admitted ordinary verbs and 139 source-only identities: 132 obligatory pronominal identities and seven presentatives/source sentinels. Passé composé is supported for 7,681 identities and unavailable for 145, including six admitted defective verbs without a past participle. There is no frequency cap. See the [full report](resource-report.json) and [explicit unavailable/excluded identities](unavailable-blocks.json).

## Admission and linguistic expectations

Original Lefff rows, tags, line references and spelling alternatives remain intact and queryable. Pinned [Lefff syntax](../../../resources/lefff-syntax/README.md) establishes ordinary/pronominal frames, auxiliary macros and explicit morphology availability. The newest file with positive ordinary frames takes precedence; older positive frames fill an absent ordinary use. The `@pron` macro distinguishes obligatory pronominal frames, rather than stale or optional `se` display text. Auxiliary exception macros and cited corrections select avoir, être or both, with separate use labels. Older transitivity labels never introduce a new auxiliary. This prevents false ordinary `j’abstiens`, `j’ai parvenu` and modern `j’ai resté`, while admitting ordinary `évaporer` and both constructions of `monter`.

[Grammalecte forms and notices](../../../resources/grammalecte-2.3.0/README.md) supply the suffix-coded simple forms and explicit participle dimensions; the original [decoder](../../../resources/grammalecte-2.3.0/conj.py) documents their representation. Lefff patterns establish defined blocks/persons, including infinitive-only and impersonal verbs. Missing compiled forms alone never establish defectiveness. Four omitted regular present participles use their explicit `v-er` pattern. Expected finite person slots are checked before a block is reported supported.

Generated rows retain provenance: `grammalecte` for supplemental morphology, `lefff-syntax/...` for the four pattern forms, `reference/<use>` for cited corrections, and `ordinary/v1/<auxiliary>/<use>` for compounds. Their source line anchors an existing Lefff identity; it does not assert that a generated expression occurs on that line. Unspecified original participles remain unspecified. Explicitly dimensioned participles supply être agreement; an unspecified Lefff `K` never creates gender/number alternatives. Avoir reference forms retain their source dimensions and do not infer preceding-object agreement from a lookup.

Sources and factual expectations are independently inspectable; reference tables are not redistributed as a new dataset:

- [Canada Translation Bureau, aimer](https://nos-langues.canada.ca/fr/conjugart/aimer): literal expectations independently check each of the 23 blocks, including literary forms and imperative compounds.
- [Usito, conjugaison verbale](https://sanction.usito.usherbrooke.ca/articles/th%C3%A9matiques/contant_1): ordinary constructions, agreement, and the distinct `ayant aimé`, `en aimant`, `en ayant aimé`.
- [Académie française, aller](https://www.dictionnaire-academie.fr/conjuguer/A9A1226): être and explicit participle alternatives, including `je suis allé / allée` and singular/plural vous. Incompatible subject/agreement combinations fail.
- [OQLF auxiliary synthesis](https://vitrinelinguistique.oqlf.gouv.qc.ca/24584/la-grammaire/le-verbe/auxiliaires/tableau-synthese-de-lemploi-des-auxiliaires-avoir-et-etre): auxiliary choice by use and modern être-only rester. Existing cited descendre/accourir/demeurer corrections remain.
- [Larousse, occire](https://www.larousse.fr/conjugaison/francais/occire-conditionnel/9794): positive future/conditional forms omitted by the other sources, including `j’occirai` and `j’occirais`. The [Académie table](https://www.dictionnaire-academie.fr/conjuguer/A9O0126) independently licenses all three compound imperative persons, including both SP/1s and YP/2s matches for `aie occis`.
- [Académie, forclore](https://www.dictionnaire-academie.fr/conjuguer/A9F1241) and [Le Robert, stupéfaire](https://dictionnaire.lerobert.com/conjugaison/stupefaire): compound imperatives remain available despite absent simple imperatives; Le Robert also attests the rare `il stupéfait` present.
- [Académie, choir](https://www.dictionnaire-academie.fr/conjuguer/A9C2105): both auxiliaries, singular present imperative and all three compound imperative persons. `ayons chu` retains subjunctive and imperative interpretations.
- [Académie, paître article](https://www.dictionnaire-academie.fr/article/A9P0092) and [table](https://www.dictionnaire-academie.fr/conjuguer/A9P0092): explicitly dimensioned `pu / pue / pus / pues` and compounds restricted to falconry. `j’ai pu` resolves both paître and pouvoir; the restriction remains visible.
- [Académie fifth edition, avenir](https://www.dictionnaire-academie.fr/article/A5A2084) and [Littré](https://www.littre.org/definition/avenir): historical verb, third persons only, using the admitted venir model. `il est avenu`, `elles sont avenues`, `il aviendra` and historical spellings remain accessible; first-person `je suis avenu` fails.
- [RosaeNLG french-contractions](https://github.com/RosaeNLG/rosaenlg/tree/ff38d3275e86567142506592d6e14ce679bf70e3/packages/french-contractions), [Canada elision guidance](https://nos-langues.canada.ca/fr/cles-de-la-redaction/elision), and [Bescherelle](https://bescherelle.ca/phonetique/elision-devant-h-muet/): lexical h/y behavior. Checks include `j’habite`, `je hais`, `je hurle`, `je harcèle`, `je hèle`, `je yorubaïse` and rejection of the opposite elision.

Positive attested rare/historical forms override omissions in another reference, with visible use labels. This is a stated reference policy, not universal agreement among dictionaries. `supported` describes admitted reference forms and defined person coverage; it does not certify every lexical sense. `not_applicable` identifies explicit ordinary-use, block or person restrictions. Original non-ordinary rows can still make a simple source block supported, without licensing ordinary compounds. Traditional group metadata remains independently uncertain for 20 identities; this does not suppress their source forms or alter the ordinary-block report.

Context-dependent avoir agreement, être subject agreement and singular/plural vous are explained beside the table. Each complete alternative is independently addressable. Labels and restrictions are translated in all three UI languages. Pronominal-only source rows retain bare expressions and never acquire fabricated ordinary subjects.

## Sources and licenses

Lefff forms and the pinned syntax files retain LGPL-LR. The current upstream website advertises CeCILL-C; the actual pinned syntax files carry LGPL-LR, which governs this admission. Grammalecte data retains GPL-3.0-only. RosaeNLG's aspirated-h list retains CC BY-SA 3.0 and its original attribution; reference code is Apache-2.0 and its upstream README declares CC BY 4.0. All original bytes, lossless compressed sources, full license texts, provenance hashes, dated modification notices, corrections and four MIT conversion modules accompany the generated resource. All pinned input hashes also match staged Git bytes; owned hashed text is normalized to LF before admission so checkout conversion cannot invalidate the payload. Original upstream whitespace remains intact. No upstream executable code is imported or executed. The application license and learning database are unchanged.

## Resource and costs

[Measured report](resource-report.json) and [timings/budgets](measurements.json), Windows 11 x64, AMD64 family 25 model 117, Python 3.14.10, Node 24.22.1, pnpm 11.25.0. Each measurement starts a fresh public CLI process, validates the resource, queries and serializes the complete table. Measurements followed task builds/tests, so the filesystem was warm; these are not OS-cold disk measurements.

| Measure | Result |
| --- | --- |
| Preserved Lefff rows / expanded source slots / identities | 342,771 / 397,413 / 7,826 |
| Total source and generated slots | 780,818 |
| Original Lefff / compressed bytes | 15,677,049 / 2,843,158 |
| Original Grammalecte JSON / compressed bytes | 367,485 / 76,393 |
| Syntax/elision sources, notices and provenance bytes | 765,718 |
| Database / gzip bytes | 123,875,328 / 26,482,276 |
| Lemma index / reverse index bytes | 14,376,960 / 17,252,352 |
| Build wall time | 16,018 ms |
| First process / warm median / warm maximum query | 1,737 / 1,453 / 1,592 ms |

The database remains below the enforced 128 MiB native snapshot bound. Full expressions exceed the old simple-resource ceilings and the previous 16 MiB reverse-index allowance; this evidence adopts an explicit 18 MiB reverse-index budget for the completed resource. **The first final CLI process (1,737 ms) misses the previous 1,700 ms query budget; the five warm measurements pass.** A prior measured resource revision reached 3,297 ms on its first process; its complete measurements and resource hash remain in the timing artifact to preserve the observed variation. This limitation is retained rather than reported as a full latency pass. CLI timing includes source parsing/integrity checks and is not a measurement of native IPC latency. The native app reads the already materialized, pinned read-only SQLite snapshot and does not duplicate linguistic generation.

## Verification

Commands use managed Python on PATH, pinned pnpm, and task-owned `target/` TEMP/TMP. Rust dependencies are locked. Backend test compilation is separate from the unchanged hard 60-second runner.

| Check | Result |
| --- | --- |
| `pnpm test:conjugation` | PASS, 22 public CLI tests in 254.345 s; all-block/resource-wide coverage, exact interpretations, auxiliary/use variants, agreement, rare/historical forms, elision, source preservation, reproducibility, malformed/incompatible/truncated resources, corrupt syntax inputs and stale-converter preservation |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | PASS |
| `pnpm test:e2e --workers=2` | PASS, all 105 browser tests in 54.9 s against final real-resource fixtures, including rare/historical corrections and all ambiguous highlights. Earlier ambiguous-expression test selector corrected before this complete rerun |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS, precompiled execution with hard 60-second runner |
| `pnpm test:resources` | PASS, 7 dictionary resource tests |
| `pnpm test:tooling` | PASS after release resource copy completed; speech-package integration skipped without separately verified inputs. Portable unit checks rerun after notice-only update |
| `pnpm desktop:build --no-bundle` | PASS, Windows x64 release including production frontend |
| Real Windows `native-conjugation-check.mjs` | PASS after the same assertion correction; actual ordinary/rare/auxiliary/elision IPC and UI, 23 blocks, ambiguity highlights, feminine alternative, explicit defectiveness, invalid/missing/corrupt resource failures and normal restart |
| Export staged Git bytes into a fresh workspace, then public CLI build | PASS, [same final database SHA-256](staged-byte-rebuild.json); verifies checkout-safe pinned sources independently of working-tree line endings |
| `pnpm test:docs`, staged whitespace check | PASS; whitespace check excludes the byte-preserved upstream `lefff-syntax/README`, whose original seven-equals heading is interpreted by Git as a conflict marker |

[Native results](native-results.json), [rendered defective table](desktop.png), and [simple ambiguity table](simple.png). Lookup, browsing, failure/replacement and normal restart preserved one synthetic vocabulary record, two cards/progress records, saved speech preferences and all three audio files, including selected audio bytes. The harness starts/stops only its owned app and uses fixture copies. Reproduce from `apps/desktop` with a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/issue-41/native-new
```

Preparation uses `target/desktop-conjugation-v3`. Existing incompatible/stale output fails visibly and is preserved; rebuild into a new directory with `pnpm conjugation:build <new-directory>`. Keep both supplemental source directories beside the database. Learning data is never migrated or rebuilt by resource preparation.

Independent standards and specification reviews checked the follow-up against `1ee125d` and the accepted #41 spec (original review baseline `5d18ccf7514240ef0630aaf2f88096ce56aedf9d`). Confirmed rare-form, auxiliary, pronominal and defectiveness findings were corrected; both review axes report no remaining confirmed findings in scope.

## Deferred verification

At the user's request, no fresh offline package or other OS/architecture acceptance was run. Windows native execution does not establish enforced-network-blocked first launch; #49 retains combined fresh-package acceptance. Formal release readiness is not claimed. Pronominal generation, query typography normalization, English meanings, colors, ordering, navigation and speech remain their separate tickets. The first-process CLI latency overrun above remains a measured limitation.
