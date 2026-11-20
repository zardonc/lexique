# Selected sources for #118

The [frozen groups](groups.json) retain all rows 1851–1900. Each uses one
modern French selected sense, with authored English and Chinese explanations
scoped to that same POS and use. Source labels do not determine CEFR; all levels
remain unknown. Plain contemporary explanations select the everyday use of
computer, iron, gas, buttock and arid desert without adopting unrelated technical
uses. Quantitative rainfall criteria from the desert source are not part of the
selected everyday definition.

## Fixed source and rights boundary

[Sources](sources.json) fix 54 selected/supporting French Wiktionnaire revisions.
[Field bindings](source-excerpts.json) retain raw lines and selected IPA variant
columns. [Contexts](source-context.json) retain full paginated public contributor
registries, full-history links and original-source notices.

[Rights](rights.json) binds each of the 200 fields to its actual selected source
span, the captured French copyright policy and applicable original-edition
layers. The captured policy, revision 38903168, identifies CC BY-SA 4.0 for
article text and reuse (lines 6 and 17), exceptional external material (22),
compatible imports (26), contributor rights (29) and ShareAlike (33). The complete
[legal text](../../../resources/core-pilot-79/COPYING) accompanies that route.

Declared DAF8 imports/references retain Académie française's original collective
edition credit, its 1932–1935 publication and the established
[#99 original-edition basis](../issue-99/source-context.json). The Wiki
contribution/transcription layer and these new adaptations remain separately
CC BY-SA 4.0. An import marker is not proof of selected-sentence authorship.
The [separate review](independent-review.md) accepts all new selected-span rights
boundaries; actual converted/package notices remain parent-owned.
The Littré import in saisir's Old French section is explicitly excluded; no
Old French text or XMLittré transcription supplies a field. Quoted examples,
translation tables, pictures and audio do not enter the authored fields.

## Exact constructions and morphology

- **l'une / PRO:ind, row 1884.** The exact straight-apostrophe article redirects
  to `l’une`; that fixed French article gives the feminine singular flexion of
  `l’un`, its `/l‿yn/` pronunciation and its number forms. The `l’un` article
  identifies article + `un` composition and the partitive construction followed
  by `de`; `un` defines one person or thing among others. `une` explicitly
  distinguishes the elided indefinite pronoun from the newspaper-front-page
  noun. `le` provides elision before a vowel. The selected use is only
  `l’une de + plural set`. Source binding retains the frozen ASCII spelling;
  no generic query alias or new annotation is admitted.
- **vive / ONO, row 1870.** The selected French section is the acclamation
  interjection, definition 53 and IPA 52. The `vive/vivent` table and plural note
  support the retained number restriction. The adjective `vif` and ordinary
  subjunctive forms of `vivre` are separately labelled exclusion/flexion evidence.
  The existing [#110 ONO/interjection pattern](../issue-110/source-context.json)
  supplies the established binding convention; the exact frozen ONO stays intact.
- **lors / ADV, row 1893.** The source restricts this time adverb to composition
  and explicitly lists `lors de`, `lors des`, `lors du`. The fixed `lors de`
  article supplies composition and at-the-time-of semantics. The fields describe
  the adverb component in that modern construction; `/lɔʁ/` is the component's
  pronunciation. The locution is not a replacement group or admitted query alias.
- **préféré, perdu, fatigué.** Selected adjective definitions and agreement
  tables remain distinct from separately captured verbal past-participle sections.
- **extérieur and professionnel.** Adjective and noun groups keep separate
  selected definitions. Nominal professionnel explicitly records the feminine
  professionnelle; propriétaire and élève explicitly have common gender.
- **gaz and ouest.** The source tables preserve their unchanged plural spellings.

Every relationship has `relation_role=source_binding_only`,
`alias_admission=not_admitted`, and independent review passed for source binding.
Full capture and preparation scripts remain at the paths in
[source-captures.json](source-captures.json).
