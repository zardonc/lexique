# Selected sources and boundaries

The accepted content selects one sense per exact frozen word/POS group: **50 groups / 200 fields**. The canonical records use **89 fixed Wiktionary sources** for the 45 ticket headwords in French and English editions. Each used page has its actual title, page ID, revision ID, timestamp, permanent revision URL and contributor-history attribution. [Independent review](independent-review.md), [admission](admission.json) and the canonical [field review](field-review.json) record the accepted source, rights and quality decisions. The [capture registry](source-captures.json) binds the records to full API responses and main-slot wikitext, preserved under integration's `target/implement-90-99/worker-audit/author-94/`. Retrieval uses the earlier `audit84_fetch.py`, `audit86-fetch.mjs` and `fetch-87-evidence.py` capture patterns with ticket-specific destinations.

French and Chinese definitions are new Lexique explanations of the cited selected lexical meaning. English explanations retain the same meaning and its restrictions. These are labelled adaptations in every field; they are not presented as upstream French or Chinese definitions. IPA is selected from the French headword's explicit `pron` template, with its recorded syllable dots and slash delimiters. Source language headings and POS headings are retained in [selected context](source-context.json). Source titles match the exact frozen NFC headwords; no alias, accent removal, capitalization substitution or inferred CEFR is introduced.

The accepted content licence is CC BY-SA 4.0. Article contributors are attributed by the permanent article and history links; new wording is attributed to Lexique contributors. The combined resource's [licence](../../../resources/core-pilot-79/COPYING) and [source obligations](../../../resources/core-pilot-79/NOTICES.md) retain the applicable obligations. Every selected field passed independent rights review at its actual text boundary. A page's CC licence is not treated as permission to relicense imported prose.

Page import and bibliography notices are retained per source. The accepted French meaning spans for `seigneur/NOM` (religious God), `pareil/NOM` (an identical thing), `quatre/ADJ:num` (anteposed cardinal four) and `simple/NOM` (racket-sport singles) have independently reviewed lexical/prose boundaries and original-rights evidence in the canonical context and source records. Notices naming the Académie française's eighth dictionary do not establish individual original-author rights. The accepted fields are scoped Lexique explanations of the reviewed spans. Other French-page uses supply pronunciation only. English Littré references on pages such as `sale` and `courir` are retained as bibliography; the selected meanings are the short lexical statements “dirty” and “to run”, not imported Littré prose.

The difficult groups remain separate:

| Group | Selected use and restriction |
| --- | --- |
| `impossible/ADJ`, `impossible/NOM` | Impossible property versus `l’impossible`, what cannot be done. |
| `froid/ADJ`, `froid/NOM` | Low temperature versus a figurative chill/strain between people. |
| `pareil/ADJ`, `pareil/NOM` | Similarity or identity with another person or thing versus a thing identical to another. |
| `simple/ADJ`, `simple/NOM` | Uncomplicated versus racket-sport singles, with one player on each side. Both selected uses are independently accepted. |
| `pire/ADJ`, `pire/NOM` | Comparative of `mauvais` versus nominal `le pire`. |
| `taire/VER` | Transitive `taire quelque chose`, keeping a matter quiet; no unrestricted intransitive claim. |
| `seigneur/NOM` | Religious `le Seigneur`, God; no historical landowner sense is counted as everyday. |
| `autant/ADV` | Equal quantity; retains `autant de` before a noun and `autant que` for comparison. |
| `ressembler/VER` | Intransitive `ressembler à`; no direct-object construction. |
| `flic/NOM`, `cul/NOM` | Slang/potentially derogatory police term and vulgar buttocks term, respectively. |

Full pages, literary quotations, etymologies, unselected senses, translation tables and media are excluded from the tracked shards. All 50 groups have independently accepted four-field content, with no annotation exception.
Native lookup/save/reopen, saved-state preservation and offline verification
results and remaining gates are recorded in the parent
[aggregate verification](../issue-90-99/README.md). Learner acceptance and wider release gates remain open.
