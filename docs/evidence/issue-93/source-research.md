# Selected sources and scope

All 50 selected uses / 200 fields passed independent source, rights and quality
review. [Independent review](independent-review.md), [admission](admission.json)
and the canonical [field review](field-review.json) record the accepted content.
The fields are adaptations of fixed French Wiktionary lexical spans
under the existing CC BY-SA 4.0 resource policy. The source histories supply
Wiktionnaire contributor attribution; the last-editor capture metadata is not
a selected-sentence authorship finding. Lexique supplies the recorded French
adaptations and English/Chinese translations. The existing
[resource licensing ADR](../../adr/0002-source-and-resource-licensing.md),
[full licence](../../../resources/core-pilot-79/COPYING), and
[resource notices](../../../resources/core-pilot-79/NOTICES.md) remain applicable;
the combined core carries the applicable source/adaptation notices.

The complete fixed response is retained as `target/issue-93/bundled.api.json`;
the typographic-apostrophe page has its own `apostrophe.api.json`. The capture
registry binds every title/page ID/revision/timestamp to the retained main-slot
wikitext. Full original captures are preserved under integration's
`target/implement-90-99/worker-audit/author-93/`. `source-context.json` contains
only selected French language/POS,
definition/IPA locations and source-page import/bibliography markers.

For 28 pages citing or declaring DAF8 material, the original Académie française
eighth-edition entries were also captured through CNRTL under
`target/issue-93/originals/`. The recorded original credit is Académie française,
*Dictionnaire de l’Académie française*, eighth edition, 1932–1935. An import
marker, a bibliography citation, an original public-domain edition and the
Wiktionnaire adaptation have distinct boundaries. A bibliography citation alone
does not prove authorship of the selected sentence. The retained original
captures support the accepted independent selected-span audit. No CNRTL
interface prose, TLFi definitions or
externally attributed literary examples are selected as dictionary fields.

Prior research candidates were inspected at
`C:/Users/inthe/IdeaProjects/lexique-d2-resource-research/target/d2-research/dictionary-modern-batch50-next9-review-20261118/pre-increment/reviewed-core-bindings.json`.
Forty-four ticket groups have candidate leads there. Their research acceptance
is not resource admission, and their XMLittré 3.0 payloads are not relabelled as
Wiktionary 4.0 content. The canonical accepted wording is bound to the
fixed French pages and independently reviewed source spans.
No prior admitted row is rewritten.

Retrieval follows the existing multi-title Node fetch pattern in
`C:/Users/inthe/IdeaProjects/lexique-84-89/target/implement-84-89/audit86-fetch.mjs`.
The original-source capture pattern was inspected in `fetch-87-evidence.py`;
the earlier finalizers were read for the source/rights/review separation.
They were not run against another worktree. The worker's capture/build helpers
and full logs remain in its ignored evidence root.

## Uses requiring explicit restrictions

- `propre/ADJ` is the clean/washed sense. `propre/NOM` is the characteristic
  noun in **le propre de**. They remain separate rows 609 and 610.
- `dîner/NOM` and `dîner/VER` use the evening meal and eating it in France.
  Regional midday uses are explicitly outside these selected senses.
- `dessus/ADV` refers to an already identified upper surface without an
  expressed noun complement. `dessus/NOM` denotes the upper part itself.
  The obsolete preposition is not substituted for either row.
- `général/NOM` denotes the military commander. `général/ADJ` denotes broad
  applicability. `blanc/NOM` is the singular colour noun; its adjective is
  separately retained.
- `baiser/NOM` is an ordinary kiss, with its recorded `/bɛ.ze/`. `baiser/VER`
  is the vulgar intransitive sexual use, with recorded `/be.ze/`. The restricted
  or outdated kissing verb is not taught as an unrestricted everyday verb.
- `idiot` is marked pejorative in both its adjective and person-noun uses.
  Historical medical classification is not selected.
- `drôle/ADJ` means funny. `drôle/NOM` at row 646 means a child in the
  **Bordelais/Poitou regional use**; the fixed French page explicitly labels
  that sense by region and labels the rogue sense obsolete. Contemporary
  corroboration is the July 2024 Reprises des savoirs *Se loger* PDF, page 1,
  headed *Paroles de drôles* with a child gloss for the Saintonge/Poitou term:
  [publication](https://www.reprisesdesavoirs.org/wp-content/uploads/2024/07/15.20Se20loger-1.pdf).
  Its actual PDF and text extraction are retained under `target/issue-93/originals/`.
  The publication is read-only usage evidence; its prose is not imported into
  the fields. Independent review accepted the regional everyday-use fit;
  the accepted sense remains restricted to Bordelais/Poitou.
- `cinq/ADJ:num` is cardinal before the counted noun, not postposed ordinal
  or a noun. `/sɛ̃k/` is the recorded citation IPA; the use and supporting source
  spans retain the exceptions for *cinq minutes*, *cinq cents*, *cinq mille*
  and further variation.
- `bonsoir/NOM` is the greeting as a masculine invariable noun, not the
  directly uttered interjection. No grammatical subclass is replaced.

## Spelling relationship

Frozen row 605 remains exactly `d'abord/ADV`. Its real fixed ASCII-apostrophe
[revision 21181001](https://fr.wiktionary.org/w/index.php?title=d%27abord&oldid=21181001)
line 1 redirects to `d’abord`. The selected first-event/action definition is
line 13 and its IPA line 8 in the real typographic-apostrophe
[revision 39799412](https://fr.wiktionary.org/w/index.php?title=d%E2%80%99abord&oldid=39799412).
The accepted relation is limited to this exact group and selected use, with
its independent spelling review recorded in
[spelling-relations.json](spelling-relations.json).

Every row's accepted text/use and source/line are recorded in the canonical JSON.
Independently accepted completeness is **50/50 groups / 200/200 fields**.
The fixed denominator remains 4,087 groups. Native lookup/save/reopen, offline
and saved-state preservation checks are pending at integration; see the parent
[aggregate verification](../issue-90-99/README.md). Learner acceptance and wider
release gates remain open.
