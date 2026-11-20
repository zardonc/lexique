# Issue #118 independent semantic and selected-rights review

**50 exact groups / 50 modern selected uses / 200 fields accepted unchanged;
zero unresolved fields.** Every FR/IPA/EN/ZH field was independently read against
fixed French/POS/form and neighbouring sense context. Frozen rows 1851–1900,
headwords, subclasses, frequency strings and order remain intact. The
denominator is **4,087 groups / 3,773 headwords**.

Reviewer: **Independent semantic and selected-rights reviewer for issue #118;
not the content author**. This fresh context is separate from the author and
parent integrator. This is agent review; human learner acceptance remains pending.
Full ticket [#118](https://github.com/zardonc/lexique/issues/118) and parent
[#78](https://github.com/zardonc/lexique/issues/78) were read from their JSON
captures in the specification checkout. Author SHA:
`13e2dd8ad39980a87cfe5e85dc1646444634e273`. Pinned reviewer base:
`3090384ec76b862a41fdffa8d1b4700e9b3a0224`.

Owned checkout: `C:/Users/inthe/IdeaProjects/lexique-review-118`, branch
`codex/d2-review-118`. Reviewed changes cover this evidence directory and
`scripts/test_core_batch_118.py`. Patch command:
`git diff 3090384 HEAD -- docs/evidence/issue-118 scripts/test_core_batch_118.py`.
Full and test-only patches are `reviewer.patch` and `reviewer-test.patch` under
the reviewer checkout's `target/issue-118-review/`.

## Selected content

[Field review](field-review.json) records original proposals, exact accepted
texts, individual quality/right reasons, fixed source bindings and independence,
plus all 50 accepted uses and 50 spelling/construction relationships.
[Original candidates](review-candidates.json) preserve the author's entire
groups and pending author-stage statuses. [Reviewed groups](groups.json) contain
the accepted content. No lexical or pronunciation replacement was needed.

| Frozen group | Accepted boundary |
| --- | --- |
| 1870 `vive/ONO` | The interjection section supplies acclamation, /viv/, the vive/vivent table and optional plural note. Fish, vif feminine and ordinary vivre flexions are separate. |
| 1884 `l'une/PRO:ind` | The exact ASCII redirect binds to typographic l’une, its feminine-singular l’un flexion and /l‿yn/. L’un selects de plus a plural set; un supplies one person/thing among others; une distinguishes the pronoun from newspaper une; le supports elision. |
| 1893 `lors/ADV` | The source restricts the time adverb to composition and lists lors de/des/du. The fixed lors de article supplies composition and event-time meaning. /lɔʁ/ is component IPA. |
| préféré, perdu, fatigué | Adjective definitions and agreement tables remain distinct from separately labelled verbal past-participle sections. |
| extérieur, professionnel | Adjective/noun pairs retain separate selected definitions and frames. Professionnel's noun retains feminine professionnelle; propriétaire and élève retain common gender. |
| puer, démarrer, prétendre | Bad smell and vehicle starting are explicitly intransitive. Prétendre que retains claiming/asserting in EN/ZH. |
| sou, fesse, désert | Modern money use, one buttock, and an arid environment remain explicit, excluding obsolete denomination, paired-body meaning and merely empty-place uses. |

All relations remain source binding only, with no admitted query alias.
There are zero drops, substitutions, new annotations or specialist exceptions.
EN/ZH are scoped explanations of the same French use, not upstream bilingual
quotations. All CEFR values remain unknown.

## Fixed sources and rights

Author archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/author-evidence/issue-118/`.
Original author evidence:
`C:/Users/inthe/IdeaProjects/lexique-batch-118/target/issue-118/`.
Ten five-group field projections and ten context projections retain the
examined content. The audit matched **54** raw articles to first-party API
titles, page IDs, revision IDs, timestamps, last editors and full raw text.
All **200** field and **273** supporting bindings match exact raw lines;
there are **248** distinct checked source locations.

Eight contributor-response parts reach terminal pagination for both request
batches. All **54** selected/supporting registries match, with **3,609** names
summed per source. Current registries extend beyond fixed revisions.
Article/history links supply attribution; neither a last editor nor an import
marker proves individual sentence authorship.

[Per-source review](source-rights-review.json), [field rights](rights.json) and
[notices](NOTICES.md) bind actual selected spans, contributors, original notices,
modifications and obligations. Captured CC BY-SA 4.0 sections 2–4, Wikimedia
Terms section 7, French copyright policy revision **38903168**, and DAF8 template
revision **22113837** were read independently. The **41** declared DAF8 original
boundaries reuse the established Académie française 1932–1935 collective-edition
basis and original credit, separately from Wiki transcription/contribution and
Lexique adaptation rights.

Selected lexical, grammar/form, construction and IPA spans pass that boundary.
Neighbouring quotations, media, etymology, translation tables and other languages
do not enter selected fields. Saisir's Old French Littré import is excluded;
bibliographic Littré references do not admit modern digital transcriptions.
Full legal text, source/history attribution, original notices, modification
information and ShareAlike remain required for the parent artifact.

## Verification and limitations

Windows x64, PowerShell **7.6.9**, uv Python **3.14.10**. The focused two-test
suite passes, requiring exact membership, preserved proposals, accepted
field/use/right decisions, source/contributor bindings, construction restrictions
and unchanged canonical-admission counts. Documentation/link and whitespace
checks cover batch documents and the owned diff.

Full logs, projections and tested-head record:
`C:/Users/inthe/IdeaProjects/lexique-review-118/target/issue-118-review/`:
`focused-final.log`, `source-audit.log`, `source-audit.json`, `docs-check.log`,
`docs-links.json`, `diff-check.log`, and `final-verification.json`.
The final record identifies the tested committed SHA and merged integration tip.

No independent live refetch or statute/edition refresh was run. All **200 fields
remain pending parent canonical admission and conversion**. Actual assembled
and packaged notices, native lookup/save/reopen, offline execution, human learner
acceptance, full/Lite sizes and release remain parent-owned gates.
