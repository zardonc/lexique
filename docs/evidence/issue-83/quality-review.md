# Issue #83 independent quality review

Reviewed: 2026-11-18. Scope: [issue #83](https://github.com/zardonc/lexique/issues/83),
the exact frozen rows 101–150. Checkout:
`C:\Users\inthe\IdeaProjects\lexique-83`, branch `codex/d2-content-003-83`,
source base `13b7fec87ec2c5a99b10be2cd03dc20ed0b5c978`.
The reviewer is independent of the field author, source explorer and application
implementer. This review covers the uncommitted field/source inputs and their
accepted projection.

**All 50 exact groups and 200 fields are independently accepted.** Each group
has one reviewed sense with a French definition, applicable headword IPA,
English explanation and Chinese explanation bound to the same selected use.
There are **49 modern everyday uses and one specifically approved specialist
exception**, `par/NOM` at row 128. The exception remains outside literal everyday
coverage. The earlier `ça/NOM` and zero-field `quoi/ADJ` decisions remain separate.

The acceptance payload is `target/issue-83/accepted-field-review.json`; its
`fields` and `usage` arrays use the existing accepted-review format. The durable
projection belongs in [field-review.json](../../../resources/core-pilot-79/field-review.json),
with source records in [curated-inputs.json](../../../resources/core-pilot-79/curated-inputs.json).
Every field records its accepted text, source ID, fixed revision, line, reviewed
excerpt, contributor-history URL and attribution. Every usage records its
accepted construction and exclusions. No additional spelling alias is admitted.

## Content and grammatical binding

All French, IPA, English and Chinese fields were read independently against the
fixed raw source lines and their grammatical sections. English and Chinese are
accurately scoped explanations of the selected French meanings. Gender, number,
speaker/addressee roles, negative constructions, direct objects and infinitive
constructions remain readable. Homographic POS groups retain distinct meanings.

| Frozen group | Accepted use and source boundary |
| --- | --- |
| sur/PRE | Physical position above and supported by an object or surface; fixed definition line 19 and contributor example line 20. The narrower support condition is explicit in all meaning fields. |
| toi/PRO:per; lui/PRO:per | Tonic singular addressee `toi` is separate from clitic `te`. Clitic `lui` covers either gender, without a preposition before the verb or after an affirmative imperative; fixed `lui` revision 39770630, definition 18 and note 25. Tonic `lui/elle` is excluded. |
| ma, cette, votre, son and ton | Exact frozen forms and determiner subclasses are retained. `ma` selects a feminine singular noun before a consonant; `son` and `ton` select masculine singular nouns before a consonant. `votre` selects a singular noun with plural addressees or one politely addressed person. `cette` remains feminine singular. Possessor and possessed-noun gender are distinguished. |
| rien/PRO:ind; jamais/ADV | Negative uses with `ne`; positive/conditional `anything` or `ever`, omission of `ne` and standalone answers are outside the selected constructions. |
| bon/ADJ, ADV, NOM and ONO | Expected qualities, literal pleasant smell in `sentir bon`, voucher and acknowledging interjection remain separate. ADV meaning comes from [sentir bon, revision 39754433](https://fr.wiktionary.org/w/index.php?title=sentir%20bon&oldid=39754433), definition 7; `bon` IPA comes from its own fixed article. Figurative smell/prospect uses are excluded. |
| là/ADV and ONO | Separate other-place adverb and comforting interjection. The interjection selects the soothing part of definition 294, backed by uncited contributor examples 295–296; threats and reprimands are excluded. `/la/` is an explicitly recorded pronunciation variant. |
| pourquoi/CON; comment/CON | Indirect questions about reason and manner. Modern lexical class remains interrogative adverb; frozen CON denotes its subordinate-clause function. [OQLF GDT 26560285](https://vitrinelinguistique.oqlf.gouv.qc.ca/fiche-gdt/fiche/26560285/adverbe-interrogatif) explicitly lists `comment` and `pourquoi`, distinguishes direct and indirect questions, and identifies the indirect use as a subordonnant. No lexical-conjunction claim is attributed to Wiktionnaire. |
| même/ADJ, ADV and PRO:ind | Identity before a noun, inclusive/emphatic `even`, and the article-bearing pronoun remain separate. Pronoun meaning uses [le même, revision 35210050](https://fr.wiktionary.org/w/index.php?title=le%20m%C3%AAme&oldid=35210050), definition 17, pronoun heading 5 and form table 6–15. The frozen headword remains `même`; `le même`, `la même` and `les mêmes` are construction evidence. |
| petit/ADV | Only physical small writing in `écrire petit`. Licensed size definition 11 and IPA 10 are adapted to that construction. Read-only [TLFi PETIT III.A](https://www.cnrtl.fr/definition/petit) attests `écrit petit` in an adverb section. Dated `un petit` and figurative `voir petit` are excluded. |
| par/NOM | Golf's conventional number of strokes for one hole, a masculine noun; fixed `par` revision 39871373, noun heading 291, IPA 294 and definition 295. Actual achieved scores, other specialist senses and the preposition are excluded. |

The project owner explicitly approved only the row-128 golf exception in the
active #83 implementation chat on **2026-11-18 UTC**. The usage review carries
`approved_by: project_owner`, that date, the #83 URL and the single-hole scope.
Other rows receive everyday acceptance only for their independently reviewed
selected use.

## Selected-source rights and context

The 200 accepted fields are eligible at their **selected-text boundaries** under
CC BY-SA 4.0. French contributor spans and Lexique's new scoped adaptations use
that route; prior XMLittré-derived definitions and translations are excluded.
XMLittré material remains separately CC BY-SA 3.0 and is not relabeled as 4.0.
The combined admitted-text route retains share alike, contributor and adapter
credit, fixed-version/history links, applicable import notices and actual
modification records, together with the [complete licence](../../../resources/core-pilot-79/COPYING)
and [resource notices](../../../resources/core-pilot-79/NOTICES.md). Project MIT
code does not relicense the content.

Primary rights sources were retrieved independently at fixed revisions:
[French copyright convention 38903168](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168),
[Wikimedia Terms 598588](https://foundation.wikimedia.org/w/index.php?title=Policy:Terms_of_Use&oldid=598588),
[Import:DAF8 22113837](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AImport%3ADAF8&oldid=22113837)
and [R:DAF8 27621166](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AR%3ADAF8&oldid=27621166).
Terms §7(g) requires applicable underlying licences and imported-source credit;
§7(h) requires adaptation and licence notices. CC BY-SA 4.0 §3(a) and §3(b)
require retained attribution, modification information and share alike.

The original declared import credit is **Académie française, Dictionnaire de
l'Académie française, huitième édition (1932–1935)**. This batch retains declared
DAF8 notices for `alors`, `aussi`, `chose`, `jamais`, `même`, `penser`, `pourquoi`,
`quand` and `très`. `sur` retains its existing article-level import notice and
the separately evidenced earlier contributor preposition boundary. Reference
markers for TLFi, Littré, Larousse, Usito, DAF9 and CFC remain source context;
they do not admit unselected definitions.

The fixed `même` article has separate boundaries: selected ADJ heading 7,
IPA 9 and definition 10, and ADV heading 148, IPA 151 and definition 152 have
no inline Alexandria/Reverso attribution. Alexandria at 211 belongs to the
unselected NOM section beginning at 209. Reverso at 240 introduces the unselected
pronoun definitions 241, 244 and 247; that body and note are excluded. Pronoun
IPA 239 is uncited, while the accepted meaning and article forms use the clean
`le même` page.

Only 18 independently checked contributor support spans are eligible for
bundling. Five cited literary/journalistic locations are research-only metadata.
Their source-container licence does not establish rights to the quotations.
TLFi, Robert and OQLF prose, the Reverso/Alexandria bodies, the CFC cartographic
definition, literary quotations, images, audio and whole source pages are outside
the admitted text. Grammar facts from read-only sources support the new scoped
explanations without copying their prose. This is not a whole-page or corpus
licensing conclusion.

## Verification and limits

`uv run --python 3.14.10 target/issue-83/quality-review/review_fields.py` passed
on Windows. The check compared the independently recorded accepted texts with
the normalized draft, the issue's literal membership and all 4,087 manifest
positions. It verified 200 field/source bindings against 37 raw fixed snapshots,
13 retained API captures by page/revision/content, 50 actual source notices and
all 23 supporting spans, with 18 eligible spans and five excluded quotation
locations. The payload contains exactly 200 unique field identities and 50
usage reviews, with 49 everyday statuses and one scoped owner exception.

`uv run --python 3.14.10 target/issue-83/quality-review/inspect_assembly.py`
also passed against the actual assembled data. This data-only inspection verified
all 200 field projections, 50 usages, retained excerpts, source contexts and
notice exclusions, including consistency with the resource README and NOTICES.
Earlier groups, fields, usages, spelling relations and unresolved records remain
unchanged. The assembled context retains only `{{Import:CFC}}` at `ton` line 131
and only the Reverso reference at `même` line 240, excluding their prose. The
reused `fr_sur` context includes all four primary `sur/PRE` locations while
preserving the earlier adjective locations. The assembled totals are 154
accepted groups and 616 fields, including two specialist exceptions; literal
everyday `complete_groups` is 152 of the unchanged 4,087-group denominator.
The report whitespace check and its four local file links also passed.

Retained review evidence is rooted at
`C:\Users\inthe\IdeaProjects\lexique-83\target\issue-83\quality-review`.
The accepted-text decisions, both checkers, result JSONs, execution logs, fixed primary
rights captures and reviewed-input patch stay ignored there. The reviewed-input
patch uses `git diff --no-index -- NUL target/issue-83/content-draft.json` and is
retained as `content-draft-review.patch` against the source base above.

Converter/native/package execution, saved-learner-data checks, human learner
acceptance and release readiness are outside this content review and were not
run by this reviewer. Parent-owned execution must separately verify the packaged
materials and exclusions. Complete three-language coverage and CEFR admission
remain separate project gates.
