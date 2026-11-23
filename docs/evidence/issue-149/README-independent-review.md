# Independent review of #149

All 50 frozen groups and 200 fields are independently eligible for coordinator
assembly. All 200 quality decisions and selected-rights decisions pass; all
50 selected uses and spelling relations pass. There are 194 unchanged texts,
six corrected texts, zero unresolved fields and zero pending owner choices.
No annotation, specialist approval or runtime alias is requested by this shard.

The reviewer is separate from the author and coordinator, working on
`codex/d2-review-149` in `review-149`. Author commit:
`ba24dc74199cffe2c39f5096455eee2900e9a643`. Review base:
`f1a2fb0cba08d0132654d9fda418a1c89e3f713e`. Required integration tip merged before
delivery: `44899e1574eefbcd28f5af564c2375c67a7a61a9`. Exact #149/parent #78 cached
specifications and repository/reviewer guidance were read; #149 scope is
unchanged by the parent's later annotations and specialist approvals.

## Content and use decisions

Each French definition, isolated IPA, English explanation and Chinese
explanation was read against its actual retained fixed French text, grammatical
frame and selected/supporting source spans. [field-review.json](field-review.json)
records actual reasons for every field, use and spelling decision. These are
independent editorial decisions; the audit checks their consistency and source
bindings. [groups.json](groups.json) is accepted content, while
[authored](authored/snapshot.json) and [review-candidates.md](review-candidates.md)
retain the author's proposals. All 13 supplied authored snapshot files remain
byte-identical to the author commit.

| Row / group | Corrected fields | Accepted restriction |
| --- | --- | --- |
| 3407 `organe/NOM` | Chinese | A functional constituent of an organism; removed the unsupported requirement that it be inside the organism. |
| 3429 `choquer/VER` | English, Chinese | Figurative transitive offence; removed unrestricted shock/astonishment. |
| 3444 `lady/NOM` | French, English, Chinese | Daughters' courtesy title applies to daughters of dukes, marquesses or earls. |

Each correction first failed a focused resource/admission check, then passed
with the smallest supported change. [accepted-replacements.json](accepted-replacements.json)
retains exact proposed/accepted texts and source spans. The `lady` correction
also uses retained Debrett's courtesy-title guidance and modern Académie
corroboration; modern external prose was not copied into learner fields. Its
French IPA `/le.di/` remains unchanged.

The review preserves selected grammatical subclasses, constructions and
register: the possessive-pronoun component `sien`, preposed cardinal `onze`,
the explicit `surpris`/`endormi` adjective frames, feminine breakage `casse`,
serious-penalty person noun `condamné`, familiar `boss`, transitive wear `user`
and pronominal `adapter`. Accepted use restrictions remain in the use ledger;
no unsupported broadening or extra group was introduced.

## Source and original-expression rights

The audit verifies 53 fixed source articles in seven revision API files,
literal UTF-8 raw/API equality, 200 selected field spans, 209 supporting excerpt
records and 1,045 occurrences of supporting review spans. Twelve contributor
response parts form seven complete continuation chains. The retained registries
credit 2,969 named article memberships and 567 anonymous article counts; these
are article credits, not sentence authorship. Fixed source/history links and
modification notices remain in [NOTICES.md](NOTICES.md).

All selected historical/imported boundaries were inspected individually.
[original-expression-review.json](original-expression-review.json) records 34
actual DAF8 original-expression comparisons, including unmarked or
bibliography-only correspondence for `organe`, `gendarme`, `jalousie`, `poignée`,
`casse` and `condamné`. The last is located under original `condamner`.
Selected modern wiki wording is distinguished from unrelated historical prose.
No article import marker or general expiry substitutes for the actual selected
comparison. Isolated IPA records use phonetic contributor data; article-level
historical lexical layers are separately retained as excluded IPA context.

The six initially pending meaning fields for row 3433 `silencieux/ADJ` and
row 3443 `menu/NOM` are now resolved. CNRTL's linked payloads lacked those DAF8
entries. The Académie's first-party historical articles
[A8S0941](https://www.dictionnaire-academie.fr/article/A8S0941) and
[A8M0869](https://www.dictionnaire-academie.fr/article/A8M0869) were captured and
compared with fixed lexical lines 22 and 89 respectively. The menu entry's
substantivized noun paragraph identifies the detailed written meal list.
The earlier pending decisions and failed lookups remain archived under
`pre-primary-status` and the original early handoff files; they are superseded
by the final rights ledger. Learner texts for these two groups remain unchanged.

The original Académie institutional collective edition is 1932–1935; the
established CPI L123-3 basis gives conservative expiry **2006-01-01**. Fresh
official statute requests returned HTTP 403, so no freshly captured statute is
claimed. The retained established #129 basis and actual original expressions
were read. Wiktionnaire transcription/adaptation and Lexique adaptations retain
CC BY-SA 4.0. The complete legal code, French policy revisions 38903168 and
22113837 and Wikimedia Terms section 7 were read; historical policy CRLF
serialization is explicitly disclosed. [selected-rights.md](selected-rights.md)
records attribution, modification and ShareAlike obligations.

The CFC cartographic definition at `cadre` line 103 is excluded from both fields
and support. `lesbianisme` contributes only its ordinary lexical concept;
embedded Canadian reference metadata and quoted example prose are excluded.
No literary/news quotation, modern dictionary prose, image, audio, etymology or
other-language section is copied into a learner field. Compatible support
sources clarify facts without importing their external expression.

## Verification and delivery limits

The approved resource/admission seam is exercised by
[test_core_batch_149.py](../../../scripts/test_core_batch_149.py). Commands and
actual results are recorded in [verification.json](verification.json), with full
RED/GREEN and final logs in the archive. Documentation links and diff checks
are included. These checks validate frozen membership, frequencies, exact
four-field/source/use alignment, contributors and accepted review consistency.

Full proof is archived at
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/reviewer-evidence/issue-149`:
raw author sources/authorities, independent inspections and explicit decisions,
original and corroborating captures, prior pending audit, final ledgers, helpers,
check logs, complete patch, test patch and final commit metadata. The final
`review-report.json` is the archive entry point. Ownership is limited to
`docs/evidence/issue-149/**` and `scripts/test_core_batch_149.py`.

Canonical conversion, combined source materials/notices, global assembly,
native lookup/save/reopen, learner acceptance, release readiness, tracker and
push remain coordinator-owned and unrun here. No frontend, Rust, browser or
native behavior claim is made by this resource review.
