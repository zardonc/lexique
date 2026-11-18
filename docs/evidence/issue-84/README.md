# D2 content batch 004 (#84): sourced draft

[Issue #84](https://github.com/zardonc/lexique/issues/84) now has candidates for
all **50 exact frozen groups in rows 151–200**, with **50 selected senses and
200 nonempty fields**. The [source research](source-research.md) records the
selected French definitions, IPA and newly authored English/Chinese explanations.
The [review candidates](quality-review.md) await a fresh reviewer.

| Batch state | Count |
| --- | ---: |
| Frozen groups / drafted groups | 50 / 50 |
| Draft senses / fields | 50 / 200 |
| Fixed Wiktionnaire source articles | 43 |
| Fresh API captures / unchanged copied captures | 11 / 32 |
| Independently accepted batch groups / fields | 0 / 0 |
| Pending field / usage reviews | 200 / 50 |
| Reused batch groups / new spelling aliases | 0 / 0 |
| Permanent target groups / headwords | 4,087 / 3,773 |

The draft appends only this batch's curation, sources, excerpts, source context,
field reviews and usage reviews to the existing monolithic JSON materials.
All 154 earlier groups / 616 fields, earlier sources, reviews, contexts, spelling
relations and unresolved records compare equal to baseline `42c7205`. The pilot
groups at rows 360, 422, 423, 483 and 1911 remain unchanged. No learner data paths
were read or written.

Shared content version and coverage counters remain unchanged for coordinator
integration. The authorized one-line local `ROWS` extension reaches row 200;
the converter's validation and admission behavior is unchanged. The four changed
resource materials have their updated local checksum entries. No prepared
resource is admitted from these pending-review inputs.

## Verification

Checks ran on 2026-11-18 UTC, Windows x64, in
`C:\Users\inthe\IdeaProjects\lexique-issue-84`, branch `codex/d2-content-84`,
with PowerShell 7, uv-resolved Python 3.14.10 and pnpm 11.25.0. Logs and raw
research evidence remain under `target/issue-84/`.

| Check | Actual result and retained log |
| --- | --- |
| Frozen-membership TDD cycle | Failed before the 50-group draft; passed after it. `tdd-red-membership.log`, `tdd-green-membership.log` |
| Admission TDD cycle | Failed before the authorized row extension; passed after it, verifying refusal at the pending-review boundary. `tdd-red-admission.log`, `tdd-green-admission.log` |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_84.py -v` | 3 passed; exact membership/four fields, selected difficult constructions and rejection of a pending source. `focused-tests.log` |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-84/validate_and_document.py` | Passed; 200 exact raw line bindings, 43 revision/capture bindings and prior-record preservation. `validation.log`, `source-validation.json`, `preservation.json` |
| `uv run --python 3.14.10 python -B -X utf8 scripts/core_pilot.py prepare target/issue-84/prepared` | Refused with `Unreviewed field source or incompatible licence`, exit 1; expected pending-review gate. `conversion.log` |
| `pnpm test:docs` | Passed; 181 tracked documents / 1,440 repository links, with 91 local evidence links counted separately. `docs-check.log` |
| `git diff --cached --check` | Passed; no whitespace diagnostics. `diff-check.log` |
| Native dictionary lookup, trusted save and normal restart | Pending coordinator validation after independent admission; no native acceptance claim |

The source researcher does not independently accept their own wording.
The existing resource's admitted-group totals remain 154 / 4,087, with 3,933
unaccepted groups, until independently audited integration changes them. The
50 drafted groups do not reduce those missing-content totals.

## Limits and coordinator handoff

The fresh independent reviewer must assess all 200 fields and all 50 usages,
including actual selected-span rights and the specific taxonomy correspondences
for `autre/PRO:ind` (155), `est-ce que/ADV` (163) and `seul/NOM` (181).
No frozen group lacks a sourced candidate, and no scope exception is taken.
The coordinator owns final acceptance, combined row/version/counter changes,
produced-resource coverage/size/reproduction, native application checks, learner
acceptance, final code/quality review and release evidence. Frontend, Rust,
packaging and broader native/offline checks were not run for this content draft.
