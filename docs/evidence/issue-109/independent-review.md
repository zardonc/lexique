# Issue #109 independent content and rights review

**50 groups / 200 FR/IPA/EN/ZH fields accepted; no missing fields.** Exact frozen rows 1401–1450, POS subclasses, order and both frequency strings remain. The denominator is **4,087 groups**. No runtime alias or owner exception is approved.

Ticket [#109](https://github.com/zardonc/lexique/issues/109), parent [#78](https://github.com/zardonc/lexique/issues/78). Reviewed author head and reviewer diff base: `524ac2d949d21d7eb79d33fcdbc9c6c9d880f5fd`. Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-109`, branch `codex/d2-content-109`. This reviewer is independent of the author and merger. Changes cover the evidence directory and the focused accepted-content comparison. Retained patch: `C:/Users/inthe/IdeaProjects/lexique-batch-109/target/issue-109/reviewer-handoff.patch`, produced by `git diff 524ac2d949d21d7eb79d33fcdbc9c6c9d880f5fd -- docs/evidence/issue-109 scripts/test_core_batch_109.py`.

## Decisions

Every actual proposed field was read against its fixed first-party French wikitext, specific grammatical frame and same-use restrictions. [Field review](field-review.json) records unchanged accepted texts and individual reasons for every field, use and spelling relation. [Groups](groups.json) contains reviewed content; [review candidates](review-candidates.json) preserves the author's original groups and pending statuses. EN/ZH are accurate Lexique explanatory translations, not upstream bilingual quotations. Each IPA is a selected recorded French variant, including the `sous-titrage` inflection template.

`quels/ADJ:int` keeps the exact masculine plural and noun-preceding identity question. Its own flexion article supplies spelling/IPA, while `quel` supplies the interrogative-adjective sense. `mien/PRO:pos` and `mienne/PRO:pos` require `le mien` / `la mienne` and the first-person possessor, with matching antecedent gender/number. The fixed phrase articles explicitly identify their component words; the standalone components supply `/mjɛ̃/` and `/mjɛn/`, distinct from whole-phrase pronunciation. The 2021 `le mien` and 2006 `la mienne` attestations support modern use. Bare possessive adjectives and plural forms are excluded.

`to/NOM` passes as an ordinary regional West African food: fixed French article lines 8–17 establish exact `to`, masculine noun, `/to/`, cereal flour/water, cooked paste or porridge and sauce. All three meanings retain that culinary/regional scope. English `to`, ISO language codes, adjacent photos/audio and other-language entries supply no selected wording. The spelling variant `tô` does not become a runtime alias. No specialist exception is introduced.

`cent/NOM` selects the first noun section's number 100, `/sɑ̃/`, separately from the quantity adjective and currency `/sɛnt/`. `enfoiré` adjective and noun retain vulgar, pejorative and insulting labels in FR/EN/ZH, excluding friendly `copain`. `enceinte/ADJ` explicitly binds its feminine form to the pregnancy sense of `enceint` and keeps the almost-always-woman restriction; the masculine rarity label is not misapplied to ordinary feminine usage. Aspirated `haïr` and mute `honnête` remain distinct. `juger de`, `conseiller à quelqu’un de` and article-bound possessives retain their constructions.

All **50 fixed first-party revisions** were independently refetched, comparing complete wikitext, title, page ID, timestamp and last editor to the saved captures. [Capture records](source-captures.json) links the comparison report. Provenance matching supports the independent content reading; it does not identify the author of every sentence or establish semantic acceptance.

[Rights](rights.json) records CC BY-SA 4.0 sections 2–4, Wikimedia Terms 7 and the current French copyright-policy/DAF8 template. These authorities were independently fetched for #109 and compared equal to the current authorities read in the immediately preceding #107 review. Selected lexical wording is separated from quotations, etymology, bibliography, other languages, translations and media. No non-DAF8 external-text import is selected; bibliography alone does not prove selected-sentence authorship.

Applicable DAF8 credit uses the established Académie française collective-edition 1932–1935 basis and conservative 2006 expiry. Contributor adaptations and Lexique explanations remain CC BY-SA 4.0. Combined redistribution requires fixed article/history attribution, applicable original-edition notices, supplied copyright/licence/disclaimer notices, full legal text/link, modification/translation notices, same-licence distribution and no obstructive restrictions or technical measures. These obligations are identified for every accepted field; actual combined-package compliance remains a parent gate.

## Validation and limitations

Windows owning-worktree evidence remains under `C:/Users/inthe/IdeaProjects/lexique-batch-109/target/issue-109/`, tied to the reviewed author head plus this retained review patch:

- Focused baseline and final field/source/membership checks: `reviewer-focused-baseline.log` and `reviewer-focused-final.log`.
- Independent revision comparison: `reviewer-remote.log` and `reviewer-primary/remote-verification.json`.
- Exact unchanged texts, five meaningful text/source mutation checks, original draft, handoff-schema, diff and local links: `reviewer-handoff-check.log`, `reviewer-diff-check.log` and `reviewer-links.json`.

The direct statute endpoint returned HTTP 403; no live statute refresh is claimed. Conversion, actual combined notices, native lookup/save/reopen and release gates remain parent-owned. No integration merge or full app suite was run. This review does not close #109.
