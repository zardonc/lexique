# Issue #109 fixed sources and selected uses

The [review candidates](review-candidates.json) preserve the original 50-group
author draft. [Groups](groups.json), [field review](field-review.json) and
[admission](admission.json) record 50 groups / 200 fields independently accepted
for quality, selected-span rights, use and spelling. See the
[independent review](independent-review.md) for the final judgments.

## Captures and attribution

[Source captures](source-captures.json) identify **50 fixed French Wiktionnaire
articles**: 46 exact ticket headwords and the supporting `quel`, `le mien`,
`la mienne` and `enceint`. Two first-party API responses retain the actual
returned revisions, timestamps and raw wikitext. These are current-query
captures, not claims about an October cutoff. Revision URLs and complete
article-history URLs are in [sources](sources.json).

Seven main contributor responses and one supporting response retain **3,234
distinct contributor names summed across articles**. Both paginated sequences
end without continuation. The full current registries are also retained in
[source context](source-context.json); they extend beyond the fixed revisions
and do not establish selected-sentence authorship. Full linked histories supply
article attribution. Lexique’s selection, adaptation and translation layer is
credited separately.

Every field has an exact one-based source line in
[source excerpts](source-excerpts.json). IPA additionally records the column
and exact variant within the source line. The selected isolated spelling is
pronounced; article-plus-pronoun phrases, liaison and all regional variants are
not conflated with that IPA. FR is a scoped adaptation. EN/ZH are new explanatory
translations of the selected French use, with the construction/register retained,
rather than purported upstream bilingual quotations.

The proposed Wiktionnaire contribution and Lexique adaptation licence is
**CC BY-SA 4.0**. Its full [legal text](../../../resources/core-pilot-79/COPYING)
and the established Wikimedia rights authorities in
[curated inputs](../../../resources/core-pilot-79/curated-inputs.json) remain
integration-owned. The handoff records source URLs, revision/history links,
attribution, modifications and explicit original-work layers.

**27 pages** contain `Import:DAF8`. Actual raw notice lines are retained, with
original Académie française eighth-edition credit (1932–1935). The author’s
proposed original-work route reuses the project’s recorded collective-work basis:
[Code de la propriété intellectuelle, L123-3](https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20),
with conservative expiry on 2006-01-01 using the final 1935 publication.
[Prior selected-source evidence](../issue-99/source-context.json) records that
edition-level assessment. Wiki transcription/adaptation and Lexique’s new
adaptations retain their separate CC BY-SA 4.0 layers. This inherited
original-edition basis is a proposal for the new selected spans; their separate
review is pending. Neither a page licence nor an import marker proves the origin
or eligibility of a selected sentence.

Bibliographic identities and import markers remain in tracked context. Full
unselected examples stay in ignored raw captures. No literary quotation, audio,
image, whole article or unrelated dictionary sense enters the proposed fields.

## Construction and spelling bindings

| Frozen group | Fixed source evidence and selected scope |
| --- | --- |
| 1402 `quels/ADJ:int` | `quels` revision 35421624, lines 2–4: exact masculine-plural interrogative adjective. `quel` revision 39953698, lines 7–10: before a noun, asks its identity. IPA is `/kɛl/` from the exact form. Exclamative and interrogative-pronoun uses remain separate. |
| 1413 `mien/PRO:pos` | Bare `mien` revision 38517604, line 9 supplies `/mjɛ̃/`; its adjective meaning is marked obsolete. `le mien` revision 39008342, lines 3, 5–8 establishes the definite-article construction and first-person singular possessor, masculine singular; lines 13–14 supply `au mien`/`du mien`. The selected meaning is the modern pronoun in this construction. |
| 1428 `mienne/PRO:pos` | `mienne` revision 38517623, lines 2–5 fixes the feminine singular form and `/mjɛn/`. `la mienne` revision 36203435, lines 3, 5, 16–17 establishes its possessive-pronoun construction, feminine singular and first-person singular possessor. |
| 1429 `cent/ADJ:num` | `cent` revision 40000908, lines 7–11: cardinal before a noun, exact quantity 100; isolated `/sɑ̃/`. The postposed ordinal and figurative quantity are excluded. |
| 1430 `cent/NOM` | Same fixed revision, lines 261–265: the masculine number noun 100 with `/sɑ̃/`. The separate currency noun at 417–425 has its own pronunciation and is excluded. |
| 1447 `enceinte/ADJ` | `enceinte` revision 39895779, lines 7, 11–12 fixes the feminine form, `/ɑ̃.sɛ̃t/` and common reference to women rather than female animals. `enceint` revision 40003215, lines 13, 22 gives the pregnancy meaning; the rare-masculine label is not applied to the common feminine use. |

Every relation is explicitly recorded as **source binding only**, with
`alias_admission: not_admitted`. Exact frozen spellings stay unchanged. The
construction pages support `mien`/`mienne`; they do not replace those frozen
groups with new headwords or admit obsolete bare adjective uses. The separate
reviewer must assess the component-to-construction mappings.

## Other selected-use boundaries

`conseiller/NOM` and `conseiller/VER` retain separate senses and source sections.
The verb selects advice on an action with `de + infinitive`; the noun selects
someone giving advice. `criminel/ADJ` selects being guilty of a crime; its
separate noun selects a person found guilty, excluding merely accused persons.
`enfoiré/ADJ` and `enfoiré/NOM` retain their separate qualification/person uses
and the **vulgar, pejorative, insulting** register in all explanatory fields.
The familiar friendly noun sense is excluded.

`faîte` selects the ridge of a roof, with the circumflex retained. `to` selects
the French masculine name of a West African dish made with cereal flour and
water, served cooked with sauce; its French language/POS/IPA frame is separate
from English `to` and language codes. This is an everyday culinary candidate,
with its West African context explicit, not a proposed specialist exception.

`haïr` retains aspirated h. `honnête` uses the moral-probity adjective and mute h.
`lâche` selects pejorative lack of courage rather than loose objects.
`formidable` selects modern favourable fantastic/extraordinary use rather than
the obsolete fear-inspiring sense. `enchanté` selects the delighted adjective,
not the greeting interjection or magical sense. `foi` selects religious belief,
not the differently pronounced Papuan-language noun. `test` selects checking a
hypothesis/fact in its second French noun section, not an invertebrate shell.

`livrer`, `briser`, `chasser`, `attirer`, `allumer`, `écraser`, `arracher` and
`mêler` select the explicitly recorded transitive uses. `inventer` selects being
first to conceive something new and practically useful without requiring its
realization; the invention-development and fabrication senses remain separate.
`juger` selects expressing an opinion, including the sourced `juger de` frame.
`sous-titrage` selects the displayed-subtitles result; its IPA comes from the
French `fr-rég` template, not an invented pronunciation.

## Evidence and pending decisions

Raw API responses, blank-line-preserving wikitext, contributor pagination,
selection/preparation scripts and execution logs are retained at
`C:/Users/inthe/IdeaProjects/lexique-spec-100-109/target/implement-100-109/worker-audit/author-109-final/target/issue-109/`.
The [capture manifest](source-captures.json) gives exact per-source paths.

Author preparation validated all 200 raw bindings, French/POS frames and exact
membership. The focused candidate check passed after an expected RED on the
missing handoff. Candidate field text totals **7,664 Unicode characters**; the
longest field is **140 characters**. These are authored-content measurements,
not compiled-resource or native-memory measurements.

No exact-row fields remain unresolved after independent review of all 200 texts,
50 uses and spelling/construction relations. Native lookup/save/reopen,
conversion, combined packaging and final admission remain parent-owned and
unrun for this handoff.
