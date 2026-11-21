# Issue #126 independent content and selected-rights review

**50/50 exact frozen groups and 200/200 fields pass independent review.**
Rows 2251–2300 retain their headword, Lexique POS subclass, order and both
frequencies. There are 50 selected everyday uses, one Chinese wording
replacement, zero missing or pending content/rights fields and no exceptions
or runtime aliases. The global denominator remains 4,087 groups.

Reviewer: Independent semantic and selected-rights reviewer #126, separate
from the author and parent integration owner. Reviewed author commit:
`60805fce80f89f79127fa8c248f633791f9a5842`, [PR #193](https://github.com/zardonc/lexique/pull/193).
Checkout: `C:/Users/inthe/IdeaProjects/lexique-review-126`,
branch `codex/d2-review-126`.

[field-review.json](field-review.json) is a
`separate_independent_field_review`: each of the 200 fields has its accepted
text, specific semantic reason, fixed source and selected-rights reason.
All 50 uses and exact source spellings have separate decisions. The reviewed
schema follows [issue #119](../issue-119/independent-review.md) and the accepted
issue #123 reviewer record in
`C:/Users/inthe/IdeaProjects/lexique-review-123/docs/evidence/issue-123/`.
No author status or coverage counter supplies the independent decision.

The Chinese field for row 2273 `exagérer/VER` is
**« 夸大；赋予某事物超出实际的重要性 »**. Its selected definition describes
overstated importance. The original proposal's additional degree branch is
preserved in [groups-author-proposal.json](groups-author-proposal.json) and
the field review's `proposed_text`.

The source boundaries preserve adjective `exprès` with /ɛk.spʁɛs/; the second
noun section's shelf sense of `rayon`; ironing `repasser`; personal emotion
in `sensible`; and the separate adjective/noun groups of `moral`.
`méfier` retains mandatory **se méfier de** and component IPA /me.fje/.
French lowercase `f` is supported by its French letter section and the short
nominal constructions `Une f`, `Un grand f`, `Une petite f`; its selected IPA
is the letter name /ɛf/, not consonant /f/. None of these bindings admits a
runtime spelling alias.

## Selected rights and source frames

All **49 fixed articles**, **200 field spans** and **186 selected source
frames** pass byte comparison with the retained first-party revision captures
and fixed-revision recaptures. Complete contributor pagination agrees with
the retained named registries and anonymous counts. The complete public
histories contain **7,902 revisions**. History attribution covers anonymous
contributors; suppressed identities are unavailable and are not invented.
Contributor lists do not identify sentence-level authors.

[rights.json](rights.json), [source-context.json](source-context.json) and
[sources.json](sources.json) retain the independently reviewed contribution,
import and historical-original boundaries. The collective-work authority is
**CPI L123-3**, with the original Académie eighth edition (1932–1935)
conservatively expired **2006-01-01**, following the established
[issue #119 original-work review](../issue-119/rights-review.md). The live
records use that statute; the authored `L126-3` labels remain in the original
proposal snapshots. The referenced 1762/1798 Académie original editions and
Littré's nineteenth-century original prose have separately expired economic
rights. Littré uses **CPI L123-1**, death 1881, expiry **1952-01-01**.
Original author/edition credit remains; modern publisher editorial material
and XMLittré markup are outside these scopes. Import notices and references
do not prove which original author supplied a selected sentence.

Fixed Wiki transcription/adaptation and Lexique FR adaptations/EN-ZH
explanations retain **CC BY-SA 4.0**, including attribution, fixed source and
history links, modification notices, ShareAlike and no added restrictions.
The complete [legal text](COPYING), Wikimedia Terms section 7, copyright
policy revision 38903168 and import templates are retained with the author
captures. Applicable legal authorities were reused; this review made no live
upstream requests.

The selected `/flam/` comes from `flamme`'s `fr-rég` line 23. Its 1871
pronunciation quotation and TLFi-linked lines 28–29 do not supply a selected
field or source frame. `demi-heure` definition/IPA lines 8/7 exclude the DAF9
etymological dating at line 3. `exagérer` selects line 7, excluding the
Littré-referenced alternative at line 17. `f`'s short local noun examples
supply grammatical use only. No attributed literary/news quotation, media
or other-language meaning supplies accepted content.

## Verification

Approved seam: spec Testing Decision 2, conversion output/admission record.
The live [admission ledger](admission.json) records the actual independent
result. Author proposals remain byte-identical to commit `60805fc` in the six
`*-author-proposal.json` files; [review-candidates.json](review-candidates.json)
also remains unchanged.

| Check | Result | Reviewer log under `target/issue-126-review/` |
| --- | --- | --- |
| Chinese selected-scope RED → GREEN | Expected failure, then pass | `tdd-red-content.log`, `tdd-green-content.log` |
| Collective-work authority RED → GREEN | Expected failure, then pass | `tdd-red-rights.log`, `tdd-green-rights.log` |
| Independent admission RED → GREEN | Pending decisions rejected, then five checks pass | `tdd-red-independent.log`, `tdd-green-independent.log` |
| Fixed source, contributor/history and frame audit | Passed | `fixed-source-byte-audit.log`, `source-audit.json`, `source-audit-complete.log` |
| Canonical overlap | All 50 identical records pass; changed adjective IPA rejected | `source-audit-complete.log` |
| Final focused tests and diff/link checks | Passed | `final-tests.log`, `diff-check.log`, `link-check.log` |

Run the focused check with
`uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_126.py`.
The tests compare every field with the independent accepted text and rights,
retain proposal values, verify exact raw spans and reviewed source frames,
and allow later canonical overlap only when the content/provenance records
are identical. They do not assert final pending author counts or require
absence from a subsequently assembled canonical resource.

Complete reviewer execution evidence and reproduction scripts:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-126/issue-126-review/`.
Original raw captures, complete histories and legal authorities:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-126/` and the supplied
integration `target/issue-126/` archive. These ignored files remain available
for parent archival.

## Limitations

Canonical assembly, actual combined package/notices, native exact lookup and
save/reopen, learner acceptance and release/offline gates remain parent-owned
and unrun here. Independent content acceptance does not assert runtime
admission or native readiness. No shared pipeline, canonical resource,
tracker or native claim changed.
