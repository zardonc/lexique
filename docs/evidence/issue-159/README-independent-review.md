# Issue #159 independent review

**48 groups / 192 fields are independently eligible for admission.** All 50
frozen groups, 200 fields and 50 selected uses were inspected. Two owner-approved
zero-field annotations retain eight explicit missing fields.
The original review covered 200 selected texts; all pass the separate rights
review, and only 192 belong to the admissible shard.

Reviewer: independent field/use and selected-rights reviewer on
`codex/d2-review-159`, separate from the author and integration owner.
Scope: [#159](https://github.com/zardonc/lexique/issues/159) and
[parent #78](https://github.com/zardonc/lexique/issues/78).
Author commit: `024e688f6d16f4916fd125f4f8643e9bfd4774af`;
author head: `bc456ce2fdbf9a09571ca1d07626e4b8f58b3db8`.
Review base: `e7bbe0afe7ab156024f8c2bee61fd57a0e2057bc`.
Checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/review-159`.
Retained proof: `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/reviewer-evidence/issue-159`.

[Field decisions](field-review.json) retain every exact proposed/accepted text,
source span, independent semantic reason and separate rights reason.
[Use decisions](field-review.json) retain construction, register, spelling,
excluded meanings and the exact everyday/POS eligibility decision.
[Authored snapshots](authored) preserve the original package files unchanged.
[Full rejected proposals](rejected-proposals.json) retain the two unadmitted
groups and their original field/use/source decisions separately. The original
review archive remains unchanged.

| Review result | Count |
| --- | ---: |
| Eligible groups / uses | 48 |
| Eligible unchanged fields | 188 |
| Eligible changed fields | 4 |
| Rejected proposal fields / uses | 8 / 2 |
| Explicit annotation missing fields | 8 |
| Selected-text rights decisions passed | 200 |
| Dropped or substituted groups | 0 |
| Owner-approved zero-field annotations | 2 |
| New specialist exceptions | 0 |

[Exact replacements](accepted-replacements.json) make the source restrictions
explicit: Chinese `steak` retains intended pan cooking; French, English and
Chinese `proie` retain a living animal. Its accepted use also retains that
living-animal restriction. All other field texts are unchanged.

## Approved annotation scope

**Row 3922, `haver/VER`:** French Wiktionnaire revision **39585385**, lines
**5–7**, establishes a transitive mining verb and **h aspiré**. The retained
TLFi research page also supplies mining/quarrying, transitivity and initial
aspiration. It supplies no ordinary everyday use. The four mining fields are
semantically supported and rights-reviewed, but remain ineligible under the
current everyday requirement.

The owner approved row 3922 as a zero-field annotation. Its mining proposal
and independent rejection remain in the separate research ledger; no specialist
sense is admitted.

**Row 3944, `da/ONO`:** French Wiktionnaire revision **39926011**, lines
**27–30**, labels the selected construction **particule**, **familier** and
**vieilli**, and describes occasional isolated use as an **adverbe**. It
establishes neither the exact frozen ONO binding nor an everyday contemporary
use. The retained CNRTL page returns a noun/Wiktionnaire-derived section; its
navigation label is not evidence for this particle's ONO classification.

The owner approved row 3944 as a zero-field annotation. The frozen ONO
identity and both frequency strings remain unchanged, with no selected sense
and four missing fields. Its old-particle proposal remains separate.

The exact reply on 2026-11-23, America/Vancouver, was **Annotate both groups
with zero fields**. [Owner approval](owner-approval.json) applies only to rows
3922 and 3944. [Field/use decisions](field-review.json) contain 192 selected
fields / 48 uses, two annotation exceptions and eight unresolved missing fields,
each carrying its matching owner decision. [Admission](admission.json) reports
zero unresolved ordinary-content blockers and eight annotation missing fields.

## Source and rights evidence

All **50** raw article files equal their retained first-party API content bytes.
All **200** selected spans and supporting spans match the fixed revisions and
French frames. Eight contributor responses cover 50 article registries with
**2,503** named credits; continuation tokens match the next recorded requests,
and anonymous counts match the retained context. Registries can postdate the
fixed revisions; full history attribution is retained without inventing
selected-sentence authors.

[Selected rights](selected-rights.md) distinguish original DAF8 works from wiki
transcription/adaptation and Lexique explanations. All 39 DAF8-marked/reference
articles retain conservative original credit. Bibliographic references and
adjacent quotations are checked at their actual boundaries. [Notices](NOTICES.md)
and [complete licence](COPYING) satisfy the shard's CC BY-SA 4.0 obligations;
the combined converted artifact remains coordinator-owned.

## Verification and limitations

Windows x64, Python 3.14. Focused restriction regression: four failures against
authored texts, then passed with accepted texts. Full focused batch suite:
**7 tests passed**. The source audit passed. Documentation/link and whitespace
checks are recorded in the original [verification](review-verification.json).
The scoped owner implementation passed the annotation regression and all eight
batch tests; [owner-scope verification](owner-scope-verification.json) records
its preservation and documentation checks separately.

Native lookup/save/reopen, canonical conversion, packaging, learner acceptance
and release validation were not run in this scoped review. The eight annotation
missing fields retain the exact denominator. The reviewed shard is ready for assembly;
#159's integration gates remain pending. A fresh official
L123-3 retrieval returned HTTP 403; the reviewed, established historical
original-work basis in #99/#123/#129 supplies the edition and expiry evidence.
