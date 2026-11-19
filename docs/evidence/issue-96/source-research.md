# Sources and selected uses

The [ticket](https://github.com/zardonc/lexique/issues/96) retains the exact
50 word/POS pairs at frozen rows 751–800 under
[spec #78](https://github.com/zardonc/lexique/issues/78). Its membership and
frequency strings are copied unchanged from the
[frozen manifest](../../../resources/core-pilot-79/scope-manifest.csv).
Credit Boris New and Christophe Pallier, Lexique 3.83, under CC BY-SA 4.0;
the existing [membership notice](../../../resources/core-pilot-79/MEMBERSHIP-NOTICES.txt)
remains applicable.

All **50 selected uses / 200 fields** passed independent source, rights and
quality review. [Independent review](independent-review.md),
[admission](admission.json) and the canonical [field review](field-review.json)
record the accepted content and exact boundaries.

## Fixed source capture

The [registry](source-captures.json) retains 94 used fixed Wiktionnaire and
English Wiktionary sources, with actual page IDs, revision IDs, timestamps,
fixed URLs, history links and local full-capture paths. Ten French pages came
from the first-party API selecting a revision at or before
`2026-10-01T00:00:00Z`. The other 39 French pages and 49 English pages were
retrieved through the same-origin browser API and fetched again by explicit
revision IDs. These are actual fixed revisions, without a claim that every
source has the October cutoff. All 98 captures, including four unused pages,
remain ignored under the owning evidence root, preserved under integration's
`target/implement-90-99/worker-audit/author-96/`.

The [excerpts](source-excerpts.json) give one-based line and column locations
and exact selected text. [Contexts](source-context.json) retain the French
language and POS sections, necessary construction/form evidence and applicable
French-section bibliography/import notices. No whole page, image or audio is
staged. A last editor is capture metadata, not a selected-sentence author.
Linked article/history attribution identifies the Wiktionary contributors;
the independent selected-text review is recorded separately from capture metadata.

## Text and rights boundaries

FR, EN and ZH meanings are new Lexique explanations adapted from the selected
French lexical meaning on English Wiktionary. They are labelled adaptations,
not quoted upstream bilingual translations. IPA transcribes one actual
matching French headword template variant; slash delimiters are added.
The [source records](sources.json) retain contributor attribution, fixed
versions, the accepted CC BY-SA 4.0 route and per-field modification statements.

French article references include imported Académie eighth-edition material
and other dictionaries. Only factual IPA/headword and grammatical form spans
are selected there; their imported definitions, quotations and etymologies are
not sources for the three drafted meaning fields. English further-reading
references do not authorize copying the referenced dictionary prose. Every
selected span passed a separate rights and quality audit at its actual boundary.
Bibliography labels alone are not proof of original authorship.

The selected contributor/adaptation material retains CC BY-SA 4.0; preserve
fixed article/history attribution, modification information and the existing
[complete licence](../../../resources/core-pilot-79/COPYING). The combined
[notices](../../../resources/core-pilot-79/NOTICES.md) retain the applicable obligations.
Project code retains its separate MIT licence.

## Exact grammatical relationships

`or/CON` introduces a fact or premise in reasoning; `or/NOM` is gold as a
material. `lequel/PRO:int` asks which one; `lequel/PRO:rel` is selected after
a preposition. Their distinct fixed English definitions and French POS frames
are retained. No broad pronoun match substitutes for either subclass.

Frozen row 770 remains the exact straight-apostrophe **`l'un/PRO:ind`**.
Its fixed French page explicitly redirects to typographic-apostrophe `l’un`.
That target supplies the masculine singular paradigm and `/l‿œ̃/`; the fixed
English `l'un` page supplies the partitive construction, including
`l'un des meilleurs vins`. The accepted spelling record binds only those two
exact forms for this selected use. It does not bind bare `un`, `l'une`,
`les uns` or reciprocal `l'un l'autre`, and adds no runtime alias.

`toute/ADJ:ind` retains its feminine singular determiner use. The fixed
French `toute` flexion section and English `toute` determiner section link it
to `tout`; the English `tout` determiner paradigm and meaning support the
selected whole/all use. Adverbial `toute contente` and pronoun uses are excluded.
`million/NOM` retains the exact frozen numerical noun, supported by the
masculine French noun frame and English cardinal-noun template.

`amoureux/NOM` selects a boy or man who romantically loves someone, chiefly in
children's speech. `gueule/NOM` selects a person's mouth with informal/rude
restrictions. `bête/ADJ` retains the offensive qualification; `bête/NOM` is the
separate animal use. All 50 groups have accepted four-field content, with no
annotation exception. Native lookup/save/reopen, offline and saved-state
preservation checks are pending at integration; see the parent
[aggregate verification](../issue-90-99/README.md). Learner acceptance and wider
release gates remain open.
