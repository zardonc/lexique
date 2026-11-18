# Issue #85 fixed source research

The [capture registry](source-captures.json) records **38 fixed captures**:
37 French Wiktionnaire pages copied from retained first-party research responses
and one newly fetched English Wiktionary response. They cover all 36 frozen
headwords, the canonical `quelqu’un` page and the English `avant` page. The
cutoff is **2026-10-01 00:00:00 UTC**. Capture origins and exact local paths
remain recorded; copying an earlier capture is not described as a fresh fetch.

Raw captures are under `target/issue-85/sources/`, with the English API response
in `target/issue-85/avant-en-api.json`. The tracked selected excerpts retain
the exact lines needed by the fields. Full source articles, external quotations
and media are not added to the distributable content.

## Selected boundaries

Each of the 48 selected uses has four proposed fields. The French definition,
English and Chinese are scoped adaptations of the selected contributor
definition; headword IPA is selected from an explicitly recorded variant.
Every field retains its own source ID, fixed revision, original line and raw
excerpt. Current author credit points to the complete contributor history;
adaptations are credited to Lexique contributors with CC BY-SA 4.0 retained.
The existing [legal text](../../../resources/core-pilot-79/COPYING) applies to
eligible contributor content. Page-level `Import:DAF8` notices are retained
separately for the reviewer to examine actual imported boundaries. They do not
establish rights for every sentence on a page. No independent rights verdict
has been supplied for these new spans.

`avant/ADV` uses **English Wiktionary revision 92304586**,
**2026-08-25T05:06:34Z**, French section line **100**, for the ordinary temporal
meaning beforehand/earlier. Its IPA comes separately from French Wiktionnaire
revision **39865821**, line **158**. French and Chinese adaptations follow
that English selected sense. Other fields use their respective fixed French
definitions, with grammar and construction restrictions stated explicitly.

`quelqu'un/PRO:ind` retains the original ASCII apostrophe. The fixed ASCII
page **revision 35035** redirects to `quelqu’un`; the canonical selected
pronoun definition is **revision 39815086**, line **8**, and IPA line **7**.
The proposed spelling relationship is recorded for independent review. No
global search alias, frozen spelling substitution or learner-data rebinding
is added.

The difficult selected uses remain narrow:

- `mal/ADJ` is confined to **bon gré mal gré**; the adjective definition and
  the fixed-expression example support this construction. Productive adjective
  replacement of mauvais is not implied.
- `beau/ADV` is confined to **avoir beau + infinitive**, expressing vain effort.
- `voilà/PRE` is the temporal **ago** construction with an elapsed duration,
  not the presentative verb.
- `ses`, `tes` and `vos` preserve Lexique's `ADJ:pos`; the source's possessive
  determiner terminology and possessor/possessed number restrictions remain
  explicit. `quel` retains `ADJ:int` before a noun.
- `donc/CON` retains the frozen connector subclass; the source labels the
  selected result relation an **adverbe conjonctif**.
- `grand/NOM` is the grown-up sense in children's speech. `mère/NOM` selects
  giving birth; the separately attested caregiving relationship is excluded.

## Source gaps

**Row 218 `avant/ADJ`.** Fixed Wiktionnaire revision **39865821** has the
adjective relating to Avant-lès-Marcilly at line 322, which is unsuitable for
this everyday target. English Wiktionary revision **92304586** supplies an
ordinary adverb, preposition and noun but no adjective section. The
[Académie ninth edition, avant II, section III](https://www.dictionnaire-academie.fr/article/A9A3345)
confirms an ordinary invariable adjective for front-position uses. This is a
grammatical lead, not admitted text: a redistributable selected wording and its
rights boundary remain unresolved. A proposed review candidate is *front,
positioned in the front part, as with a vehicle's front wheels*; no field is
claimed sourced and admitted for this group. No locality adjective is substituted.

**Row 238 `déjà/ONO`.** French Wiktionnaire revision **39362190** has only an
adverb section for déjà. Its lines 10, 22 and 30 concern temporal already,
previously and asking for forgotten information. No onomatopoeia section or
modern eligible onomatopoeia sense was located. General web research did not
establish one. The related adverb remains solely in frozen row 237. An
interjection reading is not substituted for the frozen `ONO` subclass.

Both groups remain in the denominator with zero selected fields. A scope
decision belongs to the owner; this draft makes none.
