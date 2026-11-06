# Ordinary active expressions and reference tables

[Issue #41](https://github.com/zardonc/lexique/issues/41) adds ordinary compound and nonfinite constructions to the resource CLI and desktop. All 23 block identities are present. Exact expressions resolve with or without a valid subject, including subjunctive clauses; candidates and every matching cell remain accessible. Unsupported expressions return no match rather than matching their component words.

Source Lefff rows, tags, line references and spelling alternatives remain intact. Explicit Grammalecte forms fill available simple forms and participle dimensions. Generated rows are identified by `raw_tag`: `grammalecte` for supplemental forms and `ordinary/v1/<auxiliary>/<use>` for compounds. Their `source_line` anchors the existing Lefff identity; it is not a claim that the compound occurs on that source line. Gender and number are taken from explicitly dimensioned participles, never expanded from an unspecified Lefff `K`. Each complete expression is separately addressable. Context-dependent avoir agreement is explained next to the reference table; subject agreement and singular/plural vous are explained without inferring sentence context.

## Admission and linguistic expectations

The existing pinned [Grammalecte source and notices](../../../resources/grammalecte-2.3.0/README.md) supply the suffix-coded forms and the independently inspectable [decoder design](../../../resources/grammalecte-2.3.0/conj.py). No upstream executable code is imported, copied into application code, or executed. Its data remains GPL-3.0-only; Lefff remains LGPL-LR. The original JSON, decoder, attribution, license texts, updated dated modification notice and all three MIT conversion modules accompany the generated resource. The application license and learning database are unchanged.

Independent expectations are factual checks against these primary language references; their tables are not redistributed as a new dataset:

- [Canada Translation Bureau, aimer](https://nos-langues.canada.ca/fr/conjugart/aimer): finite/simple/compound pairings, literary forms, imperative and compound participle. The CLI regression verifies a literal expectation in each of the 23 blocks independently.
- [Usito, conjugaison verbale](https://sanction.usito.usherbrooke.ca/articles/th%C3%A9matiques/contant_1): gerunds, ordinary construction and agreement rules. Checks include the distinct forms `ayant aimé`, `en aimant`, and `en ayant aimé`.
- [Académie française, aller](https://www.dictionnaire-academie.fr/conjuguer/A9A1226): être auxiliary and explicit participle alternatives. Checks include first-person masculine/feminine and plural vous expressions; incompatible subject/agreement combinations are rejected.
- [OQLF auxiliary synthesis](https://vitrinelinguistique.oqlf.gouv.qc.ca/24584/la-grammaire/le-verbe/auxiliaires/tableau-synthese-de-lemploi-des-auxiliaires-avoir-et-etre): transitive ordinary constructions license avoir despite incomplete upstream auxiliary flags. Listed movement uses of entrer/rentrer, retourner, sortir/ressortir and tomber license être with a distinct use label. Existing cited descendre/accourir/demeurer corrections remain. Missing intransitive auxiliary metadata is not converted to a universal avoir default.

`aie aimé` independently exercises both subjunctive first-person and imperative second-person interpretations. Subject-bearing queries filter those interpretations appropriately. Surcomposés are not generated; the historical second conditional past is represented only by the subjunctive pluperfect block.

## Resource and costs

[Complete measured report](resource-report.json). Windows 11 x64, AMD64 family 25 model 117, Python 3.14.10, Node 24.22.1, pnpm 11.25.0. Each query starts a new public CLI process, validates the resource, queries and serializes the table. The build touched the database before the first query: these establish cold-process and warm-filesystem costs, not OS-cold disk latency.

| Measure | Result |
| --- | --- |
| Preserved Lefff rows / expanded source slots / identities | 342,771 / 397,413 / 7,826 |
| Total source and generated slots | 684,232 |
| Original Lefff / compressed bytes | 15,677,049 / 2,843,158 |
| Original supplemental JSON / compressed bytes | 367,485 / 76,393 |
| Database / gzip bytes | 102,318,080 / 22,799,410 |
| Lemma index / reverse index bytes | 12,472,320 / 14,770,176 |
| Build wall time | 12,847 ms |
| First process / warm median / warm maximum query | 1,009 / 990 / 1,044 ms |

The #39 simple-resource size ceilings (40 MiB database, 8 MiB reverse index) are exceeded by materializing full expressions and their validated subject alternatives. The expanded resource uses a 128 MiB native snapshot bound; the reverse-index budget is 16 MiB. The existing 1,700 ms complete-CLI query budget passes. This trades larger replaceable resource storage and per-query snapshot memory for one shared build representation; native code does not duplicate linguistic generation rules.

Every report block accounts for all 7,826 identities. `supported` means the block has admitted forms, not that every person or lexical use has been certified. `not_applicable` marks verified ordinary-use inapplicability or strict impersonal person constraints; `missing` marks absent construction inputs; `unconfirmed` marks unresolved admission/metadata. Missing simple forms alone never establish defectiveness.

## Verification

Commands use the managed Python directory on PATH, pinned pnpm, and task-owned `target/` TEMP/TMP on Windows. Rust commands use locked dependencies. Initial test compilation is separate from the unchanged hard 60-second backend runner.

| Check | Result |
| --- | --- |
| `pnpm test:conjugation` | PASS, 17 public CLI tests; includes 23 independent block expectations, all compound interpretations, both auxiliaries/use labels, agreement, source preservation, reproducible bytes, malformed/incompatible/truncated resources and preservation on failure |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | PASS |
| `pnpm test:e2e --workers=2` | PASS, 105 browser tests; real-resource compound fixtures, two-match ambiguity, variants, h/y uncertainty, errors, keyboard operation, languages/themes and existing application regressions |
| `pnpm rust:fmt`, `pnpm rust:check` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS, precompiled execution with hard 60-second runner |
| `pnpm test:resources` | PASS, 7 dictionary resource tests |
| `pnpm test:tooling` | PASS; speech-package integration skipped without its separately verified inputs |
| `pnpm desktop:build --no-bundle` | PASS, Windows x64 release including production frontend |
| Real Windows `native-conjugation-check.mjs` | PASS; actual compound IPC, table rendering, all ambiguous highlights, feminine alternative, no-match, invalid/missing/corrupt resource failures and restart |

[Native results](native-results.json) and [rendered native table](desktop.png). Lookup, browsing, resource failure/replacement and normal restart preserved one synthetic vocabulary record, two cards/progress records, saved speech preferences and three audio files, including selected audio bytes. The harness starts and stops only its owned application, using fixture copies. Reproduce from `apps/desktop` with a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/issue-41/native-new
```

Build preparation uses `target/desktop-conjugation-v3`, preserving older format-2 output. Generated resources must retain the supplemental directory beside the database. Rebuild in a new directory with `pnpm conjugation:build <new-directory>`; learning data is never migrated or rebuilt by this operation.

Independent standards and specification reviews used the pre-task commit `5d18ccf7514240ef0630aaf2f88096ce56aedf9d` and working diff. Neither review has remaining confirmed findings in the implemented supported behavior; the resource-wide completeness gate below remains open. Staged document/link and whitespace checks pass.

## Remaining acceptance gates

**#41 remains open for resource-wide linguistic completeness.** Passé composé has supported forms for 5,768 identities, 155 verified ordinary-inapplicable identities, 1,890 unresolved identities and 13 missing construction inputs. Of the unresolved identities, 1,256 have unconfirmed ordinary-use metadata; the report retains the missing lemma and auxiliary lists. Some admitted transitive forms coexist with unresolved other uses. A sample passing all 23 blocks does not settle these resource-wide gaps. Additional independently justified lexical metadata/forms are required before claiming complete ordinary support for every usable source verb.

Unresolved h/y subject elision remains explicitly labelled and displayed as a source form. Agreement beyond reference forms requires sentence context. Pronominal generation, query typography normalization, English meanings, colors, ordering, navigation and speech remain their separate tickets. The Windows check does not prove a fresh package under enforced network blocking; #49 retains combined offline acceptance. Other architectures/macOS and formal release readiness were not tested.
