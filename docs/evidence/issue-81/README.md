# D2 content batch 001 (#81)

[Issue #81](https://github.com/zardonc/lexique/issues/81) completes frozen rows
0001–0050, retaining every headword and grammatical subclass. The fifty groups
contain 200 reviewed fields; five groups reuse the pilot. The complete development
resource now contains **57 groups / 228 fields**, with **4,030 of 4,087 groups
incomplete**. Content version: `core-81-v1`, resource format 1.

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

Both independent review axes pass. `pnpm test:docs` and `git diff --check` pass.

## Limits

This is a development content batch. Learner acceptance, CEFR labels, the
remaining 4,030 groups, whole-machine offline acceptance, other-platform execution
and formal release gates remain open. The broader resource-update/backup matrix
is recorded in [#80](../issue-80/README.md).
