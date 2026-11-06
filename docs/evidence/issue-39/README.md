# Ordinary verb metadata validation

[#39](https://github.com/zardonc/lexique/issues/39) extends the public conjugation CLI with traditional groups, ordinary auxiliary/use metadata, original raw flags, cited corrections, and explicit unresolved coverage. All Lefff forms and matches remain available. Desktop integration and complete M3 generation admission remain separate gates.

## Inputs, design, and licensing

Reuse the pinned [Lefff source and LGPL-LR materials](../../../resources/lefff-3.4/README.md). Add [Grammalecte 2.3.0 metadata](../../../resources/grammalecte-2.3.0/README.md) from mirror commit `08511c222029b3be20bf547563315d10fb48d44b`. Its original JSON supplies 8,403 identities and explicit type flags; metadata is projected only onto existing Lefff identities. The complete original JSON, decoder reference, attribution, GPL 3 license, corrections, hashes, and conversion version are retained. No upstream generator is executed.

The maintained [Grammalecte project](https://grammalecte.net/) and pinned [decoder](https://github.com/Pofilo/grammalecte/blob/08511c222029b3be20bf547563315d10fb48d44b/gc_lang/fr/modules/conj.py) establish the layout. Its default-to-avoir behavior is unsuitable for incomplete auxiliary fields; this CLI retains an unconfirmed state. The existing [RosaeNLG design](https://github.com/RosaeNLG/rosaenlg/tree/ff38d3275e86567142506592d6e14ce679bf70e3/packages/french-verbs) separates form lists from heuristic auxiliary selection. The maintained [verbecc source](https://github.com/bretttolbert/verbecc/tree/a5cab52168cda317a358222448a8a7c5fe58657a) was also inspected: its Verbiste-derived list supplies templates rather than the required explicit auxiliary/use distinctions. Neither alternative contributes new code or data.

Lefff forms remain LGPL-LR; Grammalecte-derived metadata remains GPL-3.0-only; Lexique's independent converter remains MIT. Tables retain separate provenance and notices. Future product distribution must include both sources and corresponding conversion/correction source, not just a database or the application's MIT notice.

## Independently expected examples

The test literals use the agreed traditional classification and independent grammar references, rather than converter output:

| Lemma | Group | Ordinary auxiliary/use |
| --- | --- | --- |
| aimer | 1 | avoir |
| finir | 2 | avoir; finir/finissons pattern |
| aller | 3 | être; correction to upstream internal group 1 |
| partir | 3 | être; sharing finir's infinitive suffix does not imply group 2 |
| avoir | 3 | avoir; upstream auxiliary group 0 mapped to the agreed category |
| être | 3 | avoir; same group mapping |
| descendre | 3 | avoir for transitive use; être for intransitive use |
| falloir | 3 | avoir; impersonal, without invented first-/second-person cells |

References: [aimer](https://www.dictionnaire-academie.fr/conjuguer/A9A1016), [finir](https://www.dictionnaire-academie.fr/conjuguer/A9F0830), [aller](https://www.dictionnaire-academie.fr/conjuguer/A9A1226), [descendre](https://www.dictionnaire-academie.fr/conjuguer/A9D1797), [falloir](https://www.dictionnaire-academie.fr/conjuguer/A9F0145), and the [Ministry of Education grammar terminology](https://eduscol.education.gouv.fr/sites/default/files/document/guide-la-grammaire-du-francais-terminologie-grammaticale-67998.pdf) for finir versus partir. These establish factual expectations; complete definitions/tables are not copied into the resource.

[Corrections](../../../resources/grammalecte-2.3.0/corrections.json) also retain source-cited auxiliary distinctions for accourir and demeurer. Homographic ressortir uses retain an unconfirmed group pending sense separation. Original flags remain visible beside corrections.

## Actual coverage and measurements

[Public results](actual-results.json), the full [coverage/size report](coverage-and-sizes.json), and [samples/environment](measurements.json) are retained.

| Measurement | Result |
| --- | --- |
| Source verb rows / expanded slots | 342,771 / 397,413, unchanged |
| Queryable lemmas / forms | 7,826 / 301,208, unchanged |
| Supported / unconfirmed traditional group | 6,573 / 1,253 |
| Supported / unconfirmed / inapplicable ordinary auxiliary | 1,310 / 6,361 / 155 |
| Missing supplemental identities | 1,252; their forms remain queryable |
| Excluded source rows | 0; unresolved artifacts retain source-line provenance |
| Original Lefff / compressed bytes | 15,677,049 / 2,843,158 |
| Original metadata / compressed bytes | 367,485 / 76,393 |
| SQLite / compressed bytes | 35,667,968 / 10,038,376 |
| Lemma index / reverse index bytes | 7,311,360 / 7,942,144 |
| Build wall clock | 3,841 ms |
| First process / warm filesystem median / warm maximum | 841 / 752 / 788 ms |

Measurements use Windows 11 x64, Python 3.14.10, SQLite 3.53.1, and the recorded hardware. Every sample invokes a fresh public CLI process, including startup, validation, lookup, and JSON output. The first sample is cold-process; ten repeats warm filesystem caches. The build already touched the database, so OS-cold disk latency and persistent native-reader latency are not established.

Initial regression budgets on this dataset/environment are 40 MiB for the database, 8 MiB for the reverse index, and 1,700 ms for the complete CLI query. They round measured sizes upward and allow approximately twice the largest observed query duration. They are implementation ceilings, not later UI latency commitments.

## Verification and reproduction

Build into a new directory and retain the database's supplemental directory:

```sh
pnpm conjugation:build target/issue-39-reproduction
pnpm conjugation:query target/issue-39-reproduction/conjugation.sqlite3 sommes
pnpm conjugation:table target/issue-39-reproduction/conjugation.sqlite3 descendre
pnpm test:conjugation
pnpm typecheck
pnpm test:docs
git diff --check
```

For measurements, time the query once and then ten more times; retain all durations and report the first separately from the remaining median/maximum. Table/query JSON adds `verb` with group, auxiliary, ordinary-use states, use labels, raw identity/type, and correction references. Build stdout and public `report.json` include resource-wide counts and complete missing/unconfirmed lemma lists.

The 14-test public CLI suite covers the independent examples, all original sommes/suis interpretations and spelling variants, unknown versus inapplicable metadata, byte-identical repeated builds, and copied source/license materials. Failure checks reject missing, corrupt, truncated, incompatible, or incomplete inputs; incompatible generated format; missing metadata rows; and changed generated metadata, including JSON null. Existing valid resources remain unchanged after rejected builds.

On this Windows sandbox, set `TEMP` and `TMP` to a writable directory beneath `target/`. Tests ran with managed Python 3.14.10 and the same unittest discovery command used by `pnpm test:conjugation`; typechecking used pnpm 11.25.0. Rust and browser/native app checks are not applicable to this Python/resource-only change.

| Check | Result |
| --- | --- |
| `python -X utf8 -m unittest discover -s scripts -p test_conjugation_sample.py -v` | Passed: 14 tests, 55.060 seconds; each CLI invocation retains its 60-second timeout |
| `pnpm typecheck` | Passed |
| `pnpm test:docs` | Passed: 104 Markdown documents, 627 local links; 67 existing local-evidence links counted separately |
| `git diff --cached --check` | Passed |
| Independent standards and spec reviews | Passed; no unresolved findings |

## Limits and remaining gates

Auxiliary information is unconfirmed for 6,361 lemmas, including incomplete upstream `zz` records and multiple auxiliaries without validated sense mappings. Group is unconfirmed for 1,253 lemmas. The CLI reports these gaps instead of defaulting to avoir, guessing from suffixes, hiding verbs, or claiming complete generation coverage. The ordinary generation ticket must resolve the gaps needed for its claims.

Replaceable resource format 2 rejects old version-1 samples; existing outputs remain untouched and can be regenerated into new directories. No learning database migration or access occurs. English meanings, full compound/pronominal generation, ending spans, desktop integration, real first-launch offline package acceptance, and other-platform execution belong to later tickets.
