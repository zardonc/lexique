# Source and rights assessment for #91

Each source record fixes an actual Wiktionary revision, timestamp, page ID, title,
URL, contributor-history URL and CC BY-SA 4.0 legal-text URL. Fresh initial API
captures were refetched by revision ID, following the earlier `audit84_fetch.py`
pattern. Both 44-page responses matched the original main-slot content. The
selected batch uses 87 distinct page/revision records; full captures include one
additional French `celui` page available for corroboration.

French headword IPA is selected from explicitly written pronunciation templates,
not generated from spelling or extracted from audio. Where variants exist, the
first recorded headword variant is selected. The word-joining tie in `d’autres`
is removed in `/dotʁ/`; the captured original notation is retained. Pronunciation
is for the frozen headword, including infinitive `marier` in the selected
`se marier` construction and `près` within `près de`.

Most semantic prose adapts the selected **English-language French-entry**
definition. French and Chinese are scoped translations/explanations; English is a
scoped adaptation. The single French semantic base is the non-quotation
`d’autres` indefinite-pronoun definition at revision 38287018, line 17. That page
has no dictionary-import notice. Page-history attribution is retained for actual
entry contributors; the last editor is not presented as the selected prose author.
Every accepted adaptation is identified under CC BY-SA 4.0.

Selected contexts retain language/POS headings, page-level import and bibliography
notices, and supporting construction spans. French pages with DAF8 or other
import references supply only recorded headword IPA, with no imported definition
prose selected. English Further reading references to TLFi, Larousse or Robert
are retained as bibliography; they confer no rights in those external dictionaries.
No quotation, translated quotation, example, audio, image or complete source page
is selected for redistribution. The full-page captures remain ignored evidence.

All 50 selected uses / 200 fields passed independent quality and selected-text
rights review. [Independent review](independent-review.md),
[admission](admission.json) and the canonical [field review](field-review.json)
record the accepted wording and boundaries. Source history attribution and the
absence of a selected quotation do not themselves prove original authorship of
every gloss. The combined resource's
[COPYING](../../../resources/core-pilot-79/COPYING) and
[NOTICES.md](../../../resources/core-pilot-79/NOTICES.md) carry the applicable
source/adaptation obligations. Full original captures are preserved under
integration's `target/implement-90-99/worker-audit/author-91/`.

| Rows | Selected-use evidence and restrictions |
| --- | --- |
| 502 `tiens/ONO` | English revision 92065114 line 18 explicitly labels a surprise interjection; French revision 39918113 lines 5–7 confirms the category. Lexique ONO is preserved; no `tenir` verb form is substituted. |
| 503–504 `noir` | Separate adjective colour and noun darkness senses, English revision 94564155 lines 68 and 82; noun IPA from French noun heading at line 678. |
| 509 `marier/VER` | English revision 92726932 line 22 explicitly selects `se marier`; every prose field preserves that construction. |
| 513–514 `possible` | Separate adjective possibility and singular substantivised feasible/achievable noun, English revision 95667464 lines 287 and 305. |
| 520–521 `près` | English revision 91504338 line 25 supports near/close; line 40 expressly calls `près` a preposition before a noun, requires `de` in everyday usage and says `près de` means near something. `près/PRE` selects this construction; the bare formal/legal institutional use at line 32 is excluded. The frozen headword remains `près`, and the adverb without a noun complement remains distinct. |
| 523–524 `d'autres` | French ASCII-spelling revision 2264762 line 1 explicitly redirects `d'autres` to `d’autres`. English `d'autres` revision 92726464 line 10 and `un autre` revision 66208565 lines 7 and 9 support the plural determiner. French `d’autres` revision 38287018 line 17 supports the standalone pronoun of unspecified other people. |
| 529–530 `dur` | Physical hardness adjective and tough-person noun are separate, English revision 95535023 lines 259 and 301. The accepted noun use is explicitly informal, as in French line 292; no solidity or specialist music sense is selected. |
| 532 `décider` | English revision 93428501 lines 17 and 24 bind the decision use and `de` before an infinitive. |
| 534–535 `certain` | Certainty adjective at English revision 95670073 line 282, indefinite determiner at line 305; line 303 records gender/number forms. |
| 538 `chef` | English revision 93684282 line 430 selects boss/leader; French revision 40036704 line 97 records the person noun and feminine alternatives. |
| 543 `ceux` | English revision 85384072 line 10 is masculine plural of `celui`; `celui` revision 93064438 line 23 supplies the relative/prepositional complement requirement. French revision 39772628 lines 5 and 16–17 records demonstrative form/IPA. |
| 547–550 `malade`, `parent` | Separate adjective and person-noun uses. `parent/NOM` selects family relative, not the father/mother use; adjective selects family relatedness. |

All 50 groups have accepted four-field content, with no annotation exception.
Native lookup/save/reopen, saved-state preservation and offline verification
results and remaining gates are recorded in the parent
[aggregate verification](../issue-90-99/README.md).
Learner acceptance and wider release gates remain open.
