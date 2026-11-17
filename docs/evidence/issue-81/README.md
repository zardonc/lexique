# D2 content batch 001 (#81)

[Issue #81](https://github.com/zardonc/lexique/issues/81) retains all fifty frozen
groups in rows 0001–0050 and their 200 reviewed fields; five groups reuse the
pilot. **All fifty groups meet the approved scope: 49 everyday groups and one
documented specialist exception**.
The development resource contains **57 groups / 228 reviewed fields**, with
**57 accepted groups and 4,030 of 4,087 groups unaccepted**. Literal everyday
coverage remains **56 complete groups and 4,031 incomplete groups**. Content version:
`core-81-v1`, resource format 1.

Frozen row 34, `ça/NOM`, retains the reviewed psychoanalytic *id* sense and all
four accurately scoped fields. The fixed Wiktionnaire revision 39860132, line
129, labels that noun as psychoanalytic; the accepted usage does not establish
an everyday nominal use. On 2026-11-16 (America/Vancouver), the project owner
approved this exact frozen group as the sole specialist exception. Its masculine,
singular and uncountability restrictions remain explicit; `ça/PRO:dem` stays
separate. The [primary-source research](../../d2-ca-nom-research.md) records the
lexical and rights evidence. The converter requires the bound #81 approval and
reports acceptance separately from literal everyday coverage. Field completeness
remains 200 for the batch and 228 for the resource. #81 remains open for merge
closeout; learner acceptance and full-release gates remain open.

## Content and redistribution

The [curation](../../../resources/core-pilot-79/curated-inputs.json) binds every
French definition, IPA, English and Chinese meaning to its selected POS/use,
fixed source revision/span, contributor history, licence and actual modifications.
The [independent field review](../../../resources/core-pilot-79/field-review.json)
retains exact accepted text and per-field quality/rights evidence. All twelve
original pilot groups retain their content. The distinct article/pronoun,
indefinite/numeral adjective, locative/nonlocative `en`, and five `tout` groups
remain separate; psychoanalytic `ça` retains its explicit domain restriction.

The [source context](../../../resources/core-pilot-79/selected-source-context.json)
retains the separate selected-source rights review, import notices and grammar
corroboration. Selected data and membership use CC BY-SA 4.0; code retains MIT.
The produced resource includes the actual licence, notices, curated inputs,
accepted review, selected excerpts, frozen membership, converter and coverage.
Independent artifact inspection compared all new fields with their accepted text
and verified the actual redistribution materials and fixed denominator.

Frozen `l'` and `d'` retain their exact membership spellings. Fixed redirects
support only their explicit curly-apostrophe aliases; arbitrary folding is not
introduced. Native and frontend validation retain strict grammatical categories,
including all twelve additional subclasses, with labels in all three locales.

## Verification

Windows x64 execution evidence is retained under
`C:\Users\inthe\IdeaProjects\lexique-81\target\issue-81`; the source base is
`a60f216727b3bd9d50d3c2176fbd7223e9efb493` plus the batch commit's changes.

| Check | Result |
| --- | --- |
| `pnpm test:core` | 5 passed: exact frozen membership, four-field review binding, reproducibility, rejection and unchanged output |
| `pnpm test:tooling` | Passed, including portable resource version validation |
| `pnpm lint`, `pnpm typecheck` | Passed |
| `pnpm test:e2e` | 391 passed, including all twelve new subclasses through lookup and rendering |
| `pnpm rust:test:build`, `pnpm test:rust` | Passed; 194 backend tests, with the existing hard 60-second runner limit |
| `pnpm rust:check`, `pnpm rust:fmt` | Passed |

### Everyday use and coverage

Windows x64 coverage checks use Python 3.14.10 through
`uv run --no-project --no-python-downloads --cache-dir target/uv-cache --python 3.14.10`.
The checkout is
`C:\Users\inthe\IdeaProjects\lexique\target\issue-81-review-fix-20261116\worktree`,
based on `b354b78a5fde916ccbc880b086fb1572bcbe1cc1` plus the coverage changes.
Logs and prepared artifacts are retained in its `target/review-81/` directory;
the source diff is `verified-source.diff`.

| Check | Result |
| --- | --- |
| `python -B -X utf8 -m unittest discover -s scripts -p test_core_pilot.py -v` | 7 passed, including the specialist-use gap and missing, invalid or unexplained everyday-use decisions; `core-suite.log` |
| `python -B -X utf8 scripts/core_pilot.py prepare target/review-81/after` | Passed: 49 complete batch groups, 200 batch fields, 56 complete global groups and 4,031 incomplete; `prepare.log` |
| `python -B -X utf8 target/review-81/after/core_pilot.py prepare target/review-81/standalone --source-dir target/review-81/after --project-license target/review-81/after/LICENSE-MIT` | Passed; all 17 prepared files reproduce byte-for-byte, and the 286,720-byte SQLite matches the retained batch artifact; `standalone.log` and `artifact-compare.log` |
| `pnpm test:docs`, `git diff --check` | Passed |

### Approved specialist exception

The 2026-11-16 (America/Vancouver) approval resolves the noun's selected-scope
requirement while retaining its psychoanalytic restrictions. Windows x64 checks
use Python 3.14.10 via `uv run --python 3.14.10`; the checkout is
`C:\Users\inthe\IdeaProjects\lexique\target\ca-nom-research\worktree`, based on
`7de4b213298c219bf812562faea79c6621631025` plus the exception changes.
Complete logs, prepared artifacts and `verified-source.diff` are retained under
that checkout's `target/ca-nom/` directory.

| Check | Result |
| --- | --- |
| `python -B -X utf8 -m unittest discover -s scripts -p test_core_pilot.py -v` | 8 passed, including acceptance without inflated everyday coverage and rejection of missing, unapproved, unexplained or wrongly bound exceptions; `core-suite.log` |
| `python -B -X utf8 scripts/core_pilot.py prepare target/ca-nom/after` | Passed: 50 accepted batch groups (49 everyday + 1 approved specialist), 200 batch fields; 57 accepted global groups, 56 everyday groups and 4,030 unaccepted groups; `after-prepare.log` |
| `python -B -X utf8 target/ca-nom/after/core_pilot.py prepare target/ca-nom/standalone --source-dir target/ca-nom/after --project-license target/ca-nom/after/LICENSE-MIT` | Passed; all 17 prepared files reproduce byte-for-byte, and before/after SQLite is byte-identical at 286,720 bytes; `standalone.log`, `artifact-compare.log` |
| `pnpm test:docs`, `git diff --check` | Passed |

The dictionary fields, provenance, frozen identities, aliases and resource version
are identical in the produced SQLite. The approval changes curation/review scope
and preparation reports; native and browser evidence remains attached to the
original batch implementation below.

## Native execution and package

`pnpm desktop:build --no-bundle` and `python -X utf8 scripts/package_portable.py
--output-dir target/issue-81/packages-final` passed. The portable ZIP contains
183 verified files and is 35,212,817 bytes; the core SQLite is 286,720 bytes.

From `apps/desktop`, with the pinned Python/MSVC environment, the native command
was:

```powershell
node e2e/native-d2-pilot-check.mjs pilot ../../target/issue-81/packages-final/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../resources/core-pilot-79/curated-inputs.json ../../target/issue-81/native/pilot-05
```

The successful run compares actual native lookup with every audited sense and
field-source record, saves all admitted headwords through the trusted provider,
checks the NFC and four evidenced spelling aliases, and verifies normal restart.
The batch representative `le` reopens in notebook details with both article and
pronoun fields and provenance matching the audited input. The helper uses the
existing 100-word page capacity to keep all saved batch headwords selectable.
Duplicate/retry handling, forged digest/reference rejection, notes, examples,
card identities and FSRS state pass the existing preservation assertions.

`native/pilot-05/results.json`, `run-metadata.json`, `lifecycle.jsonl` and
`cleanup.json` retain executable identity, calls, assertions and cleanup.
External WebView requests are empty, and remaining owned hosts/workers/helpers
are zero. The native runner retains its hard 60-second execution limit.

The field and source reviews pass; the approved specialist scope for `ça/NOM` is
recorded above. `pnpm test:docs` and `git diff --check` pass for the recorded batch.

## Limits

This is a development content batch. Learner acceptance, CEFR labels, the
remaining 4,030 unaccepted groups, whole-machine offline acceptance, other-platform execution
and formal release gates remain open. The broader resource-update/backup matrix
is recorded in [#80](../issue-80/README.md).
Literal everyday coverage remains incomplete for 4,031 groups. Browser and native
suites were not rerun for the coverage and scope changes; their evidence above
belongs to the batch implementation.
