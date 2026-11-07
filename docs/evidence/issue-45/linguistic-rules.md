# Validated conjugation coloring rules

The coloring rules describe linguistic endings over the admitted complete forms.
They do not read a model's cut, compressed suffix, shared stem, or longest common
prefix. Python's public CLI and the rebuilt Rust reader use the same
[ending rules](../../../scripts/conjugation_endings.json). Offsets count Unicode
characters in the original displayed spelling, with an exclusive end.

## Span convention and independent expectations

The main verbal word excludes subject/reflexive pronouns, imperative pronoun
suffixes, the subjonctive conjunction and the gerund marker `en`. A compound has
an auxiliary span and a main participle span; an intervening pronoun stays plain.
Adjacent auxiliary and main spans include their separating space. Expressions
retain their original form as a suffix, so subject/conjunction prefixes shift
the validated spans without changing the form.

For regular verbs, the selected tense determines the ending. The present uses
`e/es/e` for group 1 and `s/s/t` for group 2, and `ons/ez/ent` in the plural.
The second group's present stem therefore includes `i` or `iss`. Imparfait uses
`ais/ais/ait/ions/iez/aient`; subjonctif présent uses
`e/es/e/ions/iez/ent` on its admitted stem. Future and conditional endings begin
with `r`, leaving the preceding thematic `e` or `i` plain. Passé simple and
subjonctif imparfait include their tense-specific vowel in the ending; infinitive
and past-participle endings follow the explicit regular family. These are
tense-dependent endings, rather than a single subtraction from the infinitive.

| Independent reference | Rule / literal check |
| --- | --- |
| [OQLF present endings](https://vitrinelinguistique.oqlf.gouv.qc.ca/24213/la-grammaire/le-verbe/conjugaison/terminaison-des-verbes-reguliers-et-irreguliers-au-present-de-lindicatif) and [Hatier present endings](http://moncoach.bescherelle.com/exercices/fran%C3%A7ais/conjuguer/utiliser-la-terminaison-qui-convient-au-pr%C3%A9sent) | `finis` → `s`; `finissons` → `ons`; `lave` → `e`. Stem changes do not enter the ending. |
| [OQLF futur simple](https://vitrinelinguistique.oqlf.gouv.qc.ca/24655/la-grammaire/le-verbe/conjugaison/formes-du-futur-simple) and [conditionnel](https://vitrinelinguistique.oqlf.gouv.qc.ca/24137/la-grammaire/le-verbe/conjugaison/formes-du-conditionnel) | `laverai`, `paierai`, `payerai`, `finirai` → `rai`; `laverions` / `finirions` → `rions`. |
| [Hatier passé simple](https://www.bescherelle.com/faq/quelles-sont-les-terminaisons-du-passe-simple) | `lavâmes` → `âmes`; `finîmes` → `îmes`. |
| [Pinned Lefff morphology](../../../resources/lefff-syntax/morpho.fr), tables `v-er`, `v-ir2`, `v-haïr`, `v109` | Full simple-tense, imperative, infinitive, present/past participle families; `lavassions` → `assions`, `finît` → `ît`, `lavées` → `ées`, `finies` → `ies`, `haïmes` → `ïmes`. |
| [Larousse rassir](https://www.larousse.fr/conjugaison/francais/rassir/9860) | Traditional group 2; explicit participial alternatives `rassi/rassis`, `rassie/rassise`, `rassis`, `rassies/rassises`, corroborating Lefff `v109`. Only admitted alternatives are rendered. |
| [#45](https://github.com/zardonc/lexique/issues/45) | `nous avions lavé`: verbal `[5,16)`, main `[12,16)`, ending `[15,16)`; `lavais`: ending `[3,6)`. |

The CLI checks retain `paie/paye`, accent/stem changes in `mangeais`, `mangeons`
and `plaçais`, feminine/plural participles, `lave-toi`, `sois-toi lavé` and
`en se lavant`. No pronoun, auxiliary in mode 3, or whole-word substitute enters
an ending span. Every third-group/irregular identity has a null ending in every
cell, including recognizable endings. Modes 1 and 2 use the verbal/main spans
for every category.

## Classification and retained source forms

The [cited corrections](../../../resources/grammalecte-2.3.0/corrections.json)
identify `mûrir` as group 2, consistent with
[the Académie](https://www.dictionnaire-academie.fr/conjuguer/A9M3201) and Lefff
`v-ir2`. `messeoir` is group 3, consistent with
[Larousse](https://www.larousse.fr/conjugaison/francais/messeoir/11745) and Lefff
`v47`. `reparcourir` is group 3, supported by its admitted `v-courir` pattern and
`reparcourons/reparcouru`, independently checked against
[Scholingua](https://www.scholingua.com/fr/fr/conjugaison/reparcourir). Original
raw flags remain traceable. These annotations change classification only.

The retained unassigned source symbols under `précéder` and `égaler` have no
linguistic ending. Lefff also retains `courbatu/courbatue/courbatus/courbatues`
under `courbaturer`. The
[Académie's distinction](https://www.dictionnaire-academie.fr/article/QDL025)
identifies `courbatu` as an adjective and `courbaturé` as the participle of
`courbaturer`; an `-u` ending for this first-group verb is not inferred. These
source forms and their existing derived expressions remain queryable, with a
plain ending and an unconfirmed-ending cue. The public build report accounts
for these source-only cases separately from validated regular cells and
unresolved regular rules.

## Source, implementation and licensing

No additional conjugation corpus or executable upstream code is imported.
The existing admitted Lefff/Grammalecte construction rules supply the complete
forms and metadata, and preserve source-line anchors. The shared regular rule
table is a factual projection of LGPL-LR Lefff morphology and retains that
license. Conversion and rendering code remain MIT; Grammalecte metadata keeps
GPL-3.0-only. The cited grammar/dictionary pages supply independent expectations,
not copied definitions or redistributed tables. Packages retain the original
sources, their separate notices and the matching converters and ending table.
