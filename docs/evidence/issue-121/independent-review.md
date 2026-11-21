# Issue #121 independent content and selected-rights review

**50 exact frozen groups / 200 fields accepted**, comprising **197 unchanged
fields and three exact replacements**, with **zero unresolved or pending
independent fields**. All rows 2001–2050 retain their headword, POS subclass,
frequency strings and order. There are no reused baseline groups, new
annotation/specialist exceptions or runtime aliases. The denominator remains
4,087 groups. Canonical resource admission remains parent-owned.

Reviewer: **Independent semantic and selected-rights reviewer #121**, separate
from the content author and parent integration owner. This is an agent review;
human learner acceptance remains a separate gate. Reviewed author commit:
`4e466b0`, [PR #193](https://github.com/zardonc/lexique/pull/193),
[issue #121](https://github.com/zardonc/lexique/issues/121), parent
[spec #78](https://github.com/zardonc/lexique/issues/78). Exact local ticket/spec
captures were read from
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.

Checkout: `C:/Users/inthe/IdeaProjects/lexique-review-121`, branch
`codex/d2-review-121`. Initial reviewer diff base after merging the clean
integration branch: `92c208d2154c262035c8781463e3218463fac04f`.
The review covers committed author evidence plus this review's changes in
`docs/evidence/issue-121/` and `scripts/test_core_batch_121.py` only.

## Independent field and use decisions

Every actual FR/IPA/EN/ZH field was read against its fixed French definition,
language/POS frame, applicable recorded pronunciation, neighbouring sense
boundaries and restrictions. The [separate field review](field-review.json)
uses `kind=separate_independent_field_review` and records 200 exact accepted
texts with field-specific semantic/rights reasons, 50 use decisions and 50
source-spelling decisions. The independently reviewed
[#119 schema](../issue-119/field-review.json) supplies the established contract;
the accepted #123 content and selected-rights review were also inspected in
`C:/Users/inthe/IdeaProjects/lexique-review-123/docs/evidence/issue-123/`.

| Exact field | Accepted text | Selected source basis |
| --- | --- | --- |
| 2044, French | Discours qui cherche à faire comprendre un texte, une idée ou une œuvre en donnant des renseignements, des détails ou des motifs. | Fixed revision 39657201, line 8 describes discourse that seeks understanding |
| 2044, English | An explanation; an account intended to make a text, idea or work understandable through information, details or reasons. | Same purposive explanation, without requiring success |
| 2044, Chinese | 解释、说明；旨在通过补充信息、细节或理由，使人理解文本、思想或作品的说明。 | Same explanatory purpose and information/detail/reason scope |

Row 2036 passes with its **dated-or-regional, familiar** marriage restriction.
The retained current Larousse collège entry explicitly records a regional
alternative; Wiktionnaire and Usito mark it dated. A scoped regional everyday
marriage term is eligible here. Neither an identified region nor nationwide
modern currency is inferred. All three meaning fields retain the restriction.

Row 2039 passes as the **exact lowercase French letter noun**. The letter
section supplies masculine invariant grammar, nominal article constructions
and `/be/` as the letter name. Its consonant sound, uppercase substitution and
the distinct abbreviation are excluded. Row 2041 passes **only** the modifier
construction explicitly assigned adjectival value at fixed line 26. Its
insulting register and anger/contempt remain in all meaning fields; the
standalone noun, literal passive and softened regional alternatives are
outside the selected use.

The two noun/adjective pairs at rows 2035–2036 and 2049–2050 remain separate.
Feminine fashion at row 2047 is distinct from the masculine noun. Transitive
immersion, causative habituation and teaching retain their object/complement
frames; relocation and obstacle collision remain intransitive. Silent initial
h, isolated IPA variants, additive versus equal-proportion adverb meaning,
mass sugar, optional bread in soup, and ordinary hyperbolic duration were
checked. Row 2024's on-demand payment instrument does not generalise the old
bearer-payment wording to every cheque.

## Rights, source frames and original proposals

All **48 selected source frames pass at the selected text boundary** in
[sources.json](sources.json) and [source-context.json](source-context.json).
The [selected-rights review](rights-review.md) resolves the contribution,
adaptation and applicable original-work boundaries for all 200 fields.
**39 articles import or reference DAF8**: the 24 explicit imports plus 15
bibliographic references receive conservative original-work credit. Neither
page markers nor current contributor lists establish sentence-level authorship.

[Original candidates](review-candidates.json),
[original admission](admission-author-proposal.json) and the complete
[authored snapshots](authored-snapshots/) retain the original proposals and
pending statuses. All six snapshots were compared with author commit
`4e466b0`; source excerpts, captures and original proposal documents remain
available. The live [admission ledger](admission.json) records the actual
independent result. Its 200 missing canonical fields and zero newly admitted
fields describe the separate parent assembly boundary, not pending review.

## Verification and retained evidence

Complete reviewer execution evidence is under
`C:/Users/inthe/IdeaProjects/lexique-review-121/target/issue-121-review/`.
Author raw evidence remains in
`C:/Users/inthe/IdeaProjects/lexique-batch-121/target/issue-121/` and its retained
integration mirror at
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/issue-121/`.

The [fixed-source audit](../../../target/issue-121-review/source-audit.json)
passes **48 raw/API bodies and 48 retained fixed-revision replay bodies
byte-for-byte**, **200 field spans**, **190 supporting spans**, all **seven
terminating contributor pages**, **3,336 named article memberships**, and
anonymous counts. Sixty retained files match the integration mirror. The
reviewer made no fresh dictionary requests; retained exact captures avoid
retrying rate-limited upstreams. Reused policy files are byte-identical to
their archive; their CRLF-to-LF text normalisation against policy API bodies is
recorded separately from the exact lexical byte comparison.

TDD retained [content RED](../../../target/issue-121-review/content-red.log),
[content GREEN](../../../target/issue-121-review/content-green.log),
[acceptance RED](../../../target/issue-121-review/acceptance-red.log) and
[acceptance GREEN](../../../target/issue-121-review/acceptance-green.log).
The [focused final result](../../../target/issue-121-review/green.log) passes
**five tests**, without skips. It checks all frozen identities, exact accepted
content, original proposals, independent decisions, source/rights/frame
bindings and raw contributor evidence. Later canonical overlap is allowed only
when the reviewed records' content, use, provenance and frozen membership agree.
It does not freeze parent-owned pending counts as the final outcome.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_121.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121-review/source_audit.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

The [final verification record](../../../target/issue-121-review/final-verification.json)
records the delivered SHA, latest merged integration, commands and outcomes.
Full reviewer and test patches are retained as `reviewer.patch` and
`reviewer-test.patch`; their command is
`git diff 92c208d2154c262035c8781463e3218463fac04f HEAD -- docs/evidence/issue-121 scripts/test_core_batch_121.py`.

## Remaining gates

Canonical assembly, combined resource obligations, packaged notices, global
coverage/size reports, actual native lookup/save/reopen, learner acceptance,
platform/offline execution and formal release remain parent-owned and unrun
here. Shared pipeline, canonical resources, trackers, application code and
learning data are unchanged by this review.
