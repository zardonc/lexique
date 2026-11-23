# #153 author handoff: frozen rows 3601–3650

**50 exact groups / 200 proposed fields; zero independently admitted fields.**
The frozen headword/POS pairs, order and both frequency strings are preserved.
Each record proposes one selected use with French, IPA, English and Chinese
fields. CEFR remains unknown. No alias, annotation or specialist exception is
admitted. Forty-nine uses are proposed as everyday; `by/NOM` remains unresolved.

[groups.json](groups.json) contains the proposals;
[review-candidates.md](review-candidates.md) presents their wording and scope.
[admission.json](admission.json), [field-review.json](field-review.json) and
[rights.json](rights.json) retain pending independent quality, rights, usage
and spelling decisions, with reviewer=null. Author checks do not establish
independent acceptance. Original proposals are preserved under `authored/`.

## Source and construction evidence

Fifty-five fixed first-party French Wiktionnaire revisions and nine complete
paginated contributor responses support the source records. The shared
[capture verifier](../../../scripts/core_batch_test_helpers.py) compares actual
API/raw bytes and revisions. Every field/support span is checked at its exact
line and column, with French language and selected POS/use bindings.
[source-context.json](source-context.json),
[source-excerpts.json](source-excerpts.json) and
[source-captures.json](source-captures.json) preserve those pointers. Named
registries, anonymous counts and full linked histories provide attribution;
they do not establish selected-sentence authorship.

`vôtres/PRO:pos` selects the plural possessive component of `les vôtres`, for
masculine/feminine referents and plural or polite singular `vous`. Recorded
article/inflection evidence and the isolated `/votʁ/` component are retained.
The nominal family/companion and obsolete adjective uses are excluded.

`volé/ADJ` selects stolen property, with fixed `volé` participle morphology/IPA
and the separate transitive theft meaning in `voler`. Captured Le Robert
material confirms the property adjective; its prose supplies no field.
CNRTL's rare adjective for a theft victim is a different use and is excluded.
This is an author source/construction proposal awaiting independent review.

## Unresolved requirement

Frozen row 3612 `by/NOM` has only the identified French pond-drainage meaning
and `/bi/` pronunciation. The fixed article and external CNRTL/Littré reading
do not establish modern everyday use. The article's `R:Rivarol` reference also
needs actual selected-original-expression comparison. Its four proposals are
retained, with this blocker applying to all four fields; no exception is
approved. English `by`, `bief`, `bye` and other languages are not substitutions.

[selected-rights.md](selected-rights.md), [NOTICES.md](NOTICES.md) and
[COPYING](COPYING) retain the CC BY-SA 4.0 contribution/adaptation route,
separate DAF8 original-work credit and its established 2006-01-01 expiry basis.
New imported-expression and final combined-package review remain pending.

## Verification and integration boundary

The focused resource check failed before authoring because all 50 records
were absent. The rights check failed before the rights package existed.
The completed author boundary suite checks exact frozen membership, four
fields, literal source spans, attribution/pagination, construction restrictions
and rejection of pending author review by the existing admission function.
A corrupted raw capture is rejected by the shared verifier.

Commands and actual results are recorded in
[author-verification.json](author-verification.json). Full logs, helper origins,
raw captures, fixed policy/legal captures and replay helpers are preserved at:

`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-153/`

The baseline remains `core-149-v1`, 3,431 admitted groups / 13,724 fields,
19 approved annotations / 76 missing annotation fields, and the fixed
4,087-group / 3,773-headword denominator. This author handoff adds no admitted
coverage. Global assembly, conversion/notices, native lookup/save/reopen,
upgrade preservation and release claims belong to parent integration after
independent decisions. Those checks and learner acceptance have not run for
this batch. No frontend, Rust, persistence or learner data is changed here.
