# Issue #89 selected-source research

This draft covers exactly frozen rows **401–450**, retaining every original
headword and Lexique POS subclass. There are **48 new selected-use candidates /
192 proposed fields**, and **two unchanged reused groups / eight accepted fields**,
`fort/ADJ` and `fort/ADV`. No new field is independently accepted. All fifty
identities and proposed text are in [review-candidates.json](review-candidates.json).

## Fixed sources

[source-captures.json](source-captures.json) identifies 41 French Wiktionnaire
articles at the established **2026-10-01 00:00:00 UTC** cutoff. Thirteen were
captured from the first-party API in this task. Twenty-eight complete API
responses were copied byte-identically from earlier read-only research. Each
record names the original path, local API response, raw wikitext, page ID,
revision, timestamp, fixed URL and contributor-history URL. Raw captures and
execution logs remain under `target/issue-89/` in this worktree. The ten initial
successful captures and three later captures are distinguishable from the copies.

Source lines below are one-based lines in those exact raw captures. French
definitions are selected/adapted contributor prose. English and Chinese are
new scoped explanations by Lexique of the selected French use. They are not
purported upstream bilingual terms. IPA selects one explicitly recorded variant;
it does not assert exclusive pronunciation or liaison coverage.

| Row | Frozen group | Selected candidate use | Definition / IPA |
| ---: | --- | --- | --- |
| 401 | `sang/NOM` | Masculine noun: the circulating bodily fluid. | [rev. 39765267](https://fr.wiktionary.org/w/index.php?title=sang&oldid=39765267) 11 / 9 |
| 402 | `place/NOM` | Feminine noun: occupied or available physical space. | [rev. 39771477](https://fr.wiktionary.org/w/index.php?title=place&oldid=39771477) 12 / 11 |
| 403 | `espérer/VER` | Verb: hope for a desired outcome, with a noun, infinitive or clause. | [rev. 39770953](https://fr.wiktionary.org/w/index.php?title=esp%C3%A9rer&oldid=39770953) 26 / 9 |
| 404 | `gros/ADJ` | Adjective: large physical volume or circumference. | [rev. 39771576](https://fr.wiktionary.org/w/index.php?title=gros&oldid=39771576) 17 / 16 |
| 405 | `gros/NOM` | Masculine singular noun: the main part of a whole, in le gros de. | [rev. 39771576](https://fr.wiktionary.org/w/index.php?title=gros&oldid=39771576) 524 / 523 |
| 406 | `désoler/VER` | Transitive verb: cause someone great distress. | [rev. 37055393](https://fr.wiktionary.org/w/index.php?title=d%C3%A9soler&oldid=37055393) 23 / 6 |
| 407 | `eux/PRO:per` | Stressed third-person masculine plural personal pronoun; selected standalone IPA without liaison. | [rev. 39773048](https://fr.wiktionary.org/w/index.php?title=eux&oldid=39773048) 10 / 9 |
| 408 | `retrouver/VER` | Transitive verb: regain a previously lost or forgotten object. | [rev. 39771515](https://fr.wiktionary.org/w/index.php?title=retrouver&oldid=39771515) 27 / 8 |
| 409 | `ville/NOM` | Feminine noun: a substantial urban settlement. | [rev. 39818820](https://fr.wiktionary.org/w/index.php?title=ville&oldid=39818820) 12 / 11 |
| 410 | `terre/NOM` | Feminine noun: the physical ground or soil. | [rev. 39848327](https://fr.wiktionary.org/w/index.php?title=terre&oldid=39848327) 11 / 10 |
| 411 | `gagner/VER` | Transitive verb: earn something through work; work subset of the source definition. | [rev. 39771236](https://fr.wiktionary.org/w/index.php?title=gagner&oldid=39771236) 8 / 7 |
| 412 | `semaine/NOM` | Feminine noun: the seven-day calendar cycle; selected IPA variant /sə.mɛn/. | [rev. 39862519](https://fr.wiktionary.org/w/index.php?title=semaine&oldid=39862519) 8 / 7 |
| 413 | `acheter/VER` | Transitive verb: purchase something for money; selected IPA variant /a.ʃə.te/. | [rev. 39847034](https://fr.wiktionary.org/w/index.php?title=acheter&oldid=39847034) 7 / 6 |
| 414 | `longtemps/ADV` | Adverb: duration for a long time. | [rev. 39772427](https://fr.wiktionary.org/w/index.php?title=longtemps&oldid=39772427) 9 / 8 |
| 415 | `gars/NOM` | Familiar masculine noun: a boy; selected IPA variant /ɡa/. | [rev. 39518963](https://fr.wiktionary.org/w/index.php?title=gars&oldid=39518963) 11 / 10 |
| 416 | `chambre/NOM` | Feminine noun: the bedroom subset of the source habitable-room definition. | [rev. 39909790](https://fr.wiktionary.org/w/index.php?title=chambre&oldid=39909790) 11 / 9 |
| 417 | `hé/ONO` | Familiar calling interjection; restricted to close familiarity; aspirated h, IPA /e/. | [rev. 39752629](https://fr.wiktionary.org/w/index.php?title=h%C3%A9&oldid=39752629) 11 / 10 |
| 418 | `côté/NOM` | Masculine noun: right or left side of a body. | [rev. 39827549](https://fr.wiktionary.org/w/index.php?title=c%C3%B4t%C3%A9&oldid=39827549) 11 / 8 |
| 419 | `ensemble/ADV` | Adverb: people or things acting or being with one another. | [rev. 39911454](https://fr.wiktionary.org/w/index.php?title=ensemble&oldid=39911454) 10 / 9 |
| 420 | `ensemble/NOM` | Masculine noun: elements forming a whole. | [rev. 39911454](https://fr.wiktionary.org/w/index.php?title=ensemble&oldid=39911454) 157 / 156 |
| 421 | `rappeler/VER` | Transitive verb: call a person again; selected IPA variant /ʁa.ple/. | [rev. 39773210](https://fr.wiktionary.org/w/index.php?title=rappeler&oldid=39773210) 7 / 6 |
| 422 | `fort/ADJ` | Reused; existing selected use and all fields remain byte-identical | Unchanged admitted pilot record |
| 423 | `fort/ADV` | Reused; existing selected use and all fields remain byte-identical | Unchanged admitted pilot record |
| 424 | `fort/NOM` | Masculine noun: a defensive military building, as encountered in descriptions of places. | [rev. 39827775](https://fr.wiktionary.org/w/index.php?title=fort&oldid=39827775) 355 / 354 |
| 425 | `cas/NOM` | Masculine noun: an occurring or possible situation; selected IPA variant /kɑ/. | [rev. 39869835](https://fr.wiktionary.org/w/index.php?title=cas&oldid=39869835) 18 / 17 |
| 426 | `mot/NOM` | Masculine noun: an ordinary meaningful language unit; selected IPA /mo/. | [rev. 39770829](https://fr.wiktionary.org/w/index.php?title=mot&oldid=39770829) 21 / 20 |
| 427 | `seulement/ADV` | Adverb: exclusivity or limitation. | [rev. 39772714](https://fr.wiktionary.org/w/index.php?title=seulement&oldid=39772714) 7 / 6 |
| 428 | `toucher/VER` | Transitive verb: physical hand contact with an object. | [rev. 39903995](https://fr.wiktionary.org/w/index.php?title=toucher&oldid=39903995) 8 / 7 |
| 429 | `voici/PRE` | Presentative introducing a nearby person or thing to a listener. | [rev. 39869758](https://fr.wiktionary.org/w/index.php?title=voici&oldid=39869758) 7 / 6 |
| 430 | `monter/VER` | Intransitive verb: move upward to a higher location. | [rev. 39902997](https://fr.wiktionary.org/w/index.php?title=monter&oldid=39902997) 9 / 8 |
| 431 | `désolé/ADJ` | Masculine singular adjective in être désolé, the source’s weakened apology use. | [rev. 39841689](https://fr.wiktionary.org/w/index.php?title=d%C3%A9sol%C3%A9&oldid=39841689) 19 / 8 |
| 432 | `quitter/VER` | Verb: leave a place; spatial subset of the source se retirer definition. | [rev. 39753758](https://fr.wiktionary.org/w/index.php?title=quitter&oldid=39753758) 20 / 6 |
| 433 | `police/NOM` | Feminine noun: the public administration maintaining safety and order; definition 23 read with definition 14; selected IPA /pɔ.lis/. | [rev. 39823088](https://fr.wiktionary.org/w/index.php?title=police&oldid=39823088) 23 / 13 |
| 434 | `suite/NOM` | Feminine noun: a continuation of something already begun. | [rev. 39772614](https://fr.wiktionary.org/w/index.php?title=suite&oldid=39772614) 22 / 11 |
| 435 | `matin/NOM` | Masculine noun: the period from sunrise to noon. | [rev. 39863376](https://fr.wiktionary.org/w/index.php?title=matin&oldid=39863376) 16 / 9 |
| 436 | `emmener/VER` | Transitive verb: take a person from the current location to another place. | [rev. 39770690](https://fr.wiktionary.org/w/index.php?title=emmener&oldid=39770690) 7 / 6 |
| 437 | `devant/ADV` | Spatial adverb without an expressed noun complement. | [rev. 39770502](https://fr.wiktionary.org/w/index.php?title=devant&oldid=39770502) 95 / 94 |
| 438 | `devant/PRE` | Spatial preposition introducing the person or thing faced. | [rev. 39770502](https://fr.wiktionary.org/w/index.php?title=devant&oldid=39770502) 7 / 6 |
| 439 | `continuer/VER` | Transitive verb: carry on an activity already started. | [rev. 39899247](https://fr.wiktionary.org/w/index.php?title=continuer&oldid=39899247) 7 / 6 |
| 440 | `cher/ADJ` | Adjective: high monetary price. | [rev. 39858326](https://fr.wiktionary.org/w/index.php?title=cher&oldid=39858326) 18 / 9 |
| 441 | `cher/ADV` | Adverb: high monetary price, as in payer cher. | [rev. 39858326](https://fr.wiktionary.org/w/index.php?title=cher&oldid=39858326) 238 / 237 |
| 442 | `cher/NOM` | Substantivized noun in the polite or affectionate address forms mon cher and ma chère. | [rev. 39858326](https://fr.wiktionary.org/w/index.php?title=cher&oldid=39858326) 16 / 9 |
| 443 | `enfin/ADV` | Adverb: occurrence after a long wait. | [rev. 39771029](https://fr.wiktionary.org/w/index.php?title=enfin&oldid=39771029) 8 / 7 |
| 444 | `pardon/NOM` | Masculine noun: the act of forgiving. | [rev. 39871719](https://fr.wiktionary.org/w/index.php?title=pardon&oldid=39871719) 10 / 9 |
| 445 | `pardon/ONO` | Polite apology interjection. | [rev. 39871719](https://fr.wiktionary.org/w/index.php?title=pardon&oldid=39871719) 108 / 107 |
| 446 | `mari/NOM` | Masculine noun: a male spouse, without restricting the spouse’s gender. | [rev. 39770059](https://fr.wiktionary.org/w/index.php?title=mari&oldid=39770059) 10 / 9 |
| 447 | `mauvais/ADJ` | Adjective: a thing with defects; selected IPA variant /mo.vɛ/. | [rev. 39847064](https://fr.wiktionary.org/w/index.php?title=mauvais&oldid=39847064) 23 / 11 |
| 448 | `là-bas/ADV` | Spatial adverb: a distant place; selected IPA variant /la.ba/. | [rev. 39875305](https://fr.wiktionary.org/w/index.php?title=l%C3%A0-bas&oldid=39875305) 9 / 8 |
| 449 | `point/ADV` | Negative adverb used with ne in literary or regional French. | [rev. 39772006](https://fr.wiktionary.org/w/index.php?title=point&oldid=39772006) 810 / 809 |
| 450 | `point/NOM` | Masculine noun: sentence-ending punctuation; subset of the source punctuation definition. | [rev. 39772006](https://fr.wiktionary.org/w/index.php?title=point&oldid=39772006) 59 / 8 |

## Source, language and construction boundaries

`gros/NOM` selects **le gros de**, the main part of a whole, using the separate
substantivized noun section (524 / 523). Person labels, old money and weights
are excluded. `fort/NOM` is the defensive building noun; existing adjective
and adverb records are reused without adding another copy.

`eux/PRO:per` remains the masculine plural tonic pronoun, including a mixed
group; its standalone IPA excludes liaison. `hé/ONO` selects a familiar call
to someone very familiar, retaining the source familiarity restriction and
aspirated-h information. The old hierarchical wording is narrowed to close
familiarity rather than imported into the learner explanation. `pardon/ONO`
selects the polite apology interjection. Frozen ONO remains unchanged for both.

`devant/ADV` has no expressed nominal complement; `devant/PRE` introduces one.
`cher/ADJ` selects high price; `cher/ADV` selects adverbial high price in
**payer cher**. `cher/NOM` selects the address **mon cher / ma chère**.
[Le Robert, cher](https://dictionnaire.lerobert.com/definition/cher) explicitly
identifies those forms as noun use. Its grammar fact supports the noun mapping;
Wiktionnaire affection/politeness definitions 10 and 16 support the new wording.
Robert/TLFi prose is research-only and is absent from the resource fields.

`voici/PRE` selects the everyday presentative of a nearby person or thing.
Wiktionnaire calls that use a defective verb (heading 5, definition 7, IPA 6),
whereas [Le Robert, voici](https://dictionnaire.lerobert.com/definition/voici)
classifies it as a preposition. The frozen PRE identity and both source
taxonomies remain explicit. The temporal **voici + duration** source is
literary and excluded. This source-classification mapping awaits independent
acceptance; no frozen group is substituted.

`désolé/ADJ` uses the explicit adjective apology construction **être désolé**
at line 19, with the uncited contributor example at 20. It does not borrow the
separate interjection definition. `police/NOM` uses administration definition
23 with safety/order definition 14; its unrelated CFC typography text is
excluded. `point/NOM` selects sentence-ending punctuation from definition 59
and the uncited contributor example at 60; geometry and the negative adverb
remain distinct. `côté`, `cote` and `côte` are not spelling aliases.

## Rights and authorship

The source container route is **CC BY-SA 4.0**, with full article-history
credit to Wiktionnaire contributors. Lexique contributors receive credit for
selection, scoped adaptation and explanatory translations. Each proposed
field records its own fixed source/version/span, language, content-licence
route, modification information and pending review status in
[curated-inputs.json](../../../resources/core-pilot-79/curated-inputs.json).
Exact selected source prose is retained in
[selected-source-excerpts.json](../../../resources/core-pilot-79/selected-source-excerpts.json).
The shared [COPYING](../../../resources/core-pilot-79/COPYING) and
[NOTICES.md](../../../resources/core-pilot-79/NOTICES.md) retain the legal
text, attribution and share-alike route; project MIT does not relicense text.

The established fixed authorities are Wikimedia Terms revision **598588**, §7,
the French copyright convention revision **38903168**, and CC BY-SA 4.0
§§2–3. They require underlying import obligations, contributor credit and
modification/share-alike notices. The draft does not infer a selected span’s
ownership solely from a page reference or container licence.

Twenty of the 41 source pages declare **Import:DAF8**. Their actual declared
credit, **Académie française, Dictionnaire de l’Académie française, eighth
edition (1932–1935)**, is retained with marker and line locations in
[selected-source-context.json](../../../resources/core-pilot-79/selected-source-context.json).
The selected-span/import authorship boundary remains for the final rights
audit; no claim is made that every marked page is entirely a DAF8 import or
entirely new contributor text. All 192 new field rights reviews remain pending.

Reference/import markers are stored without externally attributed prose.
In particular, CFC text in `police` and `point`, literary examples, Robert/TLFi
text, images, audio and unselected definitions are excluded. The 192 selected
field spans contain no inline import or external-reference marker. Supporting
contributor spans are identified separately; corpus quotations are not admitted.

## Concrete gap

**Row 449, `point/ADV`, remains unresolved for modern everyday eligibility.**
Fixed revision **39772006**, line **810**, describes the negative adverb as
literary or regional. A concrete contemporary everyday regional use has not
been established. Its four texts are research candidates with that restriction;
they are not accepted content. No owner exception has been requested or assumed.
The original identity remains in the denominator. Noun punctuation is not a
replacement for this group.

## Source checks

`uv run --python 3.14.10 python -B -X utf8 target/issue-89/validate_sources.py`
passed: 41 exact API/page/revision/content bindings, 28 byte-identical source
copies, 192 field spans, 99 selected definition/IPA/support locations, source
languages and cutoff timestamps. These checks establish reproducible source
bindings, not independent semantic or rights approval. The result and command
log are `target/issue-89/source-validation.json` and `source-validation.log`.
