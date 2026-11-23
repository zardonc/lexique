# Issue #147 independent field and expression review

**50 exact groups, 50 everyday uses and 200 fields pass separate review.**
There are **195 unchanged fields and five changed fields**; 49 use descriptions
are unchanged and one is changed. No selected field remains unresolved.

Reviewer: **Independent field and selected-expression reviewer #147; separate
from author and integration coordinator**. This is a separate agent review,
not authenticated human or learner signoff. The exact cached #147 and #78
specs, reviewer guide, pipeline notes and repository guidance were read.
The established #129 format is used for evidence structure only.

Author head: `d75ae7e95ef266629d636a86feb555f463a2ed7d`.
Review base: `b85dfe73f9db3285588d7349d18f0a3922e33da6`.
Checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/review-147`.
Branch: `codex/d2-review-147`. The integration tip merged before final delivery
is recorded in the archived closeout. Only this evidence directory and
`scripts/test_core_batch_147.py` contain reviewer changes.

## Accepted content

Frozen rows 3301–3350 retain exact membership, POS subclasses, frequencies and
order. No annotation, specialist exception or alias is admitted. All four
fields explain the selected use. [Field review](field-review.json) contains
200 exact proposed/accepted texts and sources with per-field reasons, plus
50 use and 50 spelling/construction decisions.
[Replacements](accepted-replacements.json) retain exact texts and spans.

| Selected field | Accepted text |
| --- | --- |
| `avenue`, FR | Voie de circulation en ville, de taille et d’importance variables. |
| `avenue`, EN | An avenue: an urban street whose size and importance may vary; feminine noun. |
| `avenue`, ZH | 大街：城市道路，其规模和重要程度不一；阴性名词。 |
| `poignarder`, EN | To strike, wound or kill someone with a dagger; transitive verb. |
| `manche`, FR | Élément d’un vêtement qui couvre tout ou partie du bras. |

`avenue` retains variable size and importance. `poignarder` distinguishes striking,
wounding and killing. `manche` is the feminine sleeve with whole/partial arm
coverage. `allié`, `détenu`, `prostitué`, `survivant` and `explosif` use their
actual noun frames. `détenu` IPA comes from its nominal line. `chèvre` is explicitly
the female animal. `batterie` is connected rechargeable accumulators.
`homicide` is killing a human without a mandatory intent condition. `tsar` is
contemporary title vocabulary with a historical referent. French `golf` is the
sport; retained R&A Rule 1.1 corroborates its modern ball/club/hole mechanics.

## Sources and rights

All **50 raw articles match five first-party API responses byte for byte**.
The audit verifies **200 primary spans, 256 supporting spans and 2,100 span
occurrences** across the source and review ledgers. The complete seven-response
contributor chain gives **3,063 named article-contributor memberships** and
**486 anonymous article counts**. These are article credits, not unique people
or proof of a selected sentence’s author. Policy raw text uses the separately
recorded CRLF-to-LF API normalization; article bytes require literal equality.

[Specific expression review](original-expression-review.json) records **20 short
expression uses and 30 uses of minimal factual fragments**, with an individual
expression and meaning reason for every use. Ninety primary field source spans
select narrower facts; the rest of each factual use is retained as exact
support. The longer parent zoological prose for `chèvre` is narrowed as well.
Unselected CFC definitions for `marge` and `grain` supply no accepted content;
only their import tokens identify that excluded boundary.

Thirty-four articles retain DAF8 original-work import credit. The historical
French 2006 term calculation does not prove worldwide clearance and is not
the selected-expression acceptance route. [Selected rights](selected-rights.md)
records the exact short-expression/fact basis, attribution, CC BY-SA 4.0,
modification and ShareAlike obligations. Longer imported expressions,
literary/news examples, media and etymology supply no accepted wording.

Fourteen exact handoff files are preserved in `authored/review-base/`; all seven
pre-existing authored snapshots remain unchanged. Proposals and accepted
decisions are distinct. [Identity](independent-review-identity.json) records the
actual role, worktree and author/base pointers.

## Verification and limitations

| Command | Result |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_147.py` | Passed: nine resource/admission checks |
| Same Python flags, `target/issue-147-review/audit.py` | Passed: exact source identities/bytes/spans, contributor continuation, authority copies and snapshots |
| `pnpm test:docs` | Recorded in archived closeout |
| `git diff --check` and scoped-path check | Recorded in archived closeout |

The focused source/wording failures and passing checks are retained with the
full audit at:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/reviewer-evidence/issue-147`.
The archive includes copied fixed captures, authorities with original capture
metadata, actual decisions, helpers, logs, reviewed evidence and scoped patch.

Canonical conversion, combined packaged obligations, all-group native lookup,
trusted save/reopen, learner-data preservation and learner acceptance are
coordinator-owned and unrun here. Frontend, browser and Rust checks were not
run because no runtime/UI/Rust source changed. No whole-machine offline or
release readiness is claimed. There is no pending new owner choice for #147.
