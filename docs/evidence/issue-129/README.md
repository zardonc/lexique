# Issue #129 independent content and selected-rights review

**50 exact groups / 50 everyday selected uses / 200 fields accepted by separate
independent review.** There are **199 unchanged fields and one Chinese replacement**,
with zero missing reviewed fields or unresolved blockers. Frozen rows **2401–2450**,
all headwords, POS subclasses, frequency strings and order are preserved. The
global denominator remains **4,087 groups**. No annotation exception, specialist
exception or runtime alias is admitted.

Reviewer: **Independent semantic and selected-rights reviewer #129; separate
from author and parent integration owner**. This is an agent content/rights
review; learner acceptance remains a separate gate.
[Ticket #129](https://github.com/zardonc/lexique/issues/129) and the full spec were
read from `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.
Author commits: `22a4909` and `ea797f3`; handoff `e071fa4`,
[PR #193](https://github.com/zardonc/lexique/pull/193).
Clean integration was merged before review, giving review base
`55971e6301c930cb381a6ef67349eece0de35269`.
Checkout: `C:/Users/inthe/IdeaProjects/lexique-review-129`, branch
`codex/d2-review-129`. Only this evidence directory and
`scripts/test_core_batch_129.py` contain reviewer changes.

[Groups](groups.json) hold accepted content. [Field review](field-review.json)
uses `separate_independent_field_review`, following the independently reviewed
[#119 schema](../issue-119/field-review.json) and accepted
[#123 example](../issue-123/field-review.json). Each of the 200 records has the
exact proposed and accepted text, selected source, semantic decision, rights
decision and independent reviewer. All 50 uses and spelling bindings have
specific reasons. [Authored snapshots](authored/snapshot.json) preserve original
[groups](authored/groups.json), [admission](authored/admission.json),
[field proposals](authored/field-review.json), [rights](authored/rights.json),
[sources](authored/sources.json), [context](authored/source-context.json) and prose.

## Accepted content and boundaries

| Field | Accepted text | Selected basis |
| --- | --- | --- |
| 2417 `boucler/VER`, Chinese | 用搭扣扣住、扣紧；及物动词。 | `boucler`, revision 39447424, definition line 8: fastening/tightening with a buckle |

The [replacement record](accepted-replacements.json) keeps the exact proposal
and reason. French, IPA and English are unchanged. EN/ZH are scoped Lexique
explanations of the selected French sense, rather than upstream bilingual quotes.

`indien/ADJ` describes India and its inhabitants; exact lowercase `indien/NOM`
names the Orangina-and-grenadine drink. The uppercase nationality demonym is not
substituted. `communiste` and `chrétien` retain distinct adjective and person-noun
frames. `clef` explicitly binds to the evidenced spelling variant `clé` for the
mechanical-key meaning, with IPA from `clef` itself. This source relation does
not admit a runtime alias.

`écoute` selects listening from a line also containing obsolete watchkeeping.
`se balader` retains its pronominal requirement, and IPA remains the isolated
`balader` infinitive. `songer à`, `manque de`, `incapable de`, `nier que` and
`céder quelque chose à quelqu’un` retain their evidenced constructions;
`pas forcément` means “not necessarily”. The noun/adjective frames and selected
transitive/intransitive uses were inspected individually.

Exact `ha/ONO` has a French onomatopoeia frame and surprise meaning, with literal
recorded `/ha/`; its repeated-laughter sense is unselected. This interjection
value is not a general rule that French h aspiré is pronounced `/h/`.
`hélas/ONO` retains its interjection frame and `/e.las/` with final `/s/`.
Selected variants for `ours`, `passager`, `mai`, `soulever`, `ticket` and familiar
`foot` are applicable to their actual uses.

## Selected rights and source audit

All **48 fixed French articles**, their **200 selected field spans**, and
**2,400 recorded supporting-span occurrences** pass the independent retained
capture audit. Each article's raw UTF-8 bytes equal its first-party API revision
content. All French language/POS/head/IPA frames pass at the selected boundary.
Complete contributor pagination covers **8 responses in two finished chains**,
**3,153 named article-contributor memberships** and **487 anonymous counts**.
These counts represent article credits rather than unique people or
sentence-level authorship.

[Selected rights](selected-rights.md), [rights records](rights.json),
[source context](source-context.json), [sources](sources.json),
[exact excerpts](source-excerpts.json), [notices](NOTICES.md) and the
[complete licence](COPYING) identify the actual retained obligations.
The original DAF8 collective-work expiry is **2006-01-01**, under the established
70-year L123-3 basis from final 1935 publication. Proposed `1966-01-01` values
remain in authored snapshots and explicit proposal keys. `passager` also retains
Littré original-work credit and its **1952-01-01** L123-1 expiry.

Raw source and authority evidence:
`C:/Users/inthe/IdeaProjects/lexique-batch-129/target/issue-129/`.
Independent audit and complete execution logs:
`C:/Users/inthe/IdeaProjects/lexique-review-129/target/issue-129-review/`.
Retained copyright-policy/template text matches the captured API after CRLF to
LF normalization; the audit records this separately from literal article byte
equality. No fresh upstream requests or retries were made.

## Verification and remaining gates

Approved seam: resource/admission records against the exact frozen manifest and
independently reviewed field/source/rights records. Four focused tests verify
exact content/proposals, corrected original-work dates, all source frames and
credits, and acceptance counts. Canonical overlap is allowed only when the
complete reviewed group is identical; it is not counted as author-time reuse.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_129.py
uv run --python 3.14.10 python -B -X utf8 target/issue-129-review/audit.py
pnpm test:docs
git diff --check
```

Ignored reviewer files retain `rights-red.log` / `rights-green.log`,
`content-red.log` / `content-green.log`, `review-red.log` / `green.log`,
`source-audit.log` / `source-audit.json`, `frames.txt`, `boundaries.json`,
documentation results and final verification metadata.

[Live admission](admission.json) distinguishes **200 independently accepted
fields** from **zero newly admitted canonical fields / 200 pending canonical
fields**. Current canonical/native version remains `core-119-v1`. Canonical
assembly, actual combined conversion/notices, global coverage and size reports,
native lookup/save/reopen, learner acceptance and offline/platform/release
verification remain parent-owned and unrun by this reviewer.
