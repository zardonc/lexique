# Issue #117 independent semantic and selected-rights review

**50 groups / 50 everyday uses / 200 fields accepted unchanged; zero corrections
and zero unresolved fields.** Frozen rows 1801–1850 retain exact spellings, POS
subclasses, frequencies and order. The denominator remains 4,087 groups.
No annotation, specialist exception or runtime alias is added.

Reviewer: **Independent semantic and selected-rights reviewer #117**, a fresh
context separate from the author and parent integration owner. This is an agent
content review, not human learner acceptance. Author commit:
`352d26695550607a0ab6b2d81a75dfa05ac25780`. Reviewed integration/diff base:
`7b9510481dde923b368700a7323c6ef19bc40b03`. Checkout:
`C:/Users/inthe/IdeaProjects/lexique-review-117`, branch `codex/d2-review-117`.
The full [ticket](https://github.com/zardonc/lexique/issues/117) and
[parent](https://github.com/zardonc/lexique/issues/78) were read from their fixed
captures in the supplied specification checkout.

## Field decisions

All 200 actual FR/IPA/EN/ZH fields were read against their fixed source and
French use in ten five-group projections. Neighboring-sense and import/reference
projections retain the source boundaries. [Field review](field-review.json)
records original and accepted text, individual language-specific reasons, exact
revision/line/column and supporting bindings, rights reasons, 50 modern-use
decisions and 50 spelling/form decisions. The complete
[author candidates](review-candidates.json), `proposed_text`, `proposed_usage`
and proposed binding evidence remain unchanged.

- `pisser` and `couille` retain vulgar/popular registers; `prof` and
  `rattraper` retain familiar usage. Neutral meanings do not erase those labels.
- `résister à`, direct `contacter quelqu’un`, `ajouter X à Y` and suitability
  `convenir à` retain their constructions. `veiller` is intransitive staying
  awake. `promener` leads a person/animal; `venger` takes the wronged person.
- Exact `celle-ci` is the feminine singular proximal demonstrative with
  `/sɛl.si/`. Bare `celle`'s continuation requirement does not bind this
  compound. `celui-ci`'s separate recent-reference sense is excluded.
- `direct/ADJ` selects a route without detour; its noun selects live television
  and `en direct`. `particulier/ADJ` selects a peculiar characteristic; its
  noun selects a private individual. `bref/ADJ` selects short duration; its
  adverb summarises. Their meanings and source sections remain separate.
- `génie` selects an exceptionally gifted person, `tribunal` the judging body,
  `millier` exactly a thousand units, and `terme` an ordinary word/expression.
  Domain indexing does not require specialist exceptions for these everyday uses.
- `prof` binds its familiar teacher noun to `professeur`, excluding academic
  rank and the unrelated depth abbreviation. Its own masculine/feminine frame
  and `/pʁɔf/` remain explicit.
- Exact `toutes/ADJ:ind` uses the indefinite-adjective flexion and `tout`
  totality sense with a feminine plural noun. Exact `toutes/PRO:ind` uses its
  own pronoun flexion and plural-pronoun sense for all previously mentioned
  feminine referents. Both have `/tut/`; neither uses the adverb flexion or
  singular neuter sense.
- `affreux` selects hyperbolically unpleasant things; `brave` selects courage.
  Personal ugliness, depravity and kind/decent `brave homme` remain excluded.
  Body/clothing nouns retain their ordinary selected uses; French singular
  `un pantalon` denotes one garment.

All remaining individual judgments are explicit in the field/use records.
EN/ZH are accurate new explanations of the same French use and restrictions.
Every IPA is an explicitly recorded French citation-form variant.
Spelling relations remain `source_binding_only` with
`alias_admission=not_admitted`.

## Evidence and checks

Author archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/author-evidence/issue-117/`.
Original author target:
`C:/Users/inthe/IdeaProjects/lexique-batch-117/target/issue-117/`.
Reviewer evidence:
`C:/Users/inthe/IdeaProjects/lexique-review-117/target/review-117/`.

The audit matches 50 raw/API revision pairs, titles, IDs, timestamps and last
editors; all 200 primary and 238 supporting spans; eight terminating contributor
parts; and three first-party authority captures. The registry total is 3,335
names summed per article. Current registries may extend beyond selected
revisions and do not identify sentence authors; fixed article and full history
credits remain necessary.

[Selected rights](selected-rights-review.md), per-source/context reasons and
200 field-rights records accept the chosen boundaries. All 38 DAF8
import/reference pages retain original-edition credit. Literary/news prose,
etymologies, media and unrelated senses supply no expressive field wording.
`moteur` adapts the machine definition following the image syntax.

Windows 11 x64, PowerShell 7, uv CPython 3.14.10 and pnpm 11.25.0 were used.
The focused two-test suite, source audit, documentation/link check and owned
whitespace checks pass. Complete logs, projections, `reviewer.patch`,
`reviewer-test.patch` and `final-verification.json` remain in the reviewer root.
The final record identifies the committed tested head and merged integration tip.

## Limitations

Fixed supplied captures were independently read and compared; no independent
live refetch or statute refresh is claimed. The DAF8 assessment applies the
established original-edition basis. Canonical admission remains zero new fields
with 200 pending. Parent conversion, actual packaged obligations, native
lookup/save/reopen, learner acceptance and release remain pending. Frontend,
Rust, native and offline checks were not run. No ticket is closed.
