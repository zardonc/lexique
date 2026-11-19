# Batch #87 fixed source research

The frozen ticket covers rows **0301–0350**, retaining the exact Lexique
headword, grammatical subclass, frequency values and order. Forty-nine groups
have selected four-field candidates; `dont/PRO:int` has four explicit incomplete
fields. Source research and candidate extraction do not constitute admission.

The [source index](source-index.json) records 44 French Wiktionnaire revisions
at or before **2026-10-01 00:00:00 UTC**: 41 frozen headwords, with `œil` as the
source spelling for frozen `oeil`, and the additional `oeil`, `au revoir` and
`sur le tard` captures. Twenty-four were obtained directly through the official
API. Twenty were recovered from the existing read-only research checkout;
their owning API or fixed-revision registry/wikitext paths are recorded as
`copied_from`. All actual fixed URLs, page IDs, revisions, timestamps and capture
paths remain explicit. Recovered captures are not reported as fresh downloads.

Full responses and raw wikitext are retained in `target/issue-87/sources/`.
The tracked [source context](../../../resources/core-pilot-79/selected-source-context.json)
retains the 44 new source records, 124 selected context lines, bibliography/import
markers and actual capture provenance. The source attribution method is the
complete article-history link permitted by Wikimedia Terms section 7(b).
Eight contributor API responses ended without continuation and record all
44 public contributor inventories, comprising 3,949 registered article/user entries, plus
the reported anonymous contributor counts. Those inventories reflect the
2026-11-18 capture date, including later edits; they do not assert that every
listed contributor authored a selected pre-cutoff line. The fixed revision and
history links determine selected-text attribution.

## Source, spelling and construction boundaries

- **`dont/PRO:int`, row 305:** fixed revision 39824777 has the interrogative-adverb
  heading at line 93, `/dɔ̃/` at line 95, and the explicitly obsolete `d’où/où`
  definition at line 96. The contemporary Le Robert capture describes relative
  pronoun uses. Neither this obsolete use nor the modern relative sense is an
  eligible ordinary interrogative-pronoun field set. No four-field candidate is
  proposed; the known source pronunciation remains evidence only.
- **`dont/PRO:rel`, row 306:** the relative heading, headword and `de qui/de quoi`
  equivalents are at lines 7, 9 and 10 of the same revision. The four fields
  describe a `de` complement attached through a relative clause. The separate
  obsolete interrogative, instrument and including senses are excluded.
- **`revoir/NOM`, row 320:** the noun heading, masculine headword and definition
  are at lines 68, 70 and 71 of revision 39885599. Line 72 supplies the nominal
  farewell construction. `au revoir` revision 39885605, lines 5 and 8, establishes
  the contemporary polite farewell. Le Robert's modern entry also records
  `au revoir` as an interjection and a masculine invariable nominal expression.
  The proposed bare-lemma binding is restricted to this formula, expressing
  seeing someone again. It does not import unrestricted literary noun uses or
  relabel the independent verb. The fresh reviewer must check this exact binding.
- **`tard/NOM`, row 346:** revision 39874709, lines 102–107, explicitly labels
  the noun and gives `sur le tard` in the late-evening scope. The same source's
  adverb remains separate. `sur le tard` revision 34892223, line 7, corroborates
  the fixed construction; its broader late-in-a-long-period meaning is outside
  the selected evening field. Le Robert labels modern `tard` a masculine noun
  in `sur le tard`, with a late-age explanation. The evening restriction and its
  modern acceptability need separate review.
- **`puis/CON`, row 343:** revision 39772454, lines 79, 81–83, labels a
  coordinating conjunction with `et`, including `et puis`. Le Robert calls the
  lexical item an adverb and records enumeration/new-reason constructions with
  `et puis`. The frozen `CON` is retained for that coordinating use; chronological
  `puis/ADV`, the regional causal use and `pouvoir` forms remain separate.
- **`oeil/NOM`, row 328:** revision 38475518, lines 4, 7–8, explicitly identifies
  `oeil` as a typographic variant of `œil`. The organ-of-sight definition and IPA
  are at lines 14 and 13 of `œil` revision 39820476. The single added spelling
  record binds only this sense to frozen `oeil`; it is pending independent review.
- **`hein`, `merde` and `eh` under `ONO`:** fixed source headings classify the
  selected uses as interjections. Requesting repetition, a vulgar angry/surprised
  exclamation, and admiration/surprise respectively retain the frozen ONO label.
- **`foutre/VER`:** line 19 selects the modern very informal/vulgar `faire` use.
  The explicitly old sexual definition at line 8 and the separately attributed
  TLFi line 53 are excluded. The register remains in all three meanings.
