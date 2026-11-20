# Issue #113 independent semantic and selected-rights review

**50 uses / 200 fields accepted; 195 unchanged and five exact replacements;
zero unresolved fields.** Frozen rows 1601–1650 retain every headword, POS,
frequency string and their order. The denominator remains **4,087 groups**.
No annotation exception or runtime alias is admitted.

Ticket [#113](https://github.com/zardonc/lexique/issues/113),
parent [#78](https://github.com/zardonc/lexique/issues/78).
Reviewed author head and diff base: `e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451`.
Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-113`.
Independent checkout: `C:/Users/inthe/IdeaProjects/lexique-review-113`, branch
`codex/d2-review-113`. This reviewer is separate from the author and parent
integrator. No external human review is claimed.

## Exact semantic decisions

Every selected French sense, explicit IPA and proposed English/Chinese field
was read against actual fixed raw source and French POS/construction evidence,
in projections of five groups. [Field review](field-review.json) records every
exact proposed and accepted text, individual reason and source binding, plus all
50 accepted use contexts and spelling relations. [Original proposals](review-candidates.json)
remain unchanged, including their pending statuses. EN/ZH are Lexique explanations
of the French use, with matching restrictions, not upstream bilingual quotations.

| Exact field | Accepted wording | Source basis |
| --- | --- | --- |
| 1615 `rat`, Chinese | Generic straw is 秸秆 | Revision 40039545, line 20: `paille` has no rice restriction |
| 1627 `demi`, French/English/Chinese | Explicit before-noun hyphenation and invariability | Revision 39890664, lines 101–103; after-noun gender-only agreement retained |
| 1644 `mordre`, Chinese | Explicit bitten object and its surface | Revision 40015209, line 10: teeth press so as to `entamer` |

Original Lexique 3.83 TSV row 83159 independently confirms exact lowercase
`mlle`, feminine singular `NOM` and phonetic value `madmwazEl`. Fixed `Mlle`
revision 39033727 establishes title, expansion, typography, plural and
`/mad.mwa.zɛl/`; `mademoiselle` supplies youth/unmarried convention, declining
adult use, Quebec scope and French administrative replacement. Cambridge and
OQLF corroborate only capitalization, expansion, pronunciation and address
facts; no proprietary definition, translation, example or prose supplies a
field. The relationship is a source binding with `alias_admission=not_admitted`.
Standard capitalized writing does not replace the frozen lowercase key.

`celui-ci` retains proximal masculine singular reference and whole-compound IPA.
`toi-même` retains second-person singular emphatic subject use and the hyphen,
with `toi`/`même` component evidence. `mets/NOM` selects prepared food, identical
singular/plural spelling and `/mɛ/`; the later plural-of-`met` `/mɛt/` and
verb-of-`mettre` sections are explicitly excluded. `troisième` adjective and noun
keep ordinal rank and explicit `/tʁwa.zjɛm/`, excluding school/fraction senses.
`vidéo` adjective selects everyday devices/applications; the noun selects recorded
playable audiovisual media with its feminine/Canadian masculine context.
`obéir à`, intransitive `voyager`, transitive physical `guérir` and direct naming
`nommer` retain their constructions. `suspect` retains suspicion rather than
guilt, `maudit` reproachful complaint and `sein` ordinary body vocabulary. All
50 individual reasons assess everyday scope and exclude unrelated alternatives.

## Sources and selected rights

All **49 fixed revisions** were independently refetched from the first-party
French API in seven batches of seven. Complete wikitext, title, page ID,
timestamp and last editor match the author captures. Separate saved API/raw-span
checks cover all 200 field bindings, 50 use records and supporting evidence.
Complete contributor names match the seven-part API registry, whose final
response has no continuation. Full history credit covers anonymous contributors.
The current registry may extend beyond a fixed revision and does not prove
selected-sentence authorship. [Revision report](source-revision-review.json)
records exact results; complete refetches remain in the reviewer log root.

[Rights](rights.json) records selected fields, supporting excerpts, complete
contributors, original notices and an individual selected-boundary decision for
every source. Captured primary CC BY-SA 4.0 sections 2–4, Wikimedia Terms 7,
French copyright policy revision 38903168, DAF8 template 22113837 and CFC template
29619964 were independently read; the three wiki authorities match their saved
API/raw captures. Literary/news quotations, etymology, unrelated senses, other
languages, bilingual tables and image/audio assets do not supply fields.
Bibliography alone does not establish original authorship of a selected sentence.

The 37 DAF8 import/reference sources retain applicable Académie française
1932–1935 edition credit. The established issue-99 collective-work L123-3 basis
and conservative 2006-01-01 expiry were independently read and matched; no live
statute refresh is claimed. Contributor transcription/modification and Lexique
adaptations/explanations remain separately CC BY-SA 4.0. `objet` line 77 and
`valeur` line 75 have CFC cartography imports; selected fields instead bind the
concrete object at line 9 and price at line 18. The CFC notice is identified at
its own boundary and does not supply those definitions.

Combined redistribution must retain fixed article/history and original-work
credit, supplied copyright/licence/disclaimer notices, complete legal text/link,
modification notices and editable corresponding inputs. Adapted text and
applicable database rights retain ShareAlike without obstructive restrictions
or technical measures. These obligations are identified for every accepted
field; their satisfaction by the actual bundle remains the parent's gate.

## Validation and limits

Evidence root: `C:/Users/inthe/IdeaProjects/lexique-review-113/target/issue-113-review/`.
Platform: Windows, PowerShell 7.6.9, CPython 3.14.10 via uv. Execution evidence
applies to the pinned author SHA plus the scoped reviewer patch. The retained
`reviewer-handoff.patch` uses
`git diff e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451 -- docs/evidence/issue-113 scripts/test_core_batch_113.py`.

- `regression-red.log` reproduces the Chinese wording issue; `focused-green.log` passes **3 tests** covering exact frozen records, 200 texts/bindings, original proposals, replacements, rights and spelling metadata.
- `capture-integrity.json`, `remote-*-comparison.json`, `authority-integrity.json` and `supplemental-integrity.json` record complete matching, contributors, original TSV and rights-basis checks.
- `groups-1601-1605.json` through `groups-1646-1650.json`, `all-rights-markers.json`, `construction-raw.json` and authority text extractions retain full inspected context.
- `handoff-check.log`, `docs-check.log` and `diff-check.log` record final structural and documentation checks.

Canonical conversion, actual combined notices, native lookup/save/reopen,
saved-data preservation, learner acceptance and release gates were not run.
The review does not close #113 or change canonical admission/alias counts.
