# Issue #84 fixed source selections

The draft covers all **50 exact frozen groups in rows 151–200**, one selected
use and four proposed fields per group. All **200 field and 50 usage reviews
are pending**. No batch field has independent acceptance or resource admission.
The [capture index](source-index.json) identifies all 43 fixed French Wiktionnaire
versions, their complete contributor-history links and retained raw captures.

## Sources and selected text

The cutoff is **2026-10-01 00:00:00 UTC**. Eleven articles were captured from the
first-party revision API in this worktree. Thirty-two were copied from the
read-only research worktree's fixed-source registries. The retained originals
and registry paths are recorded individually; byte comparison passed for every
copy. Fresh responses were checked by title, page ID, revision, timestamp and
raw content. All 200 field spans match the retained raw one-based lines.

French text is a new clean adaptation of the selected French definition.
English and Chinese are new explanatory translations of that same selected use,
with explicit grammatical restrictions. They are not claimed as upstream
English/Chinese translation terms. IPA selects a recorded variant. The complete
proposed text and exact field bindings live in [curation](../../../resources/core-pilot-79/curated-inputs.json),
[field-review candidates](../../../resources/core-pilot-79/field-review.json) and
[selected excerpts](../../../resources/core-pilot-79/selected-source-excerpts.json).

The table's definition line also supplies the English and Chinese meaning
source. Context lines for forms and constructions are separately retained in
[selected source context](../../../resources/core-pilot-79/selected-source-context.json).

