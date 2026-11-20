# Issue #110 fixed sources and authored uses

The [groups](groups.json) contain complete authored FR/IPA/EN/ZH content for all
50 exact frozen groups. [Field review](field-review.json) preserves each proposed
text and pending independent state. [Admission](admission.json) distinguishes
200 authored fields from zero accepted fields.

## Fixed capture and attribution

[Source captures](source-captures.json) identify **51 fixed French Wiktionnaire
articles**: 48 exact headwords and supporting `certain`, `celui-là` and `urgent`.
Two retained first-party API responses supply actual revision IDs, timestamps,
page IDs and raw wikitext. They are current-query captures, not a historical
cutoff claim. Every article has its fixed revision URL and complete history URL
in [sources](sources.json).

Eight contributor responses, in two sequences terminating without continuation,
retain **3,315 distinct contributor names summed across the 51 articles**.
Names shared between articles are counted separately. The full registries are
also retained in [source context](source-context.json). They are current
registries that can extend beyond a fixed revision, not proof of a selected
sentence's author. Full article histories provide contributor attribution.
Lexique's selection, adaptation and translation layer is credited separately.

Every field has an exact one-based source line in
[source excerpts](source-excerpts.json). IPA also records the position of its
exact selected variant. The isolated frozen spelling is pronounced; another POS
pronunciation, an article phrase, liaison or all regional variants are not
substituted. FR is a scoped adaptation. EN/ZH are authored explanatory
translations of that selected French use, not alleged upstream bilingual
quotations. Unknown CEFR levels remain null.

## Rights and material obligations

[Rights](rights.json) proposes **CC BY-SA 4.0** for the fixed Wiktionnaire
contribution/adaptation and new Lexique adaptations. It retains actual source
and history URLs, attribution, modifications, licence and disclaimer obligations,
and references the existing full [legal text](../../../resources/core-pilot-79/COPYING).
The parent bundles these records through the existing converter/material packer.
Project-owned code remains separately MIT.

