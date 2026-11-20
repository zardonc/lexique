# Issue #100 selected-source evidence

All 50 exact word/POS pairs at frozen rows 951–1000 have one independently accepted
sense and 200 nonempty fields. The [groups](groups.json) retain exact
frequency strings, distinct POS identities, construction/register restrictions
and `cefr: null`. Source binding and independent field/rights review passed in [field review](field-review.json) and
[admission](admission.json).

## Fixed sources

[Source captures](source-captures.json) record **42 French Wiktionnaire articles**,
their actual current-query revisions/timestamps, page IDs, fixed `oldid` URLs,
history URLs, acquisition URLs and absolute API/wikitext paths. Four complete
multi-title API bodies supplied these pages. They are current-query captures,
not claimed to come from an earlier cutoff. Each original response body is
retained as fetched bytes. Wikitext preserves blank lines and is compared with
the corresponding response revision before fields are written.

Seven contributor responses cover the 42 pages with complete pagination; the
last has no continuation. The registry is current and may include contributors
after a selected revision. Full linked article histories supply attribution;
registry totals do not establish who wrote an individual selected sentence.

The supplied `dictionary-modern-batch50-next19-20261119` research scope was
inspected: its 50 evaluated groups have no intersection with this ticket.
Its raw records do not contribute fields or citations to this handoff.

FR fields are scoped adaptations of selected French lexical meanings. EN/ZH
are new Lexique explanatory translations of those same uses, credited
separately from Wiki contributions. They are not quoted upstream bilingual
glosses. IPA retains one applicable recorded French variant in slash notation;
audio/media and whole-sentence pronunciation are absent.

[Source excerpts](source-excerpts.json) bind every field to a fixed version,
one-based line/column and exact original text. [Source context](source-context.json)
records French language frames, POS/headword/gender/number frames, selected
pronunciation and additional grammar evidence. Exact source spellings retain
accents, hyphens and case; no unreviewed alias is introduced.

## Original authors and redistribution

The original responses retain all actual import/bibliographic boundaries.
Tracked context contains import/reference and bibliography metadata, rather than
unselected quotation bodies. In particular:

- DAF8 imports and references identify **Académie française, Dictionnaire de
  l’Académie française, eighth edition (1932–1935)**. The edition-level
  public-domain assessment uses the final 1935 publication and the collective
  work term in [CPI L123-3](https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20),
  giving conservative expiry on 2006-01-01. Historical wording and Wiki modifications remain separately identified;
  see the independent selected-content review. This statutory pointer was
  retained from the existing accepted rights record; no live statute refresh
  is claimed.
- `étranger` explicitly has **Import:Littré** as well as DAF8. Fixed import and
  reference templates identify **Émile Littré, Dictionnaire de la langue
  française (1872–1877)**. The fixed French rights convention explicitly names
  historical Littré as public-domain material. Original prose remains separate
  from the Wiki transcription/adaptation and Lexique translation layers.
- `ligne` line 156 contains **Import:CFC** in a cartographic definition. Its
  fixed template identifies the Comité français de cartographie's 1990 glossary
  and a CC BY-SA 4.0 donation. This definition is outside the selected
  text-row sense at line 61; its text is not a candidate field.
- `boîte` line 11 includes TLFi and Usito pronunciation references after the
  alternate variant. The selected `/bwat/` template is located precisely;
  protected dictionary prose and an unreviewed `boite` alias are absent.
- Other TLFi, Robert, historical dictionary and literary references retain
  their own source boundaries in metadata. A reference or a page-wide licence
  does not authorize copying their prose. No cited literary example is one of
  the four candidate fields.

