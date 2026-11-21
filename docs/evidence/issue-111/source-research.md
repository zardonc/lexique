# Batch #111 source research

The author completed one scoped everyday sense for each of the 50 frozen groups.
The selected definitions and IPA use 42 fixed Wiktionnaire articles and one
fixed French Wikipedia article. English and Chinese fields are newly authored
explanations of the selected French use. They are not upstream bilingual
quotations. Exact definitions, IPA variants, frames and source lines are in
[source-excerpts.json](source-excerpts.json); contributor and original-work
evidence is in [source-context.json](source-context.json).

## Binding decisions

- `telle/ADJ:ind` selects `tel` line 73 under the indefinite-adjective header,
  rather than the separate similarity adjective or pronoun. `tel` line 74
  explicitly has feminine singular `telle` before a noun. Exact `telle` line 6
  supplies IPA. The captured form entry and construction support bind the
  frozen spelling without a runtime alias.
- `pilote/ADJ` selects the preliminary small-scale use of the French
  `projet pilote` construction in Wikipedia revision 223861804, line 3.
  Lexique supplies its frozen ADJ classification. The captured dictionary
  `pilote` article has noun and verb entries, so no dictionary adjective label
  is claimed. Exact French `pilote` line 16 supplies IPA. The Wikipedia source
  paragraph describes feasibility testing before a larger project; the author
  scopes the three meaning fields to that construction. This combined binding
  requires the parent's separate semantic and construction review.
- `bus/NOM` selects `/bys/`, not the `/by/` of `boire` or `bu` forms. Its same
  singular/plural spelling and Canadian feminine usage are retained. The
  regional gender note is captured at line 38, including the named references
  by André Thibault and Benoît Melançon; their blog prose is not copied.
- Lowercase `italien/NOM` is the singular language name, not capitalized
  `Italien` for a person. `malheureux/NOM` selects the sufferer of illness or an
  accident, keeping masculine/same-plural spelling and feminine `malheureuse`.
- `vierge/NOM` retains the selected feminine-person and with-a-man restriction.
  Its adjective row uses the separately sourced person/sexual-experience
  sense. Neither is replaced by an unrelated intact-object or religious noun.
- `moindre` selects the comparative; definite superlative and negative
  `le moindre` constructions are excluded. `double` keeps the twice-value
  adjective and twice-amount noun separate. `faible/NOM` is the weakest part
  of a thing, not a weak person or a fondness.
- `envers` keeps its reverse-side noun separate from its relational preposition;
  `tendre` keeps food texture separate from pulling a thread/rope/surface taut;
  `futur/NOM` selects time after the present, retaining the source's English
  influence note, rather than a grammatical tense or future spouse.
- `entraîner`, `piquer`, `produire`, `menacer` and `représenter` retain the chosen
  transitive use and exclude separate pronominal or construction senses.
  `lorsque` is a temporal subordinating conjunction with one attested isolated
  schwa pronunciation; no elision or liaison alias is added.
- `bénir` invokes divine protection on people; its religious scope remains
  explicit in FR/EN/ZH. `fusil` is a shoulder-fired firearm, with no unsupported
  rifled-barrel-only restriction. `montre` is carried or worn and is not limited
  to a wristwatch.

## Source and rights evidence

All 43 source records retain page/revision IDs, timestamps, fixed URLs and full
history links. Seven complete contributor API parts cover 2,863 recorded names
across those article registries (2,857 Wiktionnaire plus six Wikipedia names;
counts are summed per article, not globally unique people). Full raw captures
are retained under `target/issue-111/sources/` in the owning author checkout.
API content is compared with retained wikitext during preparation.

The oldest captured Wikipedia revision, 147650450, records initial contributor
Amqui on 2018-04-18. It contains the same preliminary small-scale project
definition and identifies no imported original in its creation comment.
This is author evidence; it is not independent original-authorship proof.

Twenty-six DAF8 markers retain the Académie française eighth-edition original
layer and the established collective-publication/public-domain proposal.
Current Wiki adaptations, selected grammar evidence, French adaptations and
authored translations retain their separate CC BY-SA 4.0 layer. Bibliographic
identities are retained without selected literary example bodies. The actual
[licence text](COPYING) and [notices](NOTICES.md) accompany the author package;
[rights.json](rights.json) records every field and its pending separate review.

