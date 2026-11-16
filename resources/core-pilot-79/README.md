# Lexique FR–EN–ZH D2 pilot

This development resource contains twelve independently reviewed modern everyday
senses in twelve frozen headword/POS groups. Its permanent target is **4,087
groups / 3,773 headwords**; **4,075 groups remain incomplete**. Learner acceptance,
the complete D2 release and other platform/release gates remain open. It supplies
no CEFR labels.

[curated-inputs.json](curated-inputs.json) contains the selected French definition,
IPA, English and Chinese fields, exact POS/use, source revision/spans and actual
modifications. [field-review.json](field-review.json) preserves the separate
reviewer's exact accepted text. Six auxiliary wording corrections and the full
apostrophe redirect have been applied. [selected-source-excerpts.json](selected-source-excerpts.json)
contains only selected excerpts; [selected-source-context.json](selected-source-context.json)
retains applicable source/import notices, fixed article/history attribution and
separately labelled older authorship/merger evidence. Older revisions do not
replace the fixed current content versions.

[scope-manifest.csv](scope-manifest.csv) preserves the accepted four-column
membership and row order. Frozen `coeur` remains distinct from source `cœur`;
frozen `aujourd'hui` remains distinct from source `aujourd’hui`. Their two exact
query bindings have source evidence. This pilot adds no general accent, ligature,
apostrophe or noun-form folding.

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
review and the produced SQLite. `coverage.json` lists all incomplete groups and
their required fields.

Whole Wiktionary pages, unselected dictionary bodies, literary/comic quotations,
images and audio are outside this resource. OQLF corroborates grammar facts;
its prose is excluded.
