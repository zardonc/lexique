# Lexique FR–EN–ZH D2 core

This development resource contains **57 independently reviewed senses / 228
fields** in 57 frozen headword/POS groups, content version `core-81-v1`.
[Batch 001 #81](https://github.com/zardonc/lexique/issues/81) retains all fifty
groups in frozen rows 0001–0050 and their **200 reviewed fields**, including five
reused pilot groups and 45 new groups. **All fifty groups meet the approved batch
scope: 49 everyday groups and one documented specialist exception**. Frozen row
34, `ça/NOM`, retains its sourced psychoanalytic *id* sense with masculine,
singular and uncountability restrictions. The project owner's #81 decision is
recorded in the usage review; this row remains outside literal everyday coverage.
All twelve original pilot groups remain in the resource.
Its permanent target is **4,087 groups / 3,773 headwords**; **57 groups are
accepted and 4,030 remain unaccepted**. Literal everyday coverage is **56 complete
groups and 4,031 incomplete groups**, including the approved specialist noun.
Learner acceptance,
the complete D2 release and other platform/release gates remain open. It supplies
no CEFR labels.

[curated-inputs.json](curated-inputs.json) contains the selected French definition,
IPA, English and Chinese fields, exact POS/use, source revision/spans and actual
modifications. [field-review.json](field-review.json) preserves the separate
reviewer's exact accepted text, selected-use evidence, explicit everyday-sense
decisions and the project owner's specialist exception. Construction, register
and domain restrictions remain part of the complete meanings.
[selected-source-excerpts.json](selected-source-excerpts.json)
contains only selected excerpts; [selected-source-context.json](selected-source-context.json)
retains applicable source/import notices, fixed article/history attribution and
separately labelled older authorship/merger evidence. Older revisions do not
replace the fixed current content versions.

[scope-manifest.csv](scope-manifest.csv) preserves the accepted four-column
membership and row order. Frozen `coeur` remains distinct from source `cœur`;
frozen `aujourd'hui` remains distinct from source `aujourd’hui`. Frozen `l'` and
`d'` retain their article/pronoun and preposition groups; source `l’` and `d’`
have fixed redirect evidence. The four query aliases use these specific source
relationships, with separate article and pronoun records for `l’`.

Selected/adapted content and licensed membership use **CC BY-SA 4.0**. Read
[NOTICES.md](NOTICES.md), [COPYING](COPYING), and
[MEMBERSHIP-NOTICES.txt](MEMBERSHIP-NOTICES.txt). The actual Lexique upstream
credits and source notices are [LEXIQUE-UPSTREAM-README.txt](LEXIQUE-UPSTREAM-README.txt)
and [lexique-provenance.json](lexique-provenance.json). Application/converter
code uses the separate MIT licence.

From a repository checkout, run `pnpm core:prepare` and `pnpm test:core`. The
prepared directory includes the actual curation, selected excerpts, notices,
licences, coverage, converter and input hashes. From that standalone directory,
reproduce it with `python core_pilot.py prepare rebuilt --source-dir . --project-license LICENSE-MIT`.
The preparation self-check verifies every admitted field against its independent
review and the produced SQLite. `coverage.json` lists literal everyday gaps and
missing fields alongside accepted groups and the approved exception. `report.json`
records scope acceptance separately from everyday-sense completeness. The converter
accepts the specialist decision only for frozen row 34, `ça/NOM`, bound to #81
and the project owner's approval. Field completeness counts all reviewed fields.

Whole Wiktionary pages, unselected dictionary bodies, literary/comic quotations,
images and audio are outside this resource. OQLF and the Canadian Translation
Bureau corroborate grammar facts; their prose is excluded.
