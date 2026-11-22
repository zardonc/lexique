# Issue #136 author package

**50 frozen groups / 200 sourced field candidates** cover rows **2751–2800**.
The four proposed fields and selected/excluded uses are readable in
[review candidates](review-candidates.md), with exact records in
[groups](groups.json), [field review](field-review.json) and
[source research](source-research.md). Independent acceptance is **0 groups /
0 fields**. `man/NOM` remains an explicit author blocker.

## Scope and review inputs

- Ticket #136 and updated parent #78: retained JSON under
  `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/issue-130-139`.
- Author checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-136`,
  branch `codex/d2-content-136`, author diff base
  `e4005d868880dac12c84c412773049eea737bc67`.
- Changes: only `docs/evidence/issue-136/**` and
  `scripts/test_core_batch_136.py`. The canonical core remains the separate
  integration owner's responsibility.
- Baseline: `core-129-v2`, 2,435 admitted content groups; no reused group or
  field is double-counted. The denominator remains **4,087**. All 50 fixed
  identities, frequencies and order are preserved, with no new annotation,
  specialist exception, substitution or runtime alias.
- Original proposals: [authored groups](authored/groups.json),
  [prose](authored/prose.json), [review](authored/field-review.json),
  [rights](authored/rights.json) and the other `authored/` JSON records.

## Evidence and stable fallback

**Stable author evidence:**
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-136`.
[Admission](admission.json) and [source captures](source-captures.json) reference
this retained fallback, rather than depending on the ignored worktree cache.
It contains source/API captures, all contributor pagination and request URLs,
policy/legal captures, selection and author helpers, source frames,
reproduction output, actual check logs and the scoped review patches.

54 fixed French Wiktionnaire articles supply lexical and construction evidence;
one additional Bob/PetitBob notice supplies research-only appellative semantics.
Exact source revisions/spans, context, credits and modifications are retained in
[sources](sources.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [rights](rights.json) and
[candidate notices](NOTICES.md). The [CC BY-SA 4.0 text](COPYING) accompanies the
proposed Wikimedia/Lexique contribution layer. Original DAF8 and Littré layers
retain their unchanged, previously reviewed original-work boundaries; no new
field inherits an independent acceptance from those decisions.

## Validation

Platform: Windows x64, PowerShell 7, uv-managed Python 3.14.10. The authorized
seam is the delivered resource/admission package. It detects frozen membership
drift, incomplete/misbound fields, false acceptance, missing rights records and
missing archived evidence; it does not establish bilingual quality or native
behavior.

| Command | Result |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_136.py` | Passed, three focused boundary checks; actual expected failures retained in `red.log`, `red-rights.log`, `red-archive.log` |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-136/reproduce.py` | Passed, 50 groups / 200 proposed fields and ten original author JSON records reproduced byte-for-byte |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-136/fallback-checkout/scripts/test_core_batch_136.py` | Passed, three checks from a checkout with no local source cache; stable fallback supplied captures |
| `pnpm test:docs` | Passed, tracked documentation format and repository links |
| `git diff --cached --check` | Passed, scoped staged package |

The archive's `final-verification.json` records the actual author commit,
integration tip and tested SHA. `package.patch` contains the full scoped author
diff; `test.patch` isolates the resource-boundary test. For reproduction, restore
the archived helper/capture tree into this checkout's `target/issue-136` and run
`reproduce.py`; it writes a separate reproduction directory and compares the
retained original proposals. Do not run root-resolving helpers directly from
the external archive.

## Limitations

`man/NOM` has a semantic proposal and French address IPA, but explicit modern
NOM eligibility and Bob/PetitBob original-prose/source attribution rights are
unresolved. Bob does not supply an explicit NOM label; Wiktionnaire labels the
address use as an interjection. No reusable Bob/PetitBob source-text licence or
complete individual contributor registry is asserted. The proposed /man/ comes
from that French address use; PetitBob's /mɑ̃/ is not selected. See the exact
blocker in [admission](admission.json). This group is preserved, with no exemption.

All **200 fields** await independent quality and selected-span rights review;
all **200 required admitted fields remain missing**. Combined conversion, packaged notices,
native lookup/save/reopen and learner acceptance remain pending. Application,
frontend, Rust, native/offline and release checks were not run for this author
package. Candidate completeness does not close #136 or the wider core gate.
