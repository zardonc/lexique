# D2 content batch 006 (#86): sourced draft

All **50 frozen groups in rows 251–300** have one selected-use proposal and
**200 nonempty draft fields**: French definition, applicable IPA, English and
Chinese explanation. **No new group or field is independently accepted.**
There are 50 new groups and zero reused groups. Membership, headword spelling,
Lexique subclasses and the global denominator of 4,087 groups remain unchanged.

The [source research](source-research.md), [fixed source table](source-table.md)
and [capture registry](source-registry.json) identify the selected versions and
spans. [Quality review](quality-review.md) identifies the exact candidates and
two unresolved source-binding questions. The machine-readable
[validation record](validation.json) records the scoped counts and preservation
check.

The batch adds records to the existing
[curation](../../../resources/core-pilot-79/curated-inputs.json),
[field review](../../../resources/core-pilot-79/field-review.json),
[selected excerpts](../../../resources/core-pilot-79/selected-source-excerpts.json)
and [source context](../../../resources/core-pilot-79/selected-source-context.json).
All batch fields and usages remain pending; review records have proposed text,
null accepted text and no independent reviewer. Source extraction and mechanical
validation establish a reviewable draft, not resource admission.

## Preservation and integration

The branch started clean at `42c720556fd1195cca39ba704761877d1c0df48a`, which
was the integration branch base. Every pre-existing curation, field/usage review,
source context and selected-excerpt record retains its exact original bytes.
Original pilot rows 360, 422 and 423 remain unchanged. No learner database,
saved reference, digest, note, example, card, progress or audio file is touched.

The only shared converter delta is the authorized addition of rows 251–300 to
`ROWS`. The checksum manifest updates only the four changed JSON materials.
Shared content version, global counters, resource README, notices, prior evidence
and runtime remain unchanged. Their combined updates belong to the coordinator.
The existing source notices and CC BY-SA 4.0 legal text are reused.

Integration tip `d55b6bd` is merged, including #89's pending records. The JSON
resolution is an additive union and retains both branches' record bytes.
Rows 401–450 remain owned by #89; reused pilot rows 422 and 423 appear once.
The merged local ROWS contains both batches, while the coordinator owns its
final combined extension. All three original pilot records (360/422/423) also
passed an exact-byte comparison with the pinned base after this merge.

The coordinator also owns shared `ADJ:int` and `PRO:pos` validators and localized
labels, independent source/quality audit, final admitted conversion, native
lookup/save/restart, coverage, packaging, PR and review. The test infrastructure
from `b8931d4` and its integration follow-up is included by merge before delivery.

## Verification

Focused commands use uv's Python 3.14 with UTF-8 explicitly enabled:

```powershell
uv run --python 3.14 python -X utf8 scripts/test_core_batch_86.py
uv run --python 3.14 python -X utf8 scripts/core_pilot.py prepare target/issue-86-prepared
git diff --check
pwsh -NoProfile -File scripts/check-docs.ps1
```

The focused checks cover frozen membership, plaire's four sourced fields and
rejection of pending content at the produced-resource boundary. All 200 source
spans and revision/URL bindings were checked against the retained first-party
captures. Preservation checks compare with the pinned base, including exact
record bytes. Full logs remain under ignored `target/issue-86-*.log`.

| Check | Actual result |
| --- | --- |
| Focused Python tests | Passed: 3 tests |
| Fixed capture/span and preservation check | Passed: 200 fields and all pre-existing records |
| Direct conversion | Failed as expected: `Unreviewed field source or incompatible licence`; no admitted output |
| Diff check | Passed |
| Repository documentation check | Passed: 185 documents, 1,457 local links, 0 problems; 91 local evidence links skipped |
| Combined material checksums | Passed: all 11 existing manifest entries |
| Integration preservation | Passed: additive #86/#89 union; both branches' records and original pilot bytes retained |
| Native application and combined admission | Pending coordinator integration and independent review |

## Limitations

- All 200 fields require independent quality and selected-span rights review.
- Row 270, `ouais/ADV`, lacks an established exact modern adverb source; the
  captured sources classify the selected affirmative response as an interjection.
- Row 274, `vivre/NOM`, proposes an explicitly evidenced modern plural `vivres`;
  its singular food use is dated. Frozen-group eligibility remains unresolved.
- The converter must reject this pending draft. No admitted artifact, native
  batch lookup/save/restart, learner acceptance or release claim is made.
- Shared runtime validation and final combined metadata/checks are pending with
  the coordinator.
