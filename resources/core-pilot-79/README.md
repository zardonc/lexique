# Lexique FR–EN–ZH D2 core

This development resource contains **154 independently reviewed senses / 616
fields** in 154 frozen headword/POS groups, content version `core-83-v1`.
[Batch 003 #83](https://github.com/zardonc/lexique/issues/83) adds all fifty
groups in frozen rows 0101–0150 and their **200 reviewed fields**, with no reused
groups. Its approved scope contains **49 everyday groups and one specialist
exception**: row 128, `par/NOM`, selects the conventional stroke count required
to complete one golf hole. The golf domain, masculine noun and single-hole
standard remain explicit; the achieved score is excluded. The owner's decision
and independent acceptance are recorded in [field-review.json](field-review.json).

[Batch 002 #82](https://github.com/zardonc/lexique/issues/82) retains all fifty
targets in frozen rows 0051–0100. **49 groups / 196 fields are accepted**:
47 new groups / 188 fields and two unchanged reused groups, `bien/ADV` and
`bien/NOM`. Frozen row 87, `quoi/ADJ`, has no evidenced modern everyday adjective
use and keeps four explicitly unresolved fields. The owner approved that row as
an annotation exception with zero admitted fields, completing #82 under its
amended scope. Its original curation/review records and machine coverage status
remain unchanged: the row is unaccepted and reports four missing fields.
Pronoun content does not replace that target.

[Batch 001 #81](https://github.com/zardonc/lexique/issues/81) retains all fifty
groups in frozen rows 0001–0050 and their **200 reviewed fields**, including five
reused pilot groups and 45 new groups. **All fifty groups meet the approved batch
scope: 49 everyday groups and one documented specialist exception**. Frozen row
34, `ça/NOM`, retains its sourced psychoanalytic *id* sense with masculine,
singular and uncountability restrictions. The project owner's #81 decision is
recorded in the usage review; this row remains outside literal everyday coverage.
All twelve original pilot groups remain in the resource.
Its permanent target is **4,087 groups / 3,773 headwords**; **154 groups are
accepted and 3,933 remain unaccepted**. Literal everyday coverage is **152 complete
groups and 3,935 incomplete groups**, including both approved specialist nouns.
Learner acceptance,
the complete D2 release and other platform/release gates remain open. It supplies
no CEFR labels.

[curated-inputs.json](curated-inputs.json) contains the selected French definition,
IPA, English and Chinese fields, exact POS/use, source revision/spans and actual
modifications. [field-review.json](field-review.json) preserves the separate
reviewer's exact accepted text, selected-use evidence, explicit everyday-sense
decisions and the project owner's specialist exceptions. Construction, register
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
have fixed redirect evidence. Frozen `t'` and `s'` use fixed redirects to `t’`
and `s’`, with separate conjunction and personal-pronoun records for `s’`.
The six query aliases use these eight specific spelling records.

Frozen `où/PRO:int` and `où/PRO:rel` retain Lexique's subclasses; the selected
source describes interrogative and relative adverbs. OQLF corroborates the
interrogative-adverb classification and permits both relative-pronoun and
relative-adverb terminology. The usage and modification notices record that
correspondence. `y/NOM` selects the ordinary masculine, invariable letter name.

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
accepts specialist decisions only for frozen row 34, `ça/NOM`, bound to #81,
and row 128, `par/NOM`, bound to #83, with the respective project-owner approvals.
Field completeness counts all reviewed fields.
Batch #82 reports all fifty targets separately from the 49 admitted groups;
row 87 remains in coverage with four missing fields and its unresolved reason.
Batch #83 reports fifty accepted groups, 200 reviewed fields, 49 everyday groups
and its single approved specialist group. Frozen `pourquoi/CON` and `comment/CON`
retain Lexique's subclasses for indirect subordination; the selected sources
classify the lexemes as interrogative adverbs that act as subordinators.
The adverbial `bon` selects literal `sentir bon`; `petit/ADV` selects `écrire petit`.

Whole Wiktionary pages, unselected dictionary bodies, literary/comic quotations,
images and audio are outside this resource. OQLF and the Canadian Translation
Bureau corroborate grammar facts; their prose is excluded.