**29 articles** contain `Import:DAF8`, including the supporting `certain` and
`urgent`. Actual raw notices and original Académie française eighth-edition
credit (1932–1935) are retained. The author reuses the established collective-work
basis in [issue #99 source context](../issue-99/source-context.json):
[Code de la propriété intellectuelle, L123-3](https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20),
with conservative expiry on 2006-01-01 using final publication in 1935. A page
import marker does not establish the original author of a selected sentence.
Wiki transcription/adaptation and Lexique adaptations retain their own
CC BY-SA 4.0 layers. Independent selected-span rights review remains pending.

Fresh author captures of the French copyright policy and DAF8 template,
CC legal text and Wikimedia Terms returned HTTP 200. The direct statute endpoint
returned HTTP 403; the inherited edition-level basis is identified rather than
claiming a live statutory check. These author captures are not independent
review. Their exact paths and fixed policy revisions are in [rights](rights.json).

Bibliographic identities remain in tracked context; unselected literary and news
examples remain in ignored raw captures. No such quotation, media, whole article,
bilingual table or unrelated-language content becomes an authored field. The
short anonymous `Une remarque entendue` example is retained as adjective
construction evidence, with its fixed article credit.

## Construction and spelling bindings

| Frozen group | Fixed evidence and selected restriction |
| --- | --- |
| 1465 `gare/ONO` | `gare` revision 39920880, lines 154–157: invariable familiar warning interjection, telling someone to move aside to let someone/something pass. The exact Lexique ONO identity is retained; it is not the railway noun or a form of `garer`. IPA `/ɡaʁ/` selects the recorded `/a/` variant. |
| 1471 `celle-là/PRO:dem` | Exact `celle-là` revision 39684070, lines 5–8: demonstrative pronoun, feminine singular of `celui-là`, with the explicit form table and `/sɛl.la/`. `celui-là` revision 39973552, lines 5–8 supplies the nonproximal referential meaning. The frozen feminine form remains the headword. |
| 1478 `certaines/ADJ:ind` | `certaines` revision 32694780, lines 4–7: exact feminine plural adjective and `/sɛʁ.tɛn/`. `certain` revision 39848777, lines 41–42 supplies indefinite-adjective semantics and the before-noun restriction. Its form table at line 6 supports the gender/number relation. The separate indefinite-pronoun section is excluded. |
| 1481 `vu/ADJ` | `vu` revision 40002800, lines 63–66: adjective meaning seen/noticed. The adjective's own `fr-accord-rég` template at line 64 supplies `/vy/`; no preposition or verb pronunciation is used in its place. The selected form is masculine singular, with `vue`, `vus`, `vues` agreement forms. |
| 1491 `entendu/ADJ` | `entendu` revision 39771282, lines 5–7 and 20–21: adjective `/ɑ̃.tɑ̃.dy/`, regular agreement and the knowing/complicit part of `Bien compris, complice`, with the anonymous adjective example. The obsolete skilled sense, organized sense, agreement interjection and literal heard participle are excluded. |

`nerveux`, `jaloux` and `entendu` retain the exact masculine-form IPA and their
captured agreement templates; `nerveuse`, `jalouse` and `entendue` are not query
aliases. `fiancé/NOM` is the man engaged to marry; the source separately gives
`fiancée` for a woman. `aveugle` retains distinct adjective and person-noun
senses despite their shared spelling and pronunciation. `marche` remains
distinct from accented `marché`. `lâche/NOM` remains distinct from the previously
admitted adjective group.

All spelling relationships are **source bindings only**, with
`alias_admission: not_admitted`. There is no substituted spelling, added runtime
alias, new annotation or owner exception.

## Other selected-use boundaries

`lâche/NOM` retains the insulting coward sense; `emmerder/VER` retains vulgar,
pejorative bothering in FR/EN/ZH. `nerveux/ADJ` selects a sensitive or irritable
person; `jaloux/ADJ` selects jealousy concerning a loved person. The shared-use
restrictions distinguish these from anatomy and envy of another's advantages.

`contrat` is the obligation-creating agreement and need not be written.
`destin` is fate beyond one's will. `urgence` selects urgency, with fixed
`urgent` evidence for no delay, rather than plural hospital `urgences`.
`folie` selects an unreasonable action or idea, avoiding historical medical
wording. `sexe` selects sexual activity. `vache` selects the domestic female
bovine, without imposing current milk production. `prêtre` selects the general
religious officiant rather than the separate fish-name section.

`exploser` and `reculer` select literal intransitive uses. `énerver`, `supplier`,
`étonner`, `placer`, `déposer`, `remplacer`, `accuser`, `nourrir`, `presser`,
`accorder`, `ranger`, `élever`, `accrocher`, `coller` and `serrer` retain their
selected transitive scopes. `accuser` does not assert proved guilt. `remplacer`
identifies the replaced thing as its direct object. `ranger/VER` uses
`/ʁɑ̃.ʒe/`, not the soldier/boot noun pronunciation. `presser` selects extracting
juice/liquid; `coller` selects joining with glue.

The remaining nouns and adjectives retain the exact selected source section and
exclusions recorded in the handoff: walking `marche`, railway `gare`, a delimited
`terrain`, ground traces `piste`, general `appareil`, factory `usine`, men's suit
`costume`, symbolic information-exchange `code`, bodily `danse`, countryside
`campagne`, purpose `intention`, handheld travel `valise`, actual `réalité` and
fast-moving `rapide`.

## Counts and execution boundary

Authored field text totals **7,774 Unicode characters**; the longest field is
**118 characters**. These are source-content measurements, not compiled-resource,
installed-package, lookup-time or native-memory measurements. No assigned exact
group already had admitted content at base `fa4ea3e`; [reuse](reuse.json) is empty
and earlier canonical records remain unchanged.

Raw APIs, blank-line-preserving wikitext, contributor pagination, rights captures,
selection/preparation scripts and RED/GREEN logs remain at
`C:/Users/inthe/IdeaProjects/lexique-batch-110/target/issue-110/`.
Author preparation verifies the API revision against raw wikitext and validates
all 200 selected bindings. Independent quality/rights review, assembled conversion,
notices packaging and actual native lookup/save/reopen remain parent-owned gates.
