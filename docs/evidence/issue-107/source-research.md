# Issue #107 selected source boundaries

The 50 exact groups have one selected same-use candidate each, with FR/IPA/EN/ZH
fields and complete restrictions in [groups](groups.json). [Source context](source-context.json)
and [excerpts](source-excerpts.json) retain the actual French section, lexical POS,
headword, grammatical frame and selected definition. All four fields are drafts
for independent review.

## Fixed originals

The [47 source records](sources.json) use the revisions actually returned by the
first-party API. [Capture records](source-captures.json) point to the complete
original response and raw article text at
`C:/Users/inthe/IdeaProjects/lexique-batch-107/target/issue-107/sources/`.
Raw line numbering preserves blank lines; IPA uses the exact template substring
and column. Quoted example bodies, images and audio remain in the ignored raw
captures and are not field content or bundled notices.

The existing #99 archive supplied the capture/adaptation pattern and established
DAF8 original-edition rights record. The #104 author helpers supplied the current
API and candidate preparation pattern. Their paths are retained in the ignored
author evidence. The scoped inputs and original-raw map in
`C:/Users/inthe/IdeaProjects/lexique-d2-resource-research/target/d2-research/dictionary-modern-batch50-next19-20261119/`
have no overlapping #107 headwords. They supply no selected field bindings here.

## Selected use and grammar

| Frozen group | Fixed selected boundary and restriction |
| --- | --- |
| 1302–1303 `nord/ADJ`, `nord/NOM` | Adjective definition 250/frame 249 versus cardinal noun definition 12/frame 11; invariable adjective and masculine noun remain distinct. |
| 1304–1305 `nu/ADJ`, `nu/NOM` | Naked adjective definition 11/frame 10 versus nudity noun definition 196/frame 186, revision 39823603. The additional same-article adjective definition supports the explanatory naked-state wording. Greek letter and artwork senses are excluded. |
| 1306 `censé/ADJ` | Definition 10/frame 9, revision 39943517. The retained Le Robert capture explicitly supplies adjective plus infinitive at extracted lines 48–49. New explanations retain `censé + infinitive`, distinct from `sensé`. |
| 1307 `officier/NOM` | Military officer definition 16/frame 9; no specific rank or verb substitution. |
| 1308 `ennuyer/VER` | Transitive causing boredom at definition 9/frame 8; no `s’ennuyer` substitution. |
| 1311 `prêter/VER` | Temporary use conditional on return, definition 32/frame 8; absolute use commonly concerns money. One recorded /pʁe.te/ variant is selected. |
| 1314 `opération/NOM` | Everyday surgery sense, definition 29/frame 10; calculations and military operations are excluded. |
| 1316 `atteindre/VER` | Transitive reaching at definition 15/frame 6; formal `atteindre à` is excluded. |
| 1317 `rouler/VER` | Intransitive movement on wheels, definition 112/frame 7; transitive rolling and deception are excluded. |
| 1319 `débarrasser/VER` | Transitive clearing a table or desk, definition 13/frame 6; pronominal `se débarrasser de` is excluded. |
| 1320 `arrivé/ADJ` | Successful-person adjective definition 17/frame 9, revision 39705165; neither the arrival participle section nor the person noun supplies this group. |
| 1322–1323 `parti/ADJ`, `parti/NOM` | Informal intoxication adjective definition 11/frame 10 versus political-party noun definition 93/frame 79, revision 39772401. Noun definition 80 separately supports the united-group explanation. Heraldry and the `partir` participle are excluded. |
| 1325 `excuse/NOM` | Justifying reason at definition 11/frame 10; no interjection or verb flexion substitution. |
| 1331 `plaindre/VER` | Transitive pity at definition 7/frame 6; complaining with `se plaindre` is excluded. |
| 1333 `contrôler/VER` | Checking compliance/regularity/accuracy, definition 21/frame 7; directing or dominating is excluded. |
| 1334 `planète/NOM` | Contemporary Solar System definition 122/frame 20, revision 39984949. The direct solar orbit, gravity-shaped form and cleared orbital neighbourhood conditions remain in all meanings; dwarf planets and natural satellites are excluded. |
| 1335 `taper/VER` | Typing at definition 34/frame 9; unrelated hitting and slang senses are excluded. |
| 1337 `lune/NOM` | Lowercase feminine singular common noun for Earth's moon, definition 14/frame 13. The astronomical proper-name capitalization remains a separate fact; no uppercase alias is added. |
| 1338 `démon/NOM` | Evil spirit in Christian belief, definition 60/frame 14; belief context retained, computing daemon excluded. |
| 1339 `e/NOM` | Wiki revision 39961718 places the everyday lowercase letter at French definition 51, `lettre` section 48 and frame 50. Translation heading 66 explicitly calls it the letter name. /ø/ is one frame-50 letter-name variant, not every sound encoded by the grapheme. |
| 1341 `mémoire/NOM` | Feminine memory faculty at definition 13/frame 12; masculine thesis/memoir and computing memory are excluded. |
| 1342 `magasin/NOM` | Retail shop definition 28/frame 9. Raw definition starts `#` without a space; original line and column remain unchanged. Warehouse, weapons magazine and historical publication are excluded. |
| 1343 `téléphoner/VER` | Definition 8 and lexical frame 7, revision 39770420; applicable IPA is separately at pronunciation line 67. Same-article unquoted example 12 supplies `téléphoner à quelqu’un`; no cited example prose is copied. |
| 1344 `dépendre/VER` | Indirect-transitive dependency definition 52/frame 44, distinct from the direct-transitive take-down section. `dépendre de` is retained. |
| 1345 `sacré/ADJ` | Religious adjective definition 11/frame 10; emphatic, noun and participle uses are excluded. |
| 1348 `félicitation/NOM` | Definition 15/frame 7, revision 39330898: predominantly plural congratulations. Definition 8 supplies the action/compliment meaning; exact frozen singular headword and its pronunciation are retained, with no plural runtime alias. |
| 1349 `parfaire/VER` | Transitive completing so nothing is missing, definition 7/frame 6; financial and legal senses are excluded. |

