# Selected source evidence for #117

The package contains one scoped everyday selected sense for every exact frozen
row 1801–1850. The [groups](groups.json) contain the complete French definition,
isolated-headword IPA, original English explanation and original Chinese
explanation for the same use. Their [field bindings](source-excerpts.json) retain
immutable revision IDs/URLs and exact one-based source lines/columns.

## Fixed sources and selected rights

Fifty first-party Wiktionnaire revisions cover the 46 distinct frozen headwords
and the supporting entries `tout`, `celle`, `celui-ci` and `professeur`.
Complete API responses, exact raw wikitext and source index remain under
`target/issue-117/sources/` and `target/issue-117/source-index.json`.
Two content requests and eight contributor-response parts are retained.
The [source context](source-context.json) carries every captured contributor
name and linked full article history. The current contributor registry can
extend beyond a fixed revision; it is attribution evidence, not selected-sentence
authorship or independent quality acceptance.

Thirty-eight selected/supporting pages mark or reference the Académie française
eighth edition (1932–1935). The original collective-work edition, Wiki
transcription/adaptation and Lexique adaptations are recorded separately.
The original edition's public-domain proposal reuses the project's fixed
article L123-3 and publication evidence from [#99](../issue-99/source-context.json),
using final publication in 1935 and conservative expiry on 2006-01-01.
The independent selected-span review applies that established basis to this batch.
No additional imported-work template was found in these captured pages.
Bibliographic references and quoted examples elsewhere on a page are preserved
in raw evidence and do not supply expressive field wording.

[Rights](rights.json) provides one explicit rights record for each of the 200
fields, contributor-context pointers, original-work layers, authorship of new
translations, modifications, [complete CC BY-SA 4.0 text](../../../resources/core-pilot-79/COPYING)
and required attribution/ShareAlike/notice obligations. Current CC legal text,
Wikimedia terms and three fixed French rights/import-policy revisions were
captured in full. Individual independent decisions now accept all 200 selected
fields and their supporting boundaries; original author assessments remain
alongside the decisions. Combined parent packaging remains pending.

## Construction, spelling and flexion

| Frozen rows | Selected binding |
| --- | --- |
| 1801, 1810 | `pisser` is popular/vulgar intransitive urination; `couille` is the vulgar feminine testicle noun. Register is retained in all semantic fields. |
| 1806–1808 | Physical `résister à`, transitive `ajouter` with optional `à`, and suitability `convenir à`; agreement and impersonal senses are excluded. |
| 1809 | Exact traditional `chaîne` spelling and physical linked-chain sense. No spelling alias is admitted. |
| 1813 | `celle-ci` explicitly supplies feminine singular flexion and /sɛl.si/; `celui-ci` supplies proximal semantics. `celle`'s required relative/prepositional continuation does not apply to standalone `celle-ci`. |
| 1822–1823 | Direct-route adjective and masculine live-broadcast noun have separate source sections; `en direct` has its own retained construction span. |
| 1824–1827 | Particular-characteristic adjective/private-individual noun and short-duration adjective/summarising adverb stay distinct. |
| 1828, 1832, 1835, 1840–1841 | Explicit source flexion tables support prudente, tribunaux, nationale/nationaux/nationales, légère and affreuse. |
| 1833 | `millier` is the collective noun for a thousand units, with a `de` complement; historical weight and unspecified-large-quantity senses are excluded. |
| 1836 | Exact `prof` noun section 1 is the familiar teacher abbreviation of `professeur`, with /pʁɔf/ and masculine/feminine gender. The unrelated profondeur abbreviation is excluded. |
| 1839, 1842 | Transitive `promener` takes the person/animal being led; transitive `venger` takes the wronged person. Bare pronominal uses are not substituted. |
| 1847 | Exact `toutes/ADJ:ind` is feminine plural in the indefinite-adjective flexion section; semantics bind to `tout`'s totality determiner, accompanying a feminine plural noun. |
| 1848 | Exact `toutes/PRO:ind` is independently bound through its pronoun flexion section to `tout`'s plural indefinite-pronoun sense. It stands for already mentioned feminine people/things; final t is pronounced /tut/. |
| 1850 | Courage adjective, same masculine/feminine form; kindness and the person noun are excluded. |

Short conventional grammatical forms selected from examples have explicit
columns and a factual role. Surrounding literary/news prose and its bibliography
remain in ignored full captures; no such expressive prose is translated into a
field. All 50 spelling relations have `relation_role=source_binding_only` and
`alias_admission=not_admitted`, including the four cross-entry semantic bindings.
CEFR remains unknown.

## Preparation and coverage

The retained `fetch.py`, `contributors.py`, `selection.py`,
`prepare_candidates.py`, `capture_rights.py` and `finish_author_package.py`
reuse [#115's author format](../issue-115/README.md) and the existing
[#109 review format](../issue-109/README.md). Preparation validates actual
API IDs/timestamps, raw content equality, source language/POS, IPA variant
positions and exact frozen memberships. The focused test reads the delivered
author records and retained source spans at the approved resource boundary.

There are no reused assigned groups, no new owner exceptions, no canonically
admitted new fields and no runtime aliases. Independent acceptance is 50 groups
and 200 unchanged fields, with zero unresolved fields. The fixed denominator
stays 4,087 groups; all 200 batch fields remain unadmitted canonically. The parent
owns canonical inclusion, conversion, notices and native verification.
