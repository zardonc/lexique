# Issue #110 independent selected-content review

**50 exact groups / 50 selected senses / 200 fields accepted independently.**
There are **0 unresolved selected fields**, **0 new annotation exceptions** and
**0 admitted query aliases**. Two Chinese translations use accepted replacements.
All 200 canonical batch fields remain pending parent admission and conversion.

## Review identity and inputs

The fresh semantic and selected-rights reviewer worked in
`C:/Users/inthe/IdeaProjects/lexique-review-110`, branch `codex/d2-review-110`,
pinned to `ba97a83619d0f952e5df51d1fd4549dcc348118b`. The reviewed content author
commit is `a34b1ae`; this reviewer did not author that commit or its proposals.
Author checks supply capture evidence, rather than independent judgments.

The [ticket](https://github.com/zardonc/lexique/issues/110) and
[parent](https://github.com/zardonc/lexique/issues/78) were read from their fixed
JSON captures in the parent specification checkout. The independent record
follows [#109](../issue-109/field-review.json) and the
[converter](../../../scripts/core_pilot.py) requirements.

Every group and all four fields were read in ten projections of five groups.
The fixed source definitions, French/POS/form frames, IPA, neighboring sense
boundaries, constructions and article import/reference markers were inspected.
The [field review](field-review.json) contains individual accepted texts,
language-specific reasons, exact source bindings, context locations, usage and
spelling judgments, and independent reviewer identity. The
[source context](source-context.json) records 51 individual source boundaries.

## Accepted wording and constructions

| Frozen row | Field | Accepted outcome | Reason |
| ---: | --- | --- | --- |
| 1489 `accrocher/VER` | Chinese | `把某物挂或固定在钩子、钉子或衣帽架上。` | `portemanteau` denotes a coat rack; `衣架` suggested the distinct clothes hanger |
| 1495 `code/NOM` | Chinese | `符号系统；编码：用于交换信息的一套符号体系。` | The selected symbol system includes maritime signals; leading with `代码` suggested programming |

The original [review candidates](review-candidates.json), `proposed_text` and
`proposed_usage` remain intact. Selected usage now explicitly records
`supplier quelqu’un de + infinitive`, `remplacer X par Y` and
`accorder quelque chose à quelqu’un`. The field review retains the relevant
fixed construction evidence and reasons.

`gare/ONO` retains the frozen subclass and the familiar warning interjection.
`celle-là` and `certaines` retain their exact feminine singular/plural bindings.
`vu/ADJ` retains the explicit `/vy/` base in its adjective agreement template.
`entendu/ADJ` selects the knowing/complicit use supported by `remarque entendue`
and `sourire entendu`. All spelling records retain
`alias_admission=not_admitted`; the canonical runtime alias set is untouched.

## Evidence and checks

Execution used Windows x64, PowerShell 7, uv Python **3.14.10** and pnpm
**11.25.0**. Fixed author captures/logs remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-110/target/issue-110/`.
Independent full logs are under
`C:/Users/inthe/IdeaProjects/lexique-review-110/target/issue-110-review/`.

The capture check compared all 51 raw captures with their fixed API revisions,
all 200 field bindings, spelling/form evidence and complete contributor pages.
The eight contributor responses reproduce **3,315 article-specific name
entries** and terminate both pagination streams. These are article registries,
rather than proof of a selected sentence’s author.

| Check | Result | Retained evidence |
| --- | --- | --- |
| Scoped Chinese regression before correction | Expected RED, two failing row assertions | [red.log](../../../target/issue-110-review/red.log) |
| Fixed captures and source/form bindings | Passed | [capture-check.json](../../../target/issue-110-review/capture-check.json) |
| Pinned focused Python check | Passed, 3 tests | [green.log](../../../target/issue-110-review/green.log) |
| Documentation format and repository links | Passed | [docs.log](../../../target/issue-110-review/docs.log) |
| Staged diff whitespace | Passed | [diff-check.log](../../../target/issue-110-review/diff-check.log) |

## Pending parent gates

The [selected-rights report](selected-rights.md) and [admission record](admission.json)
separate independent field acceptance from canonical admission. The parent owns
binding-only normalization, final canonical inclusion, reproducible conversion,
fixed-denominator coverage/size reports, combined packaged notices and actual
native lookup/save/reopen. Native storage, UI, offline and release checks were
not run here. Global baseline content, the 4,087-group denominator and existing
learner data are unchanged. Direct live L123-3 retrieval was unavailable; the
fixed policy and established original-edition evidence are identified in the
rights report.
