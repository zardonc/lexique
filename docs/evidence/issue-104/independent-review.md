# Issue #104 independent selected-content review

**50 groups / 200 exact FR/IPA/EN/ZH fields independently accepted; zero missing fields.** All frozen rows 1151–1200, exact spellings, grammatical subclasses, order and frequency strings remain unchanged. The denominator is **4,087 groups**. No runtime alias or annotation exception is approved. Native and package admission remain parent-owned.

This reviewer is separate from the content author and integration merger. Reviewed committed author head and review diff base: `147f9493b07c604c897fb553e85bb1a584b4ec30`. The author branch incorporated integration `87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5` before review. Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-104`. Retained execution/source evidence: `C:/Users/inthe/IdeaProjects/lexique-batch-104/target/issue-104/`, reviewed at that author head on Windows x64, PowerShell 7 and uv Python 3.14.10. Reviewer changes stay within this evidence directory and the focused batch check.

## Independent content decision

I read each proposed field, its fixed first-party wikitext, French language/POS/spelling frame, applicable IPA, constructions, exclusions and source notices. [Field review](field-review.json) records the exact unchanged accepted text and individual linguistic/rights reasons for every field and usage. [Source-quality judgments](source-quality.json) give a separate decision for all 50 frozen rows. [Review candidates](review-candidates.json) preserves the original 50-group draft and its pending statuses byte-for-byte.

`frais/NOM` is the plural-only masculine expenses noun, separate from the adjective. `commandant/ADJ` has its own adjective frame and agreement examples, distinct from both the commander noun and verbal participle. `coupable`, `dingue` and `voisin` retain independent adjective/noun identities; the familiar register of both `dingue` uses remains visible. These select ordinary meanings, not specialist exceptions.

`hors/PRE` preserves spatial exclusion, `hors de` and aspirated h; `parmi/PRE` retains plural/collective complements. `moquer/VER` preserves the exact lemma and explicitly requires `se moquer de` for its selected ridicule sense; it neither becomes a query alias nor silently transfers a transitive use. `remonter` is intransitive ascent again, `réaliser` is bringing a project into reality, and `retenir` is memorisation. `contraire/NOM` is the opposite concept, not its adjective.

## Selected-text rights

I captured and read the relevant passages from **42 DAF8 endpoints: 33 returned entries and nine returned no entry**. Captures and URL/status records are in `reviewer-originals/captures.json`; the independently captured DAF8 `frais` expenses noun is also retained there. Empty endpoints for `avenir`, `contraire`, `commandant`, `hors`, `coupable`, `parmi`, `moquer`, `ridicule` and `voisin` supply no exact-original correspondence. Their short selected Wiki lexical explanations and applicable frames are reviewed directly; no proprietary or quoted example wording is selected or attributed from those responses.

Available DAF8 passages match or corroborate the selected imported bases, including `oreille`, `art`, `ravir`, `beauté`, `laisse`, `course`, `manière`, `traverser`, `soin`, `cour`, `remonter`, `milieu`, `secours`, `proposer`, `inspecteur`, `projet`, `rôle`, `ange`, `avance` and `emporter`. Wiki changes such as factual generalisation, chronology in lists and knowledge/skills explanations retain their separate adaptation layer. Source references and current contributor/last-editor metadata do not establish authorship of every selected sentence.

`langue/NOM` selects the communication system at **line 41**, including spoken, signed and written expression. Its cartography sentence at **line 86** and `Import:CFC` marker, together with the nearby modern glaciology quotation, are unselected and supply no accepted field wording. The CFC template's own fixed licensing statement is retained separately for the boundary check. Modern and foreign-language references are identified as reference metadata, without assigning their terms to the selected French wording.

[Rights](rights.json) records article/history credit, applicable original-edition notices, supplied copyright/licence/disclaimer notices, CC BY-SA 4.0 text/link and same-licence adaptation distribution, modification notices, and no additional restrictions or obstructive technical measures. I independently read the fixed French copyright policy, CC legal sections 2–4, Wikimedia Terms 7 and the primary L123-3 capture retained in the completed [#102 review](../issue-102/independent-review.md). The original 1932–1935 collective DAF8 edition has conservative expiry on 1 January 2006; Wiki transcription/adaptations and Lexique selection, IPA transcription and FR/EN/ZH adaptations remain separately CC BY-SA 4.0. Legal captures are copied into this worktree's `reviewer-primary/` directory. Unselected quotations, audio and images stay outside the accepted content.

## Validation and limitations

- Focused author baseline: one test passed; `reviewer-author-baseline.log`.
- Author-stage metadata/equality mismatch reproduced after independent review; `reviewer-contract-red.log`. The focused check now validates unchanged candidate source bodies and exact accepted content separately, preserving original pending candidates.
- Final focused check: two tests passed; `reviewer-focused-final.log`. All 200 original API/body/span bindings, source revisions/timestamps/page identities, frozen memberships/frequencies and exact accepted field/usage text are checked.
- Complete seven-file/source-context joins, candidate-byte preservation, Markdown links and scoped whitespace checks: `reviewer-handoff-check.log`, `reviewer-links.json`, `reviewer-diff-check.log`. Full source readings and original responses remain under the owning target directory.

The handoff has 46 accepted source records. Final combined notices, canonical conversion, native lookup/save/reopen, integration and release gates remain parent-owned. This review does not close [#104](https://github.com/zardonc/lexique/issues/104). No full app suite or all-revision remote refetch was run.
