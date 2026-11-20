# Issue #100 independent selected-content review

**50 groups / 200 exact FR/IPA/EN/ZH fields independently accepted; zero unresolved rows.** All frozen rows 951–1000, exact spellings/POS subclasses and both frequency strings remain present. The denominator is 4,087 groups. No runtime alias or annotation exception is admitted.

This reviewer is separate from the author and merger. Reviewed author head: `c37db48246e3710c11891f5d9738f3122c234e17`; diff base: `87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5`. Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-100`. Windows evidence: `target/issue-100/`. Ticket [#100](https://github.com/zardonc/lexique/issues/100), parent [#78](https://github.com/zardonc/lexique/issues/78).

## Selected content and source quality

I read all four actual fields for each group against the fixed first-party wikitext, French language/POS/headword frames and construction/register restrictions. [Field review](field-review.json) records exact accepted text and individual linguistic and rights reasons for 200 fields and 50 usages. [Review candidates](review-candidates.json) preserves the original pending author package unchanged. EN/ZH are accurate Lexique explanatory translations of the same selected French uses.

`fier/ADJ` row 959 uses the ordinary pride descriptor at revision 39715331, line 39, inside the adjective section at lines 8–10; `/fjɛʁ/` applies. It does not borrow the verb's `/fje/`. Row 960 separately requires `se fier à`. `chacun/PRO:ind` row 958 retains the singular indefinite-pronoun frame and no-liaison restriction.

`dommage/ONO` row 971 is the attested standalone interjection at revision 39912382, lines 132–134. Its applicable `/dɔ.maʒ/` comes from the exact same French spelling at line 9; the noun row 970 retains damage/harm independently. `merveilleux/NOM` row 978 selects the general experiential/imaginative sense at revision 39971666, line 94, with masculine singular `le merveilleux` at line 83. It excludes the technical literary intervention, historical dandy and regional pastry senses. Lowercase `allemand/NOM` row 980 is the language noun at revision 40037457, lines 181–185; no capitalized person noun is substituted.

The selected `fenêtre`, `bouche`, `ligne` and `mur` meanings describe ordinary buildings, the human mouth, written text and physical walls. Their domain labels do not make these specialist exceptions. Pronominal verbs, adjective/noun distinctions, singular-only language nouns and selected pronunciation variants remain explicit.

## Rights

[Rights](rights.json), [sources](sources.json) and [source context](source-context.json) record selected-span eligibility and combined obligations. I read fixed Wikimedia Terms revision 598588, section 7(a–c, g–h), French copyright convention revision 38903168 and the CC BY-SA 4.0 legal text. The #102 retained primary L123-3 capture confirms the collective-work publication term. DAF8's final 1935 publication gives conservative expiry 2006-01-01; historical Littré 1872–1877 is separately public-domain.

Historical original-edition captures are comparison evidence, not proof that every current gloss was written by that edition. The recorded `riche` and `chacun` wording corresponds to historical DAF8; modern `bruit` wording differs. Actual import/reference metadata, Wiki contributor transcription/adaptation and Lexique translation credit remain separate. Fixed article and full-history URLs provide attribution; current registries and last editors do not establish sentence authorship.

`ligne`'s CFC cartographic definition at line 156 is unselected; row 986 uses the page-character meaning at line 61. `boîte`'s TLFi/Usito references follow the alternate pronunciation in line 11; the selected factual `/bwat/` transcription and container definition copy no proprietary dictionary prose. Literary/news quotations, media and upstream translation tables supply no fields. Preserve article/history credit, applicable original notices, copyright/licence/disclaimer notices, modification credit, CC BY-SA 4.0 and its legal text/link, without additional restrictions or obstructive technical measures.

## Verification and limitations

Three focused checks pass, including exact API/wikitext equality for 42 articles, all 200 field locations and meaningful-content preservation against the untouched author candidates. The focused test compares content and provenance separately from review-status labels. Full logs: `reviewer-baseline.log`, `reviewer-contract-red.log`, `reviewer-final.log`. Source/context checks: `reviewer-source-check.json`; selected reading/import/original comparison evidence: `reviewer-reading.log`, `reviewer-import-boundaries.json`, `reviewer-original-correspondence.json`. Diff and local-link results: `reviewer-diff-check.log`, `reviewer-links.json`.

No remote refetch of article revisions or fresh statutory request is claimed. Actual assembled notices, canonical conversion, native lookup/save/reopen and release gates remain parent/integration work. No full application suite was run; this content handoff does not close #100's implementation gates.
