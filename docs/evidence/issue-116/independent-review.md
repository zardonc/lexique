# Issue #116 independent semantic and selected-rights review

**50 exact groups / 50 senses / 200 fields accepted; 197 unchanged and three
accepted replacements; zero unresolved fields.** Every frozen row 1751–1800,
headword, subclass, frequency string and selected-sense identity is preserved.
The denominator remains **4,087 groups**. No exception or runtime alias is added.

Ticket [#116](https://github.com/zardonc/lexique/issues/116), parent
[#78](https://github.com/zardonc/lexique/issues/78). Author commit:
`b73f2527a222b8d4a08e537c77e99484439773f8`; author handoff head:
`3dc3a250ca8e6ea06fe3d772857cef18743eb08c`. Reviewed integration and diff base:
`7ceacbe8533a50a7367f9acb0eb2776749f63d91`. Reviewer checkout:
`C:/Users/inthe/IdeaProjects/lexique-review-116`, branch `codex/d2-review-116`.
This fresh agent reviewer is separate from the content author and parent
integration owner; no human learner acceptance is claimed.

## Field and construction decisions

All 50 groups and all actual FR/IPA/EN/ZH texts were read in ten five-group
projections against fixed raw French definitions, grammatical frames, recorded
IPA and neighboring sense/import/reference boundaries. Individual decisions,
source bindings, accepted wording, rights reasons and reviewer identity are in
[field-review.json](field-review.json). The complete original groups remain in
[review-candidates.json](review-candidates.json); proposed texts, usage and
author rationale remain intact alongside the independent decisions.

| Frozen row / field | Accepted text | Fixed source |
| --- | --- | --- |
| 1756 `nombre`, FR | Quantité non précisée de personnes ou de choses. | Revision 39864643, line 19: `Quantité indéterminée` |
| 1756 `nombre`, EN | An unspecified number or quantity of people or things. | Same selected definition |
| 1756 `nombre`, ZH | 若干，一定数量：未指明具体数目的人或事物。 | Same selected definition |

The accepted `nombre` use retains unspecified quantity rather than teaching
an exact count or mathematical number. Its `/nɔ̃bʁ/` remains unchanged.

`tien/PRO:pos` is explicitly the masculine singular component in `le tien`,
with second-person singular possession and the required definite article.
The component `/tjɛ̃/` is independently attested in `tien`; the bare obsolete
adjective does not supply the selected meaning. `oeuf` binds only to the
explicit typographic variant `œuf`, its culinary sense and singular `/œf/`.
Both spelling tables record plural `/ø/` and the additional `/œf/` variant.

`sauvage/NOM` selects the solitude/avoidance-of-company person noun; ethnic
and violent-person alternatives are excluded. `franc` sincerity/currency,
`espagnol` adjective/language, `supérieur` spatial adjective/authority noun
and `commun` shared adjective/collective noun remain separate. The latter
uses the fixed short lexical examples `le commun des hommes/des lecteurs`.

`déconner`, `nana`, `bêtise` and pronominal `se disputer` retain the selected
registers and constructions. `star` is the French feminine celebrity noun,
with French `/staʁ/`; `miss` selects the beauty-contest winner and retains
traditional `misses` versus rectified invariant `miss` in its use record.
`retraite` retains employee and regular-pension conditions. `coffre` permits
front or rear car luggage storage. Modern coat/room adaptations retain their
ordinary functions without imposing historical length/decoration rules.

EN/ZH are new explanatory translations of the same selected French uses,
rather than upstream bilingual quotations. All 50 spelling relations retain
`alias_admission=not_admitted`; none authorizes a canonical query alias.

## Fixed sources and rights

All **47** raw articles equal their retained first-party API revisions,
including title, page ID, revision, timestamp and last editor. All **200**
primary field bindings and **209** supporting records match raw lines.
The final check covers **1,504** primary/supporting occurrences in individual
field, use and spelling records. Eight contributor responses reproduce all
47 named registries, **3,260 article-level name occurrences**, anonymous counts
and terminal pagination. Current registries may extend beyond a fixed revision
and do not establish selected-sentence authorship; complete histories provide
the attribution route.

[Rights review](rights-review.md) records the separately read CC BY-SA 4.0
sections 2–4, Wikimedia Terms section 7, French copyright policy revision
38903168 and DAF8 template revision 22113837. Both fixed wiki authorities match
their retained API/raw captures. The 21 DAF8-import articles retain Académie
française eighth-edition credit and the independently read issue-99 original
collective-work L123-3 basis, conservatively expired 2006-01-01. Wiki and Lexique
contribution layers remain separately CC BY-SA 4.0. Bibliography alone does not
assign original authorship to a selected sentence.

## Verification

Platform: Windows x64, PowerShell 7, uv CPython **3.14.10**. Author captures/logs:
`C:/Users/inthe/IdeaProjects/lexique-batch-116/target/issue-116/`.
Full reviewer evidence:
`C:/Users/inthe/IdeaProjects/lexique-review-116/target/issue-116-review/`.

- `regression-red.log`: three scoped `nombre` assertions fail before replacement.
- `focused-green.log`: focused batch suite passes **2 tests**, covering exact
  frozen membership, all proposals/accepted fields, replacements, source/rights
  decisions, constructions, spelling and pending canonical admission.
- `source-audit.json` / `final-source-check.json`: complete fixed revisions,
  all spans, contributor pagination/anonymous counts, unchanged original
  candidates, rights-authority captures and complete legal text pass.
- `docs.log` / `diff-check.log`: repository documentation and whitespace checks.
- `reviewer.patch` / `reviewer-test.patch`: retained final scoped patches using
  `git diff 7ceacbe HEAD -- docs/evidence/issue-116 scripts/test_core_batch_116.py`.
- `final-verification.json`: tested committed head, latest integration merged,
  exact counts and final check statuses.

## Pending parent gates

Canonical admission, reproducible combined conversion, actual packaged notices,
native lookup/save/reopen, saved-data preservation, learner acceptance and
release/platform gates remain parent-owned and unrun. No application or native
suite was run here. Fixed supplied sources and authorities were reviewed; no
live refetch or statute refresh is claimed. Zero unresolved selected fields
does not close the ticket or complete the whole core.
