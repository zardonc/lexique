# Issue #124 author package

All **50 exact frozen groups, rows 2151–2200**, have one selected everyday use
and **200 sourced FR/IPA/EN/ZH candidate fields**. There are **zero reused groups**,
missing authored fields, proposed annotations or specialist exceptions. The
global denominator remains **4,087 groups**. Independent semantic and
selected-rights decisions remain pending for all 200 fields; newly admitted
fields remain zero.

The [ticket](https://github.com/zardonc/lexique/issues/124) and
[parent spec](https://github.com/zardonc/lexique/issues/78) supply the scope.
The approved resource/admission seam is exercised by the
[focused check](../../../scripts/test_core_batch_124.py).

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact identities, frequencies, selected uses and all four candidate fields |
| [review-candidates.json](review-candidates.json) | Author group objects for the separate reviewer |
| [reuse.json](reuse.json) | Exact comparison with admitted core-119-v1; no reuse |
| [sources.json](sources.json) | 57 fixed revisions: 56 French-edition articles and one English-edition French spelling section |
| [source-excerpts.json](source-excerpts.json) | 200 fixed field spans with exact columns and construction/spelling support |
| [source-context.json](source-context.json) | Import/reference boundaries, 3,291 named registry entries and anonymous counts |
| [source-captures.json](source-captures.json) | Raw API/wikitext paths and 12 terminating contributor responses |
| [field-review.json](field-review.json) | Individual field, use and spelling proposals; independent decisions pending |
| [rights.json](rights.json) | Per-field rights proposals, authority captures and redistribution obligations |
| [selected-rights.md](selected-rights.md) | Contribution, original-work and selected import boundaries |
| [source-research.md](source-research.md) | Every selected use, fixed FR/IPA lines and complete restrictions |
| [NOTICES.md](NOTICES.md) | Fixed-source credits, historical originals and modification notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [admission.json](admission.json) | Authored counts and separate pending admission/integration gates |

The two `dessous` and two `rond` POS groups retain separate source frames.
Construction evidence preserves `sans cesse`/`n’avoir de cesse`, exclusively
pronominal `s’envoler`, the article in `la tienne` and prepositional `soi`.
English Wiktionary supplies a fixed modern French nonstandard spelling binding
for exact `voeu`; the French-edition Middle French article is audit-only.
Traditional `événement` is explicitly bound to `évènement`. The source spelling
relations propose no query aliases. Meaning/use restrictions also retain urban
`palais`, informal `moche`/`job`, regional gender for `job`, numeral positioning
and `vingt` pronunciation conditions.

## Verification and reproduction

The focused check passes **2 tests**: exact membership/four-field proposals and
fixed source/contributor/rights reviewability. The TDD record contains a missing
candidate-package RED followed by GREEN, then a missing-rights-package RED
followed by GREEN. All **57 raw article captures** match their retained
first-party revision content byte for byte. Contributor continuation chains
terminate and include all returned named identities and anonymous counts.
Offline rebuilding reproduces all **10 JSON evidence files byte for byte**.

Run from `C:/Users/inthe/IdeaProjects/lexique-batch-124`:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-124/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_124.py -v
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

The ignored [capture root](../../../target/issue-124/source-index.json) retains
original requests/responses, raw wikitext, contributor continuations, authorities,
selections and preparation/reproduction scripts for parent archival. The
original partial captures and earlier logs are preserved. `reproduce.py` replays
retained responses; `reproduce_sources.py` requests the exact revision IDs.

Logs: [membership RED](../../../target/issue-124/resume-membership-red.log),
[membership GREEN](../../../target/issue-124/resume-membership-green.log),
[rights RED](../../../target/issue-124/rights-red.log),
[GREEN](../../../target/issue-124/green.log),
[byte reproduction](../../../target/issue-124/reproduction.log) and
[final verification](../../../target/issue-124/final-verification.json).

## Remaining gates

Independent semantic, selected-span rights and construction/spelling review
remain pending for all 200 fields. The author proposals record zero independent
acceptance. Canonical assembly, combined converter output/notices, resource
admission and native lookup/save/reopen belong to the parent. Learner acceptance,
offline/platform and formal release checks remain unrun. Fresh network
recapture encountered Wikimedia HTTP 429; the retained first-party responses
support complete offline byte reproduction. Its
[failure log](../../../target/issue-124/fixed-source-reproduction.log) is retained.

The branch is `codex/d2-content-124`. Current integration is merged before
handoff; the exact merged integration SHA and final check results are recorded
in the verification file linked above.
