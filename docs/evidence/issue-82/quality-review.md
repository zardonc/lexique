# Issue #82 independent quality review

Reviewed: 2026-11-17. Frozen membership: rows 51–100; source base `39e79a1`.
The reviewer is independent of the content author and application implementer.

**47 new groups / 188 exact fields are independently accepted**, together with
47 selected-use reviews and three explicit spelling relationships. The two
admitted pilot groups at rows 70–71 retain their original text and evidence.
This gives 49 batch groups with accepted or reused content. Row 87 has zero
admitted fields under the annotation exception approved on 2026-11-18 UTC;
#82 is complete within its amended scope, with 49/50 content coverage.

The acceptance payload is retained at
`target/issue-82-quality-review/accepted-review.json`. Its `fields` and `usage`
arrays use the existing accepted-review projection keys. It also contains the
47 accepted groups, selected-boundary source/context acceptance, original import
credits, exact source excerpts, three spelling review projections and the four
unresolved row-87 fields.

## Exact text and selected uses

All French definitions, English meanings, Chinese meanings and IPA values were
reviewed against the fixed raw source lines and their grammatical sections.
The selected fields preserve the specific object/subject roles, number, gender,
infinitive constructions, comparison or answer polarity, and elision conditions.
The selected noun meanings of pour, dire, pouvoir, savoir, moi and plus remain
separate from their other POS groups. The source IPA is applicable to each
selected use; plus/ADV uses the quantity definition’s explicit pronunciation.

