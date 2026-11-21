# D2 #129 author package

This package proposes **50 exact groups, 50 selected senses and 200 fields**
for frozen rows **2401–2450** in [ticket #129](https://github.com/zardonc/lexique/issues/129).
Every selected sense has a French definition, applicable IPA, English meaning
and Chinese meaning for the same POS and use. The 47 distinct ticket headwords
use **48 fixed French articles**, including the separately captured `clé`
spelling target for `clef`. Complete contributor pagination retains **8 API
responses, 3,153 named article-contributor memberships and 487 anonymous
article-contributor counts**. These are article credits, not distinct people
across the batch or proof of a selected sentence's individual author.

[Groups](groups.json) contain the complete candidate content.
[Field review](field-review.json) contains 200 pending field records with fixed
sources and grammatical evidence. [Admission](admission.json) preserves the
**4,087-group** denominator. [Reuse](reuse.json) compares the exact groups with
the admitted `core-119-v1` input: **0 reused groups and 0 reused fields**.
No annotation exception, specialist exception or runtime alias is proposed.

## Evidence

- [Sources](sources.json): fixed revisions, timestamps, article/history links,
  applicable attribution, licences and original-work layers.
- [Captures](source-captures.json): retained API responses and raw wikitext,
  complete contributor pagination, preparation and selection paths.
- [Source excerpts](source-excerpts.json): all 200 exact field spans and
  French language, POS, spelling, inflection and construction evidence.
- [Source context](source-context.json): full named contributor registries,
  anonymous counts, import/reference boundaries and 50 spelling bindings.
- [Rights](rights.json), [selected rights](selected-rights.md),
  [notices](NOTICES.md) and [legal text](COPYING): the proposed redistribution
  route, field-level modifications and separately credited original works.
- [Selected-use notes](source-research.md): everyday meanings and restrictions
  requiring independent semantic and construction review.

The fields contain **10,730 Unicode characters** in total; the longest field
contains **173 characters**. These are candidate-text measurements, not native
response, SQLite, lookup-performance or packaged-size measurements.

## Author verification

Approved seam: the candidate resource/admission record, against the immutable
ticket membership and retained fixed source bytes. The focused checks detect
missing groups/fields or selected-use labels, altered membership, wrong field languages,
incomplete contributor pagination, divergent raw captures and unsupported
independent-acceptance claims. The replay check detects preparation drift.

Run from `C:/Users/inthe/IdeaProjects/lexique-batch-129`:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_129.py
uv run --python 3.14.10 python -B -X utf8 target/issue-129/reproduce.py
```

The membership check has retained [red evidence](../../../target/issue-129/red.log)
and [green evidence](../../../target/issue-129/membership-green.log).
The selected-source/rights check has [red evidence](../../../target/issue-129/rights-red.log)
and the final [two-check green result](../../../target/issue-129/green.log).
[Offline replay](../../../target/issue-129/reproduction.log) reproduces all
**9 JSON evidence files byte for byte** from retained raw inputs, selection and
the existing preparation pattern, without fetching live sources.

The [owned documentation check](../../../target/issue-129/owned-docs-check.json)
passes for four Markdown documents and 26 repository links; six ignored local
evidence links are counted separately. The staged diff whitespace check passes.
[Repository-wide `pnpm test:docs`](../../../target/issue-129/full-docs-check.log)
also passes for 343 Markdown documents, 2,627 repository links and 231 separately
counted local evidence links.

The ignored execution root is
`C:/Users/inthe/IdeaProjects/lexique-batch-129/target/issue-129`.
Keep its `sources/`, `rights/`, source index, selection, preparation/reproduction
scripts and logs with the parent evidence archive. Recapture scripts are
separate from offline replay. Fresh article and contributor requests succeeded;
the existing exact rights-authority captures are reused with their original
successful statuses, versions and timestamps retained.

## Pending gates

All **200 authored fields await independent quality and selected-span rights
review**. Independent acceptance and newly admitted fields remain **zero**;
the batch retains **200 missing admitted fields**. The author's rights proposal
does not establish acceptance of an imported sentence or final combined notices.

The parent owns independent review, canonical conversion, packaged obligations,
all-group native lookup, representative trusted save/reopen, offline execution
and learner acceptance. These checks were not run here. Application code,
canonical resources, user learning data and saved references/digests are outside
this author's changes.
