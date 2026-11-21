# Issue #122 independently accepted content

All **50 exact frozen groups at rows 2051–2100 / 200 FR/IPA/EN/ZH fields**
pass independent semantic, use, spelling and selected-rights review. **193 fields
are unchanged; seven fields and one use have exact accepted replacements;
zero fields are unresolved or missing.** There is zero reuse, zero annotation
exception and no new runtime alias. The global denominator remains **4,087 groups**.

The [ticket](https://github.com/zardonc/lexique/issues/122) and
[parent specification](https://github.com/zardonc/lexique/issues/78) were read
in full from `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.
Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-122`, branch
`codex/d2-review-122`. Author commit: `435b0000b431bf18fe21a05752523242a0dc7750`;
author final integration head: `5d12853`. Review start: `7937180a4ab592cb4d0e8f0e7af4cffd80cb74e8`.

| Record | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen identities, frequencies, accepted uses and fields |
| [field-review.json](field-review.json) | Every original proposal, accepted field/use and independent reason |
| [accepted-replacements.json](accepted-replacements.json) | Seven exact field replacements and one use replacement |
| [review-candidates.json](review-candidates.json) | Unchanged authored proposal objects |
| [author-admission.json](author-admission.json) | Unchanged authored admission/count snapshot |
| [admission.json](admission.json) | Live independent acceptance, missing/pending counts and parent gates |
| [rights.json](rights.json) | Every selected field's original rights proposal and accepted boundary reason |
| [rights-review.md](rights-review.md) | Primary legal authorities and actual import/original-work decisions |
| [independent-review.md](independent-review.md) | Complete review scope, corrections and verification |
| [sources.json](sources.json), [source-context.json](source-context.json) | Fixed versions, contributor registries, imports and accepted source bindings |
| [source-excerpts.json](source-excerpts.json) | Exact field and supporting raw spans |
| [source-captures.json](source-captures.json) | Author raw sources, API captures, pagination and reproduction pointers |
| [NOTICES.md](NOTICES.md), [COPYING](COPYING) | Credits, modification notices and full CC BY-SA 4.0 legal text |

The [focused check](../../../scripts/test_core_batch_122.py) passes **six tests**:
exact membership, accepted fields/uses/rights and proposal preservation, fixed
raw source/registry verification, and the three corrected meaning/grammar cases.
It compares any already assembled row to the reviewed shard; overlap is not
treated as missing or counted as another group. Detached checkouts validate
tracked source bindings; the owning reviewer check also verifies actual raw bytes.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_122.py -v
```

Raw author evidence remains at
`C:/Users/inthe/IdeaProjects/lexique-batch-122/target/issue-122/`.
Independent scripts, source comparisons, legal retrieval and RED/GREEN logs
remain at `C:/Users/inthe/IdeaProjects/lexique-review-122/target/issue-122-review/`.
`final-verification.json` records the tested commit, merged integration tip,
Windows/Python environment, commands and outcomes. Author preparation scripts
reproduce the authored proposal; reviewed content is the accepted shard above.

## Limitations

Canonical resource assembly, actual combined notices and package compliance,
native lookup/save/reopen, human learner acceptance and remaining offline,
platform and formal release gates remain parent-owned and unrun in this checkout.
Independent acceptance is complete; canonical installation is still pending.
