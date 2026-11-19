# Issue #95 content handoff

Rows **701–750** are represented by **50 distinct candidate groups / 50 selected
senses / 200 nonempty FR–IPA–EN–ZH fields**, matching the frozen manifest exactly.
There are no reused ticket rows and no proposed membership exceptions. This is an
author-validated content slice awaiting separate quality and rights acceptance;
**zero new groups are independently admitted by this handoff**.

The owning spec is [#78](https://github.com/zardonc/lexique/issues/78), with the exact
membership in [#95](https://github.com/zardonc/lexique/issues/95). The working copy
was based on integration commit `ea570175a42fb5b205c138c93f4e731d476d5d28` before
edits. The permanent target remains 4,087 groups; this handoff changes no global
version, coverage counter, input checksum, notice package or runtime code.

## Review and integration inputs

- [groups.json](groups.json): additions keyed by frozen row, with exact POS and
  frequency strings, scoped selected uses, four fields and modifications.
- [sources.json](sources.json): fixed article versions and source identities.
- [source-context.json](source-context.json): complete article contributor lists,
  selected locations, POS/form/construction evidence, and import/reference notices.
- [source-excerpts.json](source-excerpts.json): exact fixed wikitext spans for all
  200 fields. No new spelling aliases are proposed.
- [field-review.json](field-review.json): author-prepared review candidates;
  `independence` is false, reviewer is null, and all independent statuses are pending.
- [source-captures.json](source-captures.json): full-capture locations and versions.
- [reuse.json](reuse.json): no reused rows or sources.
- [admission.json](admission.json): batch readiness and outstanding gates.

These additions are separate from the shared files in
[core-pilot-79](../../../resources/core-pilot-79/README.md). The parent must merge
accepted records by key into the current shared files after the separate reviewer
accepts the fields and actual source boundaries. The converter reads those shared
files; this candidate handoff does not bypass its admission checks.

## Source and rights boundary

All 41 ticket headwords have real first-party Wiktionnaire revision captures.
The first ten articles were selected at or before `2026-10-01T00:00:00Z`; the other
31 and the three supporting articles (`lequel`, `clair de lune`, `chaudement`)
use the actual revisions returned at retrieval. Each records its own revision
and timestamp; the entire slice is not described as an October snapshot.

Full API bytes, wikitext and contributor responses are retained at
`C:/Users/inthe/IdeaProjects/lexique-content-95/target/issue-95/`.
Contributor continuation completed for all 44 articles, yielding 3,283 registered
article-contributor records. This attributes article contributions, not authorship
of every selected sentence. Fixed history links remain attached to each source.

Wiktionnaire contributions and the recorded Lexique adaptations are designated
CC BY-SA 4.0, subject to resolving each selected original-text boundary. Retained
DAF8 import/reference notices credit the Académie française's eighth edition
(1932–1935); Littré references credit Émile Littré. A page reference does not prove
that a particular selected sentence was imported or that its redistribution is
eligible. Imported original prose needs its actual author, edition, selected span
and applicable rights checked before admission. Unselected quotations, entire
articles and media are confined to ignored captures. Bibliographic notices are
evidence, not bundled dictionary fields or grants of rights.

EN/ZH are the content author's translations or scoped explanations of the selected
French use. Every modification is recorded. The separate reviewer must check
translation quality, IPA, POS, constructions, spelling/form binding and rights.
No author check is labelled independent acceptance. The parent owns final combined
notices and redistribution verification under the existing
[COPYING](../../../resources/core-pilot-79/COPYING) and
[resource ADR](../../adr/0002-source-and-resource-licensing.md).

## Construction decisions for review

| Rows | Selected boundary |
| --- | --- |
| 704–705 | Living adjective and living-person noun remain distinct. |
| 712 | `celle` is feminine singular, followed by a relative clause or prepositional phrase; the article's inflection table is retained. |
| 720 | `pardonner` retains the fault as direct object and the person introduced by `à`. |
| 724–725 | Person adjective and abstract composure noun remain distinct; noun IPA comes from the general textual pronunciation, not the noun header's syllable division. |
| 726–727 | The two `laquelle` form sections bind separately to `lequel`'s interrogative and relative definitions. No lookup alias is added. |
| 731–733 | Temperature adjective, invariable adverb in `manger/boire chaud`, and abstract heat noun remain distinct. |
| 740 | Nominal light is restricted to ordinary `clair de lune`, with the fixed compound definition as supporting evidence; painting/tapestry senses are excluded. |
| 748 | The selected weakened `sûrement` sense means probability, not absolute certainty. |

## Verification

PowerShell 7 and `uv run --python 3.14.10 --no-project python -X utf8` were used.
Complete logs are retained in the ignored capture root.

| Check | Result |
| --- | --- |
| Focused `unittest discover -s scripts -p test_core_batch_95.py` before content | RED: missing issue #95 handoff; `red.log`. |
| Same membership/four-field check after content | GREEN: one test passed; `green.log`. |
| Author capture/span audit | All 44 article IDs/revisions and 200 exact field spans match retained first-party bytes; grammar spans match their captures. |
| Shared-file preservation | Shared resource files unchanged from the integration baseline. |
| Diff and local Markdown link checks | Passed for this slice. |

The retained author audit is reproducible with
`uv run --python 3.14.10 --no-project python -X utf8 target/issue-95/audit_handoff.py`;
its output is `target/issue-95/audit.log`. The scratch source fetch, construction
selection and handoff assembly scripts are retained beside the full captures.

The focused [test](../../../scripts/test_core_batch_95.py) checks the frozen
candidate handoff and nonempty sourced fields. It does not prove independent
quality, native lookup, save/reopen, learner-data preservation or package readiness.
Maximum candidate field length is 147 Unicode scalars; maximum IPA length is 13.

Independent quality and imported-text rights acceptance remain pending. Conversion,
combined preservation, native lookup/save/reopen, packaging, global tests and final
readiness are owned by the parent and were not run in this worker. The core remains
incomplete for release.
