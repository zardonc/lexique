# Issue #106 candidate handoff

Frozen rows **1251–1300** retain all **50 exact word/POS identities** and their
original frequencies. **49 candidate senses / 196 FR/IPA/EN/ZH fields** have fixed
source bindings. Row **1300 `d/PRE`** has **four source-blocked fields**.
**Independent quality and rights review is pending: zero groups and zero fields
are admitted.** The global denominator remains **4,087 groups**.

Scope is [ticket #106](https://github.com/zardonc/lexique/issues/106) under
[spec #78](https://github.com/zardonc/lexique/issues/78). The owning checkout is
`C:/Users/inthe/IdeaProjects/lexique-batch-106`, branch `codex/d2-content-106`,
based on `aae5273af846e57668a8d267fa6a6566c4fd6e91` from integration
`codex/d2-content-100-109`. Shared resources, converter, application and native
verification remain integration-owned.

The seven-file handoff is [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). [Review candidates](review-candidates.json) contain
the same groups with an explicit pending status. Review records carry proposed
text and usage, pending quality/rights decisions, no reviewer and no claim of
independence. All 200 required fields remain unresolved at the admission gate:
196 await separate review and four lack candidate source support.

## Fixed-source blocker

The French section of [bare `d`, revision 39772556](https://fr.wiktionary.org/w/index.php?title=d&oldid=39772556)
starts at line 46. It has letter and gaming-noun sections at lines 51–54 and
75–79. The bare-`d` preposition at lines 183–186 belongs to Tachelhit (`shi`),
whose section starts at 179. Neither the French letter-name `/de/` nor that
other-language preposition supports frozen `d/PRE`.

The separate [apostrophized `d’`, revision 39770687](https://fr.wiktionary.org/w/index.php?title=d%E2%80%99&oldid=39770687)
has French preposition, spelling and IPA evidence at lines 7–9. That capture is
contrast evidence only. An exact bare-`d` French preposition and its IPA/spelling
relationship remain unproved. The row keeps its original identity, zero senses
and four missing fields; no alias or annotation exception is admitted.

## Sources and review inputs

[Capture records](source-captures.json) identify 44 fixed first-party articles,
their page IDs, revision timestamps, oldid/history URLs and absolute original
API/raw paths. A separate fixed `d’` capture supplies the contrast above.
The multi-title response's actual returned revisions are retained; it is not
an October cutoff snapshot. Seven complete contributor-response pages retain
3,089 page-associated names. The current registry is broader than the fixed
revisions and does not prove selected-sentence authorship.

Original captures and author inputs are under
`C:/Users/inthe/IdeaProjects/lexique-batch-106/target/issue-106/`:

- `sources/batch-current-fixed.api.json` and each headword's `.wiki` preserve
  original revision text and blank lines.
- `sources/contributors-1.api.json` through `contributors-7.api.json` retain
  the contributor pagination; the final response has no continuation.
- `sources/d-apostrophe.api.json` and `sources/d-apostrophe.wiki` retain the
  separate apostrophe contrast.
- `source-index.json`, `contributors-index.json` and `apostrophe-capture.json`
  record the acquisition bindings.
- `selection.py` and `prepare_candidates.py` reproduce the candidate package
  from these originals. [Author checks](author-checks.json) record that boundary.

Every selected field includes fixed source/version, one-based line/column,
exact excerpt, licence and modification. FR definitions are scoped adaptations;
EN/ZH are new explanatory translations credited to Lexique contributors, rather
than upstream bilingual quotations. IPA selects one recorded French variant.
[Source research](source-research.md) lists each chosen use and construction.
Source context retains French/POS/spelling frames, import and bibliography
metadata, contributor names, original-edition rights and redistribution duties.
Unselected example prose remains in the ignored originals.

## Author checks and reviewer boundary

The parent-approved resource check compares exact frozen membership/frequencies,
candidate fields, proposed review text, source context, excerpts and original
API content. It detects omitted rows/fields, substituted spelling/POS, incorrect
revision/span/text, detached usage/spelling evidence and premature admission
claims. It checks the `soit/CON` alternative construction and `/swa/`, separately
from the agreement adverb `/swat/`.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 scripts/test_core_batch_106.py -v
```

The focused test recorded RED for the absent candidate package and GREEN for
the complete author handoff. Logs are [RED](../../../target/issue-106/red.log),
[GREEN](../../../target/issue-106/green.log) and
[preparation](../../../target/issue-106/preparation.log).
The [test-only patch](../../../target/issue-106/test.patch) and
[full authored patch](../../../target/issue-106/handoff.patch) provide staged
review inputs. Diff and local-link checks are retained in
[delivery checks](../../../target/issue-106/delivery-checks.log).

The separate reviewer must assess all 196 proposed fields and 49 uses, including
source-import rights and spelling/construction fit, then record actual accepted
text and reviewer identity. Candidate checks are not independent review.
Converter reconstruction, notices packaging, resource size/performance and real
native lookup/save/reopen remain integration checks after admission. Frontend,
Rust and native checks were not run for this evidence-only author handoff.
