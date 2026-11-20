# Row 1057 Mme author handoff

**Four new FR/IPA/EN/ZH fields are pending separate independent review; zero new fields are admitted.** The 192 accepted fields and their independent review records remain unchanged. The frozen membership is `1057 mme/NOM`; its lowercase dictionary-key relationship to the written French title `Mme` is already supported by [independent review](independent-review.md). Row 1051 `s/PRO:per` remains unresolved, without an alias or annotation exception.

[Exact fields and author provenance](mme-author.json) records the complete text, use, pending reviewer statuses, selected layers and source spans. The same fields are in [groups](groups.json) and [review candidates](review-candidates.json). FR/EN/ZH describe the courtesy title for an adult woman regardless of marital status. IPA is **/ma.dam/**, the pronunciation of the expanded title, rather than letter names.

The writing restrictions are readable in all three explanatory fields: capital `M`, lowercase `me` preferably in superscript, no abbreviation point, plural `Mmes`; abbreviation before a name, given name, title or role when referring to the person; full `Madame` for direct address, standalone use, letter greetings/closings and envelopes. These are independently worded grammatical explanations, corroborated against OQLF's first-party capture. OQLF and Cambridge wording, translations and examples supply no selected text.

## Exact selected first-party source

[Wiktionnaire Mme revision 36554529](https://fr.wiktionary.org/w/index.php?title=Mme&oldid=36554529), page ID **193650**, timestamp **2024-12-07T15:06:55Z**; [full contributor history](https://fr.wiktionary.org/w/index.php?title=Mme&action=history).

| Location, one-based | Field or support |
| --- | --- |
| Line 10, column 1 | FR/EN/ZH selected title definition: `# [[Madame]] (comme [[titre]]).` |
| Line 9, column 11 | IPA template: `{{pron\|ma.dam\|fr}}` |
| Line 5, column 1 | Abbreviation etymology |
| Lines 7–8, column 1 | French noun frame, written singular/plural and pronunciation |
| Line 23, column 1 | Contemporary use irrespective of marital status |

Complete original API response and wikitext:

- `C:/Users/inthe/IdeaProjects/lexique-batch-102/target/issue-102/sources/batch.api.json`
- `C:/Users/inthe/IdeaProjects/lexique-batch-102/target/issue-102/sources/193650.wiki`

[Source excerpts](source-excerpts.json) bind every field and supporting span to this fixed revision. [Source context](source-context.json) records the selected grammar and [sources](sources.json) the proposed rights boundary. The retained independent remote comparison covers the original capture; it does not review the new field wording.

## Selected layers and remaining gates

The selected lexical definition, templates and contemporary usage note contain no named imported bibliographic layer. Attribution is to Wiktionnaire contributors through the fixed article and complete history; Lexique contributors supply the scoped French adaptation, English/Chinese explanations and IPA transcription. These layers are proposed under CC BY-SA 4.0 with the existing complete notices and adaptation obligations, pending independent selected-span review.

Nearby Balzac, Godier, Belga/RTBF and Lemaitre quotations, bilingual tables and media remain unselected. `Madame` revision 39716168 line 55 cites Jean-Pierre Lacroux's *Orthotypographie*. Its [first-party site](https://www.orthotypographie.fr/) states CC BY-NC-ND 2.0 France and prohibits commercial use/modification without permission. That prose is excluded from field text and adaptations; the retained rights capture is `target/issue-102/mme-author/orthotypographie.html`.

The parent reviewer must independently assess the four exact fields, their same-use construction restrictions and selected rights. [Admission](admission.json) retains 48 accepted groups / 192 accepted fields, four source-missing fields, four unreviewed fields and eight missing admitted fields against the fixed 4,087-group denominator. Actual notices, conversion, native lookup/save/reopen and integration remain separate gates.

## Validation and reviewer inputs

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-102`, branch `codex/d2-content-102`; diff base **c2f48cb**. Scope: `docs/evidence/issue-102/` and `scripts/test_core_batch_102.py`. Tickets: [#102](https://github.com/zardonc/lexique/issues/102), parent [#78](https://github.com/zardonc/lexique/issues/78).

- PASS: `uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_102.py` — three focused resource-boundary tests.
- PASS: `uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-102/verify_sources.py` — 56 original captures, 465 bindings and 196 candidate fields.
- PASS: preservation comparison against the pre-change evidence — 48 accepted groups / 192 fields and review records unchanged; row 1051 and established spelling relations unchanged.
- PASS: `git diff --check` and local Markdown links.

Complete logs and full/test-only patches are under `C:/Users/inthe/IdeaProjects/lexique-batch-102/target/issue-102/mme-author/`. `tested-state.json` records the exact committed SHA and Windows execution boundary; `handoff.patch` and `tests.patch` capture `git diff c2f48cb -- docs/evidence/issue-102 scripts/test_core_batch_102.py` and `git diff c2f48cb -- scripts/test_core_batch_102.py`, respectively. Full-suite, runtime conversion, native and release checks were not run for this content-only handoff.
