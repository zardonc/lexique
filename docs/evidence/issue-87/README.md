# D2 content batch #87

The batch retains all **50 frozen identities in rows 0301–0350**. It supplies
**49 selected research candidates / 196 nonempty fields** and records
**four unresolved fields for `dont/PRO:int`, row 305**. All candidates await
independent field, usage and selected-span rights review. **Zero new groups or
fields are admitted.** No scope exception is requested or presumed in the data.

The [source research](source-research.md) records the fixed revisions,
construction boundaries and source obligations. The
[review candidates](review-candidates.json) preserve the exact proposed wording,
POS, restrictions, source spans and pending records. The
[quality handoff](quality-review.md) identifies the review questions and the
unresolved interrogative group.

| Batch accounting | Count |
| --- | ---: |
| Frozen targets / new identities | 50 / 50 |
| Reused pilot or earlier batch groups | 0 |
| Selected candidate senses / fields | 49 / 196 |
| Unresolved selected-use groups / fields | 1 / 4 |
| Independently accepted batch groups / fields | 0 / 0 |
| Fixed French captures, including supporting constructions/spelling | 44 |
| Direct API captures / recovered fixed research captures | 24 / 20 |
| Fixed target denominator | 4,087 groups / 3,773 headwords |

`curated-inputs.json`, `field-review.json`, `selected-source-excerpts.json` and
`selected-source-context.json` retain the monolithic format and add only this
batch's records. Earlier records, including pilot rows 360, 422 and 423, remain
unchanged. The core version, global counters, prior evidence and runtime files
are unchanged. The local checksum manifest is refreshed. The only converter
change is the authorized one-line addition of rows 301–350 to `ROWS`.

## Verification

Commands use PowerShell 7 and `uv run --python 3.14 python -B -X utf8`;
the resolved interpreter is Python 3.14.10. Logs are retained under
`target/issue-87/` in this worktree.

| Check | Result and retained evidence |
| --- | --- |
| Frozen membership regression | Red before adding the fifty groups; green after adding them. `tdd-01-red.log`, `tdd-01-green.log` |
| Selected `écouter` four-field regression | Red before the selected use existed; green after its source-bound fields were added. `tdd-02-red.log`, `tdd-02-green.log` |
| Candidate coverage and explicit interrogative gap | Red before the remaining candidates/gap existed; green at 49 senses / 196 fields and four incomplete row-305 fields. `tdd-03-red.log`, `tdd-03-green.log` |
| Focused batch tests | Four tests passed, including refusal to publish pending source material. `focused-tests.log` |
| Post-integration focused tests | Seven tests passed across #87 and #89; source/record preservation and the expected pending-review conversion failure remained unchanged. `post-merge-focused-tests.log`, `post-merge-validation.log`, `post-merge-conversion.log` |
| Real converter | Expected failure: `Unreviewed field source or incompatible licence`; no batch SQLite produced. `conversion-pending.log` |
| Fixed-span, candidate and predecessor preservation checks | Passed. `source-validation.json`, `preservation.json` |
| Diff and local evidence links | Passed. `diff-check.log`, `local-links.json` |
| Per-ticket native lookup/save/restart | Pending; the source candidates cannot produce an admitted resource. Coordinator owns the combined-resource native oracle. |

No test-only approval or fabricated independent acceptance is used. The focused
test file is [test_core_batch_87.py](../../../scripts/test_core_batch_87.py).
Source fetching and assembly helpers remain under ignored `target/issue-87/`.

## Integration requirements and limits

The converter's fixed reviewed-row, review-count, unresolved-field and spelling
alias assumptions need the coordinator's combined update. This batch adds the
`oeil`/`œil` relation and four unresolved fields for row 305, alongside the
unchanged row-87 annotation records. The exact `dont/PRO:int` selected-use gap
requires an evidenced eligible use or a specific owner decision; relative
pronoun content cannot satisfy it. Row 305 remains in the frozen denominator.

Independent audit, learner acceptance, conversion reproduction with admitted
inputs, artifact coverage/size and native lookup/save/offline/restart checks
remain pending. The shared integration tip is merged before delivery; the
coordinator owns combined version/counters, runtime validation, final coverage,
PR and tracker operations.

The branch incorporates integration tip `d55b6bd`, including native batch test
infrastructure `b8931d4`, trusted-save reuse `5f39b97` and #89's pending content.
Merge resolution retained complete records from both branches byte-for-byte,
combined their `ROWS` additions and refreshed the local checksum manifest.
