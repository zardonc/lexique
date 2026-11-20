# Issue #101 independent selected-content review

**49 groups / 196 exact FR/IPA/EN/ZH fields accepted; row 1005 remains unresolved with four missing fields.** All 50 frozen rows 1001–1050 are retained in [groups](groups.json), with exact headword/POS/frequency values. The denominator remains 4,087 groups. No spelling substitution, runtime alias or annotation exception is admitted.

The content/rights reviewers are separate from the author and merger. Reviewed author head: `da07f57e1c6236fa535512be3af2a700077bc359`; diff base/integrated checkout head: `d9f8e985a83512c1292255fc6d97f6605628fc6c`. Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-101`. Windows evidence remains under `target/issue-101/`. Ticket [#101](https://github.com/zardonc/lexique/issues/101), parent [#78](https://github.com/zardonc/lexique/issues/78).

## Accepted content and exact wording

[Field review](field-review.json) preserves the original independent reviewer's 196 exact field decisions, 49 everyday-use judgments, reviewer identities and source/rights reasons. The completing reviewer independently read every actual field and its fixed first-party French spelling/POS/IPA and same-use frame and confirms those decisions. [Review candidates](review-candidates.json) preserves the original 50-group author package unchanged.

The reviewed `adieu/ONO` Chinese explanation says long or permanent separation, without equating every use with permanent farewell. The `personnel/NOM` English explanation says people working for a service/company/household, without adding a paid-employment condition. The exclusion for `coûter` correctly identifies `couter` as rectified spelling. These are the three explicitly recorded replacements; all other selected wording and provenance are unchanged. No query alias follows from the spelling description.

`déjeuner/NOM` and `déjeuner/VER` select the France midday meal, with regional breakfast excluded. The fixed noun gloss at revision 39927173, line 15, contains both midday and morning phrases; only its midday meaning is selected. Lines 22 and 219 record the breakfast boundary. `est/NOM` retains `/ɛst/`, distinct from `être`'s `/ɛ/`. `mille/ADJ:num` and `mille/NOM` retain distinct numeral-adjective and number-noun frames. `moi-même/PRO:per` is the emphatic first-person subject use. `intéressant/NOM` preserves the generally pejorative person/`faire l’intéressant` use rather than neutral abstract content. Language nouns keep lowercase spellings. `peau`, `doigt` and bodily `vaisseau` are ordinary body vocabulary; no specialist exception is added.

## Confirmed unresolved exact row

**1005 `m/PRO:per`: FR, IPA, EN and ZH missing.** Fixed bare [m revision 39772551](https://fr.wiktionary.org/w/index.php?title=m&oldid=39772551), French section from line 162, contains the letter at lines 166–169, euphemistic `m...` noun at 196–198 and musical adjective at 207–212. It has no bare personal-pronoun entry. The letter-name `/ɛm/` belongs to row 1004 `m/NOM`.

[m’ revision 39840795](https://fr.wiktionary.org/w/index.php?title=m%E2%80%99&oldid=39840795), lines 7–14, is an explicitly apostrophized personal-pronoun elision; straight `m'` revision 20098185 redirects to it. [me revision 39891222](https://fr.wiktionary.org/w/index.php?title=me&oldid=39891222), lines 7–13, establishes the unelided pronoun. Retained Robert corroboration likewise gives `me → m'` before a vowel/mute h. None establishes apostrophe-free `m` as `PRO:per`. New exact-spelling primary evidence or an owner scope decision is required. The original four pending fields and all frozen membership values remain recorded.

## Selected-span rights

[Rights](rights.json) and the per-source/field records retain article/full-history credit, applicable original notices, copyright/licence/disclaimer notices, CC BY-SA 4.0 legal text/link, modification credit and the same-licence redistribution/no-additional-restrictions conditions. Actual French-section DAF8 import/reference markers are retained separately from Wiki adaptations and Lexique FR/EN/ZH explanations. Bibliography and container licence do not assign authorship of every modern gloss.

The reviewed Wikimedia Terms revision 598588, section 7(a–c, g–h), fixed French copyright convention 38903168 and CC legal text support this route. The established DAF8 1932–1935 collective-edition basis is retained; #102's primary L123-3 capture confirms the term and conservative 2006-01-01 expiry. Historical Littré references are public-domain comparison material; TLFi/Usito/Robert bibliography conveys no right to their protected prose. Robert m/s, me, déjeuner and intéressant captures supply corroboration only. No literary/news quotation, proprietary dictionary definition, media or translation table supplies a field. Full linked article histories provide attribution; no successful current contributor-registry capture or fresh statutory request is claimed.

## Verification and limitations

Both focused checks pass: exact frozen membership, all 196 source locations against 46 retained API/wikitext captures, accepted/candidate content comparison and only the three recorded replacements. The focused test compares actual text/provenance separately from review labels. Complete source/context join verification passed for 743 bindings. Logs: `reviewer-final.log`, `reviewer-source-check.json`, `reviewer-diff-check.log`, `reviewer-links.json`. Reading/rights evidence: `reviewer-resumed-reading.log`, `reviewer-resumed-import-boundaries.json`, `reviewer-resumed-rights.json`. The preserved incoming review patch is `reviewer-resumed-partial.patch`.

Actual assembled notices, canonical conversion, native lookup/save/reopen and release checks remain parent/integration work. No full application suite or native gate was run. Four unresolved fields keep #101 incomplete.
