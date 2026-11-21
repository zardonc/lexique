# Issue #123 author package

All **50 exact frozen groups, rows 2101–2150**, have one selected everyday use
and **200 authored FR/IPA/EN/ZH fields**. There are **zero reused groups**,
missing author fields, proposed annotations or specialist exceptions. The
global denominator remains **4,087 groups**. All 200 independent semantic and
selected-rights decisions remain pending; this package records candidate
content, with zero newly admitted fields.

The [ticket](https://github.com/zardonc/lexique/issues/123) and
[parent spec](https://github.com/zardonc/lexique/issues/78) supply the scope.
The parent-approved resource/admission boundary is exercised by the
[focused check](../../../scripts/test_core_batch_123.py).

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities, frequencies, selected uses and four proposed fields |
| [review-candidates.json](review-candidates.json) | Original author group objects for the independent reviewer |
| [reuse.json](reuse.json) | Exact-pair comparison with admitted core-119-v1; no reuse |
| [sources.json](sources.json) | 52 fixed French source revisions, attribution, modifications and original-work layers |
| [source-excerpts.json](source-excerpts.json) | 200 fixed field spans and exact language/POS/construction support |
| [source-context.json](source-context.json) | Import/reference boundaries, 3,546 named registry entries and anonymous counts |
| [source-captures.json](source-captures.json) | Retained API/wikitext paths, eight terminating contributor responses and three fixed-revision recaptures |
| [field-review.json](field-review.json) | 200 individual field proposals, 50 use decisions and 50 spelling/construction bindings; independent decisions pending |
| [rights.json](rights.json) | Per-field selected-rights proposals, authorities and redistribution obligations |
| [selected-rights.md](selected-rights.md) | Actual contribution, historical-original and import boundaries |
| [source-research.md](source-research.md) | Every selected use with fixed FR/IPA source lines and restrictions |
| [NOTICES.md](NOTICES.md) | Proposed fixed-source credits and modification notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [admission.json](admission.json) | Exact author counts and separate pending review/integration gates |

Construction evidence retains `auprès de + complement`, `se suicider`, the
masculine lowercase letter `v`, abbreviated `labo` and France/informal `fac`.
The two `physique` POS groups have separate source frames. Meaning fields
retain the malted-barley restriction for `whisky`, the metro/small-station
restriction for `station` and complementless `davantage`. No query alias is
proposed.

## Verification and reproduction

The focused check passes **2 tests**. The TDD record contains an absent-package
membership RED followed by GREEN, and a missing-rights-package RED followed by
GREEN. Every field and source frame is checked against retained first-party
responses. Fresh requests by fixed revision reproduced all **52 source
articles byte for byte**. Rebuilding from those captures reproduces all
**10 JSON evidence files byte for byte**. Contributor pagination terminates
for both the main and support request sets.

Run from `C:/Users/inthe/IdeaProjects/lexique-batch-123`:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-123/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_123.py -v
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

The ignored [capture root](../../../target/issue-123/source-index.json) retains
raw responses, request URLs, complete wikitext, contributor continuations,
rights authorities, selections and preparation/reproduction scripts for parent
archival. `reproduce_sources.py` uses exact `revid` requests; retained fixed
responses support offline replay. The original batch captures are preserved
separately from the fixed-revision recaptures.

Logs: [membership RED](../../../target/issue-123/membership-red.log),
[membership GREEN](../../../target/issue-123/membership-green.log),
[rights RED](../../../target/issue-123/rights-red.log),
[GREEN](../../../target/issue-123/green.log),
[reproduction](../../../target/issue-123/reproduction.log) and
[final verification](../../../target/issue-123/final-verification.json).

## Remaining gates

Independent semantic, selected-span rights and construction/spelling review
remain pending for all 200 new fields. The author’s source/rights proposals
are separate from independent acceptance. Actual combined conversion,
packaged notices, canonical resource admission and native lookup/save/reopen
belong to the integration owner. Learner acceptance, platform/offline and
formal release checks remain unrun. Existing canonical resources and learner
data are preserved.

The branch is `codex/d2-content-123`, based on integration ancestor
`b91c4729a0a1c93baa58f86ed666b173fb2fc253`. The current integration tip is merged
before handoff; its exact SHA and final checks are retained in the verification
record above.
