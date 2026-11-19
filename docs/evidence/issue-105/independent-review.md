# Issue #105 separate-review handoff

**Pending.** The author prepared 46 selected senses / 184 fields and four
fixed-source group blockers. No independent reviewer has accepted these fields,
uses, spelling relations or original-text rights. The [field records](field-review.json)
have proposed text, exact source bindings, pending quality/rights statuses and
no reviewer identity. [Admission](admission.json) records zero accepted fields.

Review each proposed field against its frozen headword/POS, exact API/wikitext
capture, selected French sense, applicable pronunciation and constructions.
Evaluate the EN/ZH explanatory adaptations and the original import boundaries
independently. Resolve the complete seven-file package before admission;
author-stage source checks cannot replace that decision.

The owning checkout is
`C:/Users/inthe/IdeaProjects/lexique-batch-105`, branch `codex/d2-content-105`.
The authored diff base is `aae5273af846e57668a8d267fa6a6566c4fd6e91`;
the commit SHA is supplied with the handoff. Review the committed issue-105
evidence and `scripts/test_core_batch_105.py`. The exact authored patch and
test-only patch are retained at `target/issue-105/authored.patch` and
`target/issue-105/test.patch`. Logs and snapshots are at
`C:/Users/inthe/IdeaProjects/lexique-batch-105/target/issue-105/` on Windows x64.

The four blockers are `quiconque/PRO:int` (1208), `qu/CON` (1218),
`qu/PRO:int` (1219) and `qu/PRO:rel` (1220). They are unresolved ordinary groups,
with no approved exception. The author leaves shared resource conversion,
packaging and real native lookup/save/reopen validation to integration.
