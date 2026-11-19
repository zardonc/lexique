# Issue #90 content handoff

Frozen rows **451–500** have **49 new four-field drafts / 196 populated fields**
and **one unchanged admitted reuse / four fields**, `coeur/NOM` at row 483.
All 50 exact headword/POS identities are retained. The new fields have author
validation, with **zero independently accepted new fields**. This is a content
handoff awaiting separate review and integration, not resource admission.

The authoritative additions are [groups.json](groups.json), [sources.json](sources.json),
[source-context.json](source-context.json) and [source-excerpts.json](source-excerpts.json).
[field-review.json](field-review.json) contains draft field/usage review records:
reviewer is null, independence is false, and every new acceptance remains pending.
[reuse.json](reuse.json) identifies the previously accepted heart-organ record;
its fields, source spelling relation, contexts and reviews are reused unchanged.
[admission.json](admission.json) records the pending admission state.

No admitted resource input, frozen manifest, shared converter, global counters,
version, checksum or notices package was changed. These shards are also supplied
outside the repository at
`C:/Users/inthe/.codex/tmp/lexique-spec-90-99/batch-90/` for keyed integration.
The permanent denominator remains 4,087 groups / 3,773 headwords.

## Sources and rights

[source-captures.json](source-captures.json) locates **40 fixed first-party source
captures**: 38 primary French articles, supplementary French `autre`, and English
Wiktionary `autre`'s French pronoun section. Actual returned revision IDs and
timestamps are retained at or before the established October 1, 2026 UTC cutoff.
Every selected field and supporting span binds an exact one-based wikitext line.
The API responses, complete paginated revision histories, contributor metadata
and full wikitext remain under this worktree's ignored `target/issue-90/sources/`.
No full page, media or external quotation text is selected for the shards.

French definitions adapt the fixed semantic spans to the stated selected uses.
English and Chinese are new Lexique explanations of those uses, with their
modifications recorded; they are not presented as upstream bilingual quotations.
IPA transcribes one recorded variant and does not claim exclusive pronunciation
or liaison coverage. Form relationships for `autres` and both `aucune` subclasses
are separately evidenced. The existing `coeur`/`cœur` relationship is preserved.

Each applicable article import/bibliography marker is retained in source context.
DAF8 originals retain **Académie française, eighth collective edition, 1932–1935**
credit and their public-domain boundary; later Wiki/Lexique adaptations retain
CC BY-SA 4.0. The previously captured fixed import template, edition/author
authority and original-volume metadata were copied byte-identically from
`lexique-84-89/target/implement-84-89/quality-88-evidence` into ignored
`target/issue-90/rights/`. Their original capture origin is recorded, rather than
described as a new fetch. References to TLFi, Robert, Larousse or other dictionaries
do not establish rights to their prose. Separate review must verify the actual
selected original-author boundaries and assembled redistribution obligations.
The existing [COPYING](../../../resources/core-pilot-79/COPYING) and
[membership notices](../../../resources/core-pilot-79/MEMBERSHIP-NOTICES.txt)
remain applicable; source and modification notices must be assembled centrally
after acceptance. Project code remains MIT.

## Selected-use points for the separate reviewer

- `vers/NOM` selects a line of poetry, not the plural of `ver`; `vers/PRE`
  selects spatial direction. `voler/VER` selects stealing, not flight.
- `feu/ADJ` selects invariable tan/reddish coat markings, with the ordinary dog
  and rabbit construction at [revision 39907463](https://fr.wiktionary.org/w/index.php?title=feu&oldid=39907463), line 948. Its everyday-use fit
  needs independent confirmation; the formal deceased adjective is excluded.
- `super/NOM` selects the familiar French supermarket noun, not fuel or
  fertilizer; its source restricts the selected use to France.
- `autres/PRO:ind` uses the exact plural relationship and contrasting pronominal
  semantic span from French `autre`; the English edition explicitly classifies
  French `autre` as a pronoun and records `les uns les autres`. Its different
  French nominal-section label remains visible for POS review.
- Both `aucune` rows retain their feminine form and their distinct determiner
  and pronoun sections. `leurs` is a plural possessive determiner before a plural
  noun. `celui` selects the form completed by a relative or prepositional phrase.
- `manquer` selects the intransitive missing-item subject. `répondre` retains
  `à` for the person/question. `valoir` directly takes the monetary amount.
  Excluded constructions and homonyms are explicit in every selected sense.

## Verification and pending work

PowerShell 7.6.9 and `uv run --python 3.14.10 python -B -X utf8` were used.
The focused [membership check](../../../scripts/test_core_batch_90.py) failed
before the content shards existed and passed after their addition: all 50 frozen
identities, nonempty selected uses, four populated fields and source spans.
It checks the handoff artifact plus the unchanged reuse, not an admitted SQLite
artifact. Full logs are in ignored `target/issue-90/`.

The author validation checked 196 field spans and 13 supporting spans against
actual raw/API captures, recorded IPA variants, fixed versions, paginated history
completion, frozen frequency strings and pending review statuses. It also checked
that all admitted resource inputs remain unchanged. The longest drafted field is
196 characters; no content was shortened for a resource budget.

Separate independent review of all 196 new fields, usage/POS fit and selected-text
rights remains pending. No missing text or requested scope exception is hidden;
unaccepted drafts remain outside the admitted inputs. Parent owns accepted-field
integration, conversion/reproduction, counters, checksums, notices, native
lookup/save/reopen, learner-data preservation, packaging and final checks.
No frontend build, native GUI build or shared Cargo tests were run in this worker.