Nine first-party fixed revisions preserve seven import/reference templates,
the [French rights convention](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168)
and [Wikimedia Terms of Use](https://foundation.wikimedia.org/w/index.php?title=Policy%3ATerms_of_Use&oldid=598588).
The [existing CC BY-SA 4.0 legal text](../../../resources/core-pilot-79/COPYING)
supplies the adaptation licence. Preserve fixed article/history attribution,
applicable original-author notices, the legal text/link, modification notices
and CC BY-SA 4.0 for adapted Wiki/Lexique fields when assembling an accepted
resource. Original public-domain wording is not relabelled as CC-licensed;
project code remains MIT.

Forty-eight complete CNRTL responses retain historical-edition observations:
39 default routes and nine explicit POS routes. Forty contain the requested
original-edition widget, extracted separately from the full response. Eight
responses lack it and are explicitly marked `no_requested_original_edition_content`;
their HTML is retained without being presented as the requested entry.
CNRTL's default route can select another homograph/POS. Explicit adjective,
adverb or pronoun routes supply the applicable original entry where available.
The snapshot index records the exact response key/widget and local edition-text
path, with acquisition time and original edition. Modern TLFi/Académie ninth
edition and site text elsewhere in those responses are read-only context.

## Distinct selected uses

`reposer/VER` selects **se reposer** to recover from fatigue; `échapper/VER`
selects **s’échapper** to flee. `fier/VER` selects **se fier à**, with `/fje/`.
Their pronominal construction is explicit in the selected source text and in
all explanatory translations.

`fier/ADJ` retains `/fjɛʁ/` and satisfaction of self-esteem. The fixed Wiki
adjective's first numbered gloss at line 11 reads `envie d’amour-propre`; the
selected semantic evidence is the adjective-section descriptor at line 39,
`qui éprouve une satisfaction d’amour-propre`. It is identified as a descriptor,
not represented as a numbered definition. The separately captured DAF8
adjective route corroborates dignity/pride and being proud of someone or
something. Modern dictionary prose is outside the selected Wiki field binding.

`chacun/PRO:ind` is the exact singular indefinite pronoun, with the source
feminine `chacune` recorded only as morphology. Its pronoun-frame `/ʃa.kœ̃/`
is distinct from the familiar noun's `/ʃa.kɛ̃/`; the source note says no liaison
after `chacun`. No pronoun-to-noun or adjective substitution is made.

`dommage/NOM` selects material damage or non-material injury. `dommage/ONO`
selects the attested standalone interjection **dommage !**, expressing regret,
under the exact frozen ONO identity. The interjection section has no IPA
template; its same-spelling French pronunciation comes from the explicit noun
frame, with that boundary recorded. The source does not call this an imitation
of a sound. The independent reviewer must assess the interjection-to-Lexique-ONO
mapping and applicable pronunciation; no annotation exemption is inferred.

`merveilleux/NOM` selects the general masculine singular abstract **le merveilleux**,
the aspect of experience/imagination/belief departing from the natural order
and ordinary course of things, at Wiki line 94. The definite-article restriction
is at line 83. The neighbouring literary-genre-only definition, historical
dandy, regional cake and silk are excluded. The captured DAF8 adjective route
also has the general substantive meaning involving an event or narrative,
distinct from its epic/dramatic literary meaning. Independent modern everyday
suitability remains a required review decision, not a specialist approval.

`riche`, `public`, `étranger` and `sourire` keep their noun and adjective/verb
groups distinct. Lowercase `allemand/NOM` is the singular language noun;
the nationality/person noun `Allemand` does not supply this field. `mur/NOM`
is the built wall, with no `mûr` mapping. `boîte/NOM` is a closable container
with the traditional accented spelling. `chat/NOM` selects the feline `/ʃa/`,
not internet `/tʃat/`; `supporter/VER` selects enduring something unpleasant,
with `/sy.pɔʁ.te/`, distinct from the sports-supporter noun. `rendez-vous/NOM`
retains its hyphen, unchanged singular/plural form and arranged meeting use.

`connerie` selects the source-labelled familiar stupid action/error;
`enfer` selects the source-labelled figurative familiar place of suffering or
disorder; `terrible` selects negative hyperbole. `santé` describes the body's
state whether good or bad. Physical shape/noise/window/mouth/wall, the written
text row, kitchen room and sight are scoped independently of their technical,
figurative, inflected or alternative senses.

## Preparation and verification

All preparation/acquisition helpers and complete logs remain under
`C:/Users/inthe/IdeaProjects/lexique-batch-100/target/issue-100/`.
`selection.py` contains the exact author-selected uses. `prepare.py` checks the
original revision/wikitext and builds the candidate package; `complete_context.py`
attaches the historical/rights observations without changing candidate wording.
The original HTML, API bytes and wikitext are retained separately from transformed
edition text and selected excerpts.

The focused [resource check](../../../scripts/test_core_batch_100.py) passed
membership/frequencies, all 200 exact source locations and accepted-text
preservation against the original candidates. The separate
[independent review](independent-review.md) records selected-content and rights
acceptance. Native execution and final assembled redistribution remain parent gates.
