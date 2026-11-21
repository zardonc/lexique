# Issue #122 author package

All **50 exact frozen groups at rows 2051–2100** have one sourced selected
sense and **200 complete FR/IPA/EN/ZH candidate fields**. There is **zero reuse**,
no missing authored field, no annotation exception and no proposed runtime alias.
The fixed global denominator remains **4,087 groups**. All 200 fields await
independent semantic, construction/spelling and selected-rights review;
this author package admits **zero new groups or fields**.

The [ticket](https://github.com/zardonc/lexique/issues/122) and
[parent specification](https://github.com/zardonc/lexique/issues/78) supply the
scope. The captured spec and ticket are at
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.
The handoff is
`C:/Users/inthe/IdeaProjects/lexique-agent-notes/120-129/context.md`.

| File | Purpose |
| --- | --- |
| [groups.json](groups.json) | Exact identities, frequencies, selected uses and all four proposed fields |
| [reuse.json](reuse.json) | Baseline comparison: no previously admitted group in this frozen slice |
| [sources.json](sources.json) | 52 selected fixed French revisions, licences and attribution |
| [source-excerpts.json](source-excerpts.json) | 200 exact field spans and selected language/POS/construction support |
| [source-context.json](source-context.json) | Full named registries, anonymous counts, import/reference and original-work boundaries |
| [source-captures.json](source-captures.json) | Original API captures, exact revision recaptures and reproduction scripts |
| [field-review.json](field-review.json) | Author proposals for each field, use and spelling relationship; independent review pending |
| [review-candidates.json](review-candidates.json) | Original author group objects for later reviewer comparison |
| [rights.json](rights.json) | Selected-span author rights proposals and fixed legal authorities |
| [selected-rights.md](selected-rights.md) | Contributor, historical-original, IPA and translation boundaries |
| [admission.json](admission.json) | 50 proposed groups / 200 fields, zero admitted, and separate parent gates |
| [source-research.md](source-research.md) | All selected uses and construction/spelling decisions |
| [NOTICES.md](NOTICES.md) | Source histories, attribution and modification notices |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |

All **52 selected wikitext bodies reproduce byte for byte** through six exact
revision-ID requests. Contributor pagination terminates in six main and two
support responses. The author JSON package reproduces byte for byte from the
retained sources and explicit selections. Original whole-page audit captures
include unselected material; only the recorded lexical/IPA/grammar spans supply
candidate fields. English and Chinese are new scoped explanations of the same
selected French use.

## Verification and reproduction

The TDD seam is the spec's approved resource/admission handoff: exact frozen
membership, complete fields, truthful pending admission, fixed source bindings
and selected-rights proposals. It detects dropped/substituted groups, missing
fields, false acceptance, mismatched captured spans and missing rights records.
It does not establish independent linguistic quality or native execution.

The [focused check](../../../scripts/test_core_batch_122.py) passed **two tests**.
The retained RED/GREEN records cover absent candidate membership, then absent
selected-rights evidence, before their corresponding complete packages.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-122/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-122/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-122/write_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_122.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-122/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-122/capture_fixed.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-122`, branch
`codex/d2-content-122`. The clean starting HEAD was
`b91c4729a0a1c93baa58f86ed666b173fb2fc253`, an ancestor of integration
`codex/d2-content-120-129`. Only this evidence directory and the focused test
are owned changes. Latest integration is merged before handoff.

Complete execution evidence is retained in
`C:/Users/inthe/IdeaProjects/lexique-batch-122/target/issue-122/`:
`red.log`, `membership-green.log`, `rights-red.log`, `green.log`,
`fixed-recapture.log`, `reproduction.log`, `owned-docs-check.json`,
`full-docs-check.log` and `final-verification.json`. Standalone preparation,
selection, contributor pagination, exact revision recapture and reproduction
scripts remain alongside the raw sources and fixed rights authorities.
The final verification record supplies the tested commit, integration tip,
Windows/Python environment and actual check outcomes.

## Remaining gates

Independent field/use/spelling/rights review is pending for all 200 fields.
Canonical conversion and combined redistribution obligations, actual native
lookup/save/reopen, learner acceptance and offline/platform/release checks
remain integration-owned and unrun here. Canonical resources, shared pipelines
and learner data are unchanged.
