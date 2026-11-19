# D2 content batch #85: sourced candidates

Frozen rows **201–250** retain all **50 exact headword/POS identities**.
There are **48 sourced candidate senses / 192 proposed fields**, all pending
independent quality and selected-text rights review. **No new field is admitted.**
Rows **218 `avant/ADJ`** and **238 `déjà/ONO`** retain zero selected fields and
explicit source gaps. No scope exception has been applied.

The [source notes](source-research.md) and [review handoff](quality-review.md)
identify the selected uses and the remaining work. Exact texts, source revisions,
line spans, contributor-history links, licences and modification notices are in
[curated-inputs.json](../../../resources/core-pilot-79/curated-inputs.json),
[field-review.json](../../../resources/core-pilot-79/field-review.json),
[selected-source-excerpts.json](../../../resources/core-pilot-79/selected-source-excerpts.json)
and [selected-source-context.json](../../../resources/core-pilot-79/selected-source-context.json).
The [candidate index](review-candidates.md) includes every frozen group.

## Preservation and integration

The branch starts from `42c7205` on `codex/d2-content-84-89` and includes the
coordinator's native-test infrastructure commit `b8931d4`. Changes append only
#85 groups, sources, fields, usages, context and one spelling relationship.
Earlier records, including pilot rows **360, 422 and 423**, retain their original
content. This batch reuses **zero admitted groups**. The permanent denominator
remains **4,087 groups / 3,773 headwords**; the prior **154 admitted groups /
616 fields** are unchanged.

The only converter edit adds `range(201, 251)` to `ROWS`. The checksum manifest
updates the four changed input JSON files. Shared content version, coverage
counters, resource README/notices, prior evidence, runtime and native harness
are unchanged by this batch. The coordinator owns the combined projection,
remaining source decisions, independent audit, final counters and admission.

## Checks

Commands use uv's Python 3.14 with `-B -X utf8`; logs are retained under
`target/issue-85/logs/` in this worktree.

| Check | Result |
| --- | --- |
| Focused `unittest discover -s scripts -p test_core_batch_85.py -v` | Passed, 3 tests: frozen membership; exact candidate/excerpt bindings and pending reviews; rejection before resource production |
| TDD checks | Each of the three slices has retained failing and passing logs: `tdd-membership-*`, `tdd-fields-*`, `tdd-admission-*` |
| `scripts/core_pilot.py prepare target/issue-85/conversion` | Expected failure: `Unreviewed field source or incompatible licence`; no batch SQLite or admission report produced |
| Source-capture and preservation self-check | Passed; fixed raw excerpts and IPA variants match retained captures; earlier JSON records and metadata retained |
| Diff and local documentation links | Passed; scoped documentation check retained in `documentation.log` |

The pending batch blocks production conversion until the coordinator resolves
the source gaps and independent review. The existing admission barrier is kept.

## Limitations

Eight field slots lack selected source content; the other 192 proposed fields
await independent review. Source extraction is candidate evidence. Imported
DAF8 notices require review at the selected-text boundary; no whole dictionary
or third-party quotation is admitted. `avant/ADJ` has a modern grammatical lead
but unresolved redistributable wording/rights; `déjà/ONO` has no matching sense.
Native lookup, trusted save/reopen, produced-resource measurements, combined
coverage and final packaged obligations are pending central integration.
Application lint/build, Rust, browser, native and release checks were not run
for this content draft. Learner acceptance and the wider D2/release gates remain
open.
