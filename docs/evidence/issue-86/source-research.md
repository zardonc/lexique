# Issue #86 fixed source research

The [source table](source-table.md) binds each frozen group to its precise
definition and IPA revision/line. English and Chinese fields are scoped
translations and explanations of that same selected French use. They share its
definition source; foreign dictionary wording is not imported. Actual selection,
paraphrase, construction/register qualification and IPA variant selection are
recorded per field as adaptations by Lexique contributors.

## Fixed captures and contributors

The first-party MediaWiki revisions API returned **47 complete French
Wiktionnaire captures**, with page ID, revision ID, timestamp and wikitext.
**46 pages supply selected fields or required grammatical context**. The
additional `les leurs` capture is research only and is not incorporated.
The [registry](source-registry.json) retains exact fixed URLs and article-history
attribution links. Captures were obtained on 2026-11-18; source timestamps
identify the actual content dates. Each selected revision is fixed independently
of the current live page.

The contributors API was followed through all nine response parts and returned
**4,434 registered-contributor entries across the 47 captures**. This is a sum
of per-page lists, not a count of distinct people. The selected source context
retains the applicable page's actual account names. Article/history links remain
the attribution route for all upstream contributors, including anonymous edits;
the API account list is not presented as complete individual authorship of each
selected sentence.

Raw first-party responses, UTF-8 wikitext, contributor responses and read-only
publisher captures are retained under ignored `target/issue-86-sources/`.
The tracked selected excerpts and source context preserve the material proposed
for redistribution, without full dictionary bodies or quoted external prose.

To retrieve a fixed entry independently, use its registry `revid` with the
first-party endpoint:

```text
https://fr.wiktionary.org/w/api.php?action=query&format=json&formatversion=2&prop=revisions&rvprop=ids%7Ctimestamp%7Ccontent&rvslots=main&revids=REVISION
```

Count lines from one in the returned main-slot wikitext, preserving blank lines.
The actual capture check compared all **200** field spans and their source
revision/URL bindings against those responses.

## Grammatical and spelling bindings

- `leur/PRO:per` selects the third-person plural dative clitic from `leur`,
  revision 39770609, lines 81–84. `leur/PRO:pos` selects the possessive element
  of **le leur** from revision 38979962, lines 3, 5, 7, 16–17: plural possessors
  and a masculine singular possessed referent, with the article required.
  Bare `/lœʁ/` is recorded by `leur` and also occurs within `/lə lœʁ/` in the
  possessive entry. Neither the determiner nor the personal pronoun replaces the
  possessive group.
- `quelle/ADJ:int` remains the exact frozen feminine singular form. Its fixed
  entry, revision 39771917, lines 4–6, attests the interrogative subclass and
  relationship to `quel`. Definition semantics come from `quel` revision
  39953698, line 10; IPA comes from `quelle`, line 5. Exclamative and pronoun
  uses are excluded.
- `cet/ADJ:dem` retains the frozen subclass while the source uses the modern
  demonstrative-determiner label. `cet` revision 39772629, lines 12–25, provides
  the masculine singular paradigm, IPA and `ce` relationship. `ce` revision
  40003876, lines 25–26, supplies the vowel-**sound** condition and demonstrative
  meaning. This includes silent h and excludes aspirated h.
- `nos/ADJ:pos` uses the fixed possessive-determiner entry, revision 39801228,
  lines 16–19, with plural possessors and a plural possessed referent.
- `ah/ONO` and `salut/ONO` select interjections within the established ONO
  umbrella. `ah` uses the short `/a/` emotion interjection; the separate long
  comic-book scream is excluded. `salut` selects an informal meeting greeting
  to someone addressed with tu; the leave-taking sense is excluded.
- `gens/NOM` is plural only. Its selected entry records variable agreement;
  the draft does not assign a single unconditional adjective-agreement gender.
- `mort/ADJ` means dead, while `mort/NOM` selects feminine **la mort**, death;
  masculine **un mort**, a dead person, is excluded. `premier` and `dernier`
  retain distinct adjective and person-denoting noun records.
- `moins/NOM` names the ordinary written minus sign, not the subtraction
  operation or the comparative adverb. `demain/NOM` means the future, while
  `demain/ADV` means on the next day.
- `fils/NOM` means son, with `/fis/`, not plural `fil` with `/fil/`. The fixed
  entry records both separately; their fields and sources are not merged.
- `jouer/VER` selects **jouer à un jeu**, not musical **jouer de**.
  `plaire/VER` selects being pleasing to someone, with the experiencer after à.
  `sentir/VER` selects perceiving an odour, not emitting one or a reflexive use.
  `rentrer/VER` retains its source's “Sauf au Canada” qualification for re-entry.

## Concrete unresolved bindings

**Row 270, `ouais/ADV`:** Wiktionnaire revision 39568604, lines 11–13,
classifies affirmative informal oui as an interjection. The read-only
[Le Robert entry](https://dictionnaire.lerobert.com/definition/ouais), captured
2026-11-18, also labels it an interjection. An exact modern adverb source has
not been established. The original ADV group remains present and unaccepted;
the draft makes the source-taxonomy difference explicit.

**Row 274, `vivre/NOM`:** `vivres` revision 39708130, line 5, explicitly says
it is the plural of the noun `vivre` meaning food. Lines 9–10 attest the modern
plural-only food-supplies use. `vivre` revision 39923645, lines 301, 303, 311
and 325, separately records the masculine noun, dated singular food sense and
`vivres` derivative. The selected proposal preserves frozen `vivre/NOM` and
requires the modern plural surface form. It does not claim a contemporary
singular food use or take the heraldic serpent or elevated abstract noun as an
everyday substitute. Whether this plural-bound selection satisfies the exact
frozen group remains a concrete independent scope-review question.

## Rights boundary

Every proposed field identifies the actual fixed source, selected span,
attribution/history, CC BY-SA 4.0 route, legal URL and adaptation. The existing
[COPYING](../../../resources/core-pilot-79/COPYING) and
[NOTICES](../../../resources/core-pilot-79/NOTICES.md) remain the shared legal
materials. Upstream membership credits and source obligations are unchanged.

The fixed [French contributor policy, revision 38903168](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168)
distinguishes contributor licensing from imported text. Its line 80 describes
the collective-work public-domain basis for the older Académie dictionary.
[Import:DAF8 revision 22113837](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3AImport%3ADAF8&oldid=22113837),
line 1, says that all or part of an article was extracted and may since have
been modified. Exact import/reference markers and their source line numbers
remain in the batch context; a page-wide label is not treated as proof of a
particular sentence's authorship.

Selected French wording is paraphrased for the precise modern use; English and
Chinese are scoped adaptations. Protected publisher wording, literary/comic
examples, external quotations, historical example prose, audio and images are
excluded. Read-only Le Robert captures corroborate contemporary classifications
and usage; their wording is not bundled or used as an independently licensed
translation source.

Independent selected-span rights/quality review, satisfaction of the combined
packaged obligations and actual admitted conversion remain pending. A researcher
check or this source registry is not recorded as independent acceptance.
