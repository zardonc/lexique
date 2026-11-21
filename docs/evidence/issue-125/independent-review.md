# Issue #125 independent field review

**50/50 selected uses and 200/200 fields accepted**, comprising **196 unchanged
fields and four replacements**. All 50 frozen rows **2201–2250** retain exact
identity, grammatical subclass, frequency strings and order. Zero unresolved,
pending or missing fields/uses remain. No exception or runtime alias is admitted.

Reviewer: **Independent semantic and selected-rights reviewer #125**, separate
from the content author and parent integration owner. This is an agent review;
learner acceptance is a separate parent gate. Author commit:
`9cb84bab4e7f41f15195c6fd359037b91c320580`,
[PR #193](https://github.com/zardonc/lexique/pull/193),
[issue #125](https://github.com/zardonc/lexique/issues/125).
Review worktree: `C:/Users/inthe/IdeaProjects/lexique-review-125`,
`codex/d2-review-125`; owned scope is this directory and
`scripts/test_core_batch_125.py`.

The review uses the supplied integration spec/ticket and context, current
AGENTS/CODING_STANDARDS/LOCAL-DEV-ENV, admission/domain/product guidance,
resource-licensing ADRs, the independently reviewed
[#119 schema](../issue-119/field-review.json) and accepted #123 evidence in
`C:/Users/inthe/IdeaProjects/lexique-review-123/docs/evidence/issue-123/`.
The spec approves resource/admission tests; TDD was applied at that seam.

## Exact replacements

| Frozen row / field | Accepted text | Reason |
| --- | --- | --- |
| 2228 `gay` FR | Homosexuel : attiré sexuellement ou romantiquement uniquement par des personnes du même sexe ou genre ; se dit généralement d’un homme. | Restores the source's `seulement`; generally male is retained without making the adjective male-only. |
| 2228 `gay` EN | Gay; homosexual: sexually or romantically attracted only to people of the same sex or gender, usually said of a man (adjective). | Retains exclusive attraction, sex/gender alternatives, romantic/sexual alternatives and adjective scope. |
| 2228 `gay` ZH | 同性恋的：在性或浪漫关系上只受到同性别者吸引，通常用于男性（形容词）。 | Restores exclusivity and romantic attraction; general emotional attraction would widen the selected definition. |
| 2232 `noyer` ZH | 使某人因浸入液体而窒息（及物用法）。 | Retains immersion in liquid; water-only `溺水` narrowed the FR/EN selected mechanism. |

Full originals remain in [review-candidates.json](review-candidates.json);
[field-review.json](field-review.json) and [rights.json](rights.json) preserve
each proposed and accepted text. The original count/admission ledger is
[admission-author-proposal.json](admission-author-proposal.json).
The [live ledger](admission.json) records the actual 50/200 independent result.

## Source, use and rights decisions

[field-review.json](field-review.json) is a
`separate_independent_field_review`: **200 individual field decisions**, each
with exact source, accepted text, specific semantic reason and selected-rights
reason; **50 usage decisions** and **50 spelling decisions**. All language,
POS, IPA and selected construction frames were read against fixed source bytes.
Author validation was treated as a proposal, not an independent result.

`vous-même` remains polite singular address; plural `vous-mêmes` is explanatory
and unadmitted. `grand-chose` remains invariable with a negative construction.
`cents/ADJ:num` is supported by its exact French adjective-flexion header and
**/sɑ̃/**, plus fixed `cent` cardinal and spelling notes. Added reviewed spans
include `cents` **5, 11** and `cent` **30, 36, 38, 39, 41–43**, covering
multiplied terminal hundreds, following numeral and ordinal exclusions. The
parenthesized noun explanation in `cents` **10** is not adopted. The spelling
relation is an evidence binding, not a runtime alias or a currency noun.

`printemps`'s first lexical sentence is selected without calendar/date claims.
`mystère`'s human-affairs secret excludes the religious-source definition.
`divorcer` retains spouse-subject intransitive demander/obtenir rather than the
unselected judicial prononcer branch. `interrompre` uses the recorded standard
IPA at **119**, rather than the conflicting header or southern pronunciation.
All other individual scopes, object relations, register, region and gender
restrictions are recorded in the per-field and usage reasons.

[Selected rights](selected-rights.md) pass at the actual definition/IPA/form
boundaries. **34 DAF8-marked articles** and ten additional DAF8 reference
boundaries carry separate Académie française original-work credit, using the
established CPI L123-3 collective-publication basis and conservative
**2006-01-01** expiry. `rang`'s Littré import and four further French Littré
reference boundaries retain Émile Littré original-work credit. This adds
**14 conservative historical reference layers**. References/import markers
do not prove a selected sentence's author. Fixed contributor/transcription and
Lexique adaptation/translation layers remain **CC BY-SA 4.0**, with complete
licence, source/history attribution, changes and ShareAlike. Other reference
prose, examples, media and language sections are outside this selection.

## Verification and handoff

Initial integration merge fast-forwarded cleanly from `a27f410` to `3ef6048`.
Latest integration is merged before the final report. Exact commit/merge tips,
Windows platform, commands and statuses are in the ignored
[final verification](../../../target/issue-125-review/final-verification.json).

**Five focused tests pass**, verifying reviewed content against the original
snapshots plus four literal replacements, selected rights/source-frame
acceptance and actual ledger counts. Identical future canonical overlap is
allowed; its identities, selected use and content/provenance must match.
The original 200 pending author counts are asserted only in the snapshot.

The [source audit](../../../target/issue-125-review/source-audit.json) and
[byte log](../../../target/issue-125-review/source-byte-audit-reviewed.log) pass
**51 API/raw byte comparisons, 200 selected field spans and 812 field support
spans**, exact frozen membership, and complete contributor pagination.
Main: **seven parts, 50 articles, 3,004 named credits, 460 anonymous counts**.
`cent` support: **one part, 121 named credits, 25 anonymous counts**.
Registries may include later contributors; anonymous people remain credited
through the full linked history. These totals are per article, not unique people.

Acceptance, gay and noyer **RED/GREEN** logs, complete focused GREEN,
fixed-source audit scripts/inputs, original tracked snapshots, documentation
results, owned-link checks and full/test-only patches remain in
`C:/Users/inthe/IdeaProjects/lexique-review-125/target/issue-125-review/`.
The original author evidence also remains at
`C:/Users/inthe/IdeaProjects/lexique-batch-125/target/issue-125/` and is copied
under the review root's `author-evidence/`. Source requests were not repeated.

Canonical conversion and combined packaged notices, global coverage/size,
actual native lookup/save/reopen, learner acceptance and offline/platform/
release checks remain parent-owned and unrun. Live statute refresh was not
performed; retained primary captures and established original-work authorities
were reused and independently applied to the selected boundaries.
