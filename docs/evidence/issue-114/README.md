# Issue #114 independently reviewed content

**50 exact groups / 200 FR/IPA/EN/ZH fields accepted by separate independent review; zero unresolved fields.** Frozen rows 1651–1700, headwords, POS subclasses, both frequency strings and order remain unchanged. The denominator is **4,087 groups**. No new owner annotation or runtime alias is admitted.

The [ticket](https://github.com/zardonc/lexique/issues/114) and [parent](https://github.com/zardonc/lexique/issues/78) define this batch and the approved resource/admission seam. Field acceptance remains separate from canonical resource admission: **zero newly admitted canonical fields** are claimed here.

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Reviewed exact groups, selected uses and four fields |
| [review-candidates.json](review-candidates.json) | Unchanged complete author drafts |
| [field-review.json](field-review.json) | Separate per-field, use, rights and spelling decisions and reasons |
| [sources.json](sources.json) | Fixed versions, article/history attribution, licences and modifications |
| [source-excerpts.json](source-excerpts.json) | 200 exact field bindings and 207 supporting bindings |
| [source-context.json](source-context.json) | Full contributor registries, grammar and imported-work boundaries |
| [source-captures.json](source-captures.json) | Raw paths, pagination and independent reading scope |
| [rights.json](rights.json) | Selected-span rights decisions, original-work credits and obligations |
| [rights-review.md](rights-review.md) | Applicable rights and actual-package boundary |
| [reuse.json](reuse.json) | Zero reuse against the author's base admitted input |
| [admission.json](admission.json) | Accepted counts and parent-owned admission gates |
| [source-research.md](source-research.md) | Source and selected-use findings |
| [independent-review.md](independent-review.md) | Reviewer identity, reviewed SHA, decisions and verification |

Accepted fields total **7,430 Unicode characters**; the longest is **117**. All 50 selected senses are modern everyday uses, with explicit construction and register restrictions. CEFR remains unknown.

The [focused check](../../../scripts/test_core_batch_114.py) compares exact membership, accepted texts and source spans with separate review decisions, preserves original proposals and checks authority/public-order restrictions. It uses the supplied author raw captures when available.

```powershell
uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_114.py -v
git diff 15cf830 --check
```

Reviewer execution evidence remains at `C:/Users/inthe/IdeaProjects/lexique-review-114/target/issue-114-review/`; full author captures remain at `C:/Users/inthe/IdeaProjects/lexique-batch-114/target/issue-114/`. The final verification record identifies the tested reviewer SHA and platform.

## Limitations

Canonical conversion, actual combined packaged notices, native lookup/save/reopen, learner acceptance and release checks remain parent-owned and pending. This review does not close the ticket or establish release readiness.
