# Issue #119 fixed sources and selected uses

**50 frozen groups / 49 new selected senses / 196 authored fields**, plus
unchanged accepted `côte/NOM` at row 1911. The denominator remains **4,087
groups**. All new fields have an exact source binding and await independent
semantic, selected-span rights and construction/spelling review.

## Source and rights boundaries

Five sequential requests retain the 47 distinct new frozen-headword articles.
Separate first-party captures add `continu`, `vieux`, `celui`, `le vôtre`
and `euh`. The selected source registry contains 51 French articles plus the
French interjection section of English Wiktionary’s `heu`, fixed at revision
92930671. Its line 74 explicitly records `heu` as an alternative spelling of
French `euh`; the selected modern meaning and /ø/ come from the separately
captured French `euh` article. All raw API bodies, requests, wikitext and exact
revision IDs remain under `target/issue-119/`.

[sources.json](sources.json) supplies fixed article/history URLs, contributor
attribution, licences and modifications. [source-context.json](source-context.json)
retains complete returned named registries, anonymous counts and selected
language/POS/grammar frames. Six main French contributor responses and one
French support response terminate without continuation; the separate English
`heu` registry has 43 named contributors and complete pagination. Current
registries can extend beyond the fixed revisions and do not prove the
authorship of an individual selected sentence.

The 196 new field records in [rights.json](rights.json) distinguish selected
definition/IPA wording from unselected examples, etymologies, external
references and media. Twenty-eight selected French source pages have DAF8
import markers; the established original collective-work basis credits the
Académie française’s eighth edition, 1932–1935, separately from CC BY-SA 4.0
contributor transcription/adaptation. The fixed authority captures and original
edition basis reuse the current #116/#99 evidence. Reference lists and import
markers do not establish selected-sentence authorship.

The `pistolet` CFC import occurs only in the unselected cartographic
`curvigraphe` definition at line 53. The selected firearm definition at line
17 and IPA at line 16 have no CFC wording. No attributed literary/news example,
media, other-language meaning or unselected specialist prose supplies a field.
The English `heu` article’s TLFi entry is bibliography metadata only.

French definitions are scoped lexical adaptations; EN/ZH are newly authored
explanations of those same uses. IPA selects an exact isolated variant, with
its position inside the raw source line. [NOTICES.md](NOTICES.md) and
[COPYING](COPYING) supply contribution, modification, original-work and
ShareAlike materials for the authored handoff; actual combined packaged
obligations remain a parent gate.

## Construction, spelling and inflection

`continue/ADJ` is the feminine singular of `continu`, selecting uninterrupted
duration. Both its exact feminine inflection and the lemma definition are
captured. `celles/PRO:dem` is feminine plural in the `celui/celle/ceux/celles`
paradigm; the unsuffixed use needs a relative clause or prepositional phrase.
`vieil/ADJ` is masculine singular before a vowel or silent h, selecting age
rather than the archaic unrestricted form. `vôtre/PRO:pos` is the component
of `le vôtre`: its complete table gives `le vôtre/la vôtre/les vôtres` and
plural or polite-singular `vous`. Bare obsolete adjective semantics and
unaccented `votre` are not selected.

`heu/ONO` retains the frozen spelling and separately attested modern hesitation
use of `euh`. Its English spelling record is French, not a Latin lament or
nautical noun. All five relationships are source bindings with
`alias_admission=not_admitted`; no runtime alias is added.

The two `soudain` POS groups and two `civil` POS groups remain separate.
`civil/NOM` selects civilian status, not the technical legal person category.
`saigner` selects the explicitly intransitive loss-of-blood sense despite
the broader article’s transitive label; `glisser` also selects intransitive
motion. `visiter` takes a place rather than the obsolete person-visit use;
`exiger` takes the demanded thing as direct object and retains the real or
claimed right. `apercevoir` selects visual perception, separate from
pronominal awareness constructions.

`somme` selects a feminine money amount, separate from the masculine nap.
`herbe` retains its singular collective use. `bouge`, `gaffe` and
`embêter` retain familiar register; `amant` retains its unmarried sexual
partner restriction. `patte` preserves the source’s bird-of-prey exclusion.
`poids` selects familiar mass, with the scientific imprecision explicit.
Each selected use has its own captured grammatical support and author
interpretation in [field-review.json](field-review.json).

## Frozen selected-use table

