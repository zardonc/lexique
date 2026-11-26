# Independent source, semantic and rights review #142

**45 complete groups / 180 fields** are ready for coordinator assembly.
**184 fields pass quality and rights: 182 unchanged, two Chinese replacements**;
four of those are IPA fields held with rights-incomplete groups. **16 fields
remain unresolved**: 12 textual source dependencies across four groups and four
`damoiseau` everyday-use requirements. Quality alone passes 49 groups/196 fields;
rights alone passes 188 fields. All 50 frozen groups / 200 proposals remain;
zero new canonical fields, annotations or specialist exemptions are admitted here.

## Identity and actual scope

Separate agent reviewer: `Independent source, semantic and selected-rights reviewer #142; separate from content author and coordinator`. This is documented independent agent
review, not authenticated human/learner signoff. Reviewed author HEAD `0b177a99e2d7809d18a80bb37fd2c39e7da153fd`
(`d216259` / `04ebd88`), [PR #211](https://github.com/zardonc/lexique/pull/211),
issues #142 and #78. Review base `9fa4f264509c953dba801d18ea6eaab84b2aafb2`; worktree
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/review-142`, branch `codex/d2-review-142`. Integration was first merged at `76102363c4166066fab725f761258db72bb6f035`,
then refreshed with tip `b85dfe73f9db3285588d7349d18f0a3922e33da6` as `8ac008c1af3af2605aa6bf124948bc5b8fdeec17`
before the final review commit. Shared files arrived
through that merge; the authored reviewer patch owns only `docs/evidence/issue-142/**`
and `scripts/test_core_batch_142.py`.

Every actual French definition, applicable IPA, English explanation and Chinese
explanation was independently read against retained fixed API/raw text, selected
use, language/POS, construction, register and spelling. [Field decisions](field-review.json)
and [use decisions](independent-review-decisions.json) contain the actual reasons.
Tests/audit helpers check consistency and bytes; they are not linguistic verdicts.
[Author snapshots](authored/snapshot.json) preserve all 16 original top-level
files before correction; exact proposed/accepted texts and spans remain in
[replacements](accepted-replacements.json). The external `before/` snapshot
additionally preserves original bytes without Git newline normalization.

## Actual changes and gaps

`bavarder` Chinese now preserves *sans mesure / sans discrétion* as lack of
restraint/discretion. `témoignage` now uses `证人` rather than `目击者`: witnesses
need not be eyewitnesses. Both regressions failed individually before correction.
The remaining 194 quality-accepted fields are unchanged.

`damoiseau/NOM`, row 3098, revision 39460790, nominal IPA line 7 and definition
line 9: fixed Wiki explicitly marks the gallant-man sense `ironique` and `vieilli`.
Fresh [Larousse](https://www.larousse.fr/dictionnaires/francais/damoiseau/21586)
lists ironic gallantry without `vieilli`. Fresh [Le Robert](https://dictionnaire.lerobert.com/definition/damoiseau)
defines the historical title; playful synonyms and modern-dated historical,
costume/reenactment or theatrical examples do not establish everyday gallantry.
Robert warns that its examples are automatic and not editorially reviewed.
Historical wording could describe a modern selected use under the spec, but
that new eligibility needs actual independent evidence. **Modern everyday use
of this selected sense remains unestablished; all four requirements stay unresolved.**
Pronunciation and narrow expression rights are sourced separately. No new sense,
specialist substitution, spelling alias or annotation exception has been admitted.

The rights decision assesses actual expression, not blanket article notices.
**25 of 29 DAF8-marked selected expressions clear individually** as conventional
short glosses/facts. Four developed textual boundaries remain unresolved:

| Row / word | Fixed revision / definition line | Concrete remaining gap |
| --- | --- | --- |
| 3051 catastrophe | 39826923 / 10 | Rhetorical *bouleversant le cours des choses* plus developed consequences; whole sentence matches independent DAF8 capture. |
| 3074 conscient | 39702182 / 8 | Developed inference from intimate awareness to responsibility; whole sentence matches independent DAF8 adjective capture. |
| 3076 reproduire | 39829187 / 23 | Combined copying/dissemination formulation, including *les vulgariser par un procédé quelconque*, matches independent DAF8 capture. |
| 3093 affecter | 39716112 / 59 | Metaphorical *Faire sur l’âme une impression…* matches embedded DAF8 second entry ACA8_535 in the retained CNRTL response. Article import notice alone does not prove copying. |

For those retained whole expressions, missing evidence is applicable US original
expression clearance/permission or a substantiated noncopyrightability basis.
France expiry 2006-01-01 supplies no blanket US clearance. No protection or
infringement determination is asserted. New EN/ZH wording does not itself copy
the sentence, but its retained exact upstream packet is a separate redistribution
dependency: three textual fields per group are held. All isolated IPA fields clear.
[Territorial review](territorial-rights-review.json) supplies each literal
selected/supporting span and specific reason. [Selected rights](selected-rights.md)
separates original works, contribution/adaptation licensing and packaging.

`bienvenu` retains the nominal `le bienvenu` use; `se marrer` remains explicitly
pronominal/familiar. `ô` retains emphatic feeling and applicable `/o/`, with modern
2018 campus-use evidence; fixed `particule` taxonomy is separately reconciled
with interjection/frozen ONO. `carton` selects packaging line 26 and IPA line 11;
CFC map prose line 53 is wholly excluded. Usage findings and capture identities
are in [usage corroboration](usage-corroboration.json).

## Source audit, checks and retained evidence

All 50 fixed API/raw pairs literally match; all 200 field spans and 2,678 nested
span occurrences match. Six completed contributor pages retain 2,827 named
article memberships and 443 anonymous counts. Full article credits and linked
histories are not sentence authorship. Policy/template identity, separately
recorded CRLF normalization and canonical COPYING match were checked. DAF8,
Littré and additional bibliographic original-work layers remain separately
credited; no full literary/news quotation is selected content.

[Verification](verification.json) records the final commands, tested merge SHA
and patch locations; external `final-closeout.json` retains patch hashes. Focused RED logs reproduce the missing independent
ledger, the two Chinese failures and the missing territorial decision. Final
checks cover the focused batch, fixed-source audit, documentation, links and diff.
The focused batch passed eight tests with one canonical-admission skip;
the fixed-source audit, `pnpm test:docs` (including local links), scoped diff
check and byte-preservation/count check passed. Explicit `--admitted` failed
as expected because this batch is absent from the canonical resource.
Canonical `--admitted` is expected to remain unsatisfied; native/combined
conversion, packaging and learner acceptance are coordinator-owned and unrun here.
Frontend/Rust checks are unrun because no application code changed.

Author raw inputs/helpers: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-142`.
Reviewer helpers, independent raw captures, original-byte snapshots, full logs,
owned full/test-only patches and closeout: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/reviewer-evidence/issue-142`. Working cache:
`target/issue-142-review/`. No push or tracker action was performed. These checks
establish the reviewed shard within the stated boundary; whole-machine offline,
other-platform and release readiness remain separate gates.
