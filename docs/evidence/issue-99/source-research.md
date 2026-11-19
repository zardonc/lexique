# Issue #99 selected content

All **50 frozen rows 901–950** are retained, with **49 independently accepted
senses / 196 fields** and one owner-approved zero-field annotation,
`i/ADJ:num` (934), with **four missing fields**.
[Independent review](independent-review.md), [admission](admission.json) and the
canonical [field review](field-review.json) record the accepted wording, rights
and usage. The exact frequencies, POS and restrictions are in
[groups.json](groups.json) and [review-candidates.json](review-candidates.json).

## Fixed sources and attribution

[source-captures.json](source-captures.json) records 44 real fixed French
Wiktionnaire articles: 43 exact ticket headwords and `certain`, which supplies
the indefinite adjective/pronoun meanings for plural `certains`. Revision IDs,
timestamps, fixed URLs, history URLs and original/local capture paths are retained.
Ten ticket pages were fetched directly at the October cutoff, one existing
`pourtant` API response was copied byte-identically, eleven additional cutoff
pages were fetched through Obscura, and 22 pages were selected through a current
multi-title API response. The latter keep their actual returned revisions and
timestamps; they are not described as cutoff captures. Full response bodies and
blank-line-preserving wikitext remain under the ignored evidence root.

Each accepted field has a one-based line and exact selected source text in
[source-excerpts.json](source-excerpts.json). The preparation script compares
every selected API revision and complete wikitext, checks the French-language
section and IPA spelling, and checks all frozen memberships. This establishes
source binding, not independent translation quality or rights eligibility.

Seven paginated contributor API bodies contain 3,299 distinct contributor names
summed across the 44 articles. The final response has no continuation. Full
article-history attribution is retained; this current contributor registry is
broader than the fixed revisions and does not establish selected-sentence authorship.
The content author and independent reviewer are separate; the accepted review
records cover all 49 selected uses and their explanatory translations.

FR definitions are scoped adaptations of the selected French semantics. EN/ZH
are new Lexique explanatory translations of those uses, not purported upstream
bilingual equivalents. IPA selects one recorded variant with slash notation;
it does not assert exclusive pronunciation or cover liaison. Lexique selection,
adaptation and translation credit is recorded separately from Wiki contributors.

The accepted container route is CC BY-SA 4.0, under the existing
[COPYING](../../../resources/core-pilot-79/COPYING) and
[rights authorities](../../../resources/core-pilot-79/curated-inputs.json).
[source-context.json](source-context.json) retains actual `Import:DAF8` markers
and their original Académie eighth-edition credit. Independent review accepted
the exact selected-text rights and imported-prose boundaries recorded in the
canonical sources. A container licence, contributor count or import marker does
not independently resolve them. No external literary quotation,
Robert/TLFi/Capital prose, unselected definition, audio, image or whole page is
included in the accepted fields or tracked source excerpts.

## Selected-use and POS boundaries

`certains/ADJ:ind` (913) is the exact masculine plural before a noun.
`certains/PRO:ind` (914) is the standalone masculine plural referring to people.
Fixed `certains` lines 2–5 and 10–13 prove the separate form/POS sections;
fixed `certain` lines 41–42 and 139–142 supply their distinct meanings. The
additional bindings remain explicit supporting evidence, without a new alias.

`intérieur/ADJ` and `intérieur/NOM`, `ancien/ADJ` and `ancien/NOM`,
`anniversaire/ADJ` and `anniversaire/NOM`, and `bleu/ADJ` and `bleu/NOM`
retain separate identities. `ancien/ADJ` selects former status **before the
noun**; its noun selects a longstanding group member. `anniversaire/ADJ`
retains same-calendar-day commemoration, and the noun selects the birthday date,
not the celebration. The contemporary 2021 quotation in the adjective section
is researched as usage support, with only its bibliographic identity retained
here; its prose is not a resource field.

