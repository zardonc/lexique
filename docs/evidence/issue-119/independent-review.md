# Issue #119 independent semantic and selected-rights review

**49 new uses / 196 fields accepted; 191 unchanged and five exact replacements;
zero unresolved fields.** The unchanged accepted `côte/NOM` at row 1911 adds
one reused group / four reused fields, giving **50 batch groups / 200 fields**.
Each reused field is counted once globally. All frozen rows 1901–1950,
headwords, POS subclasses, frequency strings and order remain; the denominator
is **4,087 groups**.

Reviewer: **Independent semantic and selected-rights reviewer #119**, separate
from the content author and parent integration owner. This is an agent review;
human learner acceptance remains a separate gate. Ticket
[#119](https://github.com/zardonc/lexique/issues/119), parent
[#78](https://github.com/zardonc/lexique/issues/78), read in full from the
specified captures in `C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/`.
Reviewed author head and reviewer diff base:
`d4abf52426449dd5601e989640dc91649e6196f9`, containing integration
`3090384ec76b862a41fdffa8d1b4700e9b3a0224` and author commit `3247be1`.
Checkout: `C:/Users/inthe/IdeaProjects/lexique-review-119`, branch
`codex/d2-review-119`. Review changes cover this evidence directory and
`scripts/test_core_batch_119.py` only.

## Field and use decisions

Every actual proposed FR/IPA/EN/ZH text was independently read against the
fixed raw French definition, neighbouring sense boundaries, language/POS
frame, recorded pronunciation and construction evidence. The
[field review](field-review.json) retains exact proposals, accepted texts,
individual semantic/right reasons, 49 accepted uses and 49 spelling decisions.
[Original candidates](review-candidates.json) preserve all 50 author group
objects, including the unchanged reused group and original pending statuses.
EN/ZH are explanations of the selected French sense, not upstream bilingual
quotations.

| Exact field | Accepted wording | Source basis |
| --- | --- | --- |
| 1921 `visiter`, Chinese | 参观；出于虔敬或好奇，详细游览城市、纪念建筑或博物馆。 | Fixed selected definition, line 7 retains devotional purpose and a monument, without limiting it to ancient ruins |
| 1933 `civil`, French | Dans le civil : statut ou vie des citoyens qui ne sont pas militaires. | Fixed noun section, lines 142–145 attest civilian status and `dans le civil` |
| 1933 `civil`, English | In civilian life (dans le civil); the status or life of citizens who are not members of the military. | Same noun construction; distinct from the technical protected-person category |
| 1933 `civil`, Chinese | 平民身份或生活；用于 dans le civil，指不属于军队的公民的身份或生活。 | Same civilian-status/life construction |
| 1948 `embêter`, French | Familier : énerver ou importuner quelqu’un. | Fixed entry, lines 6–11 and the retained familiar use/EN/ZH |

`continue` is feminine singular of `continu`; `celles` is feminine plural
in the `celui` paradigm and needs a relative or prepositional complement.
`vieil` retains masculine singular before a vowel or silent h. `vôtre` is
the masculine singular component of `le vôtre`, addressing plural or polite
singular `vous`; its component IPA agrees with the modern pronoun table.
English Wiktionary `heu`, revision 92930671 line 74, explicitly attests the
French alternative spelling of `euh`; the French hesitation semantics and
recorded `/ø/` supply the selected frozen ONO use.

Adjective/adverb `soudain` and adjective/noun `civil` remain distinct.
`saigner` and `glisser` are intransitive; `visiter`, `exiger` and `apercevoir`
retain their selected direct-object uses. Feminine money `somme`, collective
singular `herbe`, familiar `bouge/gaffe/embêter`, unmarried-to-the-partner
`amant`, the selected bird-of-prey exclusion for `patte` and everyday
scientifically imprecise mass `poids` retain their restrictions. All spelling
relations have `alias_admission=not_admitted`.

## Reuse and rights

[Reuse identifiers](reuse.json) were compared with the current
[curated input](../../../resources/core-pilot-79/curated-inputs.json) and
[existing accepted field review](../../../resources/core-pilot-79/field-review.json).
The complete row-1911 group is equal: `cote-noun-rib`, source `fr_cote`, and
the four identifiers `french_definition`, `ipa`, `english`, `chinese` match.
Each prior accepted text and its passed quality/rights verdict also match.
No new field, use or spelling review for `côte` is present.

[Selected-rights review](rights-review.md) records the 52-source audit,
complete returned contributor registries, anonymous counts, fixed API/raw
comparison and selected import/reference boundaries. The DAF8 collective-work
expiry is **2006-01-01** under the established 70-year basis from final 1935
publication. Original proposed dates remain separately recorded. The CFC
marker at `pistolet` line 53 is confined to an unselected cartographic
definition. Applicable attribution, original-work credit, modification,
CC BY-SA 4.0 and ShareAlike obligations are identified; the assembled package
remains parent-owned.

## Verification and handoff

Reviewer evidence: `C:/Users/inthe/IdeaProjects/lexique-review-119/target/issue-119-review/`.
The [source audit](../../../target/issue-119-review/source-audit.json),
[focused results](../../../target/issue-119-review/green.log) and
[documentation results](../../../target/issue-119-review/docs.log) retain full
outputs. Retained reviewer patch command:
`git diff d4abf52426449dd5601e989640dc91649e6196f9 HEAD -- docs/evidence/issue-119 scripts/test_core_batch_119.py`;
patches are `reviewer.patch` and `reviewer-test.patch` in that evidence root.
Final tested head, latest merged integration and check statuses are retained
in `final-verification.json` there.

Focused batch checks pass **3 tests**, including exact membership and reuse,
all 196 accepted field/right/source bindings, original proposals, five exact
replacement fields and corrected original-work expiry. The independent raw
audit passes 52 source/API revisions, full registries, source/frame spans and
four prior reused acceptances. Documentation/link and diff checks pass.
PowerShell 7 and uv Python 3.14.10 were used.

Author raw evidence remains accessible at
`C:/Users/inthe/IdeaProjects/lexique-batch-119/target/issue-119/`;
future integration archive root is
`target/implement-110-119/author-evidence/issue-119`.

## Remaining gates

No unresolved new fields, drops, substitutions, new owner exceptions or
runtime aliases remain. Canonical admission, combined conversion/notices,
global coverage and size reports, native lookup/save/reopen, learner acceptance
and offline/platform/release verification remain parent-owned and unrun by
this reviewer. Canonical resources and learner data are unchanged.
