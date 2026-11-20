# Issue #101 selected-source research

All frozen rows **1001–1050** keep their exact spelling, POS subclass, ordering
and frequency strings. [Groups](groups.json) select one precise use for each of
49 groups, with four independently accepted fields per sense. The original candidates appear in
[review-candidates.json](review-candidates.json). Unknown CEFR values remain null.

## Fixed sources and original layers

[Source captures](source-captures.json) retain 46 real Wiki pages: 42 exact
ticket headwords and four supporting spellings, `m’`, `m'`, `me` and `s’`.
Original first-party API response bytes and each complete revision are retained
under `C:/Users/inthe/IdeaProjects/lexique-batch-101/target/issue-101/sources/`.
The revisions are the actual returned versions, with their own timestamps;
they are not described as October-cutoff snapshots. No source ID is reused from
another batch; source IDs start with `b101_` and sense IDs with `b101-`.

[Source excerpts](source-excerpts.json) contain the exact selected definition
lines and IPA templates with one-based lines/columns. Supporting bindings
retain actual French-language, POS, spelling and construction frames.
[Source context](source-context.json) records page-level `Import:DAF8` notices,
bibliographic identities, original-author layers and actual selected locations.
An import marker identifies a declared page layer; it does not prove which
selected sentence originated there. Complete quotations remain in ignored raw
evidence; only their bibliographic metadata is tracked.

French definitions are scoped adaptations of the selected French semantics.
EN/ZH are new explanatory translations of those same uses, credited to Lexique
contributors. They are not claimed to be upstream bilingual glosses. IPA selects
one applicable recorded variant, using slash notation; letter names, consonant
sounds and homographic verb pronunciations remain distinct. No audio or image
is selected.

The proposed adaptation route is **CC BY-SA 4.0**, retaining fixed-article and
complete-history contributor credit, the [full legal text](../../../resources/core-pilot-79/COPYING),
licence URLs, original-source notices and explicit modification credit. The
local primary legal captures include CC BY-SA 4.0, fixed Wikimedia Terms of Use
revision 598588, and Wiki copyright revision 38903168. Wikimedia Terms section
7 distinguishes contributor licensing, imported text, attribution and reuse;
container licensing is not proof of selected-text eligibility.

For declared DAF8 layers, source metadata proposes the existing admitted
[issue-99 edition-level basis](../issue-99/sources.json): Académie française,
eighth edition, 1932–1935, collective work; article L123-3 gives seventy years
from the year after publication, with conservative expiry 2006-01-01. Wiki
transcription/adaptation and Lexique adaptations remain separately credited
under CC BY-SA 4.0. The current statute request returned HTTP 403; this package
does not claim a fresh primary-statute capture or newly fetched statutory basis. The independent selected-span decision is recorded in
[independent review](independent-review.md).

The current contributor-registry request returned HTTP 429, retained in
`target/issue-101/contributors.log`. Complete linked article histories are the
proposed attribution route permitted by Wikimedia Terms 7(b). Neither a current
registry nor a fixed revision's last editor would establish selected-sentence
authorship. The independent review records each selected span's actual
original/contributor layers and combined obligations.

The explorer's `dictionary-modern-batch50-next19-20261119` small assigned-scope
and original-record map were inspected. Its 50 evaluated groups have no overlap
with this ticket. The package contributes no field evidence here.

## Selected-use boundaries

`déjeuner/NOM` (1011) and `déjeuner/VER` (1012) select the **midday meal in
France**. The noun's fixed line 15 mixes midday and morning wording; its first
midday meaning is selected, while fixed line 22 and the verb's line 219 retain
regional breakfast as excluded context. The separately captured Robert entry
corroborates the modern midday use. Its proprietary prose does not supply fields.

