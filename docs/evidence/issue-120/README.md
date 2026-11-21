# D2 issue #120: accepted content and native admission

Frozen rows **1951–2000** contain **50 accepted groups / 200 fields**, with zero missing or pending batch fields and no reuse, new annotation exceptions or runtime aliases. The fields are bundled in `core-129-v2`; the fixed denominator remains 4,087 groups / 3,773 headwords.

## Content and rights

The [exact groups and accepted text](groups.json), [independent field/use decisions](field-review.json), [selected rights](rights.json), [source context](source-context.json), [fixed-source captures](source-captures.json) and [selected excerpts](source-excerpts.json) retain the precise POS/use, source versions, contributors and import/original-work boundaries. Original proposals remain in the preserved snapshots. The [independent-review handoff](README-independent-review.md) records the earlier review stage; its pending parent gates are historical.

The actual combined resource includes [notices](../../../resources/core-pilot-79/NOTICES.md) and [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING). The resource now contains 2,435 accepted groups / 9,740 fields; all 1,935 prior groups and compiled payloads, 15 spelling relationships / 11 aliases, and 15 zero-field annotations / 60 missing fields are preserved.

## Native verification

Windows 11 x64 with the matching rebuilt `core-129-v2` executable passed exact lookup for all 50 groups / 200 fields and trusted save/restart/reopen for all 46 headwords. The ordinary sword meanings retain their accepted scope and provenance. The closed `core-119-v1` upgrade preserved learning state, frozen digests, notes, examples, FSRS and selected audio. Both native runs retained their hard 60-second limit, zero external WebView requests and zero remaining owned processes.

[Batch results](../../../target/issue-120/native-core-129-v2-final/results.json), [run identity](../../../target/issue-120/native-core-129-v2-final/run-metadata.json), [cleanup](../../../target/issue-120/native-core-129-v2-final/cleanup.json), [upgrade](../../../target/issue-129/native-core-129-v2-final-upgrade/results.json), [other-record preservation](../../../target/fix-193/preservation.log), [build identity](../../../target/fix-193/build.json), [portable package](../../../target/fix-193/checked-package.log) and [combined admission metadata](admission.json) retain execution evidence. The [combined verification](../issues-120-129/README.md#verification) separates current v2 results from the baseline evidence.

Whole-machine offline denial, other-platform native execution, learner acceptance and formal release readiness remain unverified. CEFR labels remain unknown; #78 remains open.
