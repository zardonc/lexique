# Issue #115 fixed sources and selected uses

The package authors all 50 exact groups in rows 1701–1750 with one modern
selected sense and four bound fields each. French text is a scoped adaptation;
English and Chinese are new explanatory translations of that same use.
Register, gender, construction and spelling restrictions remain in all three
meaning fields where they affect interpretation. IPA selects a recorded
isolated-form variant, with the applicable number stated for `os`.

## Fixed captures and attribution

[Sources](sources.json) and [captures](source-captures.json) identify
**50 selected/supporting French Wiktionnaire revisions** plus the fixed
**Dictionnaire de l’Académie française, eighth edition, tome 2, page 659 (1935)**.
The first-party Wiki API retained actual revision IDs, timestamps, raw wikitext
and fixed/history URLs. The DAF8 source uses an immutable publication
edition/page identity and retained HTML/text lines, rather than a Wiki revision
claim. All exact one-based field lines and IPA variant columns appear in
[source excerpts](source-excerpts.json).

Seven main, one redirect-support and one idiom-support contributor responses
retain **3,348 contributor names summed across 52 captured Wiki articles**.
The two unused captured articles are `le tien` and the redirect
`mettre du sien`; neither supplies a field. Each paginated sequence ends
without continuation. Full registries/history credits are retained in
[source context](source-context.json). Registries extend beyond the fixed
revision and do not identify the author of an individual selected sentence.
The Académie original edition and digital presenter are credited separately
from Lexique's authored selection/adaptation/translation layer.

## Selected use and spelling boundaries

- **`gardien/ADJ` and `gardien/NOM`** remain distinct: protective adjective,
  including the sourced `ange gardien` construction, versus a person who guards.
  Likewise, **`blessé/ADJ` and `blessé/NOM`** distinguish an injured quality from
  an injured person. Neither pair is merged or counted twice as one group.
- **`oeuvre/NOM`** retains the exact frozen key. Fixed `oeuvre` revision
  `37442666` explicitly attests a variant of `œuvre` under typographic
  constraints, with feminine gender and /œvʁ/. Fixed `œuvre` revision
  `39796721`, line 21, supplies the selected feminine literary/artistic/
  intellectual creation. The masculine collective output is excluded.
- **`com/NOM`** is the exact familiar feminine shortening of
  `communication`, fixed revision `39837141`, lines 17–20, /kɔm/.
  Masculine `commentaire`, feminine `commission` and domain `.com` are
  excluded. No expansion or new spelling is substituted as the frozen key.
- **`tien/NOM`** selects personal contribution in `y mettre du tien`,
  addressed to one familiar person. DAF8 article `A8T0594`, printed page 659,
  text line 730 explicitly supplies the noun classification; lines 737 and
  739–740 supply what comes from the addressee and the concession construction.
  The current ninth edition `A9T1169*` corroborates the
  construction's present-day recording as a source fact, with no ninth-edition
  definition or explanatory prose adapted into fields. Fixed exact French
  `tien` revision `38579571`, line 9 supplies form-level /tjɛ̃/; its adjective
  classification does not stand in for the separately sourced noun. Nominal
  `du` is `de + le`; a possessive pronoun replacing a named noun, obsolete
  attributive `un tien ami` and plural family uses are excluded.
- **`os/NOM`** selects singular physical bone /ɔs/. Fixed revision
  `39950453`, line 16 separates singular and plural pronunciations.
  The usual plural /o/ and invariant spelling remain readable in FR/EN/ZH.
- **`piger`** keeps its familiar European understanding use; **`lycée`**
  explicitly selects French upper secondary education. **`péché`** is the
  ordinary religious noun, with its spelling distinct from `pêche`.
  **`renoncer`**, **`appuyer`**, **`confier`** and **`rompre`** retain
  their selected prepositional/transitivity constructions in the three meanings.

These are source-supported author selections. No runtime alias, new annotation
exception, new CEFR grade or broader dictionary union is admitted.

## Rights and modifications

[Rights](rights.json) records the CC BY-SA 4.0 Wiki contribution/transcription
and Lexique adaptation route, full legal-text link, fixed/history attribution,
modification notices and redistribution obligations. **43 sources** have an
applicable DAF8 imported/referenced original-edition layer. The existing
[issue #99 original-edition basis](../issue-99/source-context.json) identifies
the collective work, final 1935 publication and conservative 2006-01-01 expiry
under the cited primary statute. It is reused as an author proposal, without a
new independent legal determination. A page-wide import/reference marker does
not prove selected-sentence authorship; all exact selected/supporting spans and
original notices are retained for the fresh reviewer.

The CFC cartography import in `titre`, lines 76–77, is excluded from selected
fields. Literary/news example prose, translation tables, etymology, images,
audio and other-language sections do not supply fields. The fixed XMLittré
historical noun crosscheck, its README and licence header are retained under
`target/issue-115/sources/`; no XMLittré text supplies a field or packaged
excerpt. The current Académie ninth edition is factual construction
corroboration only, without a general redistribution claim for that edition.

Fresh CC legal text, Wikimedia terms and French copyright/import-policy
captures are retained with their actual URLs and retrieval status.
Combined package notices and final redistribution compliance remain subject to
the parent's selected-span review and conversion gates.

## Evidence boundary

[Field review](field-review.json) contains 200 proposed texts and exact sources,
50 selected uses, 50 spelling/construction relations and null reviewer identity.
Author validation checks the captured bindings and explicit restrictions.
[Independent review status](independent-review.md) leaves all quality,
selected-sentence/import fit and rights judgments pending.

Preparation reuses the issue #109 record layout and issue #113 author scripts.
Reproduction and logs are at `target/issue-115/` in
`C:/Users/inthe/IdeaProjects/lexique-batch-115`.
Native/converter results are not claimed by this author package.
