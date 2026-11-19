# Issue #103 selected-source research

All **50 frozen groups at rows 1101–1150** have a proposed everyday selected
use and four nonempty fields. [Groups](groups.json) preserve the exact subclasses,
frequency strings and frozen order. [Field review](field-review.json) records
200 proposed texts and their exact evidence; accepted quality and rights are
pending for a separate reviewer.

## Fixed originals and credit

[Capture records](source-captures.json) identify 47 exact ticket headwords plus
the supporting French verb `baigner`. Four first-party current-revision API
responses capture the ticket articles, and a separate response captures
`baigner`. Every article keeps its actual revision ID, timestamp, page ID,
fixed `oldid` URL, linked complete history and original/local API and wikitext
paths. These are current-query captures, not claims of an earlier cutoff.
The retained wikitext is compared with the complete original API revision body.

Eleven contributor API responses cover five paginated batches. Each final
response has no continuation. The current registry is broader than the fixed
revisions; it supplies contributor names alongside the complete linked history,
and does not identify authorship of a particular selected sentence.
Wiktionnaire contributors receive article/history credit. Lexique contributors
receive separate credit for selection, adaptation and new explanatory EN/ZH
translations. The translations are proposals derived from the French semantics,
not copied upstream bilingual glosses.

Sixteen fixed reference-template revisions identify the actual declared source
names and editions, including DAF8, DAF9, Littré, TLFi, Robert, Grand Robert,
Larousse, Usito, FranceTerme, Féraud, Meyer, DMF, FEW, Van Daele and Pauleau.
Their original API bodies and wikitext are retained. The fixed French
[copyright policy](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168)
is also captured. [Source context](source-context.json) records each actual
French-section reference/import marker with its location and template version.
The complete local `notices-audit.json` additionally retains quoted context and
bibliographic lines for review; those full quotations are not resource fields.

## Original and redistribution boundaries

The proposed Wiki/Lexique adaptation route is **CC BY-SA 4.0**, using the existing
[complete licence](../../../resources/core-pilot-79/COPYING). Each field records
its source and modification: scoped French adaptation, one recorded IPA variant
with slash notation, or new English/Chinese explanatory translation. The
article's general licence does not resolve imported or external prose. The
separate reviewer must decide the selected boundaries and combined obligations
before any content is admitted or shipped.

Declared DAF8 originals are credited to **Académie française**, eighth edition
**1932–1935**. Forty corresponding CNRTL DAF8 endpoint responses are retained
with capture times, source URLs and complete original HTML. Thirty-seven contain
entries; `plusieurs`, `excellent` and `arrière` report no entry. Readable text is
separate from original bytes and excludes script/style contents. The proposed
public-domain basis uses collective-work article L123-3, seventy years following
publication, with 1 January 2006 as the conservative expiry using 1935. Its
primary statute URL returned HTTP 403 in this session. Exact original/Wiki
correspondence, Wiki contributor changes and selected adaptations remain
separately pending.

DAF8 `bosser` describes nautical fastening, whereas this candidate selects the
modern familiar verb meaning work. DAF8 `récupérer` describes recovering property,
whereas this candidate selects recovery of strength. Their page import markers
are retained without assigning those modern meanings to DAF8. Likewise, a
Littré or other historical reference is recorded with its named author and
edition, without claiming the selected line's authorship from that reference.

`nez`, `douleur`, `dent` and `sonner` have modern external dictionary references;
`dent` and `marché` also cite DAF9. `oiseau` names Meyer's animal-science dictionary.
Those sources' text cannot inherit permission from the Wiki container. The
tracked candidates adapt the selected Wiki lexical lines; their eligibility
still requires independent original/import comparison. Modern example prose,
audio, images, etymology and unselected definitions are not candidate fields.

## Selected-use and spelling boundaries

`politique/ADJ` selects relation to public affairs and government;
`politique/NOM` selects the feminine noun for political activities and power.
`arrière/ADJ` is invariable and locative; `arrière/NOM` is the masculine rear
part. The adjective's IPA is supported by the same fixed article's separate
French pronunciation section, with its own POS frame retained.

`plusieurs/ADJ:ind` selects the indefinite adjective before a **plural noun**,
with an unspecified count greater than one, most often greater than two.
Its separate pronoun section supplies no adjective field. `tel/ADJ:ind`
selects unspecified identification before a noun, with the exact masculine
singular `tel`; captured examples support agreeing `telle/tels/telles`.
Its IPA comes from the same lemma's French adjective frame, with the indefinite
section independently recorded. Similarity, degree and standalone-pronoun
meanings are excluded.

`juif/ADJ` selects relation to Judaism; `juif/NOM` selects a follower of Judaism.
The noun explicitly restricts this use to religion, excluding ancestry without
religious adherence and the historical antisemitic insult. No uppercase ethnic
name is substituted for the exact lowercase frozen word. `héros` selects the
fictional protagonist and retains **aspirated h**; IPA does not imply elision
or liaison. `voleur` and `cousin` retain their selected male-person uses;
`tante` is a parent's sister and excludes a partner or a parent's cousin.

`bosser` retains familiar/popular European usage. `fuir` selects rapid departure
through fear; `respirer` physiological breathing; `sonner` producing a sound;
`récupérer` regaining strength, with a separately captured intransitive example.
`apprécier` selects favourable esteem, and `libérer` release of someone held
against their will. `recherche` selects scientific/scholarly work, excluding
publication of that work. `joie` selects the source's distinct cheerful-mood
definition. `frais` selects cool temperature and records both traditional and
1990 feminine-spelling paradigms as source-only evidence.

`mademoiselle` retains youthful/unmarried address, the Quebec restriction to
very young girls, possible objection from adult women and non-use in French
administrative documents. Its modern notes and definition are bound together;
the nineteenth-century married-woman meaning is excluded.

`bain` is bathing in water. Its fixed noun definition explicitly links
`baigner/se baigner`; the separately fixed verb's immersion definitions support
the explanation. This is semantic support for the noun, not an extra group or
runtime alias. `là-dedans` and `grand-père` keep their exact accented/hyphenated
spelling. All 50 groups have explicit French-headword spelling evidence;
same-spelling IPA and linked-definition relationships remain source bindings
only. [Source excerpts](source-excerpts.json) and [context](source-context.json)
retain those locations. No new query alias is admitted.

## Pending decisions

There are **zero missing candidate fields**, **200 unreviewed fields**, and
**zero admitted fields**. The separate reviewer must confirm the selected
modern meanings, constructions, all four same-use fields, applicable IPA,
spelling relationships and actual original/import rights. No unsupported exact
membership or scope exception is asserted by the author. Canonical preparation,
notices and real native/save/reopen verification remain integration work.