| Row | Exact headword/POS | Selected French use |
| ---: | --- | --- |
| 1901 | `singe/NOM` | Dans le sens courant, mammifère de l’ordre des primates, hors l’être humain et les lémuriens. |
| 1902 | `vision/NOM` | Action ou capacité de voir ; vue, dans le sens de perception visuelle. |
| 1903 | `malheureusement/ADV` | Par malheur ; indique qu’un fait est regrettable. |
| 1904 | `journaliste/NOM` | Personne participant à la rédaction des informations diffusées par les médias écrits ou audiovisuels. |
| 1905 | `gloire/NOM` | Renommée brillante et durable que les mérites, les grandes actions ou les grandes œuvres attirent à quelqu’un. |
| 1906 | `somme/NOM` | Quantité d’argent. |
| 1907 | `créature/NOM` | Être humain ou animal, considéré comme le résultat d’une création. |
| 1908 | `continue/ADJ` | Qui dure sans aucune interruption ; féminin singulier de continu. |
| 1909 | `midi/NOM` | La douzième heure depuis minuit. |
| 1910 | `heu/ONO` | Interjection marquant le doute ou l’incertitude ; graphie heu de euh. |
| 1911 | `côte/NOM` | Chacun des os qui forment la cage thoracique. |
| 1912 | `franchement/ADV` | Avec franchise ; sincèrement. |
| 1913 | `amant/NOM` | Homme qui a des relations amoureuses et sexuelles avec une personne avec laquelle il n’est pas marié. |
| 1914 | `champion/NOM` | Athlète qui a remporté la première place dans une compétition. |
| 1915 | `noble/ADJ` | Dont la qualité morale est grande ou hautement appréciée. |
| 1916 | `exiger/VER` | Demander quelque chose en vertu d’un droit légitime ou prétendu tel. |
| 1917 | `victoire/NOM` | Avantage remporté sur un rival ou un concurrent. |
| 1918 | `forêt/NOM` | Vaste terrain couvert de bois, avec de nombreux arbres proches les uns des autres. |
| 1919 | `chasseur/NOM` | Personne qui pratique la chasse. |
| 1920 | `pistolet/NOM` | Arme à feu courte et portative. |
| 1921 | `visiter/VER` | Aller voir en détail une ville, un monument ou un musée, par dévotion ou par curiosité. |
| 1922 | `saigner/VER` | Perdre du sang ; se dit d’une personne, d’un animal ou de la partie d’où le sang coule. |
| 1923 | `herbe/NOM` | Au singulier collectif, ensemble des plantes qui couvrent les prairies ou les pâturages et servent notamment à nourrir le bétail. |
| 1924 | `bouge/NOM` | Familier : café ou établissement sordide, sale et mal fréquenté. |
| 1925 | `identité/NOM` | Personnalité civile d’un individu, légalement reconnue ou constatée. |
| 1926 | `soudain/ADJ` | Qui est subit ou prompt. |
| 1927 | `soudain/ADV` | Tout d’un coup ; aussitôt. |
| 1928 | `aise/NOM` | État commode et agréable, notamment dans à l’aise. |
| 1929 | `glisser/VER` | Se mettre en mouvement en coulant sur une surface lisse ou le long d’un autre corps. |
| 1930 | `celles/PRO:dem` | Pronom démonstratif féminin pluriel remplaçant des personnes ou des choses déjà évoquées ; suivi d’une relative ou d’un groupe prépositionnel. |
| 1931 | `social/ADJ` | Relatif à une société, une communauté ou une collectivité. |
| 1932 | `civil/ADJ` | Relatif aux citoyens. |
| 1933 | `civil/NOM` | Statut des citoyens qui ne sont pas militaires. |
| 1934 | `vieil/ADJ` | D’un certain âge ; forme masculine singulière de vieux devant une voyelle ou un h muet. |
| 1935 | `gaffe/NOM` | Familier : bêtise. |
| 1936 | `vôtre/PRO:pos` | Dans le vôtre, pronom possessif masculin singulier : ce qui est à vous, au pluriel ou par politesse. |
| 1937 | `patte/NOM` | Membre d’un quadrupède, d’un oiseau autre qu’un oiseau de proie, ou d’un arthropode. |
| 1938 | `apercevoir/VER` | Commencer à voir. |
| 1939 | `offre/NOM` | Proposition. |
| 1940 | `frontière/NOM` | Limite séparant un État ou une contrée d’un autre État ou d’une autre contrée. |
| 1941 | `poids/NOM` | Dans le langage courant, masse d’un objet ; cet emploi est scientifiquement impropre. |
| 1942 | `couverture/NOM` | Grande pièce d’étoffe épaisse qui couvre le lit et se place au-dessus des draps. |
| 1943 | `réserver/VER` | Garder une chose ou une personne pour un autre temps, un autre usage ou une autre occasion. |
| 1944 | `univers/NOM` | Le monde entier ; l’ensemble de tout ce qui existe. |
| 1945 | `provoquer/VER` | Causer ; susciter. |
| 1946 | `durant/PRE` | Tout au long de ; placé avant l’expression de durée. |
| 1947 | `trembler/VER` | Être agité de petites secousses fréquentes. |
| 1948 | `embêter/VER` | Énerver ou importuner quelqu’un. |
| 1949 | `opinion/NOM` | Avis ou jugement que l’on a sur une chose. |
| 1950 | `confirmer/VER` | Rendre plus certaine une chose déjà donnée ou reçue pour vraie. |

## Measurements and remaining gates

The 196 new field texts contain the measured character count in
[admission.json](admission.json); these are authored-text measurements, not
database, runtime-memory or release-package measurements. [reuse.json](reuse.json)
retains `cote-noun-rib`, `fr_cote` and all four fixed field identifiers;
the existing accepted `côte/NOM` group is unchanged, with no pending copies.

There are zero author-identified unresolved content groups and no owner
exceptions, drops or substitutions. All 49 new groups / 196 fields still
await independent review. Canonical admission, combined notices/conversion,
native lookup/save/reopen, learner acceptance and offline/platform/release
validation remain parent-owned and unrun.
