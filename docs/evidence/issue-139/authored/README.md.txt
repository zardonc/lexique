# Issue #139: frozen groups 2901–2950

**50 exact groups, 50 selected senses and 200 sourced author proposals** are
ready for separate review. Exact headwords, Lexique POS subclasses, original
order and both frequency strings match the frozen manifest. No group is
omitted, substituted or exempted. The global denominator remains 4,087 groups
and 3,773 headwords; CEFR remains unknown.

| Resource/admission measure | Author package |
| --- | ---: |
| Frozen groups / selected senses | 50 / 50 |
| Complete proposed fields | 200 |
| Fixed French source articles | 50 |
| Reused admitted groups / fields | 0 / 0 |
| Missing authored fields | 0 |
| Independently accepted new groups / fields | 0 / 0 |
| Fields pending independent quality / rights review | 200 / 200 |
| Missing admitted fields | 200 |
| Author-unresolved content rows | 0 |
| Annotation / specialist exceptions in this batch | 0 / 0 |

The [exact ticket #139](https://github.com/zardonc/lexique/issues/139) and updated
[parent #78](https://github.com/zardonc/lexique/issues/78) define the resource and
native seams. The worker/pipeline guide, project scope/product/roadmap/readiness,
domain glossary, resource ADRs, local development guidance and TDD resource-seam
instructions were read before authoring. The clean assigned branch
`codex/d2-content-139` was brought onto integration base
`d3725aa325ed9ac7222c2be80527efb798508485` without discarding work.

## Review inputs

- [Readable exact proposals](review-candidates.md) and [groups.json](groups.json):
  all four fields, precise selected use and excluded alternatives for every row.
- [field-review.json](field-review.json): 200 field proposals, 50 usage records
  and 50 source-spelling records. All reviewer identities are null and independent
  decisions remain pending.
- [source-captures.json](source-captures.json), [source-context.json](source-context.json)
  and [source-excerpts.json](source-excerpts.json): retained API/raw revisions,
  literal lexical/IPA/POS/construction spans, full contributor pagination and
  original-work attribution boundaries.
- [rights.json](rights.json), [selected-rights.md](selected-rights.md),
  [NOTICES.md](NOTICES.md) and [COPYING](COPYING): per-field author rights proposals,
  original-work credits, modification notices and redistribution obligations.
- [admission.json](admission.json) and [reuse.json](reuse.json): proposed and
  missing-admitted counts, pending gates and comparison against `core-129-v2`.
- [authored originals](authored/): all nine original JSON proposals retained
  before separate review. Changed wording requires a new review decision.
- [verification.json](verification.json): actual author commands and results.

French definitions are scoped adaptations. English and Chinese are newly
authored Lexique explanations supported by the selected French meaning; they
are not upstream bilingual quotations. The separate reviewer must assess their
actual wording, not infer language quality from a French source or source audit.

## Selected boundaries

`dealer/NOM` selects the slang drug seller /di.lœʁ/, distinct from the verb
/di.le/. `elle-même/PRO:per` selects feminine singular emphasis on the subject
personally performing an action. It retains the exact hyphenated spelling and
does not equate emphasis with acting alone or with every reflexive construction.
`bouffe/NOM` selects feminine popular food, not its other noun or verb sections.
`choper/VER` retains familiar France usage and the single `p`; `trouille/NOM`
retains familiar strong fear.

`bible/NOM` is a printed copy of the Bible, retaining lowercase common-noun
identity. `pension/NOM` is a paid place providing both meals and lodging.
`muet/ADJ` selects temporary emotional speechlessness. `suffisant/ADJ` means
enough, with the adjective section retained separately from the participle.
`milliard/NOM` is exactly 1,000,000,000, modern English short-scale billion and
Chinese 十亿. `gang/NOM` is the masculine criminal band; feminine Quebec group
usage is excluded. `technologie/NOM` selects the explicit study-of-techniques
definition, not the vague innovation line. `peste/NOM` selects the named disease
as ordinary vocabulary; it is not a specialist or annotation exception.

`dresser`, `casier`, `muet`, `plaisanterie`, `exécution` and `gang` retain their
recorded pronunciation variants in source context while proposing one applicable
isolated IPA. `soumettre` selects submission for examination or decision;
`préoccuper` is transitive causing concern; `ralentir` is transitive reducing
speed. Their pronominal or alternative constructions remain excluded.

Thirty-one articles declare DAF8 imports. The established independently reviewed
original collective-work basis is retained separately from all new selected-span
decisions. `portefeuille`’s CFC nautical-chart definition at line 62 is excluded;
the pocket wallet at line 49 is selected. Wallet and `injuste` photographs and
captions are outside selected spans. Literary/news examples, lyrics, audio,
images and other-language meanings supply no learner field.

## Retained evidence and reproduction

Reviewed checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-139`.
Owned tracked changes reach only this evidence directory and
[scripts/test_core_batch_139.py](../../../scripts/test_core_batch_139.py).
The stable archive is
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-139`.
It retains raw/API/request captures, all six contributor pages, rights-authority
captures, exact `selection.py`, adapted preparation/rights/capture helpers,
notice and reproduction helpers, baseline-resource metadata and execution logs.
Capture metadata uses this archive as its root and fallback; symbolic original
`target/issue-139` paths resolve there. Checks work without the original cache.

From the assigned ticket worktree:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_139.py
uv run --python 3.14.10 python -B -X utf8 C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-139/reproduce.py
pnpm test:docs
git diff --check
```

The focused tests inspect deliverable records, frozen frequencies, literal
source spans, contributor continuation requests, original proposals, selected
boundaries and pending decisions. Initial missing-content failure is retained
in ignored `target/issue-139/RED.log`; the original-proposal boundary failure is
in `RED-original-proposals.log`. Reproduction uses fixed captures and recorded
baseline metadata, writes root author output, and compares all nine JSON files
byte for byte. It must be run against author output; separate accepted reviewer
output needs its own preserved inputs. Full scoped and test-only patches, tested
commit identity and final integration merge identity are retained in the stable
archive’s closeout record.

## Pending gates

All 200 fields await separate quality, selected-span rights, spelling/construction
and everyday-use review. The coordinator owns accepted replacements, combined
canonical assembly/conversion, actual packaged obligations, all-group native
lookup/save/reopen, learner-data preservation and learner acceptance. Native,
frontend and Rust checks are not run for this author-only package. This package
does not establish admission or release readiness.
