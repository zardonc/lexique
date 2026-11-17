# D2 content batch 002: fixed source evidence

The admitted content for [#82](https://github.com/zardonc/lexique/issues/82) retains the exact
frozen identities in rows 0051–0100. It supplies **47 new senses / 188 fields**
and retains **two unchanged pilot groups / eight fields**, giving **49 groups /
196 accepted fields**. Frozen row 87, `quoi/ADJ`, remains unresolved with four
missing fields; #82 remains incomplete. The [independent binding review](quality-review.md)
records the source gap for that exact adjective group.

The source base is `39e79a1`. Its prior resource contains 57 groups, including
56 ordinary groups and the approved `ça/NOM` specialist exception. The
47 new groups give 104 accepted groups, 103 ordinary groups and 416
fields, with 3,983 of the fixed 4,087 groups unaccepted. The admitted content retains
the original membership, the prior exception and the original `bien` records.
The [curation](../../../resources/core-pilot-79/curated-inputs.json) and
[field review](../../../resources/core-pilot-79/field-review.json) own those
existing records.

## Fixed evidence

The retained primary-source captures contain **36 French Wiktionnaire pages**:
34 frozen headwords and the two explicit redirect targets `t’` and `s’`. Each
capture records the page ID, revision ID, timestamp, request URL and complete
wikitext. Content is fixed at the established cutoff, **2026-10-01 00:00:00 UTC**.
Contributor credit refers to the complete article history, rather than only the
editor of the captured revision.

The machine-readable draft is
`C:\Users\inthe\IdeaProjects\lexique-82\target\issue-82-source-research\source-draft.json`.
It contains the source registry, 47 new groups, verbatim copies of the two reuses,
188 exact field excerpts, selected-use evidence, three subclass-specific spelling
relations, article source notices and the separate pending row decision.
Full captures and API responses are in its adjacent `sources/` directory.
Line numbers below refer to that retained raw wikitext. The fixed article links
identify the actual content version.

Eight older versions are retained separately in `historical-import-context/`:
[pouvoir 1621068](https://fr.wiktionary.org/w/index.php?title=pouvoir&oldid=1621068),
[nous 1629229](https://fr.wiktionary.org/w/index.php?title=nous&oldid=1629229),
[moi 1664436](https://fr.wiktionary.org/w/index.php?title=moi&oldid=1664436),
[devoir 1652303](https://fr.wiktionary.org/w/index.php?title=devoir&oldid=1652303),
[oui 1655180](https://fr.wiktionary.org/w/index.php?title=oui&oldid=1655180),
[comme 1655237](https://fr.wiktionary.org/w/index.php?title=comme&oldid=1655237),
[venir 1650838](https://fr.wiktionary.org/w/index.php?title=venir&oldid=1650838),
and [sur 1632981](https://fr.wiktionary.org/w/index.php?title=sur&oldid=1632981).
These are the latest article revisions before 2007-01-01. They preserve older
import/authorship context for independent comparison and do not replace any
selected 2026 content version.

| Row | Exact frozen group | Selected use | Fixed source: definition / IPA lines |
| ---: | --- | --- | --- |
| 51 | `pour/NOM` | Favourable argument or advantage, including *le pour et le contre* | [39879849][pour], 527 / 526 |
| 52 | `pour/PRE` | Purpose followed by an infinitive | [39879849][pour], 32 / 9 |
| 53 | `des/ART:ind` | Plural indefinite article | [39770537][des], 18 / 17 |
| 54 | `dire/NOM` | A statement attributed to someone | [39792727][dire], 421 / 420 |
| 55 | `dire/VER` | Express something through speech | [39792727][dire], 9 / 8 |
| 56 | `pouvoir/NOM` | Capacity to produce an effect | [39819957][pouvoir], 288 / 287 |
| 57 | `pouvoir/VER` | Capability with an infinitive | [39819957][pouvoir], 15 / 14 |
| 58 | `vouloir/VER` | Intention or will to do something | [39905056][vouloir], 7 / 6 |
| 59 | `mais/CON` | Coordination expressing opposition, difference or exception | [39875963][mais], 11 / 10 |
| 60 | `me/PRO:per` | First-person singular direct object | [39891222][me], 9 / 8 |
| 61 | `nous/PRO:per` | First-person plural subject | [39909636][nous], 10 / 9 |
| 62 | `dans/PRE` | Location inside a containing space | [39880146][dans], 10 / 9 |
| 63 | `savoir/NOM` | Knowledge acquired through study or experience | [39883195][savoir], 350 / 349 |
| 64 | `savoir/VER` | Certain knowledge of a fact or thing | [39883195][savoir], 7 / 6 |
| 65 | `elle/PRO:per` | Feminine singular clitic subject | [39821966][elle], 11 / 10 |
| 66 | `du/ART:def` | Definite contraction `de + le` | [39875873][du], 32 / 31 |
| 67 | `y/NOM` | Ordinary masculine letter name *i grec* | [39880230][y], 54 / 53 |
| 68 | `y/PRO:per` | Inanimate indirect object introduced by `à` | [39880230][y], 77 / 76 |
| 69 | `bien/ADJ` | A person regarded as behaving well | [39904513][bien], 403 / 402 |
| 70 | `bien/ADV` | Satisfactory or correct manner | Unchanged admitted pilot record `bien-adv-manner`, [92967453][en-bien], line 110 |
| 71 | `bien/NOM` | A possession | Unchanged admitted pilot record `bien-noun-possession`, [92967453][en-bien], line 150 |
| 72 | `où/PRO:int` | Interrogative place, with or without movement | [39876260][ou], 11 / 10 |
| 73 | `où/PRO:rel` | Relative place, with or without movement | [39876260][ou], 183 / 182 |
| 74 | `t'/PRO:per` | `te` elision before a vowel or silent `h` | [39321617][t-elision], 9 / 8 |
| 75 | `voir/VER` | Visual perception | [39774028][voir], 7 / 6 |
| 76 | `plus/ADV` | Greater quantity, comparative of `beaucoup` | [39906941][plus], 10 / 10; explicit `/plys/` |
| 77 | `plus/NOM` | An advantage or favourable additional feature | [39906941][plus], 347 / 344 |
| 78 | `non/ADV` | Negative answer | [39831231][non], 18 / 16 |
| 79 | `te/PRO:per` | Second-person singular direct or indirect object | [39762688][te], 14 / 13 |
| 80 | `mon/ADJ:pos` | Speaker's possessive determiner | [39885403][mon], 13 / 12 |
| 81 | `au/ART:def` | Definite contraction `à + le` | [39770800][au], 25 / 24 |
| 82 | `avec/PRE` | Company or accompaniment with an expressed complement | [39771853][avec], 10 / 9 |
| 83 | `moi/NOM` | Ordinary *amour-propre*, masculine singular | [39771617][moi], 257 / 256 |
| 84 | `moi/PRO:per` | Stressed first-person singular pronoun | [39771617][moi], 19 / 18 |
| 85 | `si/ADV` | Degree or intensity | [39823641][si], 223 / 216 |
| 86 | `si/CON` | Condition or supposition | [39823641][si], 22 / 21 |
| 87 | `quoi/ADJ` | Unresolved | [39880332][quoi] attests other classes; no adjective field selected |
| 88 | `quoi/PRO:int` | Interrogative object referring to things | [39880332][quoi], 9 / 8 |
| 89 | `quoi/PRO:rel` | Relative after a preposition, referring only to things | [39880332][quoi], 102 / 101 |
| 90 | `devoir/NOM` | Duty or obligation | [39864895][devoir], 170 / 169 |
| 91 | `devoir/VER` | Obligation with an infinitive | [39864895][devoir], 18 / 8 |
| 92 | `s'/CON` | Conjunction `si` elision before `il` or `ils` | [38871517][s-elision], 30 / 29 |
| 93 | `s'/PRO:per` | Personal pronoun `se` elision before a vowel or silent `h` | [38871517][s-elision], 6 / 5 |
| 94 | `oui/ADV` | Affirmative answer or agreement | [39819001][oui], 17 / 16 |
| 95 | `ils/PRO:per` | Masculine or mixed-group plural subject | [39774275][ils], 11 and supporting 16 / 10 |
| 96 | `comme/ADV` | Exclamatory degree | [39770570][comme], 20 / 9 |
| 97 | `comme/CON` | Comparison of manner introducing a clause | [39770570][comme], 211 / 207 |
| 98 | `se/PRO:per` | Third-person reflexive object | [39823639][se], 16 and supporting 14–15 / 14 |
| 99 | `venir/VER` | Movement to the speaker's or discussed person's location | [39871985][venir], 7 / 6 |
| 100 | `sur/ADJ` | Acidic or sour taste, feminine `sure` | [39823643][sur], 334 / 333 |

## Sense and spelling boundaries

Every new French definition is a scoped adaptation of the selected contributor
definition. Its English and Chinese fields translate or explain that same use;
they do not import an independent commercial dictionary translation. The curation
records actual modifications and preserves number, gender, reference, construction
and elision restrictions. Each IPA points to the fixed source line applicable to
the selected use.

The noun `y` uses the ordinary Latin-letter entry, whose source heading is
`lettre` with masculine/invariable morphology. Its separate colorimetry and
Cyrillic entries are outside this selection. Both `où` uses retain their exact
frozen pronoun subclasses while exposing the contributor headings
`adverbe interrogatif` and `adverbe relatif` for the independent taxonomy review.
The modern labels `déterminant possessif` for `mon` and modal `auxiliaire` within
the `pouvoir`/`devoir` definitions are likewise retained in context. These source
labels do not replace the frozen categories.

The three explicit spelling relationships are separate records:

| Frozen group | Fixed redirect | Selected target section |
| --- | --- | --- |
| `t'/PRO:per` | [`t'`, revision 451737][t-redirect], line 1 → `t’` | [39321617][t-elision], personal pronoun, lines 7–9 |
| `s'/CON` | [`s'`, revision 451738][s-redirect], line 1 → `s’` | [38871517][s-elision], conjunction, lines 28–30 |
| `s'/PRO:per` | [`s'`, revision 451738][s-redirect], line 1 → `s’` | [38871517][s-elision], personal pronoun, lines 4–6 |

The frozen apostrophes remain unchanged. The `s’` conjunction refers to `si`,
whereas its personal-pronoun section refers to `se`. The obsolete possessive
elisions and colloquial consonant elisions are excluded. The bound `/s‿/`
pronunciation remains explicitly linked. `sur/ADJ` uses the unaccented sour-taste
adjective and its feminine `sure`, not the separate accented word `sûr`.

## Rights and attribution

The accepted selected/adapted text follows the existing **CC BY-SA 4.0** route,
with Wiktionnaire contributor history, fixed article/version links and Lexique's
recorded selection/adaptations. The existing complete legal text is
[COPYING](../../../resources/core-pilot-79/COPYING); the owning reuse notices are
[NOTICES.md](../../../resources/core-pilot-79/NOTICES.md). The elected route is
supported by [CC BY-SA 4.0, §§1(a), 2(a), 3(a) and 3(b)](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en),
[Wikimedia Terms revision 598588, §7](https://foundation.wikimedia.org/w/index.php?title=Policy:Terms_of_Use&oldid=598588),
and the [French contributor policy, revision 38903168](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168).

Article-level `Import:DAF8` notices are retained for `pouvoir`, `nous`, `moi`,
`devoir`, `oui`, `comme`, `venir` and `sur`. The [fixed import template, revision
22113837](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AImport%3ADAF8&oldid=22113837)
states that all or part of an article was extracted from the referenced eighth
Académie edition and may have been modified since. The source context also preserves
applicable `R:DAF8`, `R:TLFi` and `R:Littré` reference lines. These records expose
source boundaries for the selected-text rights review. Literary/example
quotations, whole imported dictionary bodies, images and audio are outside the
selected fields. The frozen Lexique membership keeps its separate existing
upstream attribution and notices.

The [fixed `R:DAF8` template, revision 27621166](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AR%3ADAF8&oldid=27621166)
identifies the eighth edition as 1932–1935 and links its Académie text at CNRTL.
Its complete primary API response is retained in `rights/reference-daf8.api.json`.
The fixed French copyright-policy JSON, rendered HTML and text excerpt are also
retained in `rights/`, using the already captured revision 38903168. That policy
distinguishes contributor text from external-source exceptions and requires
compatible conditions when external text is imported directly.

## Verification

Windows source tools used PowerShell 7 and Python 3.14.10 through `uv`, with
their cache confined to the research directory. Full logs are retained beside
the draft.

| Command or check | Result |
| --- | --- |
| `fetch_sources.py` | Passed: 36 fixed page/revision captures, 34 requested frozen headwords, two explicit redirect targets; responses contain real source text |
| `inspect_sources.py` | Passed: numbered French headings, definitions, IPA and source notices retained in `french-source-lines.json` |
| `build_source_draft.py` | Passed: 47 exact new frozen rows, 188 fields, 47 unique sense IDs, two unchanged reuses, complete selected spans and three exact redirect bindings |
| Fixed `Import:DAF8` template capture | Passed: revision 22113837 and its complete original notice retained in `rights/import-daf8.api.json` |
| `fetch_import_history.py` | Passed: eight fixed pre-2007 versions retained separately from current selected content; the field draft is unchanged |
| Report membership and local links | Passed: 50 exact row numbers, five existing local links and 37 defined source references; whitespace comparison produced no diagnostics |

The draft builder consumes the retained fixed captures and current frozen
manifest; it performs no live dictionary lookup. Its assertions detect missing
or misaddressed definition/IPA spans, altered membership, duplicate sense IDs,
changed reused source versions and incorrect redirect relationships.

## Limits

The [independent review](quality-review.md) accepts the 188 new fields, selected
taxonomy, spelling relationships and selected-text/import-boundary rights.
It establishes the consulted-source gap for `quoi/ADJ`, rather than the absence
of every possible historical or dialectal use. Learner acceptance remains unrun.
Produced-resource notices, conversion, application checks and native behavior
are recorded in the [batch execution evidence](README.md).

[pour]: https://fr.wiktionary.org/w/index.php?title=pour&oldid=39879849
[des]: https://fr.wiktionary.org/w/index.php?title=des&oldid=39770537
[dire]: https://fr.wiktionary.org/w/index.php?title=dire&oldid=39792727
[pouvoir]: https://fr.wiktionary.org/w/index.php?title=pouvoir&oldid=39819957
[vouloir]: https://fr.wiktionary.org/w/index.php?title=vouloir&oldid=39905056
[mais]: https://fr.wiktionary.org/w/index.php?title=mais&oldid=39875963
[me]: https://fr.wiktionary.org/w/index.php?title=me&oldid=39891222
[nous]: https://fr.wiktionary.org/w/index.php?title=nous&oldid=39909636
[dans]: https://fr.wiktionary.org/w/index.php?title=dans&oldid=39880146
[savoir]: https://fr.wiktionary.org/w/index.php?title=savoir&oldid=39883195
[elle]: https://fr.wiktionary.org/w/index.php?title=elle&oldid=39821966
[du]: https://fr.wiktionary.org/w/index.php?title=du&oldid=39875873
[y]: https://fr.wiktionary.org/w/index.php?title=y&oldid=39880230
[bien]: https://fr.wiktionary.org/w/index.php?title=bien&oldid=39904513
[en-bien]: https://en.wiktionary.org/w/index.php?title=bien&oldid=92967453
[ou]: https://fr.wiktionary.org/w/index.php?title=o%C3%B9&oldid=39876260
[t-elision]: https://fr.wiktionary.org/w/index.php?title=t%E2%80%99&oldid=39321617
[voir]: https://fr.wiktionary.org/w/index.php?title=voir&oldid=39774028
[plus]: https://fr.wiktionary.org/w/index.php?title=plus&oldid=39906941
[non]: https://fr.wiktionary.org/w/index.php?title=non&oldid=39831231
[te]: https://fr.wiktionary.org/w/index.php?title=te&oldid=39762688
[mon]: https://fr.wiktionary.org/w/index.php?title=mon&oldid=39885403
[au]: https://fr.wiktionary.org/w/index.php?title=au&oldid=39770800
[avec]: https://fr.wiktionary.org/w/index.php?title=avec&oldid=39771853
[moi]: https://fr.wiktionary.org/w/index.php?title=moi&oldid=39771617
[si]: https://fr.wiktionary.org/w/index.php?title=si&oldid=39823641
[quoi]: https://fr.wiktionary.org/w/index.php?title=quoi&oldid=39880332
[devoir]: https://fr.wiktionary.org/w/index.php?title=devoir&oldid=39864895
[s-elision]: https://fr.wiktionary.org/w/index.php?title=s%E2%80%99&oldid=38871517
[oui]: https://fr.wiktionary.org/w/index.php?title=oui&oldid=39819001
[ils]: https://fr.wiktionary.org/w/index.php?title=ils&oldid=39774275
[comme]: https://fr.wiktionary.org/w/index.php?title=comme&oldid=39770570
[se]: https://fr.wiktionary.org/w/index.php?title=se&oldid=39823639
[venir]: https://fr.wiktionary.org/w/index.php?title=venir&oldid=39871985
[sur]: https://fr.wiktionary.org/w/index.php?title=sur&oldid=39823643
[t-redirect]: https://fr.wiktionary.org/w/index.php?title=t%27&oldid=451737
[s-redirect]: https://fr.wiktionary.org/w/index.php?title=s%27&oldid=451738
