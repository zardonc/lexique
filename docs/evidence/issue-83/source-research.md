# D2 content batch 003: fixed source evidence

The source selections for [#83](https://github.com/zardonc/lexique/issues/83)
cover the exact frozen identities in rows **0101–0150**: **49 ordinary uses**
and the owner's specifically approved **`par/NOM` golf exception**. That noun
is restricted to the conventional stroke count for one hole. The previous
`ça/NOM` and `quoi/ADJ` decisions do not extend to other rows.

The source base is `13b7fec87ec2c5a99b10be2cd03dc20ed0b5c978`. It contains
104 accepted groups. All 50 #83 groups are additional identities; accepting
four fields for each would add 200 fields and give **154 groups / 616 fields**,
with 3,933 of the fixed 4,087 groups still unaccepted. These are prospective
counts for the completed batch. This source audit records selected uses and
source locations; final field acceptance belongs to the separate quality review.

## Fixed evidence

The source registry contains **37 French Wiktionnaire captures**: the 35
frozen headwords, plus the explicit constructions `sentir bon` and `le même`.
The existing fixed `ce` source is retained as related evidence for `cette`.
The established content cutoff is **2026-10-01 00:00:00 UTC**. Each source
records its page ID, revision, timestamp, fixed URL, complete-history URL,
raw wikitext path and API/capture provenance.

Thirteen pages were obtained through successful first-party API responses in
this audit. The first ten requests succeeded before HTTP 429 interrupted the
initial fetch; `parler`, `sentir bon` and `le même` were obtained subsequently.
The other 24 complete fixed captures were copied from the read-only research
checkout. Their SHA-256 hashes match the originals. The copied responses and
their original source registries remain identified in the new registry; the
37-page result is not a claim that every page was freshly reachable.

The machine-readable draft is
`C:\Users\inthe\IdeaProjects\lexique-83\target\issue-83\source-audit\source-draft.json`.
Its adjacent `sources/` directory retains the raw captures. The same audit
directory contains `source-index.json`, `candidate-inventory.json`,
`source-validation.json`, the construction/grammar captures and the `sur`
history comparison. The earlier research checkout remains unchanged.

The draft records 50 exact identities, definition and IPA selections, supporting
locations, source notices, grammatical relationships and the approved specialist
proposal. Existing candidate records are available for **38/50 pairs**. The
other 12 pairs now have explicit source selections: `bon/ADV`, `bon/NOM`,
`bon/ONO`, `là/ONO`, `parler/NOM`, `parler/VER`, `son/NOM`, `ton/NOM`,
`par/NOM`, `comment/ONO`, `même/PRO:ind` and `petit/ADV`. Candidate pointers
retain their original field licences and previous research status.

In the table, numbers refer to one-based lines of the retained raw wikitext.
Each reference identifies the exact article revision. The definition and IPA
normally come from the same page; the two construction mappings show their
separate sources explicitly.

| Row | Exact frozen group | Selected use | Fixed source: definition / IPA lines |
| ---: | --- | --- | --- |
| 101 | `sur/PRE` | Physical supported position, as in an object on a bed or table | [sur] 19 / 18 |
| 102 | `toi/PRO:per` | Tonic second-person singular personal pronoun; not the clitic te | [toi] 17 / 16 |
| 103 | `ici/ADV` | Place immediately near the speaker | [ici] 12 / 9 |
| 104 | `rien/PRO:ind` | Negative indefinite pronoun correlated with ne | [rien] 29 / 11 |
| 105 | `ma/ADJ:pos` | First-person possessive determiner before a feminine singular noun; selected ma form | [ma] 25 / 24 |
| 106 | `lui/PRO:per` | Third-person singular epicene indirect-object clitic without a preposition, before the verb or after an affirmative imperative; distinct from the tonic pronoun | [lui] 18 / 17 |
| 107 | `bon/ADJ` | Having the qualities expected of the thing concerned | [bon] 20 / 19 |
| 108 | `bon/ADV` | Literal adverbial use in sentir bon: the smell is pleasant | [sentir bon] 7 / [bon] 707 |
| 109 | `bon/NOM` | Voucher or coupon entitling its holder to something in exchange; noun 3 | [bon] 620 / 619 |
| 110 | `bon/ONO` | Interjection marking approval, satisfaction or acknowledgement | [bon] 688 / 681 |
| 111 | `là/ADV` | A place other than the speaker current location; excludes the separate contemporary ici-like use | [là] 10 / 9 |
| 112 | `là/ONO` | Interjection used to soothe or reassure; only that subset of the mixed soothing/threatening definition | [là] 294 / 293 |
| 113 | `suivre/VER` | Move after or behind someone or something | [suivre] 8 / 7 |
| 114 | `pourquoi/ADV` | Direct interrogative request for a reason | [pourquoi] 7 / 6 |
| 115 | `pourquoi/CON` | Introduce an indirect question asking the reason; frozen CON retained for the subordonnant function, source lexical class adverbe interrogatif | [pourquoi] 7 / 6 |
| 116 | `parler/NOM` | Manner or form of language peculiar to a group, profession or region; excludes the verb | [parler] 466 / 465 |
| 117 | `parler/VER` | Use spoken language by pronouncing or articulating words | [parler] 8 / 7 |
| 118 | `prendre/VER` | Grasp or take something with the hand | [prendre] 7 / 6 |
| 119 | `cette/ADJ:dem` | Feminine singular form of the demonstrative determiner ce; not a replacement frozen headword | [cette] 17 / 16 |
| 120 | `votre/ADJ:pos` | Possessive determiner for a singular possessed noun and plural addressees, or a single addressee addressed politely | [votre] 10 / 9 |
| 121 | `quand/CON` | Temporal conjunction introducing a subordinate clause | [quand] 8 / 7 |
| 122 | `son/ADJ:pos` | Third-person possessive determiner before a masculine singular noun beginning with a consonant; owner gender does not determine the noun form; vowel liaison excluded | [son] 19 / 18 |
| 123 | `son/NOM` | Something audible; excludes bran and the Cuban music genre | [son] 101 / 100 |
| 124 | `alors/ADV` | At the time previously mentioned; temporal use | [alors] 8 / 7 |
| 125 | `ton/ADJ:pos` | Second-person singular possessor and masculine singular possessed noun, selected before a consonant; vowel liaison excluded | [ton] 20 / 19 |
| 126 | `ton/NOM` | Quality or tone of a speaking voice, supported by ordinary speech examples; excludes musical interval and phonological tone | [ton] 96 / 91 |
| 127 | `chose/NOM` | Unspecified inanimate object; narrower than the source object/idea/abstraction definition | [chose] 10 / 9 |
| 128 | `par/NOM` | Golf: conventional number of strokes for completing a single hole; specifically approved specialist exception | [par] 295 / 294 |
| 129 | `par/PRE` | Introduce the agent complement of a passive verb | [par] 41 / 18 |
| 130 | `croire/VER` | Hold a statement or proposition to be true | [croire] 7 / 6 |
| 131 | `aimer/VER` | Like or find something pleasant; excludes romantic/sexual senses | [aimer] 43 / 6 |
| 132 | `falloir/VER` | An action is necessary, in the impersonal il faut + infinitive construction | [falloir] 8 / 7 |
| 133 | `comment/ADV` | Direct interrogative asking a manner or means | [comment] 7 / 6 |
| 134 | `comment/CON` | Introduce an indirect question asking manner or means, with savoir/dire; frozen CON retained for the subordonnant function, source lexical class adverbe | [comment] 7 / 6 |
| 135 | `comment/ONO` | Interjection requesting repetition of what was said | [comment] 164 / 163 |
| 136 | `très/ADV` | High degree immediately before an adjective or adverb | [très] 15 / 14 |
| 137 | `ou/CON` | Exclusive choice in a question; Chinese must express the interrogative choice construction | [ou] 17 / 12 |
| 138 | `passer/VER` | Time elapses; restrict the general intransitive change/elapse definition to time | [passer] 80 / 9 |
| 139 | `penser/VER` | Exercise mental activity or reflect; intransitive selected use | [penser] 8 / 7 |
| 140 | `aussi/ADV` | Addition, also or in addition; excludes equality-comparison and consequence uses | [aussi] 27 / 7 |
| 141 | `jamais/ADV` | At no time, normally correlated with ne; negative temporal adverb | [jamais] 41 / 11 |
| 142 | `même/ADJ` | Identical rather than different; adjective before a noun | [même] 10 / 9 |
| 143 | `même/ADV` | Inclusive addition or emphasis, even; not the adjective or pronoun | [même] 152 / 151 |
| 144 | `même/PRO:ind` | Indefinite pronoun preceded by an article, referring to the same person or thing; selected source le même retains the frozen même lexeme | [le même] 17 / [même] 239 |
| 145 | `attendre/VER` | Stay in place for the arrival of someone or something | [attendre] 7 / 6 |
| 146 | `petit/ADJ` | Small physical size; adjective | [petit] 11 / 10 |
| 147 | `petit/ADV` | Physical adverbial write-small construction écrire petit only; excludes dated un petit and figurative voir petit | [petit] 11 / 10 |
| 148 | `petit/NOM` | Young offspring of an animal; excludes child, tarot rank and boules target | [petit] 925 / 924 |
| 149 | `trouver/VER` | Find the person or thing one was seeking | [trouver] 7 / 6 |
| 150 | `laisser/VER` | Allow someone to act, in laisser + person + infinitive | [laisser] 83 / 6 |

## Sense, construction and spelling boundaries

The frozen Lexique subclasses remain exact. Modern source labels such as
possessive/demonstrative determiner and interrogative adverb are retained as
source taxonomy rather than silently replacing the frozen `ADJ:pos`,
`ADJ:dem` or `CON` identities.

- **`bon/ADV`** selects literal `sentir bon`: the smell is pleasant. The
  fixed [construction][sentir bon] supplies definition line 7, and [bon]
  supplies `/bɔ̃/` at line 707. [TLFi's `bon`](https://www.cnrtl.fr/definition/bon)
  and [Le Robert's `bon`](https://dictionnaire.lerobert.com/definition/bon)
  corroborate the contemporary adverbial construction. Figurative business
  prospects and unrelated idioms are outside this selection.
- **`petit/ADV`** selects `écrire petit`, writing in small letters. The
  [petit] adjective definition at line 11 supplies the small-size meaning;
  [TLFi, `petit`, III.A](https://www.cnrtl.fr/definition/petit) explicitly
  documents the adverbial `écrit petit` construction. Wiktionnaire's separate
  adverb definition at line 970 is marked *vieilli* and is excluded, as are
  TLFi III.B's dated `un petit` and figurative `voir petit`.
- **`pourquoi/CON` and `comment/CON`** select indirect reason and manner
  questions. The [OQLF GDT record, updated 2022](https://vitrinelinguistique.oqlf.gouv.qc.ca/fiche-gdt/fiche/26560285/adverbe-interrogatif)
  lists both as interrogative adverbs and explains the *subordonnant*
  function in indirect questions. This supports the frozen subordinate-clause
  mapping while preserving the contemporary lexical source label. The old
  XMLittré conjunction headings remain historical corroboration with their
  separate CC BY-SA 3.0 provenance.
- **`même/PRO:ind`** selects the article-preceded construction
  `le/la/les même(s)` and preserves the frozen headword `même`. The clean
  [le même] revision 35210050 has the pronoun heading at line 5, form table
  at lines 7–10, definition at line 17 and an explicit `même` relation at
  line 21. Its selected definition has no Reverso/import marker. The
  single-word `/mɛm/` comes from [même] line 239, before the Reverso reference.
  In that article, the selected adjective heading 7/definition 10 and adverb
  heading 148/definition 152 have no inline external attribution. The
  Alexandria reference at line 211 belongs to the unselected noun section
  beginning at line 209. The Reverso reference at line 240 introduces the
  unselected pronoun definitions at lines 241, 244 and 247. Those external
  references and definitions remain source context and are excluded from the
  selected definition.
- **`cette/ADJ:dem`** keeps `cette` as the frozen identity. Its own form
  definition and IPA are selected from [cette]; the existing [ce] revision
  39770730 supplies the related base-determiner evidence. **`toi`** selects
  the tonic second-person singular pronoun, while **`lui`** selects the
  epicene third-person indirect-object clitic without a preposition, before
  the verb or after an affirmative imperative. Its contributor notes at
  [lui] line 25 supply the exact clitic construction; they are outside the
  attributed example at lines 19–21. Possessive owner gender does not determine
  the form of `ma`, `son` or `ton`; their selected scopes and pronunciation
  contexts are recorded individually. The `son` and `ton` possessive selections
  are restricted to the before-consonant context, with vowel liaison excluded;
  [son] examples at lines 20–21 retain the consonant-onset construction.

The remaining restrictions are recorded per row: soothing `là/ONO`, the
voucher noun `bon`, audible `son`, speaking-voice `ton`, an exclusive question
with `ou`, temporal `passer`, additive `aussi`, animal-offspring `petit/NOM`
and the permissive `laisser + person + infinitive` construction. These narrow
uses prevent a translation or definition from drifting into another listed
sense or subclass.

## Approved specialist proposal

Frozen row 128 remains **`par/NOM`**, with sense ID
`par-noun-golf-standard-strokes`. The owner approved this exact specialist
scope during the implementation chat. [par] revision 39871373 has the noun
heading at line 291, `/paʁ/` at line 294 and the golf definition at line 295.
The proposal is:

| Field | Proposed selected text |
| --- | --- |
| French | Au golf, nombre conventionnel de coups nécessaires pour terminer un trou. |
| English | In golf, the conventional number of strokes needed to complete a hole. |
| Chinese | 高尔夫球中，为完成一个球洞而规定的标准击球次数。 |
| IPA | /paʁ/ |

The English and Chinese lines are new explanatory translations of the fixed
French definition. They are not presented as existing translation terms.
Selection and translation changes require modification information and full
Wiktionnaire contributor-history credit under the selected CC BY-SA 4.0 route.
The article's `R:DAF8` reference at line 346 is retained as context; that
article-level reference alone does not identify the ownership of the noun prose.

## Rights and attribution

Selected contributor prose follows the established **CC BY-SA 4.0** route,
with exact fixed-version links, complete article-history credit and recorded
selection/adaptation changes. The existing complete legal text is
[COPYING](../../../resources/core-pilot-79/COPYING), and the owning output
notices are [NOTICES.md](../../../resources/core-pilot-79/NOTICES.md). Relevant
primary rules are [CC BY-SA 4.0, §§1(a), 2(a), 3(a) and 3(b)](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en),
[Wikimedia Terms revision 598588, §7](https://foundation.wikimedia.org/w/index.php?title=Policy:Terms_of_Use&oldid=598588)
and the [French contributor policy, revision 38903168](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168).
The software's MIT licence does not change resource-text rights. Frozen
membership keeps its separate [upstream attribution](../../../resources/core-pilot-79/MEMBERSHIP-NOTICES.txt).

The registry distinguishes **explicit article import notices**, **article
reference markers** and **inline external-source references**. It retains
their actual text and one-based line locations. A page-level reference does
not establish that every selected span came from that referenced dictionary.

For **`sur/PRE`**, selected definition line 19 and the contributor example at
line 20 already appear in retained [revision 16261](https://fr.wiktionary.org/w/index.php?title=sur&oldid=16261),
dated 2004-07-25. The [DAF8 import revision 943859](https://fr.wiktionary.org/w/index.php?title=sur&oldid=943859),
dated 2006-05-15 with comment `fusion daf`, adds the sour-taste adjective
material when compared with [revision 943853](https://fr.wiktionary.org/w/index.php?title=sur&oldid=943853).
The selected preposition wording and example predate that import. The retained
`sur-earliest-history.api.json`, `sur-earliest-boundary.json` and
`sur-daf8-added-lines.txt` expose that comparison; the import notice remains
recorded alongside the current selected version.

The original `sur` French and Chinese candidate fields were **XMLittré
CC BY-SA 3.0** adaptations. Their original attribution to Émile Littré and
François Gannaz / XMLittré and their applicable obligations remain intact.
The separate fixed Wiktionnaire route in this draft does not relicense those
earlier fields. The historical `pourquoi` and `comment` corroboration uses the
owning XMLittré [p.xml](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/p.xml)
and [c.xml](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/c.xml)
at commit `b44d144d12a153799e2c8f357ea91b049f8d83e2`.

Five supporting locations are **read-only external quotations/citations**:
[pourquoi] line 16, [comment] lines 9 and 16, and [ton] lines 98 and 104.
The draft sets their `content_license` to `null`, records the CC BY-SA 4.0
source-container licence separately, states that external quotation rights
are unestablished by that licence, and sets `eligible_for_bundled_text=false`.
The selected definition and IPA spans are outside those quotation blocks.
TLFi, Le Robert and OQLF captures supply accuracy/grammar corroboration;
their prose is not proposed as bundled field text. Images, audio and whole
external dictionary bodies are outside this field selection.

## Verification

The source tools use PowerShell 7 and retained fixed captures. The draft
builder consumes the saved registry and prior candidate inventory; it performs
no live dictionary lookup. Validation results are retained in
`target/issue-83/source-audit/source-validation.json`.

| Command or check | Result |
| --- | --- |
| `./target/issue-83/source-audit/prepare-source-draft.ps1` | Passed: 50 exact groups, 49 ordinary selections, one owner-approved specialist, 37 sources, 38 prior candidate groups, zero unresolved selected-use eligibility |
| `./target/issue-83/source-audit/validate-source-draft.ps1` | Passed: 50 issue/manifest identities, unchanged 4,087 denominator, all 123 definition/IPA/support quotes, 37 source bindings and cutoff timestamps, 24 unchanged copied captures, seven retained corroboration pointers and five excluded quotation locations |
| Initial validation run | Failed on the checker's specialist-label assertion; the checker was aligned to the unchanged draft's `owner_approved_specialist_use_evidenced` and quotation-role labels, then passed |
| `./target/issue-83/source-audit/build-source-report.ps1` | Passed after correcting a format-expression error in the temporary generator; table rows and reference URLs come directly from the validated draft |
| Report row, reference and local-link checks | Passed: 50 exact row identities and locations, 38 defined fixed references, three existing local legal/notice links and base/prospective accounting; live URL reachability was not rechecked |
| `git diff --no-index --check -- target/issue-83/source-audit/empty.txt docs/evidence/issue-83/source-research.md` | Passed: no whitespace diagnostics; exit 1 denotes the new-file difference |
| Application, native, offline and release checks | Not run by the source researcher; owned by the batch implementation and execution evidence |

## Limits

Previous research reviews remain background evidence for the 38 candidate
pairs. The checks in this report establish the new selected uses, exact source
locations, retained notice context and explicitly described rights boundaries.
The final four-field review must be independent from both the source researcher
and field author/implementer. Its acceptance outcome, output conversion and
notices, application/native behavior and learner acceptance belong to their
separate batch records. The machine-readable source draft keeps
`resource_admission=false` throughout this audit.

[aimer]: https://fr.wiktionary.org/w/index.php?title=aimer&oldid=39835462
[alors]: https://fr.wiktionary.org/w/index.php?title=alors&oldid=39770736
[attendre]: https://fr.wiktionary.org/w/index.php?title=attendre&oldid=39827374
[aussi]: https://fr.wiktionary.org/w/index.php?title=aussi&oldid=39900316
[bon]: https://fr.wiktionary.org/w/index.php?title=bon&oldid=39909710
[ce]: https://fr.wiktionary.org/w/index.php?title=ce&oldid=39770730
[cette]: https://fr.wiktionary.org/w/index.php?title=cette&oldid=39770755
[chose]: https://fr.wiktionary.org/w/index.php?title=chose&oldid=39765418
[comment]: https://fr.wiktionary.org/w/index.php?title=comment&oldid=39835513
[croire]: https://fr.wiktionary.org/w/index.php?title=croire&oldid=39771272
[falloir]: https://fr.wiktionary.org/w/index.php?title=falloir&oldid=39842917
[ici]: https://fr.wiktionary.org/w/index.php?title=ici&oldid=39875304
[jamais]: https://fr.wiktionary.org/w/index.php?title=jamais&oldid=39791966
[là]: https://fr.wiktionary.org/w/index.php?title=l%C3%A0&oldid=39774017
[laisser]: https://fr.wiktionary.org/w/index.php?title=laisser&oldid=39874412
[le même]: https://fr.wiktionary.org/w/index.php?title=le%20m%C3%AAme&oldid=35210050
[lui]: https://fr.wiktionary.org/w/index.php?title=lui&oldid=39770630
[ma]: https://fr.wiktionary.org/w/index.php?title=ma&oldid=39881037
[même]: https://fr.wiktionary.org/w/index.php?title=m%C3%AAme&oldid=39900491
[ou]: https://fr.wiktionary.org/w/index.php?title=ou&oldid=39842607
[par]: https://fr.wiktionary.org/w/index.php?title=par&oldid=39871373
[parler]: https://fr.wiktionary.org/w/index.php?title=parler&oldid=39838633
[passer]: https://fr.wiktionary.org/w/index.php?title=passer&oldid=39886892
[penser]: https://fr.wiktionary.org/w/index.php?title=penser&oldid=39835489
[petit]: https://fr.wiktionary.org/w/index.php?title=petit&oldid=39879780
[pourquoi]: https://fr.wiktionary.org/w/index.php?title=pourquoi&oldid=39774019
[prendre]: https://fr.wiktionary.org/w/index.php?title=prendre&oldid=39881854
[quand]: https://fr.wiktionary.org/w/index.php?title=quand&oldid=39774209
[rien]: https://fr.wiktionary.org/w/index.php?title=rien&oldid=39869204
[sentir bon]: https://fr.wiktionary.org/w/index.php?title=sentir%20bon&oldid=39754433
[son]: https://fr.wiktionary.org/w/index.php?title=son&oldid=39878143
[suivre]: https://fr.wiktionary.org/w/index.php?title=suivre&oldid=39776790
[sur]: https://fr.wiktionary.org/w/index.php?title=sur&oldid=39823643
[toi]: https://fr.wiktionary.org/w/index.php?title=toi&oldid=39298856
[ton]: https://fr.wiktionary.org/w/index.php?title=ton&oldid=39773107
[très]: https://fr.wiktionary.org/w/index.php?title=tr%C3%A8s&oldid=39831880
[trouver]: https://fr.wiktionary.org/w/index.php?title=trouver&oldid=39894594
[votre]: https://fr.wiktionary.org/w/index.php?title=votre&oldid=39844733
