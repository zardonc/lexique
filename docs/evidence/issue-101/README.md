# Issue #101 author handoff

All **50 exact frozen groups, rows 1001–1050**, are retained. **49 groups have
196 source-backed FR/IPA/EN/ZH candidate fields**. Row 1005 `m/PRO:per` has
**four missing candidate fields**. Independent admission remains **0 groups /
0 fields**; all **200 fields are unadmitted**. The permanent denominator is
**4,087 groups / 3,773 headwords**.

The seven-file package is [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). Review fields retain candidate text and null
accepted text/reviewer values. The separate reviewer owns acceptance and any
exact replacements; [independent review](independent-review.md) is pending.
[Review candidates](review-candidates.json) contain the
same frozen groups and authored fields.

[Source research](source-research.md) records the selected uses, spelling/POS
boundaries, original source layers and proposed redistribution route.
[Source captures](source-captures.json) identify original API/HTML responses,
complete blank-line-preserving wikitext, actual returned revisions, timestamps,
fixed URLs and complete article-history attribution.

## Author verification

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-101`, branch
`codex/d2-content-101`, based on integration base
`aae5273af846e57668a8d267fa6a6566c4fd6e91`.
Integration tip `87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5` was merged before
the authored handoff commit. Only `docs/evidence/issue-101/**` and
`scripts/test_core_batch_101.py` are authored changes.
Author source evidence, reproducible preparation and execution logs are under
`C:/Users/inthe/IdeaProjects/lexique-batch-101/target/issue-101/`.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-101/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_101.py
```

The focused check passes both tests: exact row/headword/POS/frequency membership,
four-field evidence consistency and pending-review semantics; and every field
or supporting Wiki span against its retained original API revision and raw text,
including language/POS frames and exact IPA columns. Full original captures are
required for the second test. The first missing-candidate RED is retained in
`target/issue-101/red-membership.log`; the membership GREEN is in
`target/issue-101/green-membership.log`. The field-evidence RED is in
`target/issue-101/red-fields.log`; final GREEN is in
`target/issue-101/green-fields-final.log`.

Scoped diff whitespace and local-link checks pass. Their logs are
`target/issue-101/diff-check.log` and `target/issue-101/docs-check.json`.
The full authored patch is `target/issue-101/author-handoff.patch`; the separate
test patch is `target/issue-101/author-test.patch`. Diff base is the merged
integration tip above; the evidence root also retains the final commit ID and
verification boundary in `target/issue-101/handoff.json`.

## Limitations

Row 1005 needs independent confirmation of its exact bare-spelling/POS blocker;
no annotation approval applies to it. All candidate quality, source eligibility
and combined redistribution obligations await separate independent review.
The source-capture record retains unavailable optional contributor-registry and
statute retrieval status; complete linked history supplies contributor credit,
and the statute basis is identified as reused from prior admitted metadata.

Shared canonical content/conversion/readme, application integration, final
packaged notices and native lookup/trusted save/reopen verification belong to
integration. No native, frontend, release or learner-acceptance result is claimed
by this author package.
