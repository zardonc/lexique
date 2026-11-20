# Issue #110 independently reviewed content

**50 exact frozen groups / 50 selected senses / 200 independently accepted
fields**, with **0 unresolved selected fields**. Two Chinese translations have
accepted replacements. Headwords, POS subclasses, frequencies and order retain
rows 1451–1500 exactly. The global denominator remains **4,087 groups**.

The [ticket](https://github.com/zardonc/lexique/issues/110) and
[parent](https://github.com/zardonc/lexique/issues/78) define the scope.
[Independent review](independent-review.md) records individual semantic,
construction, IPA and spelling judgments. [Selected rights](selected-rights.md)
records the actual import, bibliography and contributor boundaries.

## Review materials

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact groups and independently accepted fields |
| [review-candidates.json](review-candidates.json) | Unchanged author proposals at a34b1ae |
| [field-review.json](field-review.json) | Separate independent review, proposed/accepted texts, per-field reasons, context and bindings |
| [sources.json](sources.json) | 51 fixed French sources, history credit, licences and modifications |
| [source-context.json](source-context.json) | Individual reviewed source boundaries, actual import/reference markers, form and construction evidence, complete contributor registries |
| [source-excerpts.json](source-excerpts.json) | Unchanged 200 exact raw field bindings |
| [source-captures.json](source-captures.json) | Fixed first-party API/wikitext captures and full contributor pagination |
| [rights.json](rights.json) | Independent selected-boundary judgments and redistribution obligations |
| [reuse.json](reuse.json) | No reused exact group; prior canonical content remains unchanged |
| [admission.json](admission.json) | Independent counts and pending parent canonical/conversion/native gates |

The review retains all four languages/IPA fields for the same selected modern
use. The coat-rack translation at row 1489 and symbol-system translation at row
1495 are corrected with their proposals and reasons retained. The constructions
for `supplier`, `remplacer` and `accorder` are explicit. Exact spelling/flexion
records retain `alias_admission=not_admitted`, with no runtime aliases added.

## Verification

The [focused resource check](../../../scripts/test_core_batch_110.py) uses the
parent's approved resource boundary. It checks frozen membership and frequencies,
proposal preservation, accepted outcomes, independent reasons and identities,
source/version/span bindings, attribution and honest pending parent gates.
The independent raw-capture check also reads all fixed captures and contributor
pages in the author checkout.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_110.py -v
pnpm test:docs
git diff --cached --check
```

The focused Python check passes **3 tests**. The Chinese regression first failed
at both affected rows. Documentation format/repository links and staged diff
checks pass. The [independent review](independent-review.md) identifies complete
logs, capture verification, tested base and platform.

Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-110`, branch
`codex/d2-review-110`, pinned base
`ba97a83619d0f952e5df51d1fd4549dcc348118b`, content author `a34b1ae`.
Independent logs remain under `target/issue-110-review/` in that checkout.
Fixed author captures/logs remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-110/target/issue-110/`.

## Pending parent gates

Independent selected acceptance is **50 groups / 200 fields**; newly admitted
canonical content from this review remains **0 groups / 0 fields**, with all
**200 canonical fields pending**. The baseline stays at **1,436 accepted groups /
5,744 fields** and **2,651 groups without accepted content**, including the
approved annotation rows.

The parent owns final canonical admission and binding-only normalization,
reproducible conversion, fixed-denominator coverage/size reporting, actual
combined notices packaging and native lookup/save/reopen. Native, UI, offline
and release/platform checks were not run here. The direct live L123-3 capture
was unavailable; the fixed policy and established original-edition basis are
identified in the selected-rights report. Learner data and canonical runtime
aliases are unchanged.
