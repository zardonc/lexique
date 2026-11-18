# Frozen row 87: `quoi/ADJ` eligibility research

**Frozen row 87 is an approved annotation exception under [#78](https://github.com/zardonc/lexique/issues/78)
and [#82](https://github.com/zardonc/lexique/issues/82), as of 2026-11-18 UTC.** The primary sources
checked support modern pronoun, noun and exclamative uses of `quoi`, but no modern
everyday adjective sense of that exact spelling. Lexique's `ADJ` record is present
in the original source; its cause remains unproved. The approved exception
retains that disputed annotation with zero admitted fields.

The reviewed development content remains **49/50 batch groups / 196 fields**,
including 47 new groups and two reused `bien` groups; globally it remains
**104 groups / 416 fields**, `core-82-v1`. The fixed denominator is **4,087 groups /
3,773 headwords**. The sole approved specialist sense remains #81's `ça/NOM`.
The approved annotation exception preserves the frozen identity and these
content counts. #82 is complete within its amended scope, with 49/50 content
coverage and one separately reported annotation exception.

## Exact source and frozen input

The owning [Lexique provenance](../../../resources/lexique-3.83/provenance.json)
fixes version **3.83**, its complete original TSV and documentation commit
`298010e765c49673317aa5b5c3b068df0351725a`. The repository preserves those TSV
bytes in [Lexique383.tsv.gz](../../../resources/lexique-3.83/Lexique383.tsv.gz).
The following are all records whose `ortho` or `lemme` is exactly `quoi`.
Source lines count the TSV header as line 1; every record has `ortho=lemme=quoi`,
`phon=kwa`, `islem=1` and `cgramortho=ADJ,ONO,PRO:int,PRO:rel`.

| TSV line | `cgram` | `genre`, `nombre` | `freqlemfilms2` | `freqfilms2` | `freqlemlivres` | `freqlivres` |
| ---: | --- | --- | ---: | ---: | ---: | ---: |
| 104317 | `ADJ` | `m`, `s` | 1558.22 | 1558.22 | 210.88 | 210.88 |
| 104318 | `ONO` | empty, empty | 0.73 | 0.73 | 1.62 | 1.62 |
| 104319 | `PRO:int` | empty, empty | 923.03 | 18.91 | 380.07 | 3.18 |
| 104320 | `PRO:rel` | empty, empty | 923.03 | 904.12 | 380.07 | 376.89 |

Frequencies are per million words in their respective subtitle or book corpus;
lemma and word-form frequencies are separate fields. Both pronoun subclasses
carry the source lemma value 923.03, while their subtitle form values differ.
The accepted selection sums each distinct lemma/POS value once:
`1558.22 + 0.73 + 923.03 + 923.03 = 3405.01`.

The [frozen manifest](../../../resources/core-pilot-79/scope-manifest.csv), CSV
lines 88–90, preserves rows 87–89 as follows:

| Frozen row | Exact group | `subtitle_lemma_per_million` | `subtitle_pos_per_million` |
| ---: | --- | ---: | ---: |
| 87 | `quoi/ADJ` | 3405.01 | 1558.22 |
| 88 | `quoi/PRO:int` | 3405.01 | 923.03 |
| 89 | `quoi/PRO:rel` | 3405.01 | 923.03 |

The `ADJ` group therefore meets the frozen frequency rule in #78's Further Notes.
It was not introduced by the project converter. Frequency and grammatical tags
establish source membership, not an admissible dictionary meaning.

## What the data owner's category means

Boris New and Christophe Pallier's [*Manuel de Lexique 3*, document version
3.11][manual-fixed], printed page 13, Table 3, explicitly gives **“ADJ Adjectif”**.
It separately lists `ADJ:dem`, `ADJ:ind`, `ADJ:int`, `ADJ:num` and `ADJ:pos`, as
well as `PRO:int`, `PRO:rel` and `ONO`. Plain `ADJ` is not documented as a generic
category covering interrogative pronouns or possessive determiners.

Printed page 9, §2.3, says: **“Afin d'étiqueter grammaticalement nos corpus,
nous avons utilisé l'étiqueteur Cordial Analyseur.”** This documents the general
corpus-tagging method. It does not identify the originating sentence, lexical
analysis, software version or correction history of line 104317. The retained
owner [release notes][release-notes] describe 3.83 as “Minor phonological
corrections”; they supply no explanation for this record's category.

The manual inspected through the owner's served PDF is byte-for-byte identical
to the PDF at the fixed documentation commit. Its text locators are PDF page
indexes 8 and 12, extraction lines 194–201 and 312–337. The owner
[README][owner-readme] retains the manual as useful database documentation while
warning that its online-interface instructions are obsolete.

These sources establish an actual, unexplained upstream annotation. They do
not prove that Cordial produced this particular label, that it is a spelling
error for `coi`, or that a particular historical adjective caused it.

## Primary lexical evidence

Public captures below were retained on **2026-11-18 UTC**. Académie articles
identify the completed ninth edition as **2024**; their article IDs and exact
entry sections are the version/span locators. The current CNRTL portal separates
`quoi` by POS: its [bare entry](https://www.cnrtl.fr/definition/quoi) is a noun page, while the explicit
`/definition/quoi/pronom` page exposes the TLFi article.

| Primary source and exact span | Decisive evidence | Consequence for row 87 |
| --- | --- | --- |
| Académie 9, [A9Q0310][academie-i], entry heading and §I.1 | **“pronom relatif invariable”**; the pronoun can be *complément du nom ou de l'adjectif*. | Being the complement of an adjective is a pronoun function, not an adjective classification. |
| Académie 9, [A9Q0311][academie-ii], entry heading, §2 and §3 | **“pronom interrogatif”**; §2 includes the fixed construction *Quoi de* and *Quoi de neuf*; §3 includes *N'importe quoi*. | These ordinary constructions are treated in the pronoun entry. They do not supply a bare adjective sense. |
| Académie 9, [A9Q0312][academie-iii], entry heading and etymology | **“adverbe exclamatif”**, **“Emploi adverbial de quoi II.”** | The exclamation is a separate adverbial use. |
| ATILF/CNRTL, [TLFi `quoi/pronom`][tlf], sections I, II and III | **“Pron. interr. ou exclam., porteur du genre inanimé”**; **“Pron. rel. prép.”**; **“Pronom. indéf.”** | The comprehensive pronoun article supplies no adjective classification. |
| Same TLFi article, §I.C.7 and §III.D | *Quoi de + adj.* occurs under the interrogative/exclamative pronoun; §III.D labels *n'importe quoi* **“loc. pronom. indéf.”** | An adjacent adjective or an indefinite compound does not satisfy `quoi/ADJ`. |
| French Wiktionnaire, [revision 39880332][wiki], timestamp 2026-09-17T16:44:24Z, French headings at lines 6, 91, 99 and 160 | Interrogative pronoun, noun, relative pronoun and interjection; no French adjective heading. Lines 8, 101 and 162 give `kwa` for those uses. Line 277 links the homophone `coi`. | This reproduces the existing batch's fixed-source gap. A homophone's spelling or a pronunciation from another class cannot complete this adjective group. |

The current [curation](../../../resources/core-pilot-79/curated-inputs.json)
contains only `quoi-interrogative-pronoun-object-thing` and
`quoi-relative-pronoun-prepositional-thing`, bound to rows 88 and 89. It reports
row 87 separately in #82's `unresolved_groups`; it supplies no adjective sense.

## Approved scope amendment

The project owner approved the following amendment on 2026-11-18 UTC. It makes
one exception to #78/#82's ordinary modern everyday sense and four-field
requirements, while preserving the exact frozen membership and denominator:

> Retain frozen row 87, `quoi/ADJ`, as an annotation exception to the modern
> everyday sense and four-field requirements in #78 and #82.
> Preserve its headword, POS, row order and the 4,087-group denominator. Report
> #82 as 49/50 groups with reviewed content and 196 admitted fields; report row
> 87 separately with zero admitted fields. This exception is not an admitted
> specialist sense and does not establish 50/50 content completion.

The four fields (`french_definition`, `ipa`, `english`, `chinese`) remain absent.
The existing artifact records them in the batch's `unresolved_groups`, which
describes missing content. The approved exception resolves the requirement for
that row and leaves 49/50 batch groups with content. It is separate from the
existing `ça/NOM` specialist sense, which has all four admitted fields.

#78's Implementation Decision 1 still requires explicit scope and manifest
revision for group changes. This amendment changes only row 87's content
requirement; the frozen manifest and previously admitted content are preserved.

## Source rights

The existing Lexique membership uses **CC BY-SA 4.0**, with its owning
[upstream notice](../../../resources/lexique-3.83/UPSTREAM-README.txt) and
[licence](../../../resources/lexique-3.83/COPYING). That permission does not create
an adjective definition or English/Chinese meanings. The batch's existing
Wiktionnaire CC BY-SA route remains documented in [source-research.md](source-research.md)
and [resource notices](../../../resources/core-pilot-79/NOTICES.md); its fixed
`quoi` article supplies no eligible adjective field.

Académie 9 and TLFi are cited linguistic evidence, not proposed bundled field
text. [Académie's CGU][academie-cgu], §6, requires **“l'autorisation expresse et
préalable de l’Éditeur”** for reproducing, adapting or translating its content;
§7.2 permits source-attributed citations and hyperlinks. It therefore supplies
no general commercial redistribution route under the current
[resource policy](../../adr/0002-source-and-resource-licensing.md).

## Verification and retained evidence

Checks ran on Windows x64 with PowerShell 7 and uv-resolved Python **3.14.10**, in
`C:\Users\inthe\IdeaProjects\lexique\target\quoi-adj-research\worktree`, branch
`codex/refine-v03-guidance-and-quoi-adj-research`, based on merged main
`7864cde9b32a2ad8d3e296a847b0539206e9162a`. Complete public source captures,
extractions and logs are retained under `target/quoi-adj/linguistic/` in that
checkout.

| Command or check | Result and retained evidence |
| --- | --- |
| `gh issue view 78 --json number,title,state,body,url` and equivalent #82 read | Passed; exact current requirements and OPEN state retained in `github-issue-78.json` and `github-issue-82.json` |
| uv-resolved `python -X utf8 target/quoi-adj/linguistic/inspect_frozen_source.py` | Passed; all four source rows, exact frozen rows 87–89, 4,087-row denominator and 3405.01 total; `frozen-source-records.json`, `quoi-source-records.tsv`, `frozen-source-check.log` |
| Fixed manual/served manual byte comparison and fixed Wiktionnaire revision assertions | Passed; identical PDF bytes, revision 39880332 and its timestamp/headings; `primary-version-check.log` |
| Primary page capture and exact entry-section inspection | Passed; `academie-9-quoi-i.html`, `academie-9-quoi-ii.html`, `academie-9-quoi-iii.html`, `academie-primary-text.json`, `cnrtl-quoi-pronom.html`, `cnrtl-pronoun-text.json`, `lexique-manual-fixed.pdf`, `lexique-manual-primary-extract.json`, `wiktionnaire-quoi-39880332.api.json`, `wiktionnaire-quoi-39880332.txt`, `academie-cgu-text.json` |
| Focused report links, source assertions and whitespace check | Passed; `report-check.log`; no resource or frozen-manifest diff |

## Limits

The eligibility conclusion applies to the exact frozen group under #78/#82's
modern everyday requirement. It does not establish the absence of every
historical or regional adjective use. No occurrence-level corpus annotation or
row-specific maintainer correction explains the `ADJ` label here. No field or
resource was admitted; learner acceptance, application/native/offline checks and
release checks were not rerun for this documentation-only research. The scope
amendment above is approved; broader core-content and release gates remain open.

[manual-fixed]: https://github.com/chrplr/openlexicon/blob/298010e765c49673317aa5b5c3b068df0351725a/datasets-info/Lexique383/Manuel_Lexique.3.pdf
[owner-readme]: https://github.com/chrplr/openlexicon/blob/298010e765c49673317aa5b5c3b068df0351725a/datasets-info/Lexique383/README-Lexique.md
[release-notes]: https://github.com/chrplr/openlexicon/blob/298010e765c49673317aa5b5c3b068df0351725a/datasets-info/Lexique383/RELEASE%20NOTES-Lexique.txt
[academie-i]: https://www.dictionnaire-academie.fr/article/A9Q0310
[academie-ii]: https://www.dictionnaire-academie.fr/article/A9Q0311
[academie-iii]: https://www.dictionnaire-academie.fr/article/A9Q0312
[academie-cgu]: https://www.dictionnaire-academie.fr/CGU
[tlf]: https://www.cnrtl.fr/definition/quoi/pronom
[wiki]: https://fr.wiktionary.org/w/index.php?title=quoi&oldid=39880332
