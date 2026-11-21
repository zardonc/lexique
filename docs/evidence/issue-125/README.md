# Issue #125 author package

All **50 exact frozen groups, rows 2201–2250**, have one selected everyday
sense and **200 proposed FR/IPA/EN/ZH fields**. There are **zero reused groups**,
zero annotation exceptions and no missing authored fields. The denominator
remains **4,087 groups**. All **200 fields await independent quality and
selected-rights review**; none is independently accepted or canonically admitted
by this author package.

The [ticket](https://github.com/zardonc/lexique/issues/125) and
[parent specification](https://github.com/zardonc/lexique/issues/78) define the
exact scope and approved resource/admission testing seam.

| File | Purpose |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities, frequencies, uses and four proposed fields |
| [sources.json](sources.json) | 51 fixed French source revisions, attribution, modifications and licences |
| [source-excerpts.json](source-excerpts.json) | 200 exact field bindings and language/POS/form/construction frames |
| [source-context.json](source-context.json) | Full named contributor registries, anonymous counts, selected spans, import and reference boundaries |
| [source-captures.json](source-captures.json) | Retained first-party responses, wikitext, contributor pagination and script pointers |
| [field-review.json](field-review.json) | Field/use/spelling proposals for the separate reviewer; acceptance is false |
| [rights.json](rights.json) | Per-field selected-rights proposals, original-work layers, authorities and obligations |
| [reuse.json](reuse.json) | No overlap with the inspected core-119-v1 canonical input |
| [admission.json](admission.json) | Author counts, pending independent decisions and parent gates |
| [source-research.md](source-research.md) | All selected uses and fixed source spans |
| [selected-rights.md](selected-rights.md) | Selected import boundaries and the proposed redistribution route |
| [NOTICES.md](NOTICES.md) | Fixed article/history attribution, original-work credit and changes |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |
| [selection.py](selection.py) | Authored exact-use selection, text and fixed line numbers |
| [prepare_candidates.py](prepare_candidates.py) | Existing evidence-schema preparation from archived source responses |
| [finish_author_package.py](finish_author_package.py) | Selected-rights proposals and notices |
| [reproduce.py](reproduce.py) | Byte comparison of the ten generated outputs and source/contributor capture checks |

`vous-même/PRO:per` retains polite **singular** address. `grand-chose/PRO:ind`
retains the negative construction and invariable form. `cents/ADJ:num` remains
the exact plural cardinal spelling, sourced through its own adjective entry and
the fixed `cent` cardinal/number notes; its IPA is **/sɑ̃/**, distinct from the
currency-noun entries. These are source bindings with no runtime alias proposed.

## Verification and reproduction

The [focused admission check](../../../scripts/test_core_batch_125.py) passes
**two tests**. The retained TDD logs show absent membership evidence before the
complete membership package and absent rights evidence before the complete
selected-rights proposals. The author checks validate fixed spans and metadata;
the separate reviewer decides actual field/use/spelling and rights acceptance.

Reproduction passes for **51 source wikitext files**, **seven main contributor
pages plus one cent-support page**, and **nine JSON files plus NOTICES.md**.
There are **3,004 named main article credits and 121 cent-support credits**;
these are per-article registry counts, not unique people across the package.
Anonymous contributors remain credited through complete histories and retained
API counts. Registries can include later contributors.

Restore the complete retained `target/issue-125/` directory into this author
checkout before reproducing. It contains source-index.json, the original API
responses and request URLs, wikitext, full contributor pagination, rights
authorities, acquisition scripts and logs. Reproduction uses these archived
inputs rather than mutable current articles. Run against the author baseline
before canonical assembly; a later reviewed/canonical package has its own inputs.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 docs/evidence/issue-125/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 docs/evidence/issue-125/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_125.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 docs/evidence/issue-125/reproduce.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

The complete acquisition scripts are `fetch.py`, `fetch_cent.py`,
`contributors.py` and `contributors_cent.py` in the retained capture directory.
The first two record fixed revisions; contributor scripts retain every
continuation URL and terminal response. Re-running acquisition without the
archive requests current sources and does not reproduce this selection.

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-125`, branch
`codex/d2-content-125`. Only this directory and the focused test are authored.
The branch descends from integration and merges the current integration tip
before handoff. Final commit, merged tip, command results and archive inventory
are retained in `target/issue-125/final-verification.json`.

Logs and raw package: `C:/Users/inthe/IdeaProjects/lexique-batch-125/target/issue-125/`.
Key logs: `membership-red.log`, `membership-green.log`, `rights-red.log`,
`green.log`, `reproduction.log`, `docs-check.log`, `owned-links.json`,
`diff-check.log` and `integration-merge.log`. Parent archives this complete
directory before cleanup.

## Limitations

No material author sourcing blocker remains. Independent quality, use/spelling
and selected-rights decisions remain pending for all 200 fields. Rights
authorities reuse retained primary snapshots and the established original-work
bases; no live statute refresh is claimed. Canonical conversion, actual combined
packaged notices, resource/native lookup/save/reopen, learner acceptance and
release readiness are parent-owned and unrun here. This author package does not
close the ticket or establish whole-core readiness.