| Row | Frozen group | Selected use | Fixed source | Definition / IPA line |
| ---: | --- | --- | --- | --- |
| 151 | `merci/NOM` | Masculine noun for an expression of thanks, un merci. | [revision 39871928](https://fr.wiktionary.org/w/index.php?title=merci&oldid=39871928) | 78 / 77 |
| 152 | `merci/ONO` | Polite interjection expressing thanks; frozen ONO corresponds to the source interjection. | [revision 39871928](https://fr.wiktionary.org/w/index.php?title=merci&oldid=39871928) | 135 / 134 |
| 153 | `sa/ADJ:pos` | Third-person possessive determiner sa before a consonant-initial feminine singular noun. | [revision 39737822](https://fr.wiktionary.org/w/index.php?title=sa&oldid=39737822) | 16 / 15 |
| 154 | `autre/ADJ` | Indefinite adjective describing a different identity of the same type. | [revision 39879552](https://fr.wiktionary.org/w/index.php?title=autre&oldid=39879552) | 8 / 7 |
| 155 | `autre/PRO:ind` | Article/determiner-bearing indefinite pronoun, l’autre or un autre; the source places this use under nom, while modern corroboration calls it a pronoun. | [revision 39879552](https://fr.wiktionary.org/w/index.php?title=autre&oldid=39879552) | 163 / 149 |
| 156 | `ta/ADJ:pos` | Possessive determiner ta for a singular addressee and consonant-initial feminine singular noun. | [revision 39709095](https://fr.wiktionary.org/w/index.php?title=ta&oldid=39709095) | 20 / 19 |
| 157 | `arriver/VER` | Intransitive arrival at a destination. | [revision 39771089](https://fr.wiktionary.org/w/index.php?title=arriver&oldid=39771089) | 7 / 6 |
| 158 | `ces/ADJ:dem` | Plural demonstrative determiner ces; selected /se/ before a consonant. | [revision 39770653](https://fr.wiktionary.org/w/index.php?title=ces&oldid=39770653) | 25 / 24 |
| 159 | `donner/VER` | Transitive giving of owned property without payment. | [revision 39863324](https://fr.wiktionary.org/w/index.php?title=donner&oldid=39863324) | 11 / 10 |
| 160 | `regarder/VER` | Direct transitive visual attention, regarder quelqu’un ou quelque chose. | [revision 39808468](https://fr.wiktionary.org/w/index.php?title=regarder&oldid=39808468) | 8 / 7 |
| 161 | `encore/ADV` | Temporal adverb of continuation. | [revision 39835483](https://fr.wiktionary.org/w/index.php?title=encore&oldid=39835483) | 9 / 8 |
| 162 | `appeler/VER` | Direct transitive telephone contact. | [revision 39778943](https://fr.wiktionary.org/w/index.php?title=appeler&oldid=39778943) | 44 / 7 |
| 163 | `est-ce que/ADV` | Direct interrogative marker; frozen ADV retained, source taxonomy particule; unelided headword IPA. | [revision 39281202](https://fr.wiktionary.org/w/index.php?title=est-ce%20que&oldid=39281202) | 8 / 7 |
| 164 | `peu/ADV` | Quantity/frequency adverb meaning not much. | [revision 39905680](https://fr.wiktionary.org/w/index.php?title=peu&oldid=39905680) | 10 / 9 |
| 165 | `peu/NOM` | Masculine singular-only noun for a small quantity. | [revision 39905680](https://fr.wiktionary.org/w/index.php?title=peu&oldid=39905680) | 148 / 147 |
| 166 | `homme/NOM` | Adult male human, as distinct from a woman or child. | [revision 39856507](https://fr.wiktionary.org/w/index.php?title=homme&oldid=39856507) | 14 / 13 |
| 167 | `partir/VER` | Intransitive departure from a place. | [revision 39870672](https://fr.wiktionary.org/w/index.php?title=partir&oldid=39870672) | 25 / 7 |
| 168 | `mes/ADJ:pos` | First-person singular possessor with plural possessed noun; selected /mɛ/ before a consonant. | [revision 39812823](https://fr.wiktionary.org/w/index.php?title=mes&oldid=39812823) | 15 / 14 |
| 169 | `toujours/ADV` | Temporal adverb of continuous duration. | [revision 39879817](https://fr.wiktionary.org/w/index.php?title=toujours&oldid=39879817) | 8 / 7 |
| 170 | `jour/NOM` | Calendar day of twenty-four hours. | [revision 39720198](https://fr.wiktionary.org/w/index.php?title=jour&oldid=39720198) | 9 / 8 |
| 171 | `femme/NOM` | Adult female human, as distinct from a girl. | [revision 39908333](https://fr.wiktionary.org/w/index.php?title=femme&oldid=39908333) | 27 / 26 |
| 172 | `temps/NOM` | Temporal duration or interval between events. | [revision 39865587](https://fr.wiktionary.org/w/index.php?title=temps&oldid=39865587) | 10 / 9 |
| 173 | `maintenant/ADV` | Adverb referring to the present time; selected /mɛ̃t.nɑ̃/ variant. | [revision 39886721](https://fr.wiktionary.org/w/index.php?title=maintenant&oldid=39886721) | 10 / 9 |
| 174 | `notre/ADJ:pos` | First-person plural possessors and singular possessed noun. | [revision 39770637](https://fr.wiktionary.org/w/index.php?title=notre&oldid=39770637) | 11 / 10 |
| 175 | `vie/NOM` | Human lifetime from birth to death. | [revision 39772710](https://fr.wiktionary.org/w/index.php?title=vie&oldid=39772710) | 31 / 18 |
| 176 | `deux/ADJ:num` | Cardinal numeral before a consonant-initial noun; frozen ADJ:num. | [revision 39863990](https://fr.wiktionary.org/w/index.php?title=deux&oldid=39863990) | 11 / 10 |
| 177 | `mettre/VER` | Direct transitive placement in a specified location. | [revision 39804542](https://fr.wiktionary.org/w/index.php?title=mettre&oldid=39804542) | 9 / 8 |
| 178 | `rester/VER` | Intransitive remaining at a physical location; selected /ʁɛs.te/ variant. | [revision 39770817](https://fr.wiktionary.org/w/index.php?title=rester&oldid=39770817) | 8 / 7 |
| 179 | `sans/PRE` | Preposition before a nominal complement expressing absence. | [revision 39865923](https://fr.wiktionary.org/w/index.php?title=sans&oldid=39865923) | 10 / 9 |
| 180 | `seul/ADJ` | Adjective denoting absence of company. | [revision 39886448](https://fr.wiktionary.org/w/index.php?title=seul&oldid=39886448) | 10 / 9 |
| 181 | `seul/NOM` | Nominal masculine un seul referring to one person; source elliptical adjective use supported by uncited examples and modern noun corroboration. | [revision 39886448](https://fr.wiktionary.org/w/index.php?title=seul&oldid=39886448) | 47 / 9 |
| 182 | `arrêter/VER` | Direct transitive stopping of movement; selected /a.ʁe.te/ variant. | [revision 39835670](https://fr.wiktionary.org/w/index.php?title=arr%C3%AAter&oldid=39835670) | 8 / 7 |
| 183 | `vraiment/ADV` | Adverb confirming reality. | [revision 39773767](https://fr.wiktionary.org/w/index.php?title=vraiment&oldid=39773767) | 8 / 7 |
| 184 | `connaître/VER` | Direct transitive familiarity acquired through experience; original connaître spelling. | [revision 39700190](https://fr.wiktionary.org/w/index.php?title=conna%C3%AEtre&oldid=39700190) | 9 / 8 |
| 185 | `quelque/ADJ:ind` | Singular indefinite determiner with an uncountable noun; selected /kɛlk/ variant. | [revision 39772610](https://fr.wiktionary.org/w/index.php?title=quelque&oldid=39772610) | 22 / 7 |
| 186 | `quelque/ADV` | Approximation adverb before a numeral; selected /kɛlk/ variant. | [revision 39772610](https://fr.wiktionary.org/w/index.php?title=quelque&oldid=39772610) | 132 / 122 |
| 187 | `sûr/ADJ` | Adjective expressing a person’s certainty, être sûr de quelque chose. | [revision 39909552](https://fr.wiktionary.org/w/index.php?title=s%C3%BBr&oldid=39909552) | 78 / 10 |
| 188 | `juste/ADJ` | Adjective denoting fairness or justice. | [revision 39779681](https://fr.wiktionary.org/w/index.php?title=juste&oldid=39779681) | 11 / 10 |
| 189 | `juste/ADV` | Restrictive adverb meaning only. | [revision 39779681](https://fr.wiktionary.org/w/index.php?title=juste&oldid=39779681) | 275 / 256 |
| 190 | `juste/NOM` | Human noun denoting a person acting or judging equitably. | [revision 39779681](https://fr.wiktionary.org/w/index.php?title=juste&oldid=39779681) | 214 / 205 |
| 191 | `tuer/VER` | Literal transitive causation of death; selected broad definition line 30. | [revision 39856141](https://fr.wiktionary.org/w/index.php?title=tuer&oldid=39856141) | 30 / 6 |
| 192 | `mourir/VER` | Literal intransitive cessation of life. | [revision 39836447](https://fr.wiktionary.org/w/index.php?title=mourir&oldid=39836447) | 7 / 6 |
| 193 | `demander/VER` | Request something from a person, demander quelque chose à quelqu’un. | [revision 39884226](https://fr.wiktionary.org/w/index.php?title=demander&oldid=39884226) | 7 / 6 |
| 194 | `peut-être/ADV` | Adverb of uncertain possibility; selected first IPA variant. | [revision 39873831](https://fr.wiktionary.org/w/index.php?title=peut-%C3%AAtre&oldid=39873831) | 7 / 6 |
| 195 | `dieu/NOM` | Lowercase masculine common noun for a deity, not a proper-name substitution. | [revision 39819419](https://fr.wiktionary.org/w/index.php?title=dieu&oldid=39819419) | 12 / 11 |
| 196 | `fois/NOM` | Occurrence-count noun, une fois or deux fois. | [revision 39910928](https://fr.wiktionary.org/w/index.php?title=fois&oldid=39910928) | 10 / 9 |
| 197 | `oh/ONO` | Interjection of surprise; frozen ONO corresponds to source interjection. | [revision 39764266](https://fr.wiktionary.org/w/index.php?title=oh&oldid=39764266) | 9 / 8 |
| 198 | `père/NOM` | Social/paternal father role, including a father who is not the biological progenitor. | [revision 39868402](https://fr.wiktionary.org/w/index.php?title=p%C3%A8re&oldid=39868402) | 21 / 10 |
| 199 | `comprendre/VER` | Direct transitive comprehension of linguistic meaning. | [revision 39630355](https://fr.wiktionary.org/w/index.php?title=comprendre&oldid=39630355) | 26 / 6 |
| 200 | `sortir/VER` | Intransitive movement from inside to outside. | [revision 39556148](https://fr.wiktionary.org/w/index.php?title=sortir&oldid=39556148) | 8 / 7 |

## Grammatical and pronunciation boundaries

- `merci/NOM` uses the modern masculine expression of thanks, not the old
  feminine mercy noun. `merci/ONO` and `oh/ONO` retain the source interjection
  taxonomy and the frozen ONO identities.
- `autre/PRO:ind` uses determiner-bearing `l’autre` or `un autre`. Wiktionnaire
  puts its person/thing opposition under the noun heading; the freshly captured
  [Robert entry](https://dictionnaire.lerobert.com/definition/autre) explicitly
  labels the standalone use **pronom**. Only the Wiktionnaire definition is
  adapted; Robert prose is excluded from bundled text.
- `est-ce que/ADV` preserves the frozen ADV and records the source category
  **particule**. The selected use starts a direct question with subject before
  verb. Definition line 8 supplies the selected meaning. The elided spelling is
  attested at line 16 in a cited literary example; its prose is not bundled.
  This is a proposed taxonomy correspondence for the independent reviewer,
  not an assertion that Wiktionnaire calls it an adverb. The elision to
  `est-ce qu’` is construction context; it introduces no query alias.
- `seul/NOM` selects the masculine substantive **un seul**, meaning one person
  only. The licensed source has the elliptical use at line 47 and uncited
  contributor support at 48–49. The freshly captured
  [Robert entry](https://dictionnaire.lerobert.com/definition/seul) explicitly
  labels `un/une seul(e)` **nom**. The adjective meaning alone is not substituted
  for the noun. The source-category correspondence remains pending review.
- `peu/NOM` remains masculine and singular-only. `quelque/ADJ:ind` selects the
  small indefinite amount with a singular uncountable noun, avoiding the marked
  literary countable sense. `quelque/ADV` selects approximation before a numeral,
  not the concessive construction. `juste/NOM` selects an equitable person,
  not the religious sense or abstract justice.
- `sa` and `ta` select consonant-initial feminine singular nouns. `ces`, `mes`
  and cardinal `deux` select pronunciation before consonants; vowel liaison is
  outside these selections. `notre` selects multiple possessors and one singular
  possessed noun; accented `nôtre` is not a spelling replacement.
- `connaître` keeps its original accent and spelling. `père` selects a social
  paternal role. Lowercase `dieu` remains the common noun deity. No spelling
  alias, specialist exception or CEFR assignment is added.

## Rights and redistribution

The proposed selected/adapted text route is **CC BY-SA 4.0**, with the existing
[complete legal text](../../../resources/core-pilot-79/COPYING). Each field records
the actual selection/adaptation or explanatory translation; each article has
its fixed revision, complete history attribution and retained reference/import
markers. Declared DAF8 imports retain credit to Académie française,
*Dictionnaire de l’Académie française*, eighth edition (1932–1935). Article
reference markers remain distinguished from import declarations. A reference
marker alone does not establish ownership of a selected definition.

The final reviewer must confirm actual selected-span rights, retained imported
credit, modification notices and share alike under CC BY-SA 4.0 §§3(a)–3(b)
and [Wikimedia Terms §7](https://foundation.wikimedia.org/w/index.php?title=Policy:Terms_of_Use&oldid=598588).
The existing [resource notices](../../../resources/core-pilot-79/NOTICES.md)
and [membership notices](../../../resources/core-pilot-79/MEMBERSHIP-NOTICES.txt)
retain their separate obligations. These drafts do not relicense historical
XMLittré fields or bundle whole source pages.

Literary/journalistic quotations, audio, images, unselected definitions and
Robert/TLFi prose are excluded. The four grammar-page fetches are research
captures, not reusable field text; the two Robert entries supply the recorded
modern classification findings. The selected excerpt for `peut-être` contains
references attached to later reduced IPA variants; the proposed IPA is the
first full variant `/pø.t‿ɛtʁ/`, preceding those markers.

## Retained evidence and limits

`target/issue-84/` retains `source-index.json`, raw API/copy captures under
`sources/`, `source-inspection.txt`, `grammar/`, `source-validation.json`,
`preservation.json`, draft preparation and validation scripts, and focused logs.
Raw full pages stay research-only. The durable resource retains selected spans,
not the unselected page bodies.

All fifty groups have sourced candidates. The concrete unresolved decisions
are independent quality and selected-span rights for all 200 fields, including
the taxonomy correspondences at rows **155, 163 and 181**. No exception is
requested or recorded. Produced-resource admission and native lookup/save/restart
remain coordinator gates after the independent audit.