- **`frère/NOM`:** the selected source definition explicitly shares both father
  and mother. All three meanings preserve that restriction; half-siblings are
  excluded from this particular selected sense.
- **`trois/ADJ:num`:** the preposed cardinal numeral at line 13 is selected;
  postposed ordinal and nominal digit/number uses remain separate. `/tʁwa/` is
  one of the two recorded IPA variants at line 12.

Le Robert corroboration pages were successfully captured on 2026-11-18 for
`dont`, `revoir`, `tard` and `puis`, under `target/issue-87/corroboration/`.
Their complete HTML/text is read-only research material; no external dictionary
prose is proposed as bundled field text. The persistent pointers are
[dont](https://dictionnaire.lerobert.com/definition/dont),
[revoir](https://dictionnaire.lerobert.com/definition/revoir),
[tard](https://dictionnaire.lerobert.com/definition/tard) and
[puis](https://dictionnaire.lerobert.com/definition/puis).

## Selected-field rights and modifications

Every candidate field binds to a fixed French revision and exact one-based raw
span. French wording removes markup and makes the selected restriction readable;
English and Chinese are attributed translations/scoped explanations by Lexique
contributors of that same selected French use. IPA selects one recorded
headword pronunciation with slash delimiters. The
[review candidates](review-candidates.json) retain each actual modification and
the source span, licence candidate and pending quality/rights status. The
longest French/IPA/English/Chinese fields are respectively **152 / 11 / 135 / 47
Unicode scalars**.

The selected source-container and adaptation licence candidate is **CC BY-SA
4.0**, with the existing [legal text](../../../resources/core-pilot-79/COPYING),
[rights authorities and credits](../../../resources/core-pilot-79/curated-inputs.json)
and [notices](../../../resources/core-pilot-79/NOTICES.md). Fixed article/history
links, selected source excerpts, Lexique contributor modification credits and
share-alike terms must accompany admitted adaptations. The converter/code
remains under the separate project MIT licence.

The 43 retained source notices include Académie eighth-edition, older Littré,
TLFi and the unselected `œil` typography CFC marker. Bibliographic references
do not establish imported-text rights by themselves. Inline external-import
markers are retained without their unselected definition prose. Modern external
quotes, unselected senses, other-language sections, audio and images are excluded.
No historical source comparison is claimed where one has not been performed.
The separate reviewer must resolve actual selected-span authorship/import
boundaries and the corresponding attribution obligations before admission.

## Exact selections

Line numbers refer to the retained raw fixed wikitext. The four-field candidate
for each ordinary group uses the displayed definition/IPA selection and its
explicit restriction; English and Chinese are scoped translations, not extracted
English/Chinese dictionary records. Row 305 lists known evidence only.

| Row | Frozen group | Selected use / gap | Fixed source revision | Definition / IPA lines |
| ---: | --- | --- | --- | --- |
| 301 | `écouter/VER` | Transitive verb: deliberately pay attention to sound. | [écouter 39873689](https://fr.wiktionary.org/w/index.php?title=%C3%A9couter&oldid=39873689) | 8 / 7 |
| 302 | `raison/NOM` | Feminine noun: the cause or motive, rather than the faculty of reasoning. | [raison 39869727](https://fr.wiktionary.org/w/index.php?title=raison&oldid=39869727) | 77 / 7 |
| 303 | `amour/NOM` | Ordinary singular masculine noun: the feeling of love. | [amour 39818440](https://fr.wiktionary.org/w/index.php?title=amour&oldid=39818440) | 13 / 12 |
| 304 | `entrer/VER` | Intransitive verb: physical movement from outside to inside. | [entrer 39847178](https://fr.wiktionary.org/w/index.php?title=entrer&oldid=39847178) | 7 / 6 |
| 305 | `dont/PRO:int` | Incomplete: obsolete interrogative source; zero selected fields | [dont 39824777](https://fr.wiktionary.org/w/index.php?title=dont&oldid=39824777) | 96 obsolete / 95 evidence only |
| 306 | `dont/PRO:rel` | Invariable relative pronoun linking a clause to an antecedent through a de complement. | [dont 39824777](https://fr.wiktionary.org/w/index.php?title=dont&oldid=39824777) | 10 / 9 |
| 307 | `fait/ADJ` | Masculine singular adjective: the resulting state of something made or carried out. | [fait 39770742](https://fr.wiktionary.org/w/index.php?title=fait&oldid=39770742) | 11 / 10 |
| 308 | `fait/NOM` | Masculine noun: an occurrence or event; selected recorded pronunciation /fɛ/. | [fait 39770742](https://fr.wiktionary.org/w/index.php?title=fait&oldid=39770742) | 156 / 148 |
| 309 | `devenir/VER` | Intransitive change-of-state verb, commonly followed by a subject complement; selected recorded full pronunciation. | [devenir 39810515](https://fr.wiktionary.org/w/index.php?title=devenir&oldid=39810515) | 35 / 34 |
| 310 | `nouveau/ADJ` | Adjective: objectively newly existing; source masculine form nouveau. | [nouveau 39824311](https://fr.wiktionary.org/w/index.php?title=nouveau&oldid=39824311) | 15 / 14 |
| 311 | `nouveau/NOM` | Masculine person noun: a newcomer to a group; source feminine counterpart nouvelle. | [nouveau 39824311](https://fr.wiktionary.org/w/index.php?title=nouveau&oldid=39824311) | 416 / 415 |
| 312 | `hein/ONO` | Informal request-for-repetition interjection; retained frozen ONO taxonomy. | [hein 39820199](https://fr.wiktionary.org/w/index.php?title=hein&oldid=39820199) | 13 / 7 |
| 313 | `commencer/VER` | Verb: initiate an action or undertaking. | [commencer 39876197](https://fr.wiktionary.org/w/index.php?title=commencer&oldid=39876197) | 7 / 6 |
| 314 | `merde/NOM` | Feminine noun, vulgar register: literal excrement. | [merde 39876765](https://fr.wiktionary.org/w/index.php?title=merde&oldid=39876765) | 10 / 9 |
| 315 | `merde/ONO` | Invariable vulgar interjection; retained frozen ONO taxonomy. | [merde 39876765](https://fr.wiktionary.org/w/index.php?title=merde&oldid=39876765) | 338 / 337 |
| 316 | `moment/NOM` | Masculine noun: a short duration, rather than a single point in time. | [moment 39819037](https://fr.wiktionary.org/w/index.php?title=moment&oldid=39819037) | 20 / 8 |
| 317 | `voiture/NOM` | Feminine noun: the ordinary automobile sense. | [voiture 39780940](https://fr.wiktionary.org/w/index.php?title=voiture&oldid=39780940) | 30 / 12 |
| 318 | `vieux/ADJ` | Adjective: something long in existence; selected source form vieux and /vjø/. | [vieux 39834397](https://fr.wiktionary.org/w/index.php?title=vieux&oldid=39834397) | 27 / 10 |
| 319 | `vieux/NOM` | Masculine person noun for an elderly person; feminine counterpart vieille. | [vieux 39834397](https://fr.wiktionary.org/w/index.php?title=vieux&oldid=39834397) | 402 / 401 |
| 320 | `revoir/NOM` | Masculine noun bound to the everyday farewell construction au revoir; source lemma remains revoir. | [revoir 39885599](https://fr.wiktionary.org/w/index.php?title=revoir&oldid=39885599) | 71 / 70 |
| 321 | `revoir/VER` | Transitive verb: seeing again. | [revoir 39885599](https://fr.wiktionary.org/w/index.php?title=revoir&oldid=39885599) | 9 / 8 |
| 322 | `elles/PRO:per` | Feminine plural subject clitic; third person. | [elles 39720213](https://fr.wiktionary.org/w/index.php?title=elles&oldid=39720213) | 8 / 7 |
| 323 | `payer/VER` | Transitive verb: monetary payment for a good or service; selected source variant /pe.je/. | [payer 39800963](https://fr.wiktionary.org/w/index.php?title=payer&oldid=39800963) | 7 / 6 |
| 324 | `fou/ADJ` | Figurative adjective for unreasonable or reckless behaviour/things; source masculine form fou. | [fou 39886219](https://fr.wiktionary.org/w/index.php?title=fou&oldid=39886219) | 32 / 11 |
| 325 | `fou/NOM` | Figurative masculine person noun, rather than a clinical classification. | [fou 39886219](https://fr.wiktionary.org/w/index.php?title=fou&oldid=39886219) | 344 / 331 |
| 326 | `tirer/VER` | Transitive physical pulling use. | [tirer 39872099](https://fr.wiktionary.org/w/index.php?title=tirer&oldid=39872099) | 10 / 8 |
| 327 | `ouvrir/VER` | Transitive verb: open something closed. | [ouvrir 39869432](https://fr.wiktionary.org/w/index.php?title=ouvrir&oldid=39869432) | 8 / 7 |
| 328 | `oeil/NOM` | Masculine anatomical noun for the ordinary organ of sight; frozen oeil retains its own identity. | [œil 39820476](https://fr.wiktionary.org/w/index.php?title=%C5%93il&oldid=39820476) | 14 / 13 |
| 329 | `changer/VER` | Intransitive verb: undergo a change. | [changer 39841310](https://fr.wiktionary.org/w/index.php?title=changer&oldid=39841310) | 9 / 8 |
| 330 | `question/NOM` | Feminine noun: an information-seeking question. | [question 39867719](https://fr.wiktionary.org/w/index.php?title=question&oldid=39867719) | 8 / 7 |
| 331 | `tomber/VER` | Intransitive verb: physical downward fall under weight. | [tomber 39903230](https://fr.wiktionary.org/w/index.php?title=tomber&oldid=39903230) | 8 / 7 |
| 332 | `assez/ADV` | Quantity adverb: sufficiency. | [assez 39748401](https://fr.wiktionary.org/w/index.php?title=assez&oldid=39748401) | 7 / 6 |
| 333 | `foutre/VER` | Very informal/vulgar lexical verb meaning do; ordinary non-reflexive use. | [foutre 39873704](https://fr.wiktionary.org/w/index.php?title=foutre&oldid=39873704) | 19 / 7 |
| 334 | `excuser/VER` | Transitive pardon/forgive use, as in excuser quelqu’un; not reflexive apologize. | [excuser 39848912](https://fr.wiktionary.org/w/index.php?title=excuser&oldid=39848912) | 15 / 7 |
| 335 | `affaire/NOM` | Feminine noun: a matter or task requiring attention. | [affaire 39810695](https://fr.wiktionary.org/w/index.php?title=affaire&oldid=39810695) | 11 / 10 |
| 336 | `dormir/VER` | Intransitive ordinary physiological sleep. | [dormir 39803288](https://fr.wiktionary.org/w/index.php?title=dormir&oldid=39803288) | 8 / 7 |
| 337 | `combien/ADV` | Interrogative adverb asking quantity or number; frozen ADV taxonomy retained. | [combien 39842448](https://fr.wiktionary.org/w/index.php?title=combien&oldid=39842448) | 7 / 6 |
| 338 | `frère/NOM` | Masculine kinship noun, selected literal same-father-and-mother scope. | [frère 39753064](https://fr.wiktionary.org/w/index.php?title=fr%C3%A8re&oldid=39753064) | 11 / 10 |
| 339 | `travail/NOM` | Masculine noun: effort and application to a task. | [travail 39877669](https://fr.wiktionary.org/w/index.php?title=travail&oldid=39877669) | 8 / 7 |
| 340 | `idée/NOM` | Feminine noun: a suggestion. | [idée 39781862](https://fr.wiktionary.org/w/index.php?title=id%C3%A9e&oldid=39781862) | 10 / 9 |
| 341 | `eh/ONO` | Admiration/surprise interjection; frozen ONO retained and recorded /ɛ/ variant selected. | [eh 39385321](https://fr.wiktionary.org/w/index.php?title=eh&oldid=39385321) | 9 / 8 |
| 342 | `puis/ADV` | Chronological succession adverb. | [puis 39772454](https://fr.wiktionary.org/w/index.php?title=puis&oldid=39772454) | 8 / 7 |
| 343 | `puis/CON` | Coordinating use under frozen CON, especially et puis; modern dictionaries may label the lexical word adverb. | [puis 39772454](https://fr.wiktionary.org/w/index.php?title=puis&oldid=39772454) | 82 / 81 |
| 344 | `famille/NOM` | Feminine collective noun: the strict parents-and-children sense. | [famille 39900679](https://fr.wiktionary.org/w/index.php?title=famille&oldid=39900679) | 8 / 7 |
| 345 | `tard/ADV` | Time adverb: after the intended or habitual time. | [tard 39874709](https://fr.wiktionary.org/w/index.php?title=tard&oldid=39874709) | 22 / 21 |
| 346 | `tard/NOM` | Masculine noun bound to sur le tard; selected evening scope of the fixed noun definition. | [tard 39874709](https://fr.wiktionary.org/w/index.php?title=tard&oldid=39874709) | 105 / 104 |
| 347 | `truc/NOM` | Informal masculine noun for an unnamed or distrusted thing. | [truc 39720290](https://fr.wiktionary.org/w/index.php?title=truc&oldid=39720290) | 22 / 11 |
| 348 | `trois/ADJ:num` | Preposed cardinal numeral under exact frozen ADJ:num; selected recorded /tʁwa/ variant. | [trois 39827448](https://fr.wiktionary.org/w/index.php?title=trois&oldid=39827448) | 13 / 12 |
| 349 | `tant/ADV` | Quantity/degree adverb. | [tant 39772732](https://fr.wiktionary.org/w/index.php?title=tant&oldid=39772732) | 8 / 7 |
| 350 | `souvenir/NOM` | Masculine noun: a retained mental impression. | [souvenir 39786873](https://fr.wiktionary.org/w/index.php?title=souvenir&oldid=39786873) | 122 / 121 |
