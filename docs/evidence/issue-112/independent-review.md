# Issue #112 independent semantic and selected-rights review

**50 groups / 50 senses / 200 fields accepted unchanged; zero unresolved groups
or fields.** Rows 1551–1600 retain exact frozen headwords, POS subclasses,
frequencies and order. No new annotation, exception or runtime alias is admitted.
The denominator remains **4,087 groups**.

## Identity and scope

Reviewer: **Independent semantic and selected-rights reviewer #112**, a fresh
review context separate from the author and integration owner. This is an agent
review; human learner acceptance remains separate. Owning checkout:
`C:/Users/inthe/IdeaProjects/lexique-review-112`, branch `codex/d2-review-112`.
Reviewed author commit: `9864e8cc830567fc7ead4b62d3063a2ffc94dd8d`.
Reviewed integration base: `6b29f8be699ef3b489a4e5285db18ed120fe7501`.
Author's tested merge head: `d3d5477857cb4cab5ef0fba03777b621e83e51bc`.
The review covers this entire evidence directory and the focused batch test.

The [ticket](https://github.com/zardonc/lexique/issues/112) and
[parent](https://github.com/zardonc/lexique/issues/78) were read from their
supplied JSON captures. Required agent, scope, local environment, standards,
domain/readiness and resource-licensing guidance was read. Precedents #110/#111
and the completed #113/#114 review worktrees informed the record format.

Every actual FR/IPA/EN/ZH text was read in ten five-group projections, followed
by French/POS/form frames, neighbouring definitions, construction evidence and
article import/reference markers. [Field review](field-review.json) records an
individual reason and exact proposed/accepted text for all 200 fields, plus
50 accepted uses and 50 spelling decisions. [Original proposals](review-candidates.json)
retain their complete texts and pending author states.

## Content decisions

All 200 texts are accurate without replacements. EN/ZH explain the same selected
French uses and restrictions; they are Lexique translations, not upstream
bilingual quotations. Each IPA selects explicitly recorded French citation IPA.

- `rose` colour adjective/flower noun, `inconnu` adjective/person, `crétin`
  insulting adjective/person and `privé` private adjective/detective noun retain
  their separate source sections, POS and use boundaries.
- `in/ADJ` binds the French familiar invariable anglicism and `/in/`.
  `quelles/ADJ:int` binds exact feminine plural form/IPA at revision 35421628
  lines 1–4 to interrogative semantics at `quel` revision 39953698 lines 7–10.
  All meanings retain the before-feminine-plural-noun condition.
- `lourd/ADJ` selects physical weight. `lourd/ADV` selects importance/influence
  at revision 39849383 lines 213–216, with invariable `peser lourd`; adjective
  agreement and the later `beaucoup` sense are excluded.
- `lui-même/PRO:per` retains masculine singular acting-subject emphasis and
  whole-compound `/lɥi.mɛm/`. `hasard` has aspirated h; `horreur` has mute h.
  `samedi` retains the day sequence without a numbered-week convention.
- Transitive causation, tenant-side renting, physical joining/fastening and
  explicit pronominal exclusions remain. `étudiant` retains higher education;
  `étage` retains the above-ground-floor condition. Hunting, ground, spring and
  body vocabulary remain everyday senses despite their domain indexing.

All 50 spelling relations retain `alias_admission=not_admitted`. Canonical
resources, existing aliases and saved learner data are unchanged.

## Evidence correction

The selected field texts need **zero corrections**. The source-research summary
now identifies **all three CFC exclusions**, rather than only `rose`:

| Fixed source | Excluded boundary | Selected content |
| --- | --- | --- |
| `rose`, revision 39955249 | CFC cartography definition at 258 | Colour adjective at 11; flower noun at 225 |
| `source`, revision 39793245 | CFC cartography definition at 53 | Natural water spring at 10 |
| `chasse`, revision 39773099 | CFC footer at 336; typography definition at 48 | Human animal hunting at 23 |

The author already recorded these exclusions in source context. The summary now
matches the full evidence. [Selected rights](selected-rights.md) distinguishes
25 DAF8 import declarations, 12 DAF8 bibliography-only pages and nine other
sources. Import markers and contributor registries do not prove sentence authorship.

## Validation and retained evidence

Platform: Windows x64, PowerShell 7, uv CPython **3.14.10**, pnpm **11.25.0**.
Full reviewer evidence root:
`C:/Users/inthe/IdeaProjects/lexique-review-112/target/issue-112-review/`.
Author raw captures remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-112/target/issue-112/`.
The parent's future archive is `target/implement-110-119/author-evidence/issue-112`;
it has not been created by this reviewer.

| Check | Result | Full log/evidence |
| --- | --- | --- |
| Author resource-boundary baseline | Passed, two tests | `baseline.log` |
| Fixed API/raw content and exact spans | Passed, 46 revisions; 200 primary/208 supporting spans | `capture-audit.log`, `capture-audit.json` |
| Contributors, spellings and authorities | Passed, seven parts; 3,218 names summed per article; terminal pagination | `contributors-authorities-audit.json` |
| Reviewed focused batch test | Passed, two tests | `focused.log` |
| Documentation format/repository links | Passed | `docs.log` |
| Scoped diff whitespace | Passed | `diff-check.log` |

`groups-01.json` through `groups-10.json`, `frames-*.txt`, compact frames and
`source-markers.json` retain the inspected projections. Exact patch:
`reviewer.patch`, from
`git diff 6b29f8be699ef3b489a4e5285db18ed120fe7501 HEAD -- docs/evidence/issue-112 scripts/test_core_batch_112.py`.
The test-only patch is `reviewer-test.patch`. Final tested head, integration tip
merged before reporting and clean-worktree status are in `final-verification.json`.

## Limitations

Supplied fixed captures and retained authorities were independently read; no
live source refetch or live statute refresh is claimed. Contributor registries
may extend beyond fixed revisions and are not selected-sentence authorship proof.
Canonical admission/conversion, actual combined notices, native lookup/save/reopen,
saved-data preservation, learner acceptance, offline/platform and release checks
remain parent-owned pending. Structural checks do not replace the recorded
independent semantic review.
