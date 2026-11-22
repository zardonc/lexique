# Issue #142 content-author handoff

**50 exact frozen groups retain 200 FR/IPA/EN/ZH proposals.** Forty-nine groups
have an author-proposed everyday sense. Row 3098, `damoiseau/NOM`, retains an
accurately labelled dated research proposal and **four unresolved ordinary-content
fields**. All 200 fields await independent quality and selected-span rights
review; **zero new fields are admitted or independently accepted**.

Membership is rows 3051–3100 of the unchanged 4,087-group manifest. The baseline
is `ff8aae2`, `core-139-v1`: 2,932 admitted groups / 11,728 fields, 18 annotations.
This batch reuses zero admitted groups or fields. The denominator, existing
resources and learner data remain unchanged.

## Reviewable package

[Readable proposals](review-candidates.md) give every exact field and the selected
and excluded uses. [groups.json](groups.json) retains exact identity/frequencies,
gender, number, constructions, register and applicable IPA. The source records
are [sources.json](sources.json), [source-context.json](source-context.json),
[source-excerpts.json](source-excerpts.json) and [source-captures.json](source-captures.json).
[field-review.json](field-review.json) records author proposals with null reviewer
and pending independent decisions. [admission.json](admission.json) records
coverage, missing fields and the unresolved group separately.

All 50 actual revision/API texts, all 200 field spans and the six-part completed
contributor continuation match. Returned names and anonymous counts are retained
with complete linked histories; a current registry is article credit, not proof
of selected-sentence authorship. One HTTP 429 during contributor pagination was
retained and retried after the calculated 30-second backoff; successful captures
were preserved. Request URLs, response headers, failure body and retry details
remain in the raw archive.

Raw captures, legal authorities, original author helpers and complete check logs:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-142/`.
The owned ignored `target/issue-142/` is the working cache. The source ledger
records the outside archive as its fallback. Original proposals are preserved in
`authored/` for separate review.

## Difficult groups and rights

`bienvenu/NOM` selects a welcome person, notably `le bienvenu`; the adjective,
feminine greeting and frozen masculine noun remain distinct. `marrer/VER`
selects the expressly pronominal `se marrer`, with its colloquial restriction.
`ô/ONO` selects emphatic feeling expression with `/o/`; the fixed source labels
the frame `particule`, while current Le Robert classifies this exact use as an
interjection. The retained 2018 campus account attests the short `Ô joie`
construction. [Usage corroboration](usage-corroboration.json) records the source
taxonomy and modern-use facts; external prose is not imported into learner fields.

`damoiseau`’s fixed gallant-man definition explicitly says `vieilli`. Larousse’s
ironic definition omits that label, but newer retained examples concern historical
discussion, reenactment or theatrical address. These conflicting observations
do not establish an everyday sense. The four ordinary-content requirements stay
unresolved; no group, alternate spelling, specialist substitute or annotation
exception is admitted.

[Rights proposals](rights.json), [selected rights](selected-rights.md),
[notices](NOTICES.md) and [COPYING](COPYING) retain CC BY-SA 4.0 contribution and
adaptation obligations. Twenty-nine DAF8 article layers preserve Académie
française’s 1932–1935 collective original-work basis, conservatively expired
2006-01-01. `marrer` separately credits the original historical Littré edition
and its established original-work rights. Each new selected span still awaits
review. `carton`’s CFC inset-map definition at line 53 is outside its selected
packaging sense at line 26. Referenced modern works, literature, media and
external dictionary prose keep their separate rights.

## Actual checks

Commands run from the author worktree with Python 3.14.10 through uv:

| Check | Actual result |
| --- | --- |
| Initial `scripts/test_core_batch_142.py` | Failed: all 50 rows lack admitted resource content; full RED log retained. |
| Initial author-package check | Failed: author evidence absent; RED log retained. |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-142/reproduce.py` | Passed: 14 evidence files reproduced byte-for-byte from fixed captures. |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_142.py` | Passed: four author resource tests; one integration-owned admission check skipped. |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_142.py --admitted` | Failed: canonical baseline lacks this batch. Author proposals do not satisfy admission. |
| `pnpm test:docs` | Passed; full result retained in the archive. |
| `git diff --check` | Passed for the scoped author package. |

## Limits and next boundary

Independent field/rights/construction/spelling review and resolution of row 3098
remain open. Combined conversion/packaging and actual native lookup/save/reopen
are coordinator-owned and were not run here. Frontend/Rust application checks
were not run because this package changes only author evidence and the focused
resource test. Learner acceptance, whole-machine offline checks, other-platform
execution and release readiness retain their separate gates.
