# Issue #108 selected-source evidence

All 50 frozen groups at rows 1351–1400 have one sourced four-field candidate.
The 46 fixed French Wiktionnaire articles comprise 45 exact headwords and `nul`,
which supplies separate indefinite-adjective/pronoun grammar for `nulle`.
[Source captures](source-captures.json) retain the actual revision IDs,
timestamps, original request URLs, API responses, exact wikitext and article
history URLs. [Sources](sources.json), [context](source-context.json),
[excerpts](source-excerpts.json) and [field-review proposals](field-review.json)
bind the candidate text to those originals. Acceptance remains pending a
separate independent reviewer.

## Source and contribution rights

The declared Wikimedia text licence is CC BY-SA 4.0. Eligibility of each
selected transcription/adaptation remains pending review. Full linked article
histories provide contributor credit. Eight paginated current named-contributor responses are
retained separately; a current registry is not a cutoff author registry and
does not establish authorship of a selected sentence. Fixed revision editors
are also recorded.

Actual French-section DAF8 imports/references and bibliographic markers are
recorded per article. Where DAF8 is declared, the historical original is credited
to Académie française, *Dictionnaire de l’Académie française*, eighth edition
(1932–1935). The already reviewed edition-level collective-work basis under
article L123-3 and conservative expiry of 2006-01-01 are referenced from the
[accepted #99 context](../issue-99/source-context.json). This author reuses that
original-edition evidence; the source-page marker alone does not identify the
origin or rights of a particular modern sentence. Every actual selected-span
rights decision remains pending independent review.

`champ` has an `Import:CFC` marker on an unselected cartography definition.
That marker and its unresolved original-source rights are isolated in context;
the candidate selects the agricultural definition at line 14. No CFC prose is
copied into the fields or the bundled bibliography.

French fields are scoped adaptations of the selected semantics. English and
Chinese fields are new explanatory translations by Lexique contributors,
under CC BY-SA 4.0; they are not attributed to an upstream bilingual dictionary.
IPA preserves recorded variants with a formatting/context adaptation. Field
modification notices identify that layer. Unselected quotation bodies and media
remain in ignored audit captures; bundled context contains bibliographic
markers rather than the quotation text. The existing core licence texts and
redistribution pipeline remain the integration route after independent review.

## Exact source frames and restrictions

Every selected definition is in an evidenced French section and the required
POS frame. Each group retains the original frozen spelling, grammatical
subclass, frequency strings and separate sense identity. Same-spelling
noun/adjective pairs stay separate: `suivant`, `prisonnier`, `deuxième` and
`militaire`. The selected uses are limited to what their cited definition
supports. Unknown CEFR remains null.

- `repartir` uses the second verb section's intransitive new departure, not the
  first section's literary retort or accented `répartir`.
- `huit/ADJ:num` uses the anteposed cardinal definition at fixed revision
  39752961, lines 7–11. Its shared noun/adjective pronunciation table at
  lines 326–350 records /ɥit/ in isolation, phrase-finally and before a vowel,
  and /ɥi/ before a consonant. The IPA field retains those conditions. The
  postposed ordinal and `huir` verb form are excluded.
- `bravo/ONO` uses the French interjection frame, not the translingual spelling
  alphabet, another language, applause noun or hired-killer noun.
- `bonne/NOM` preserves the source's pejorative status (line 5) in all meaning
  fields and selects the female domestic/hotel servant or child-carer use.
- `suivant/NOM` selects the next person/item, supported by its noun definition
  and the separate adjective's order semantics, rather than the obsolete
  attendant/escort sense.
- `prisonnier/ADJ` has no IPA on its adjective headword line. The IPA is bound
  explicitly to the general French pronunciation section (fixed revision
  39964862, lines 149–150) and the adjective frame at lines 107–110. The
  separate noun retains its own frame and prisoner definition.
- `marre/ADV` preserves the colloquial `en avoir marre (de…)` construction,
  corroborated by the source's derived-expression link at line 44. It is not
  presented as an unrestricted adjective meaning tired.
- `île` and `goût` retain their frozen traditional spellings and source spelling
  markers. No unaccented runtime alias is admitted.
- `coincer` selects the familiar tight-space confinement use at line 10, rather
  than the obsolete wedge insertion/fixing definitions. `miracle` selects the
  everyday hyperbolic extraordinary occurrence, with that restriction retained.

## Row 1386 modern-use blocker

[Fixed `nulle` revision 39737088](https://fr.wiktionary.org/w/index.php?title=nulle&oldid=39737088)
records the feminine singular indefinite pronoun at lines 18–20 and its only
exact-form usage example at lines 21–23, from Balzac (1855).
[Fixed `nul` revision 39923182](https://fr.wiktionary.org/w/index.php?title=nul&oldid=39923182)
records the indefinite pronoun as masculine singular and invariable at lines
210–213, with literary/legal register. These sources support a historical
feminine candidate but do not resolve modern everyday eligibility or the
invariable-frame conflict. The blocker affects all four fields of row 1386's
selected use. It is recorded in [admission](admission.json), field/usage review,
source-binding relations and ignored `target/issue-108/blockers.json`.

The distinct indefinite adjective at row 1385 is evidenced by `nulle` lines
25–27 and `nul` lines 120–123, including the feminine `nulle` agreement template,
singular-only restriction, placement before a noun and literary register.
Both relations serve source binding only; neither creates a runtime alias or
merges the two POS subclasses. A separate reviewer must determine whether the
selected written-register uses satisfy the parent requirement.
