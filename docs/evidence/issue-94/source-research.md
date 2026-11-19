# Selected sources and boundaries

The draft selects one sense per exact frozen word/POS group. Ninety real fixed Wiktionary revisions cover the 45 ticket headwords in French and English editions; 88 pages supply selected definition or pronunciation spans. Each used page has its actual title, page ID, revision ID, timestamp, permanent revision URL and contributor-history attribution. The [capture registry](source-captures.json) binds these records to full API responses and main-slot wikitext under the ignored evidence root. Retrieval uses the earlier `audit84_fetch.py`, `audit86-fetch.mjs` and `fetch-87-evidence.py` capture patterns with ticket-specific destinations.

French and Chinese definitions are new Lexique explanations of the cited selected lexical meaning. English explanations retain the same meaning and its restrictions. These are labelled adaptations in every field; they are not presented as upstream French or Chinese definitions. IPA is selected from the French headword's explicit `pron` template, with its recorded syllable dots and slash delimiters. Source language headings and POS headings are retained in [selected context](source-context.json). Source titles match the exact frozen NFC headwords; no alias, accent removal, capitalization substitution or inferred CEFR is introduced.

The proposed content licence is CC BY-SA 4.0. Article contributors are attributed by the permanent article and history links; new wording is attributed to Lexique contributors. The existing [licence](../../../resources/core-pilot-79/COPYING) and [source obligations](../../../resources/core-pilot-79/NOTICES.md) remain the parent assembly's starting materials. Field rights are pending independent selected-boundary review. A page's CC licence is not treated as permission to relicense imported prose.

Page import and bibliography notices are retained per source. The selected French meaning lines for `seigneur/NOM` (religious God), `pareil/NOM` (an identical thing), `quatre/ADJ:num` (anteposed cardinal four) and `simple/NOM` (racket-sport singles) require a separate check of the exact lexical/prose boundary and any original authors/rights. Notices naming the Académie française's eighth dictionary do not establish individual original-author rights. The current drafts use new explanations; neither the original imported passages nor these source-boundary conclusions are independently admitted. Other French-page uses supply pronunciation only. English Littré references on pages such as `sale` and `courir` are retained as bibliography; the selected meanings are the short lexical statements “dirty” and “to run”, not imported Littré prose.

The difficult groups remain separate:

| Group | Selected use and restriction |
| --- | --- |
| `impossible/ADJ`, `impossible/NOM` | Impossible property versus `l’impossible`, what cannot be done. |
| `froid/ADJ`, `froid/NOM` | Low temperature versus a figurative chill/strain between people. |
| `pareil/ADJ`, `pareil/NOM` | Similarity versus an identical thing/equal, including `sans pareil`. |
| `simple/ADJ`, `simple/NOM` | Uncomplicated versus racket-sport singles. The latter is a candidate everyday leisure use requiring separate acceptance. |
| `pire/ADJ`, `pire/NOM` | Comparative of `mauvais` versus nominal `le pire`. |
| `taire/VER` | Transitive `taire quelque chose`, keeping a matter quiet; no unrestricted intransitive claim. |
| `seigneur/NOM` | Religious `le Seigneur`, God; no historical landowner sense is counted as everyday. |
| `autant/ADV` | Equal quantity; retains `autant de` before a noun and `autant que` for comparison. |
| `ressembler/VER` | Intransitive `ressembler à`; no direct-object construction. |
| `flic/NOM`, `cul/NOM` | Slang/potentially derogatory police term and vulgar buttocks term, respectively. |

Full pages, literary quotations, etymologies, unselected senses, translation tables and media are excluded from the tracked shards. The author checks establish draft wording, exact capture bindings and frozen membership; they are not independent quality evidence, learner acceptance or resource admission. No group is omitted or replaced. Independent everyday-use and original-rights decisions remain open.