No assigned group overlaps the base's canonical admitted input.
[Reuse](reuse.json) is explicitly empty. No existing content is rewritten.
There are zero unresolved exact groups or missing authored fields. All 200
fields remain pending independent semantic/rights review and therefore remain
missing from admitted coverage. The fixed denominator is 4,087 groups;
[coverage](coverage.json) separates these states.

## Selected uses

| Frozen row | Exact group | Definition source / line | IPA source / line | Authored use |
| ---: | --- | --- | --- | --- |
| 1501 | `justice/NOM` | `justice` / 15 | `justice` / 10 | Feminine noun: the justice system as an institution or power. |
| 1502 | `défense/NOM` | `défense` / 59 | `défense` / 13 | Feminine noun expressing a prohibition; défense de + infinitive, notably on notices. |
| 1503 | `arrêt/NOM` | `arrêt` / 12 | `arrêt` / 11 | Masculine noun: an act or state of stopping. |
| 1504 | `malheureux/ADJ` | `malheureux` / 19 | `malheureux` / 7 | Adjective: masculine singular or plural malheureux; feminine malheureuse(s). |
| 1505 | `malheureux/NOM` | `malheureux` / 147 | `malheureux` / 143 | Masculine noun for a person suffering illness or an accident; malheureux has the same singular and plural spelling, feminine malheureuse. |
| 1506 | `telle/ADJ:ind` | `tel` / 73 | `telle` / 6 | Exact feminine singular indefinite adjective telle before a noun; source semantics from tel/ADJ:ind and form evidence from telle. |
| 1507 | `niveau/NOM` | `niveau` / 17 | `niveau` / 11 | Masculine noun: relative physical height. |
| 1508 | `entraîner/VER` | `entraîner` / 46 | `entraîner` / 8 | Transitive verb: entraîner quelqu’un à an exercise; accented traditional spelling, one attested IPA variant. |
| 1509 | `bus/NOM` | `bus` / 21 | `bus` / 20 | Noun for public road transport; usually masculine, regional feminine documented; singular and plural bus. |
| 1510 | `tendre/ADJ` | `tendre` / 15 | `tendre` / 11 | Adjective with the same masculine and feminine singular form, describing food texture. |
| 1511 | `tendre/VER` | `tendre` / 213 | `tendre` / 212 | Transitive verb: tendre a thread, rope or surface, making it taut. |
| 1512 | `envers/NOM` | `envers` / 69 | `envers` / 68 | Masculine noun, commonly l’envers, contrasting with the visible face. |
| 1513 | `envers/PRE` | `envers` / 9 | `envers` / 8 | Preposition expressing an attitude or relationship towards someone or something. |
| 1514 | `italien/ADJ` | `italien` / 11 | `italien` / 10 | Lowercase masculine singular adjective italien; feminine italienne. |
| 1515 | `italien/NOM` | `italien` / 136 | `italien` / 135 | Lowercase masculine noun for the language, singular-only in the selected source use. |
| 1516 | `crise/NOM` | `crise` / 21 | `crise` / 10 | Feminine noun: a dangerous, decisive situation. |
| 1517 | `moindre/ADJ` | `moindre` / 8 | `moindre` / 7 | Comparative adjective, same masculine and feminine singular form; compares size, amount or degree. |
| 1518 | `tableau/NOM` | `tableau` / 11 | `tableau` / 10 | Masculine noun for a movable painting; plural tableaux. |
| 1519 | `piquer/VER` | `piquer` / 9 | `piquer` / 8 | Transitive verb: slight piercing with a pointed object. |
| 1520 | `admettre/VER` | `admettre` / 18 | `admettre` / 7 | Transitive verb: accepting a fact as true, with a noun object or a que clause. |
| 1521 | `restaurant/NOM` | `restaurant` / 16 | `restaurant` / 10 | Masculine noun for a business selling meals. |
| 1522 | `pilote/ADJ` | `Projet pilote` / 3 | `pilote` / 16 | Postnominal adjective use in projet pilote; preliminary small-scale trial before a larger project. French construction source plus frozen Lexique ADJ classification; IPA from exact French pilote. |
| 1523 | `pilote/NOM` | `pilote` / 32 | `pilote` / 16 | Noun for an aircraft operator; pilote can denote a man or a woman. |
| 1524 | `trésor/NOM` | `trésor` / 11 | `trésor` / 9 | Masculine noun for a store of valuables. |
| 1525 | `vierge/ADJ` | `vierge` / 10 | `vierge` / 9 | Adjective with the same masculine and feminine singular form, describing a person’s sexual experience. |
| 1526 | `vierge/NOM` | `vierge` / 138 | `vierge` / 137 | Feminine noun, retaining the selected source’s female and with-a-man restriction. |
| 1527 | `produire/VER` | `produire` / 24 | `produire` / 6 | Transitive verb: producing a created thing or work. |
| 1528 | `double/ADJ` | `double` / 10 | `double` / 9 | Adjective with the same masculine and feminine singular form; twice a reference value, weight or quantity. |
| 1529 | `double/NOM` | `double` / 190 | `double` / 189 | Masculine noun, le double de a reference quantity. |
| 1530 | `tombe/NOM` | `tombe` / 11 | `tombe` / 10 | Feminine noun for a burial place containing the dead. |
| 1531 | `poche/NOM` | `poche` / 27 | `poche` / 10 | Feminine noun for a garment pocket; containing-space evidence from the preceding selected source sense. |
| 1532 | `menacer/VER` | `menacer` / 29 | `menacer` / 9 | Transitive verb: a verbal threat addressed to someone. |
| 1533 | `couple/NOM` | `couple` / 71 | `couple` / 70 | Masculine noun un couple for two people linked romantically, by marriage or sexually; no sex pairing restriction added. |
| 1534 | `gâcher/VER` | `gâcher` / 29 | `gâcher` / 9 | Transitive verb, preserving the accented frozen spelling and one recorded IPA variant. |
| 1535 | `odeur/NOM` | `odeur` / 10 | `odeur` / 9 | Feminine noun for an olfactory sensation, without a pleasant/unpleasant restriction. |
| 1536 | `futur/ADJ` | `futur` / 10 | `futur` / 9 | Masculine singular adjective futur; feminine future. |
| 1537 | `futur/NOM` | `futur` / 79 | `futur` / 73 | Masculine temporal noun; source notes influence of English, which is lexical influence rather than an identified imported quotation. |
| 1538 | `souci/NOM` | `souci` / 10 | `souci` / 9 | Masculine noun for a feeling of worry or concern. |
| 1539 | `lorsque/CON` | `lorsque` / 8 | `lorsque` / 7 | Temporal subordinating conjunction before a clause; one explicitly recorded isolated pronunciation variant. |
| 1540 | `concerner/VER` | `concerner` / 7 | `concerner` / 6 | Transitive verb; a matter or thing is the subject and the affected person or thing is the direct object. |
| 1541 | `énorme/ADJ` | `énorme` / 11 | `énorme` / 10 | Adjective with the same masculine and feminine singular form, describing physical size or bulk. |
| 1542 | `programme/NOM` | `programme` / 10 | `programme` / 9 | Masculine noun for the written details of an event or sequence. |
| 1543 | `toit/NOM` | `toit` / 9 | `toit` / 8 | Masculine noun for a building’s roof. |
| 1544 | `faible/ADJ` | `faible` / 10 | `faible` / 9 | Adjective with the same masculine and feminine singular form, describing physical strength. |
| 1545 | `faible/NOM` | `faible` / 235 | `faible` / 234 | Masculine noun for the weakest part of a thing. |
| 1546 | `représenter/VER` | `représenter` / 64 | `représenter` / 6 | Transitive verb: represent one or more people under delegated authority. |
| 1547 | `bénir/VER` | `bénir` / 19 | `bénir` / 6 | Transitive verb for invoking divine protection on people, including parents blessing children; religious context retained. |
| 1548 | `fusil/NOM` | `fusil` / 18 | `fusil` / 10 | Masculine noun for a shoulder-fired firearm; not restricted to a rifled barrel. |
| 1549 | `repas/NOM` | `repas` / 8 | `repas` / 7 | Masculine noun for the food eaten at a meal; same singular/plural spelling, one attested IPA variant. |
| 1550 | `montre/NOM` | `montre` / 46 | `montre` / 9 | Feminine noun for a portable timepiece; not restricted to a wristwatch. |