`est/NOM` (1013) selects east, pronounced `/ɛst/`; the verb form of `être`
pronounced `/ɛ/` is excluded. `mille/ADJ:num` (1020) selects the invariable numeral
adjective, while `mille/NOM` (1021) selects the number 1000 as a masculine noun;
neither uses the mile/nautical-mile senses. `adieu/NOM` (1028) selects farewell
as parting for a long time or forever. `adieu/ONO` (1029) retains the frozen
subclass and selects the Wiki farewell interjection with the same duration
restriction, excluding regional casual greeting.

`personnel/ADJ` (1033) concerns an individual; `personnel/NOM` (1034) selects
**collective singular staff**. `anglais/ADJ` (1038) concerns **England** or its
inhabitants; its noun (1039) is the language and retains the singular-only use.
`français/ADJ` (1047) concerns France, its inhabitants or culture; its noun
(1048) is the language. Lowercase language nouns are not capitalized person aliases.

`intéressant/ADJ` (1041) selects arousing interest. The noun (1042) selects the
**generally pejorative person who considers themselves interesting**, in
`faire l’intéressant` / `faire son intéressant`. Fixed revision 39771367,
lines 99–107, explicitly separates the noun, its person meaning, IPA and those
constructions. The captured current Robert article separately corroborates
the dictionary's modern noun treatment; it is read-only grammar evidence, with
no copied definition or partner example in the fields. This is author research,
not a separate reviewer's acceptance.

`virer` selects familiar dismissal from employment; `régler` selects payment;
`pote` is familiar friend; `sympa` is a familiar likeable person adjective.
`vaisseau` selects the bodily channel for **blood or lymph**, retaining that
ordinary anatomical meaning rather than importing ship, architecture or heraldry
uses. `moi-même` selects the emphatic first-person **subject responsible for the
action**. `compris` selects adjective **included**, excluding its understood
sense and verb forms. `vol` selects winged-animal flight, excluding theft and
aircraft journeys. Each sense records its own remaining exclusions.

`m/NOM` (1004) and `s/NOM` (1050) select lowercase **letter-name nouns**,
pronounced `/ɛm/` and `/ɛs/`. Wiki labels these sections `lettre`; the explicit
masculine/invariable markers and separately captured Robert noun classifications
support the Lexique NOM mapping. This is not an upper/lowercase runtime alias,
nor an admission of consonant-sound or symbol pronunciations.

## Limitations: row 1005 and admission

Row **1005 `m/PRO:per`** retains four explicit missing-field records. Its fixed
[bare-m revision 39772551](https://fr.wiktionary.org/w/index.php?title=m&oldid=39772551)
has a French letter section at 166–169, a vulgar `m...` noun at 196–198 and a
musical adjective at 207–212, with **no French personal-pronoun section**.
[Apostrophized m’ revision 39840795](https://fr.wiktionary.org/w/index.php?title=m%E2%80%99&oldid=39840795)
lines 7–14 supports elision of `me`; [me revision 39891222](https://fr.wiktionary.org/w/index.php?title=me&oldid=39891222)
lines 7–13 supports the direct/indirect object pronoun under the different
unelided spelling. Straight-apostrophe `m'` revision 20098185 redirects to `m’`.
Robert's bare-m article classifies it as a masculine letter-name noun and symbol.
These facts do not establish the exact frozen bare-pronoun spelling.

The full original captures are `sources/7338-39772551.wiki`,
`sources/182951-39840795.wiki`, `sources/42279-39891222.wiki`,
`sources/160194-20098185.wiki` and `support/robert-m.html` under the owning
author evidence root. The selected contrasting spans are in
[source-excerpts.json](source-excerpts.json).

No prior annotation approval is applied, no new specialist/annotation exception
is declared, and no apostrophe-removing binding is admitted. The independent
review confirms this exact source gap; any necessary scope decision belongs to the owner.
The other 49 groups and 196 fields pass separate quality, selected-source rights
and spelling/POS review. Integration owns final conversion,
packaged notices and real native lookup/trusted save/reopen verification.