| Frozen group | Accepted taxonomy and use boundary |
| --- | --- |
| y/NOM | Ordinary Latin-letter name, masculine and invariable. The fixed source’s letter heading is corroborated by [Académie A9Y0001](https://www.dictionnaire-academie.fr/article/A9Y0001), which labels the letter name as an invariable masculine noun. Specialist colorimetry and Cyrillic senses are excluded. |
| où/PRO:int and où/PRO:rel | Separate locative interrogative and relative uses. The exact source headings are adverbe interrogatif and adverbe relatif. [OQLF 21461](https://vitrinelinguistique.oqlf.gouv.qc.ca/21461/la-grammaire/les-pronoms/pronoms-relatifs/ou-employe-comme-pronom-relatif-ou-adverbe) recognizes relative pronoun/adverb and interrogative adverb terminology. Neutral field wording and the frozen subclasses are retained. Temporal and conclusive senses are excluded. |
| mon/ADJ:pos | Speaker’s possessive determiner, with the recorded masculine-singular and vowel/silent-h feminine-singular conditions. The source’s modern determiner heading is explicit in metadata. |
| pouvoir/VER and devoir/VER | Selected capability and obligation uses with an infinitive; fixed verb headings and modal auxiliary tags remain in source evidence. |
| t'/PRO:per, s'/CON and s'/PRO:per | Three independent subclass bindings to the fixed curly-apostrophe redirect targets. The conjunction is the elision of si before il/ils; the personal-pronoun form is the elision of se. The exact frozen apostrophes remain unchanged. |
| sur/ADJ | Unaccented sour-taste adjective with feminine sure, explicitly corroborated by the older morphology line 98. |

## Selected-source rights and original credits

The accepted redistribution route is **CC BY-SA 4.0** for the contributor text,
selection and adaptations, using the [existing full legal text](../../../resources/core-pilot-79/COPYING)
and [resource notices](../../../resources/core-pilot-79/NOTICES.md). It retains the
fixed article/version and complete contributor-history URLs, actual modification
statements and Lexique adapter credit. The produced resource must retain these
source notices, license text and attribution, apply share alike to the selected
adaptations, and preserve the license’s freedom from additional restrictions.

The review separately checked all eight declared Académie imports. Their original
credit is **Académie française, Dictionnaire de l’Académie française, huitième
édition (1932–1935)**. The fixed [import notice, revision 22113837](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AImport%3ADAF8&oldid=22113837)
identifies extracted article text and subsequent modification; the fixed
[reference template, revision 27621166](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AR%3ADAF8&oldid=27621166)
identifies the original edition and dates. This original credit is carried in the
accepted source records and source context as well as the contributor credit.

| Imported page | Retained historical revision and reviewed lines | Selected-text boundary |
| --- | --- | --- |
| pouvoir | 1621068, lines 7 and 122–124 | Capability wording survives in the verb. The current capacity-to-produce-an-effect noun definition replaces different older noun senses. |
| nous | 1629229, lines 9–10 | Speaker-inclusive first-person plural subject facts survive; the selected current sense is the subject, with object and tonic uses excluded. |
| moi | 1664436, lines 9 and 24 | Older pronoun wording and a psychoanalytic noun sense differ from the selected current tonic pronoun and amour-propre. The older noun is not the selected everyday noun use. |
| devoir | 1652303, lines 20 and 93 | Moral/legal obligation verb and duty noun wording survive in current lines 18 and 170. Original Académie credit is retained. |
| oui | 1655180, line 7 | Selected affirmative-answer/agreement facts remain; the additional discourse clauses are outside the selection. |
| comme | 1655237, lines 20 and 88 | Degree exclamation and manner comparison survive as separate selected uses. |
| venir | 1650838, line 7 | The selected movement-to-reference-place definition survives. |
| sur | 1632981, lines 98–99 | The sour-taste definition and unaccented masculine/feminine forms are evidenced. |

The selected spans are short conventional semantic/grammatical formulations and
pronunciation/spelling facts. No field selects literary example prose, an
externally attributed expressive quotation or a whole imported entry body.
The retained R:DAF8, R:TLFi and R:Littré lines remain article source context;
the acceptance concerns the selected spans. The independent field projections
explicitly record passed quality and rights eligibility at that boundary.

## Reproduction and spec result

The retained builder creates the acceptance JSON from the unchanged source draft
and the explicit reviewed row set. Its assertions check 47 exact new frozen rows,
188 unique four-field reviews, all 188 raw span matches, 47 usage reviews, 36 fixed
sources, three exact redirect bindings, eight historical import reviews, unchanged
pilot reuse records and four unresolved adjective fields. JSON readback and local
Markdown link checks pass. The source draft remains unchanged.

The spec requires “All 50 listed word/POS pairs match the frozen membership
exactly” and four accurately scoped fields for each selected use. Membership is
retained exactly; the approved row-87 annotation exception has zero admitted
fields. The independent acceptance remains 49 content groups and 196 fields.
Converter, resource and native verification are recorded separately in
the [batch execution evidence](README.md).

## Row 87: quoi / ADJ

**Approved annotation exception (2026-11-18 UTC).** The reviewed primary sources do not attest a modern everyday
adjectival use of the exact spelling `quoi`. The four fields for frozen row 87
cannot be accepted from the attested pronoun uses. The manifest retains
`quoi / ADJ`; rows 88 and 89 own its interrogative and relative pronoun uses.

The [primary-source follow-up](quoi-adj-research.md) verifies the original
Lexique annotation, its category rules and TLFi's classifications. The project
owner approved its #78/#82 scope amendment; the annotation exception has zero
admitted content fields.

| Primary source | Exact evidence | Finding |
| --- | --- | --- |
| [Frozen first-release manifest](../../../resources/core-pilot-79/scope-manifest.csv) | Manifest position 87, `word=quoi`, `lexique_cgram=ADJ` | The requested group is an adjective; changing the spelling or grammatical subclass changes membership. |
| [French Wiktionary, revision 39880332](https://fr.wiktionary.org/w/index.php?title=quoi&oldid=39880332) | Wikitext headings at lines 6, 91, 99 and 160; definitions at 9, 94, 102/106 and 163 | Interrogative pronoun, noun, relative pronoun and interjection; no adjective section. The pronunciation at line 8 belongs to the interrogative pronoun. |
| [Académie, quoi I](https://www.dictionnaire-academie.fr/article/A9Q0310), [quoi II](https://www.dictionnaire-academie.fr/article/A9Q0311), [quoi III](https://www.dictionnaire-academie.fr/article/A9Q0312), ninth edition (2024) | Entry category labels | Modern relative and interrogative pronouns, with a separate adverbial exclamation. Being a complement of an adjective does not make `quoi` an adjective. |
| [Larousse, quoi](https://www.larousse.fr/dictionnaires/francais/quoi/65796) | Entry category labels and linked homophone `coi` | Relative and interrogative/exclamative pronouns. The adjective search result is the distinct linked word `coi`; it does not evidence `quoi / ADJ`. |

The retained fixed source is
`target/issue-82-source-research/sources/fr-page-139273-revision-39880332.txt`.
The row decision and the four unresolved field names are retained in
`target/issue-82-quality-review/row87-binding.json`.

Verification: the binding JSON parses with zero accepted fields and four
unresolved fields; the manifest's row 87 matches `quoi / ADJ`. The report's
whitespace and local Markdown link checks passed.

## Limits

This conclusion establishes an admission gap in the consulted modern sources;
it does not establish the absence of every historical or dialectal use. No
pronoun definition, translation or IPA has been accepted as an adjective field.
Learner acceptance and release checks remain unrun. Produced-resource conversion,
native lookup, save/reopen and preservation are recorded in the
[batch execution evidence](README.md). The content acceptance does not establish
whole-page or whole-edition redistribution rights for unselected content.
