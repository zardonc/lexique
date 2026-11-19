# Issue #98 content handoff

The candidate artifact contains **48 selected uses / 192 proposed FR/IPA/EN/ZH
fields** and **two owner-approved zero-field annotation exceptions** for rows
859 `l/ART:def` and 860 `l/PRO:per`. All 50 frozen identities and frequency
strings are exact. There are **zero independently admitted new fields**;
independent quality and rights acceptance remains pending.

[Groups](groups.json), [sources](sources.json), [selected contexts](source-context.json),
[field excerpts](source-excerpts.json) and [review drafts](field-review-draft.json)
are keyed additions for parent integration. [Annotations](annotations.json) retain
the two exact bare forms with zero senses, zero admitted fields and four missing
fields each. [Owner approval](owner-approval.json) records the explicit reply,
effect and 2026-11-18 America/Vancouver date. [Bare-form evidence](literal-l.md)
and [selected source/TSV records](literal-l-evidence.json) support the disposition.
There are no other [unresolved groups](unresolved.json) and no [reused groups](reuse.json).

The [focused membership check](../../../scripts/test_core_batch_98.py) failed
before the additions, then passed against the candidate artifact. Its amended
assertions check all 50 identities and frequencies, 48 sourced four-field senses
and the actual owner approval for the two annotations. A passing candidate check
does not establish an admitted SQLite artifact or independent semantic quality.
The fixed denominator remains 4,087 groups, including the two annotations.

All fields have a real fixed revision and one-based source span. The 88 selected
field sources were fetched by explicit revision ID; the 93 full source captures
also include the bare and apostrophized forms used for the annotation evidence.
[Capture locations](source-captures.json) identify actual API responses and exact
wikitext under `C:/Users/inthe/IdeaProjects/lexique-content-98/target/issue-98/sources/`.
Browser fetch of the actual first-party API response preserves Wiki markup.
Complete captures remain ignored; only selected spans and notices are tracked.
Returned revision dates and retrieval dates are separate; no October cutoff is claimed.

Definitions and EN/ZH explanations adapt the selected English-language French
entry sense. French IPA comes from the exact French headword/POS span; one stated
variant is selected without audio. For `situation`, transport `accident` and
sound `silence`, a broad French-entry cross-reference is narrowed using an exact
English gloss in the same revision, retained as supporting evidence. Independent
review must confirm those bindings. `remercier` retains its explicit de/pour
construction evidence; `dépêcher` selects pronominal `se dépêcher`.

The lowercase `américain/NOM` selects the meat-and-fries sandwich; it does not
borrow capitalized `Américain` as a nationality noun. Its adjective selects the
continental Americas relationship. `garde/NOM` selects the feminine act of
guarding/protection. `gosse` retains the France/colloquial restriction, `victime`
retains feminine noun gender regardless of the person's gender, and `pute`/
`salaud` retain their vulgar or insulting register. `crime` selects the strict
French legal category. No bare-`l` alias or runtime change is proposed.

The proposed Wiki selections and Lexique translations/adaptations use the existing
[CC BY-SA 4.0 route](../../../resources/core-pilot-79/COPYING). Contexts retain
actual article import/reference notices and identify the Académie française's
eighth edition and Émile Littré separately where named. Imported French definitions,
quotations and media are not field text. Reference notices do not grant rights to
external prose. Contributor attribution uses the complete linked page histories;
the last editor is not treated as the sentence author. Selected-text authorship,
quality, rights and the combined redistribution obligations require the separate
reviewer's acceptance and parent finalization.

| Check | Result | Retained log under `target/issue-98/` |
| --- | --- | --- |
| Focused membership before additions | Expected failure: absent frozen rows | `red.log` |
| Focused membership under amended scope | Passed, one test | `green-amended.log` |
| Exact fixed revisions, spans and pending statuses | Passed: 93 fixed pages, 204 field/support spans | `author-validation.log` |
| Prior shared resource/runtime preservation | Passed: no worker changes | `preservation.log` |
| Documentation links and diff | Passed | `docs.log`, `diff-check.log` |
| Independent field/rights acceptance | Pending central reviewer | [review drafts](field-review-draft.json) |
| Conversion/native/offline/packaging | Parent-owned, not run by worker | [admission status](admission.json) |

Commands use PowerShell 7, pnpm 11.25.0 and
`uv run --python 3.14.10 python -X utf8`. The focused command is
`uv run --python 3.14.10 python -X utf8 scripts/test_core_batch_98.py`.
[Author validation](author-validation.json) records measurements and actual claims.
The parent owns global input merging, version/counters/checksums/notices, accepted
field records, conversion, native lookup/save/reopen, learner-data preservation,
packaging and release checks. Existing `l'` resources and the literal-`l` negative
Rust expectation remain unchanged.
