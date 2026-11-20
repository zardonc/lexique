# Issue #107 independent content and rights review

**50 groups / 200 FR/IPA/EN/ZH fields accepted; no missing fields.** All exact frozen rows 1301–1350, POS subclasses and both frequency strings remain. The denominator is **4,087 groups**. No alias or owner exception is approved.

Ticket [#107](https://github.com/zardonc/lexique/issues/107), parent [#78](https://github.com/zardonc/lexique/issues/78). Reviewed author head: `878fee8fa59c830126065c5654792da5d9a41092`; diff base: the same head for this reviewer-only change. Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-107`, branch `codex/d2-content-107`. This reviewer is independent of the author and merger. Changes cover the evidence directory and the focused test's comparison of accepted content with the preserved draft. Retained patch: `C:/Users/inthe/IdeaProjects/lexique-spec-100-109/target/implement-100-109/worker-audit/author-107-final/target/issue-107/reviewer-handoff.patch`, produced by `git diff 878fee8fa59c830126065c5654792da5d9a41092 -- docs/evidence/issue-107 scripts/test_core_batch_107.py`.

## Decisions

Every actual field, French-language/POS frame, selected wikitext span and same-use restriction was read independently. [Field review](field-review.json) records exact unchanged accepted text and individual linguistic judgments; [groups](groups.json) contains the accepted content. [Review candidates](review-candidates.json) preserves the original author draft, including its pending statuses. EN/ZH are accurate Lexique explanations of the selected French use, not quotations from upstream bilingual tables. IPA preserves one explicitly recorded variant.

The handoff uses the existing core schema's `separate_independent_field_review`, `passed_for_selected_spans` and `passed_after_exact_accepted_replacements` acceptance values. Selected text and provenance are unchanged.

`e/NOM` row 1339 passes as the lowercase letter-name noun. Fixed `e` revision 39961718, lines 42–51, supplies French, the exact spelling, masculine/invariable grammar and `/ø/`. Le Robert's captured exact entry explicitly confirms a masculine invariable noun; its grammatical fact provides corroboration only. No proprietary definition or symbol sense is selected, and no uppercase/accented runtime alias is added. `censé` keeps the infinitive construction. `félicitation` retains the exact singular headword while explicitly stating usual plural usage.

`arrivé/ADJ` selects achieved success, supported by its separate adjective sense and modern person-use attestations, including the 1994 Médiaspouvoirs quotation. `parti/ADJ` retains familiar intoxication separately from the political noun and departure participle. `téléphoner` binds the lexical headword at line 7 to the same French article's pronunciation at line 67 and keeps `à` for its recipient. The ordinary mountain, surgery and Solar System planet senses preserve their domain restrictions; no specialist exception is introduced.

All **47 fixed first-party revisions** were independently refetched. Full wikitext, title, page ID, timestamp and last editor match the original captures. [Capture records](source-captures.json) link `reviewer-primary/remote-verification.json`; this mechanical match supports provenance and does not establish linguistic quality or sentence authorship.

[Rights](rights.json) records current CC BY-SA 4.0 sections 2–4, Wikimedia Terms 7 and independently fetched French copyright policy/DAF8 template. The selected lexical prose is separated from literary/news quotations, etymology, other-language sections, images/audio and translation tables. No non-DAF8 imported external wording is selected. Dictionary bibliography alone does not prove authorship. Applicable DAF8 edition credit is retained conservatively using the established collective-work 1932–1935 basis and 2006 expiry; contributor adaptations and Lexique translations remain CC BY-SA 4.0.

Combined redistribution requires fixed article/history credit, applicable original-edition attribution, supplied copyright/licence/disclaimer notices, full legal text/link, adaptation and translation notices, same-licence distribution and no obstructive restrictions or technical measures. These obligations are identified for all accepted fields; this content review does not establish that the eventual combined package satisfies them.

## Validation and limitations

Windows owning-worktree logs remain under `C:/Users/inthe/IdeaProjects/lexique-spec-100-109/target/implement-100-109/worker-audit/author-107-final/target/issue-107/`, tied to the reviewed author head plus this retained review patch:

- Focused baseline and final field/source/membership checks: `reviewer-focused-baseline.log` and `reviewer-focused-final.log`.
- Independent revision comparison: `reviewer-remote.log` and `reviewer-primary/remote-verification.json`.
- Meaningful-content regression, accepted counts, draft preservation, diff and local-link checks: `reviewer-handoff-check.log`, `reviewer-diff-check.log` and `reviewer-links.json`.

The direct statute endpoint returned HTTP 403; no live statute refresh is claimed. Final conversion, combined notices, native lookup/save/reopen and release gates belong to the parent. No integration merge or full app suite was run. This review does not close #107.