Every other group's selected use, exclusions, four fields and exact source spans
are in [groups](groups.json) and [field review](field-review.json). All spelling
relations are exact source bindings with `alias_admission: not_admitted`.

## The `e/NOM` bridge

The retained first-party Le Robert page at
<https://dictionnaire.lerobert.com/definition/e> explicitly labels the letter as
a masculine invariable noun. The fixed local HTML/text paths and capture timestamp
are in `source-context.grammar_corroboration`. Extracted line 43 supplies that
grammatical classification, line 45 supplies the noun label and line 46 identifies
the fifth letter. These are read-only grammatical facts; Le Robert's prose is not
reused in the dictionary fields. Wiki's definition and IPA remain the licensed
field sources. The relation records retain the exact lowercase spelling and the
French `lettre` to Lexique `NOM` bridge for independent review. Euler's constant
and the Guangxi language, in Wiki's separate noun sections, are not replacements.

## Original authors and rights

All sources credit Wiktionnaire contributors through the fixed article and full
history, with the complete current contributor registry separately retained.
Lexique contributors are credited for the scoped French adaptations and new
English/Chinese explanatory translations. CC BY-SA 4.0 attribution, licence,
modification and share-alike obligations remain explicit; dictionary content is
not relabelled MIT.

Thirty-eight articles have DAF8 bibliography or import markers. The existing
[#99 original-edition record](../issue-99/source-context.json) identifies Académie
française's eighth edition (1932–1935) as a collective work, with its original
public-domain basis under the cited French seventy-year publication rule.
That edition-level finding is retained separately from Wiki changes and Lexique
adaptations. It does not prove that a particular selected sentence came from DAF8.

Other actual references, including TLFi, Littré, LarousseXXe, Robert, Usito, DAF9
and historical works, remain metadata with their one-based locations. Their mere
presence neither establishes selected-text authorship nor licenses external
wording. In particular, `félicitation` lists LarousseXXe and Littré without a DAF8
import; its selected definition's original-text correspondence must be assessed.
`planète` has several modern and historical references; review must assess the
selected contemporary scientific criteria at their actual wording boundary.
No unselected quotation bodies are in the notices.

## Pending admission decisions

All 200 fields await independent same-use translation, pronunciation, modern-use
and selected-text rights decisions. Focused questions include the nudity noun,
successful-person `arrivé`, informal intoxication `parti`, exact `e/NOM` bridge,
separate `téléphoner` pronunciation, predominantly plural `félicitation` and
non-DAF8 original-wording correspondence. Author source checks are traceability
evidence and do not resolve those decisions. No new exception or alias is proposed.
Final conversion, notices and native evidence are integration-owned.
