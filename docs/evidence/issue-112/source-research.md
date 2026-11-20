# Selected sources and uses for #112

The fixed first-party Wiktionnaire API captures contain 45 assigned headword
pages and the supporting lemma `quel`: **46 revisions**. Each source record
retains its page ID, revision ID, timestamp, fixed URL and full linked history.
The [source index](source-captures.json), [field bindings](source-excerpts.json)
and [contexts](source-context.json) locate the retained raw evidence. Source
versions are those actually returned by the API, without a historical cutoff
claim. Full contributor pagination comprises seven captured API responses;
3,218 registered contributor names are counted across their respective pages.
Anonymous contribution counts and the full histories remain in the API/history
evidence. A page-wide registry does not prove authorship of an individual gloss.

French definitions are scoped adaptations of the selected French meanings.
English and Chinese fields are authored explanations of those same uses,
not quotations from a bilingual dictionary. IPA selects one explicitly recorded
isolated-headword variant; other recorded variants are not claims of error.
No audio, illustration or literary example sentence enters the package.
Unknown CEFR remains null.

## Exact identity and construction boundaries

| Frozen rows | Selected use and restriction |
| --- | --- |
| 1551–1552 `rose` | Colour adjective and feminine flower noun are separate groups; masculine colour noun is outside the selected noun sense. |
| 1554 `endormir` | Transitive cause-to-sleep use; `s’endormir` is a separate construction. |
| 1557 `louer` | Tenant-side renting; landlord-side letting and the praise homonym are excluded. |
| 1558 `mine` | Ore-extraction site; appearance, explosive and pencil-lead homonyms are excluded. |
| 1559 `hasard`, 1599 `horreur` | Aspirated h versus mute h is retained in the use restriction, with the exact headword-line evidence. |
| 1566 `passe` | Feminine sporting pass to a teammate; permit/card and other homonyms are excluded. |
| 1567 `couler` | Intransitive liquid flow; sinking and transitive casting/leaking are excluded. |
| 1568 `rigoler` | Informal laughter/amusement; unrelated drainage-channel verb is excluded. |
| 1570–1571 `inconnu` | Unknown adjective and unfamiliar-person noun have separate POS bindings. |
| 1572–1573 `crétin` | Insulting everyday adjective and noun; historical clinical meanings are excluded. |
| 1574–1575 `privé` | Private adjective versus the person noun `un privé`, an ellipse of `détective privé`; `privé de` is a separate construction. |
| 1577 `in` | Invariable French adjective meaning fashionable; English, Latin and other French uses are excluded. |
| 1578 `sol` | Physical ground; the musical note and currency homonyms are excluded. |
| 1580 `quelles/ADJ:int` | Exact feminine plural interrogative form. Definition semantics are from fixed `quel/ADJ:int`, form and IPA from fixed `quelles/ADJ:int`. Exclamative and pronominal uses are excluded. This is a source binding, not a runtime alias. |
| 1582 `étudiant` | Higher-education person noun, with feminine `étudiante` identified; school pupils in general are outside the selected use. |
| 1585–1586 `lourd` | Physical-weight adjective versus invariable importance adverb in `peser lourd`. The quantity adverb `beaucoup` is excluded. |
| 1588 `débrouiller`, 1590 `fâcher` | Transitive sort-out and cause-anger uses; pronominal constructions are excluded. |
| 1589 `shérif` | Territorial police/judicial office in certain English-speaking jurisdictions; powers vary by country, and this is not a general French police title. |
| 1591 `lui-même/PRO:per` | Masculine singular personal pronoun emphasising the acting subject; the hyphen is retained. Reflexive-object and `by himself` uses are separate. |
| 1592 `samedi` | Day-name noun after Friday and before Sunday; no numbered-week convention is imposed. |
| 1594 `malgré` | Preposition before a noun phrase or pronoun denoting an obstacle; the clause construction `malgré que` is excluded. |
| 1595 `étage` | Building level above the ground floor; `premier étage` is above `rez-de-chaussée`. |
| 1596 `source` | Natural water spring; information source and radiation emitter are excluded. |
| 1597–1598 `attacher`, `joindre` | Physical fastening/joining; emotional attachment, reaching a person and enclosing documents are outside these selected uses. |
| 1600 `sexuel` | Relating to sexuality; biological sex classification and slang appetite descriptions are separate. |

Each of the remaining selected groups has its ordinary isolated-headword
spelling and directly matched French/POS frame. All 50 spellings and use
restrictions are individually recorded in [field-review.json](field-review.json).
No frozen group, subclass or frequency is changed. No owner annotation,
automatic morphological alias or additional sense is introduced.

## Rights and attribution evidence

The fixed Wiktionnaire copyright convention, revision `38903168`, expressly
permits copying, distributing and modifying article text under CC BY-SA 4.0.
The fixed DAF8 import notice, revision `22113837`, identifies the original
Académie française edition. Their exact retained spans and paths are in
[rights.json](rights.json). These existing first-party policy captures from
#109 are reused in this worktree; their reuse does not transfer the prior
reviewer's acceptance to this package.

Twenty-five selected source pages declare a DAF8 layer. The author proposal
credits Académie française’s eighth edition conservatively for those pages,
with Wiktionnaire transcription/adaptation contributors and Lexique adaptation
and translation contributors credited separately. The original collective work
was published in 1932–1935; the existing project L123-3 basis uses the final year
and expiry on 1 January 2006. Import markers identify a declared layer, rather
than establishing which contributor wrote each selected sentence. Selected-span
rights review remains pending for the separate reviewer.

For adverbial `lourd`, the tracked construction evidence retains only the short
combination `pèse lourd` at its exact column in the fixed source, and identifies
Dieudonné Costes and Maurice Bellonte, *Paris-New-York* (1930). The underlying
narrative sentence is outside the selected content. The author rights proposal
uses the noncreative grammatical fact, without claiming the book is public
domain or its prose is covered by Wiktionnaire’s licence.

The CFC cartographic import on `rose` is explicitly excluded at its marker
boundary; it contributes no selected definition. Bibliographic identities
identify context, not permission to redistribute their literary quotations.
The combined package must retain the [notices](NOTICES.md), legal text, selected
spans, fixed source histories and modifications. No noncommercial restriction
is introduced.

## Author state

All 50 groups / 200 fields are complete authored records. The canonical input
contains no previously admitted assigned row, so the [reuse record](reuse.json)
is explicitly empty and prior entries remain unchanged. Semantic, rights and
spelling judgments in this package are author proposals. All independent
quality/rights states remain pending; final admission and native execution are
parent-owned.
