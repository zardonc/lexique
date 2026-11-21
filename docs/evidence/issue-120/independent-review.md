# Issue #120 independent content and selected-rights review

**50 exact groups / 50 selected uses / 200 fields accepted.** There are
**197 unchanged fields and three exact replacements**, with no unresolved
field, use, spelling or selected-rights decision. Frozen rows **1951–2000**,
headwords, POS subclasses, frequencies and order remain unchanged. There are
zero reused groups, new exceptions or runtime aliases; the denominator remains
**4,087 groups**.

Reviewer: **Independent semantic and selected-rights reviewer #120**, separate
from the author and parent integration owner. This is an agent review; human
learner acceptance remains a separate gate. Review checkout:
`C:/Users/inthe/IdeaProjects/lexique-review-120`, branch `codex/d2-review-120`.
Author commit: `45f3be5`, integrated at `a27f410`; review base:
`92c208d2154c262035c8781463e3218463fac04f`. Ticket
[#120](https://github.com/zardonc/lexique/issues/120),
[PR #193](https://github.com/zardonc/lexique/pull/193) and the integration spec
were read from the supplied fixed captures. Review scope is this evidence
directory and `scripts/test_core_batch_120.py`.

## Exact field decisions

Every proposed FR/IPA/EN/ZH text was read against its fixed raw definition,
neighbouring senses, French language/POS/headword frame, pronunciation and
construction evidence. [Field review](field-review.json) is a
`separate_independent_field_review`, with exact proposed and accepted texts,
200 specific field reasons, 50 accepted uses and 50 spelling decisions. It
uses the independently reviewed [#119 schema](../issue-119/field-review.json).

| Field at row 1995, `épée/NOM` | Accepted text |
| --- | --- |
| French | Arme blanche à longue lame affilée munie d’une poignée, utilisée pour attaquer ou se défendre. |
| English | A sword: a bladed weapon with a long sharpened blade and a handle, used to attack or defend. |
| Chinese | 剑；带柄的长锋利刀刃武器，用于进攻或防御。 |

Fixed revision **39863164, line 14** supplies the blade, handle and
offensive/defensive function. The accepted everyday weapon meaning omits the
source's inaccurate blade geometry and historical carrying detail. The
[Académie ninth edition, `épée`, A9E2134](https://www.dictionnaire-academie.fr/article/A9E2134)
corroborates the distinction between ordinary swords and triangular sporting
equipment without supplying imported or translated wording. The fixed
Wiktionnaire excerpts, history, attribution, DAF8 credit and CC BY-SA 4.0
adaptation obligations remain intact. IPA `/e.pe/` is unchanged. All other
197 field texts are accepted as proposed.

The independent Spec reviewer for PR #193 accepted these exact three meanings,
the unchanged IPA and the ordinary feminine-noun use, separately from the
correction author. The four field records and usage record retain the reviewer
and modern-sense evidence. The accepted fields describe a sword without
requiring a triangular blade or carrying equipment.

Adjective/noun `adulte`, `canon`, `débile` and `idéal` remain distinct.
`participer à` and `réagir à` preserve their indirect/trigger constructions;
`décevoir`, `enregistrer` and `fixer` retain direct-object uses, while selected
`hurler` is an intransitive human cry. `hum`, `haine` and `hurler` retain their
h aspiré frame without adding a spoken `/h/`. `pédé` retains vulgar/insulting
register and narrowly scoped self-reclamation; noun `débile` remains an
insulting judgement, separate from the obsolete clinical description.
`pis/ADV` retains contemporary fixed expressions and explicitly dates free
use. Singular toilet-seat `lunette`, complete-facility `aéroport`, petrol
`essence`, number-noun `zéro` and the broad watercourse `rivière` keep their
selected restrictions.

## Rights and source frames

[Selected-rights review](rights-review.md) independently accepts the actual
lexical/phonetic boundaries. All **46 source frames** now pass at the selected
span boundary in [sources](sources.json) and [source context](source-context.json).
Complete article/history attribution, fixed revisions, selected spans,
original-work credits, modification notices, CC BY-SA 4.0 and ShareAlike
obligations are retained. DAF8's original collective-work expiry is
**2006-01-01**, using final 1935 publication and L123-3; live layer records use
that date.

[Original candidates](review-candidates.json),
[authored admission](authored-admission.json),
[authored review](authored-field-review.json),
[authored rights](authored-rights.json),
[authored sources](authored-sources.json) and
[authored context](authored-source-context.json) preserve the original proposals,
pending decisions and dates. [Live admission](admission.json) records actual
independent acceptance and zero pending independent fields. Canonical
assembly remains pending for all 200 accepted fields.

## Verification

Full evidence root:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/`.
The current everyday-sword regression exercises the produced SQLite payload in
French, English and Chinese. Its [RED](../../../target/fix-193/red.log) and
[GREEN](../../../target/fix-193/green.log) results use the same deterministic
converter path. The [independent exact-field acceptance](../../../target/fix-193/independent-sword-review.json)
records the four-field and usage decisions.
The [RED](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/content-rights-red.log) and
[GREEN](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/content-rights-green.log) verify the
exact weapon scope and original-work date. The
[fixed-source byte audit](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/fixed-source-byte-audit.json)
and [complete audit log](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/fixed-source-byte-audit.log)
compare all 46 articles to retained first-party revision and fixed-recapture
API content, all 200 spans and complete contributor pagination: **seven
responses / 3,332 named memberships**, with separate anonymous counts.
Current registries include later editors; selected-sentence authorship is not
inferred from them. Four policy captures reproduce their exact LF or CRLF
serialization explicitly; this is distinct from the 46 exact article/API byte
matches. No new upstream requests or 429 retries were made.

The [full focused checks](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/green.log),
[documentation checks](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/docs.log),
[owned local links](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/links.log) and
[final verification](../../../target/implement-120-129/reviewer-evidence/issue-120/issue-120-review/final-verification.json)
retain commands, statuses and tested head. Author raw evidence and its
reproduction scripts are copied byte-for-byte into the ignored
`target/issue-120-review/author-evidence/` directory. Original evidence also
remains in the author and integration worktrees. PowerShell 7 and uv Python
3.14.10 were used. The focused checks allow subsequent canonical overlap only
when the whole reviewed record is identical.

## Remaining gates

Canonical conversion and reproduction, combined packaged obligations, global
coverage/size reports, native lookup/save/reopen and learning-data preservation,
human learner acceptance, offline/platform execution and formal release are
parent-owned and unrun by this reviewer. Canonical resources, shared pipeline,
trackers and native readiness claims are unchanged.
