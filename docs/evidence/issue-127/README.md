# Issue #127 author package

All **50 exact frozen groups, rows 2301–2350**, have one proposed everyday
selected sense and **200 sourced FR/IPA/EN/ZH candidate fields**. There are zero
reused groups, zero new annotations and no proposed runtime aliases. The
denominator remains **4,087 groups**. Independent quality and selected-rights
acceptance remain pending for all 200 fields; this package admits no content.

The [ticket](https://github.com/zardonc/lexique/issues/127) and
[parent specification](https://github.com/zardonc/lexique/issues/78) supply the
exact membership and approved resource/admission testing seam.

| File | Purpose |
| --- | --- |
| [groups.json](groups.json) | Exact identities, frequencies, selected uses and four candidate fields |
| [sources.json](sources.json) | 49 fixed French revisions, credits, modifications and licences |
| [source-excerpts.json](source-excerpts.json) | 200 field spans and French language/POS/construction support |
| [source-context.json](source-context.json) | Complete named registries, anonymous counts, selected spans and import/reference boundaries |
| [source-captures.json](source-captures.json) | Original responses, source bytes, pagination and ignored control-script pointers |
| [field-review.json](field-review.json) | Pending independent field/use/spelling decisions; null reviewer |
| [rights.json](rights.json) | Per-field rights proposals, original-work layers, authorities and obligations |
| [reuse.json](reuse.json) | No overlap with the inspected core-119-v1 canonical input |
| [admission.json](admission.json) | Proposed counts and pending independent/parent gates |
| [source-research.md](source-research.md) | Every selected sense and fixed definition/IPA location |
| [selected-rights.md](selected-rights.md) | Precise selected boundaries and proposed redistribution route |
| [NOTICES.md](NOTICES.md) | Fixed article/history links, original-work credits and modification notice |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |

`technique/ADJ` and `technique/NOM` retain separate source sections and sense
identities. `bite/NOM` retains its vulgar register. `monseigneur/NOM` is a scoped
honorific, especially for bishops. `pub/NOM` selects the informal **feminine
advertising message /pyb/**, distinct from the masculine bar /pœb/. `au-delà`
is a place adverb; `romantique` selects the sourced idealistic adjective.
`associé` retains its masculine noun form and evidenced feminine equivalent.
`hop` retains its aspirated-h frame. Alternative IPA variants are not extra
senses. No spelling substitution or annotation exception is proposed.

## Verification and reproduction

The [focused admission check](../../../scripts/test_core_batch_127.py) passes
two tests. The retained TDD logs cover missing group evidence and missing rights
evidence before the corresponding complete author proposals. These checks
validate candidate bindings and review status; acceptance belongs to the
separate reviewer.

Source reproduction passes for **49 exact wikitext files**, **nine contributor
responses across five fully terminated pagination chains**, and **nine JSON
outputs plus NOTICES.md**. There are **3,039 named per-article credits**, not
3,039 distinct people across the package. Anonymous contributors are credited
through their full linked histories and retained API counts. Current registries
may extend beyond the fixed revision and do not identify sentence authorship.
Reused legal-authority files and COPYING retain their original bytes.

Restore the full retained `target/issue-127/` directory to this checkout before
reproducing. It contains `source-index.json`, original API responses and request
URLs, wikitext, contributor pagination, rights authorities, acquisition and
preparation scripts, selection data and logs. The control scripts remain ignored;
the existing core converter remains integration-owned. Run against the author
baseline before independent review/canonical assembly changes these records.

```powershell
uv run --python 3.14.10 python -B -X utf8 target/issue-127/prepare_candidates.py
uv run --python 3.14.10 python -B -X utf8 target/issue-127/finish_author_package.py
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_127.py -v
uv run --python 3.14.10 python -B -X utf8 target/issue-127/reproduce.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-127`, branch
`codex/d2-content-127`. Only this evidence directory and the focused test are
tracked author changes. The complete raw package and logs remain in
`C:/Users/inthe/IdeaProjects/lexique-batch-127/target/issue-127/` for parent
archival. Final commit, integrated tip, verification results and file inventory
are recorded in `target/issue-127/final-verification.json`.

Key logs: `tdd-red.log`, `tdd-green-fields.log`, `tdd-red-rights.log`,
`tdd-green.log`, `reproduction.log`, `docs-check.log`, `owned-links.json`,
`diff-check.log` and `integration-final.log`.

## Limitations

No material author sourcing blocker remains. All 200 fields await independent
quality, use/spelling and selected-rights decisions. Original-work bases and
primary authority snapshots are reused; no live statute refresh is claimed.
Canonical conversion, combined packaged obligations, actual native
lookup/save/reopen, learner acceptance and release readiness are parent-owned
and unrun here.
