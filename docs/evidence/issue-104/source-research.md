# Issue #104 selected source boundaries

All 50 frozen groups have one same-use FR/IPA/EN/ZH candidate sense. Exact spelling,
grammatical section, pronunciation frame and selected definition are recorded in
[source context](source-context.json) and [excerpts](source-excerpts.json).
The four adjective/noun pairs `commandant`, `coupable`, `dingue` and `voisin`
retain separate selected-sense identities and separate POS/IPA frames.

## Source selection and attribution

The first-party Wiktionnaire API capture contains 46 French headwords. Its raw
revision text is preserved without line-number changes. Every source has a fixed
`oldid` URL, revision timestamp, page ID and full linked article history in
[sources](sources.json). The seven paginated contributor responses cover all
46 pages and retain 3,205 page-associated contributor names in the ignored
registry. This is a current registry, not proof of the author of a particular
sentence or of authorship at the fixed revision.

The original source archive under
`C:/Users/inthe/IdeaProjects/lexique-spec-90-99/target/implement-90-99/`
supplied the established capture/adaptation pattern and original-edition rights
record. Existing admitted data is not reused for these rows. The small scope and
raw-record map in
`C:/Users/inthe/IdeaProjects/lexique-d2-resource-research/target/d2-research/dictionary-modern-batch50-next19-20261119/`
were inspected: its evaluated and selected groups do not overlap #104. Its raw
candidate records are therefore not source bindings for this handoff.

## Selected constructions and meanings

| Frozen group | Selected use and retained restriction |
| --- | --- |
| 1151 `frais/NOM` | Expenses payable; masculine, plural only; definition 234 and frame 233. No adjective or `frai` plural binding. |
| 1160 `commandant/ADJ` | In command of a post, troops or warship; adjective section 5, definition 8 and frame 7. Separate from the noun and participle sections. |
| 1161 `commandant/NOM` | Person in charge of a command; noun section 16, definition 19 and frame 18; specific rank senses are excluded. |
| 1164–1165 `coupable/ADJ`, `coupable/NOM` | Guilty person as adjective versus the person noun; definitions 9 and 88, separate frames 8 and 87. |
| 1170 `hors/PRE` | Spatial exclusion; frame 9 records aspirated h, definition 10 supplies the spatial use, and derivative 54 explicitly records `hors de`. Other exclusion/conjunction uses are excluded. |
| 1174 `remonter/VER` | Intransitive going up again; definition 8 and intransitive frame 7. No transitive winding or reassembly sense. |
| 1175 `réaliser/VER` | Bringing a plan or idea into reality, definition 8. English becoming aware is excluded. |
| 1176 `moquer/VER` | `se moquer de` someone/something; pronominal frame 7 and definition 8. Frozen source headword remains `moquer`; `s’en moquer` and formal transitive uses are excluded. |
| 1180–1181 `dingue/ADJ`, `dingue/NOM` | Informal crazy/mad adjective versus informal person noun; definitions 13 and 73, separate frames 12 and 72. |
| 1184–1185 `voisin/ADJ`, `voisin/NOM` | Spatially neighbouring adjective versus nearby person/neighbour noun; definitions 8 and 76. Graph terminology is excluded. |
| 1197 `parmi/PRE` | Among/amid with a plural or collective noun, definition 10. The literary singular place-name use is excluded. |
| 1198 `langue/NOM` | Community language, definition 41. Tongue and the separate cartographic `Import:CFC` sense at 86 are excluded. |
| 1199 `contraire/NOM` | Logically opposite concept, noun definition 8 and frame 7; no adjective substitution. |

Each other group has its selected sense and exclusions in [groups](groups.json).
All source spellings equal the frozen headword. [Spelling records](field-review.json)
retain exact source bindings with no runtime aliases. Frequency strings and
original row order remain those in the accepted [scope manifest](../../../resources/core-pilot-79/scope-manifest.csv).

## Rights and modifications

Wiktionnaire contributors are credited through each fixed article and full history.
The Wiki transcription/adaptation layer and Lexique's scoped adaptations/new
translations retain [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en).
Its attribution, licence/link, modification notice and ShareAlike obligations
are recorded per source. It permits commercial reuse; no noncommercial term is
introduced. Project code retains its existing MIT licence separately.

For pages carrying `Import:DAF8` or `R:DAF8`, the Académie française's
*Dictionnaire de l’Académie française*, eighth edition, 1932–1935, is recorded
as a separate original collective-work layer. The prior #99 rights record cites
[Code de la propriété intellectuelle L123-3](https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20):
seventy years from 1 January following publication. Using final publication in
1935 gives the recorded conservative expiry of 2006-01-01. Original public-domain
prose is not relabelled CC BY-SA; Wiki and Lexique adaptations retain their own
rights. Page references do not prove the exact selected sentence's origin.
That selected-text/import correspondence remains pending independent review;
no live legal refresh is claimed.

The original API/raw text retains unrelated examples and their bibliography in
ignored evidence. Candidate fields include only selected lexical meaning and
applicable IPA, with separately bound grammatical context. Tracked notices retain
import/reference marker metadata rather than unrelated quoted example prose.
The CFC marker on the unselected cartographic `langue` sense is recorded without
copying that sense or transferring its rights to the selected community-language
definition.

FR removes Wiki markup and adapts only the selected meaning. IPA transcribes one
recorded French variant with slash delimiters. EN/ZH are new explanatory
translations of that same French meaning and its evidenced restrictions.
Each field records its modification and pending source-rights/quality statuses.

## Limitations

All 200 candidate fields need independent same-use, IPA, modern-use and
selected-text-rights assessment. Author bindings and contributor attribution
cannot supply that acceptance. No new annotation or specialist exception is
requested. Conversion, combined redistribution materials, native verification
and wider admission/release gates remain pending.
