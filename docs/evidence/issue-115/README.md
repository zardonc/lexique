# Issue #115 independently reviewed content

**50 exact frozen groups / 50 selected senses / 200 FR/IPA/EN/ZH fields pass
independent content and selected-rights review.** Six fields in `violence` and
`titre` retain complete selected-source conditions. There are **no unresolved
groups or fields**, no dropped/substituted identities, no new annotations and
no admitted runtime aliases. Rows 1701–1750, frequencies and order remain exact;
the denominator remains 4,087 groups.

Scope: [#115](https://github.com/zardonc/lexique/issues/115),
[parent #78](https://github.com/zardonc/lexique/issues/78) and the unchanged
[scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Records

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Accepted wording, exact frozen groups, uses and exclusions |
| [review-candidates.json](review-candidates.json) | Unchanged original author proposals |
| [field-review.json](field-review.json) | 200 proposed/accepted texts, individual reasons, 50 use and 50 spelling decisions |
| [sources.json](sources.json) | 50 fixed Wiki revisions and one fixed DAF8 edition/page |
| [source-excerpts.json](source-excerpts.json) | Unchanged exact field and supporting bindings |
| [source-context.json](source-context.json) | Complete registries, original layers and per-source review |
| [source-captures.json](source-captures.json) | Original source, authority and preparation locations |
| [rights.json](rights.json) | Preserved proposals and independent selected-rights verdicts |
| [reuse.json](reuse.json) | No assigned group admitted at the author base |
| [admission.json](admission.json) | Independent counts and separate parent gates |

[Independent review](independent-review.md) records reviewer identity, exact
inputs, corrections and checks. [Rights review](rights-review.md) records
applicable obligations. [Author research](source-research.md) preserves selection
and proposal rationale.

## Verification

Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-115`, branch
`codex/d2-review-115`. Reviewed author commit:
`d136ed09ca0581f1cce6c581636b1fd6d6e17702`; review base:
`6b29f8be699ef3b489a4e5285db18ed120fe7501`.

The source audit matches 51 sources, 200 field and 218 supporting bindings, and
nine terminal contributor-response parts containing 3,348 names across 52
articles. Actual fields and neighbouring context were independently read.
The focused suite passes **three tests**; changed-document links and whitespace
checks pass. Full tests/source/projection/docs/diff logs, owned patches and final
head/integration metadata remain under
`C:/Users/inthe/IdeaProjects/lexique-review-115/target/issue-115-review`.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_115.py
pnpm test:docs
git diff --cached --check
```

Verified author archive:
`C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/author-evidence/issue-115`.
Original logs remain at `C:/Users/inthe/IdeaProjects/lexique-batch-115/target/issue-115`.
Accepted fields total **15,778 characters / 19,610 UTF-8 bytes**; the longest is
**216 characters**. These are selected-text measurements.

## Remaining gates

Independent acceptance is separate from canonical resource admission. All
200 newly admitted canonical fields remain pending parent normalization,
conversion and actual packaged notices. Fixed-publication identity, coverage/size
reports, native lookup/save/reopen and learning-data preservation remain
parent-owned, as do learner acceptance and offline/release checks. Shared
canonical resources and learning/audio data remain unchanged. Fixed captures
were used without a live source or statute refresh.
