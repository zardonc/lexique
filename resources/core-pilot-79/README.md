# Lexique FR–EN–ZH D2 core

This development resource contains **57 independently reviewed senses / 228
fields** in 57 frozen headword/POS groups, content version `core-81-v1`.
[Batch 001 #81](https://github.com/zardonc/lexique/issues/81) retains all fifty
groups in frozen rows 0001–0050 and their **200 reviewed fields**, including five
reused pilot groups and 45 new groups. **49 of the fifty groups meet the modern
everyday-sense requirement**. Frozen row 34, `ça/NOM`, has a sourced psychoanalytic
*id* sense whose everyday nominal use remains unresolved. Its four fields remain
complete and readable; the group is included in the incomplete coverage list.
All twelve original pilot groups remain in the resource.
Its permanent target is **4,087 groups / 3,773 headwords**; **56 groups are
complete and 4,031 remain incomplete**. Learner acceptance,
the complete D2 release and other platform/release gates remain open. It supplies
no CEFR labels.

[curated-inputs.json](curated-inputs.json) contains the selected French definition,
IPA, English and Chinese fields, exact POS/use, source revision/spans and actual
modifications. [field-review.json](field-review.json) preserves the separate
reviewer's exact accepted text, selected-use evidence and explicit everyday-sense
decisions. Construction, register
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
review and the produced SQLite. `coverage.json` lists all incomplete groups,
missing fields and unresolved everyday-sense requirements. Field completeness
counts all reviewed fields, including the four complete fields of `ça/NOM`;
group completeness also requires a passed everyday-sense decision.

Whole Wiktionary pages, unselected dictionary bodies, literary/comic quotations,
images and audio are outside this resource. OQLF and the Canadian Translation
Bureau corroborate grammar facts; their prose is excluded.
