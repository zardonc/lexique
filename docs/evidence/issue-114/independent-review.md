# Issue #114 independent semantic and selected-rights review

**50 groups / 200 fields accepted; no unresolved fields.** Every actual FR/IPA/EN/ZH text and selected use was independently read against its fixed French source, grammatical frame and neighbouring context. Frozen identities, frequencies and order remain; the denominator is 4,087 groups.

Reviewer: **Independent semantic and selected-rights reviewer #114**, a fresh review context separate from the author and parent integration owner. This is an agent review, not recorded human learner acceptance. Author commit: `f13aee1359d5a638d57692e8e2432fd3929bbbed`. Reviewed author head and reviewer diff base: `15cf8309ee3539ec1c2846c084a83cdc58013d75`. Checkout: `C:/Users/inthe/IdeaProjects/lexique-review-114`, branch `codex/d2-review-114`. Changes cover this evidence directory and `scripts/test_core_batch_114.py` only.

Retained patch: `C:/Users/inthe/IdeaProjects/lexique-review-114/target/issue-114-review/reviewer.patch`, using `git diff 15cf830 HEAD -- docs/evidence/issue-114 scripts/test_core_batch_114.py`. The test-only patch is `reviewer-test.patch` at the same root.

## Field and use decisions

[Field review](field-review.json) records original proposals, accepted texts and individual semantic/right reasons for 200 fields, 50 uses and 50 spelling relations. [Original drafts](review-candidates.json) preserve the complete author's groups and pending statuses. [Reviewed groups](groups.json) contain accepted wording. EN/ZH are explanatory translations of the same selected French use, not upstream bilingual quotations.

`puissant/ADJ` selects authority or influence, supported by its first definition and family/regional-power example. The effect-producing sense starts separately at line 21. EN/ZH and the use description expressly retain authority and influence. Masculine IPA `/pɥi.sɑ̃/` comes from `fr-accord-cons`, despite the empty `pron` template.

`commissaire/NOM` selects the common police ellipsis at component line 39. The fixed `commissaire de police` article supplies meaning, composition and reverse ellipsis. EN/ZH specify public-order regulations and retain the French office name. IPA `/kɔ.mi.sɛʁ/` is the component, not the phrase pronunciation.

`faillir/VER` retains `faillir + infinitive`: nearly doing something that does not occur. The source's auxiliary label within its verb section describes this semi-auxiliary construction without changing the frozen POS. `pendre` is explicitly intransitive suspension; `bouffer` is explicitly transitive eating with very colloquial/popular register, overriding their general entry headers for the selected definitions. `blond/ADJ` hair colour and `blond/NOM` man/boy remain separate; `arrivée` is the arrival moment noun. Traditional accents/hyphens, gender and recorded IPA variants remain exact.

All binding relations have `alias_admission=not_admitted`; old runtime aliases remain unchanged. No group is dropped, substituted or assigned a new owner exception.

## Sources and rights

Ten five-group projections exposed all actual fields and fixed spans. Ten further five-group projections exposed neighbouring context and import notices; full raw sources remained on disk. The source audit matched all **50** raw articles to retained first-party API content, titles, page IDs, revision IDs, timestamps and last editors. All **200** field and **207** supporting bindings match exact raw lines.

Eight contributor-response parts retain all **50** current registries with **3,418 names summed per article** and terminal pagination. Registries extend beyond fixed revisions; complete article/history links provide the attribution route. Neither a last editor nor an import marker is claimed as a selected sentence's author.

[Rights review](rights-review.md) and [per-source rights](rights.json) record CC BY-SA 4.0 sections 2–4, Wikimedia Terms section 7, fixed French copyright policy and the DAF8 template. The **26** DAF8-marked articles retain the established Académie française eighth-edition collective-work basis and original credit. Selected lexical wording is separate from quotations, media, etymologies and bibliography; `ramasser` adapts only the definition preceding image syntax. All selected rights decisions pass at this boundary.

## Verification and limitations

Windows 11 x64, Python **3.14.10**. The focused two-test suite and full retained source/contributor audit pass. Documentation link and whitespace checks cover the changed documents and diff. Evidence remains in `C:/Users/inthe/IdeaProjects/lexique-review-114/target/issue-114-review/`: `focused-final.log`, `source-audit.json`, `docs-check.json`, `diff-check.log`, ten `project-*.txt` and ten `context-*.txt` projections, and `final-verification.json`. The final record identifies the tested committed SHA and merged integration tip.

Fixed sources and authorities were read from retained captures; no independent live refetch or statute refresh is claimed. Canonical conversion, actual assembled notices, native execution, learner acceptance and release remain parent-owned pending gates.