`honte` has aspirated h; `habitude` has mute h. `filer` selects familiar
intransitive departure; `ficher` selects familiar euphemistic **faire/travailler**,
not `s’en ficher`. `bordel/NOM` selects vulgar figurative disorder. `colonel`
selects an officer holding the French rank, with the hierarchy explicitly
supported by the neighbouring definition. `professeur` selects a teacher at
any level, rather than only the university rank. `avancer` selects physical
forward movement and `risquer` selects exposing something to possible danger.

### Row 934: approved zero-field `i/ADJ:num` annotation

[Wiki revision 39772403](https://fr.wiktionary.org/w/index.php?title=i&oldid=39772403)
has Roman lowercase **one** in the **translingual** section, lines 41–43.
Its French section starts at 48: the Latin letter name is at 49–52; a feminine
Glagolitic noun starts at 100; colloquial pronouns follow. The page provides no
French numeral-adjective section for the frozen group. The
[current Robert entry](https://dictionnaire.lerobert.com/definition/i) labels
`i` a masculine invariable noun and symbol, and explicitly writes the Roman
number as uppercase `I`. That evidence does not establish a lowercase French
numeral-adjective spelling/POS bridge or its applicable reading.

Row 935 separately admits the everyday Latin-letter noun, supported by the
French Wiki letter section and Robert’s noun classification. It does not
complete row 934. Row 934 retains its frozen identity/frequencies and zero
fields and four missing-field records. The owner-approved annotation is recorded
in [admission](admission.json); it changes the content requirement while retaining
the exact frozen identity and denominator. No `i`→`I` alias,
translingual-to-French binding or specialist sense is admitted.

### Row 949: accepted abstract noun `l’incroyable`

[Wiki revision 39827744](https://fr.wiktionary.org/w/index.php?title=incroyable&oldid=39827744)
lines 85–99 describes only Revolutionary-era dandies. That historical sense is
excluded. The accepted sense selects **what cannot be believed or is difficult to
believe**, as a masculine singular substantive with neutral value in
**l’incroyable**. Its FR/EN/ZH explanatory text adapts the adjective semantics
at line 10; IPA is at 9. This semantic/POS mapping is explicit, not a claim that
the Wiki noun section contains the proposed abstract meaning.

[TLFi A.1](https://www.cnrtl.fr/definition/incroyable) explicitly records the
masculine-singular neutral substantive usage and a 1931 Valéry example. The
[current Robert page](https://dictionnaire.lerobert.com/definition/incroyable)
also reproduces a 2020 Capital example using **le plus incroyable**, a modern
substantivization lead. Robert says those partner examples are selected
automatically and not reviewed by its editors; its noun definition itself is
historical. Local full HTML/text captures fix the observed evidence.

Independent review accepted the abstract noun's precise POS, modern everyday
suitability, explanatory wording and selected-text rights. The accepted use is
neutral, masculine and singular. Neither the historic dandy nor a specialist
exception is admitted. The
corroborating dictionaries and Capital supply read-only grammar/usage facts;
their prose does not enter the four fields.

## Evidence root and reproduction

Retained full captures and logs:
`C:/Users/inthe/IdeaProjects/lexique-content-99/target/issue-99/`.
Full original captures and logs are preserved under integration's
`target/implement-90-99/worker-audit/author-99/`.
The API attempts that encountered HTTP 429 remain logged. Obscura response bodies
are retained as UTF-8 JSON text, not claimed to be the original transport bytes.
Only the reused `pourtant` capture is explicitly asserted byte-identical to its
original path. Source IDs carry `b99_`; sense IDs carry `b99-`.

```powershell
uv run --python 3.14.10 --no-project docs/evidence/issue-99/prepare_candidates.py
uv run --python 3.14.10 --no-project -m unittest discover -s scripts -p test_core_batch_99.py
```

These retained author-stage commands checked the **candidate handoff artifact**,
exact frozen membership and source bindings. The final accepted replacements are
included in the [combined core](../../../resources/core-pilot-79/README.md).
Native lookup/save/reopen, offline and saved-state preservation checks are pending
at integration; the parent owns the [aggregate verification](../issue-90-99/README.md).
Learner acceptance and wider release gates remain open.
