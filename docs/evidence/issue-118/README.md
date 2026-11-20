# Issue #118 author package

All **50 frozen groups, rows 1851–1900**, have one authored modern selected
sense with **FR/IPA/EN/ZH: 50 senses / 200 fields**, and **zero missing authored
fields**. Their exact headwords, grammatical subclasses, frequencies and order
remain intact. All independent quality, rights and construction/spelling reviews
are pending for the parent's separate reviewer: **zero new admitted groups or
fields**. There are no new annotations or runtime aliases. The denominator remains
**4,087 groups / 3,773 headwords**.

Scope: [#118](https://github.com/zardonc/lexique/issues/118),
[parent #78](https://github.com/zardonc/lexique/issues/78), and the unchanged
[frozen manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Handoff records

| File | Content |
| --- | --- |
| [groups.json](groups.json) | 50 exact groups and 200 authored selected-use fields |
| [review-candidates.json](review-candidates.json) | Identical content reserved for the separate reviewer |
| [field-review.json](field-review.json) | 200 source bindings, 50 uses and source-only spelling/construction relations |
| [sources.json](sources.json) | 54 fixed French source revisions, licences, original credits and modification notices |
| [source-excerpts.json](source-excerpts.json) | Exact raw definition/IPA lines, IPA columns and supporting spans |
| [source-context.json](source-context.json) | Complete contributor registries, French/POS frames, original imports and flexion/construction evidence |
| [source-captures.json](source-captures.json) | Actual retained APIs, raw sources, rights authorities and preparation paths |
| [rights.json](rights.json) | 200 individual selected-field rights proposals and applicable obligations |
| [NOTICES.md](NOTICES.md) | Source, adaptation and redistribution notices |
| [reuse.json](reuse.json) | Zero assigned groups already admitted at the starting integration tip |
| [coverage.json](coverage.json) | Fixed denominator, canonical coverage boundary and authored content size |
| [admission.json](admission.json) | Authored counts and pending review, conversion and native gates |
| [handoff.json](handoff.json) | Integration pointers, verification status and retained diff/log paths |

[Source research](source-research.md) describes the selected-use boundaries.
[Independent review status](independent-review.md) reserves acceptance for the
separate reviewer.

## Verification and evidence

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-118`, branch
`codex/d2-content-118`. Integration `codex/d2-content-110-119` was confirmed and
merged at `2b0a4554704a1fc093e965fae5d78eaca1b1e6d0` before authoring; the final
integration pointer is recorded in [handoff.json](handoff.json).

The parent's approved resource seam is exercised by
[the focused check](../../../scripts/test_core_batch_118.py). RED records the
absent handoff, then the absent rights package; GREEN checks all 50 groups and
200 bound fields, exact special constructions and pending acceptance. Full logs
and source/preparation files are retained at
`C:/Users/inthe/IdeaProjects/lexique-batch-118/target/issue-118/`:
`red.log`, `fields-green.log`, `rights-red.log`, `green.log`, `docs-check.log`,
`final-diff-check.log`, `authored-package.patch`, and `tests.patch`.

Source snapshots are first-party API captures with fixed revision IDs and raw
text. The 54 selected/supporting revisions have complete linked histories and
paginated current contributor registries. Registries cover a broader current
author set than the fixed revisions; they are not individual sentence-authorship
claims. Tracked bindings remain available in detached checkouts; full raw APIs
and unselected context remain in this owning checkout.

## Limitations

All 200 fields await separate independent quality and selected-span rights
review, including the exact `l'une/PRO:ind`, `vive/ONO` and `lors/ADV` bindings.
The historical original-edition rights basis reuses the established #99 evidence;
this author makes no independent legal-refresh claim. Combined conversion,
notices, actual native lookup/save/reopen, offline execution, full/Lite size and
release gates remain parent-owned and unrun. The canonical admitted input and
saved learner data receive no change from this author package.
