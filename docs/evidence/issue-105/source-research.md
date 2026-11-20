# Issue #105 selected sources and uses

The [inventory](review-candidates.json) covers the exact 50 frozen groups in
ticket #105, under parent #78. There are 46 proposed everyday senses with FR,
IPA, EN and ZH fields, four exact-group blockers and no reused/admitted group.
Unknown CEFR values remain null.

## Fixed originals and adaptations

[Source captures](source-captures.json) retains 43 first-party Wiktionnaire API
revision responses and their blank-line-preserving wikitext: 41 exact ticket
headwords plus supporting `que` and `qu'`. Nine ticket articles were captured
at the 2026-10-01 cutoff; the remaining 34 source records come from a current
multi-title response. Each source keeps its actual revision and timestamp.
Current captures are not described as cutoff revisions.

Every candidate field binds to an exact source ID, fixed URL/revision, one-based
line and raw excerpt in [source-excerpts.json](source-excerpts.json). The
[context](source-context.json) records its French language, POS and headword/IPA
frames. These checks establish provenance binding. They do not establish
independent semantic quality or rights eligibility.

FR fields are scoped adaptations of the selected French meanings. EN and ZH
are new Lexique explanatory translations of those meanings and restrictions,
not purported quotations from an upstream bilingual dictionary. IPA selects a
recorded variant, retaining its syllable notation in slashes; it does not claim
exclusive pronunciation in all connected-speech contexts. Each field records
the corresponding modification notice.

## Rights and attribution

The proposed container route is CC BY-SA 4.0, with the existing
[legal text](../../../resources/core-pilot-79/COPYING) and
[core rights authorities](../../../resources/core-pilot-79/curated-inputs.json).
Wiktionnaire history credit and Lexique selection/adaptation/translation credit
remain separate. Eight contributor API responses completed both query series,
covering all 43 articles and 3,229 names summed across their article registries.
Full histories are linked. The current registry may include contributions made
after a frozen revision, and does not identify the author of a particular
selected sentence.

Actual `Import:DAF8`, `R:DAF8`, `R:Littré` and other bibliographic markers are
preserved with their source locations. Académie française, eighth edition
(1932–1935), and Émile Littré are credited as original bibliographic layers
where the source actually identifies them. The independent [review](independent-review.md) and [rights decision](rights.json) resolve the selected-original-text boundary for the 184 accepted fields, separately from the original candidate provenance checks. A marker does not identify every modern definition author.

Raw quoted examples, full article bodies and Robert/CNRTL page bodies remain in
ignored captures. Tracked context preserves bibliographic markers and selected
eligible-evidence candidates, without unrelated literary example prose. No
Robert, CNRTL, audio or image content becomes a dictionary field.

## Selected-use restrictions

The exact noun/adjective pairs for `responsable`, `marié`, `méchant`, `russe`
and `proche` remain distinct. `responsable/ADJ` selects accountability with a
`de` complement; its noun selects an organizational position of responsibility.
`marié/NOM` selects a male newlywed on the wedding day. `méchant/NOM` selects a
fictional villain; its adjective selects harmful/unkind behaviour.

`russe/NOM` selects the lowercase, singular-only language noun, rather than a
capitalized nationality/person label. `proche/ADJ` selects spatial proximity;
the noun selects a close friend or relative. `vacance/NOM` selects the singular
vacancy of a post, not the explicitly plural holiday senses. No plural or
feminine source relation becomes a runtime alias.

`j/NOM` selects the lowercase letter name `/ʒi/`. Wiktionnaire revision
39772412 has the French letter frame at lines 37–41, including masculine and
invariable markup. The retained Robert response corroborates nominal letter
use: derived text lines 43–46 give the grammar and letter description. This
supports review of a letter-name noun, without borrowing the joule/day symbols,
the apostrophized personal pronoun or a capitalized alias.

`sept/ADJ:num` selects the cardinal numeral before a noun, with source
pronunciation `/sɛt/`; it does not select the postposed ordinal, number noun or
rare clan noun `/sɛpt/`. `pleuvoir` selects the impersonal weather use with
`il`. `engager`, `réparer`, `construire` and `séparer` retain their selected
transitive senses. `profiter` selects `profiter de` a thing/opportunity, and
`cesser` keeps the stopping use and `cesser de` construction separate from
negated continuing expressions.

`chier` remains a vulgar intransitive defecation verb, explicitly marked in FR,
EN and ZH. `salope` selects the vulgar contempt/disloyalty insult at fixed line
68, independently of sexual conduct. Its restrictions retain the source's
especially-feminine use, occasional reference to a man and the distinction
from historical sexual shaming or consensual sexual play. It is an insult,
not a neutral assessment of a person.

## Exact-group blockers

| Frozen row | Exact group | Fixed-source gap |
| ---: | --- | --- |
| 1208 | `quiconque/PRO:int` | Revision 39477804 has relative lines 5–7 and indefinite lines 46–48, without an interrogative frame. |
| 1218 | `qu/CON` | Revision 32115306 has only a French letter-name noun at lines 12–15. |
| 1219 | `qu/PRO:int` | The same noun evidence cannot establish an interrogative pronoun. |
| 1220 | `qu/PRO:rel` | The same noun evidence cannot establish a relative pronoun. |

`quiconque/PRO:ind` (1207) and `quiconque/PRO:rel` (1209) have separate
candidate uses. The relative uses no antecedent; the indefinite uses the
standalone complement. The retained CNRTL response identifies its dictionary
as **Le Littré (1873–1874, 1878)** at derived text line 33. It is historical
corroboration, not a TLFi definition or proof of modern interrogative use.

The bare `qu` noun has letter-name IPA `/ky/`. Supporting revision 21403352 is
an actual redirect from `qu'` to distinct `qu’`. Supporting `que` revision
40046884 has conjunction and interrogative/relative frames, but no relation
establishes the literal frozen `qu` spelling in those subclasses. The negative
source/spelling findings are retained separately; no substitution, automatic
alias or annotation exception is made. All four rows retain four missing fields
in the fixed 4,087-group denominator.
