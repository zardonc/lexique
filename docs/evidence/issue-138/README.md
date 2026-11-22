# D2 issue #138: author package for independent review

**50 exact groups, 50 selected senses and 200 four-field candidates** cover
frozen rows 2851–2900 from [issue #138](https://github.com/zardonc/lexique/issues/138)
under the [updated parent #78](https://github.com/zardonc/lexique/issues/78).
**Zero new groups or fields are independently accepted or admitted.**
Independent quality, selected-span rights, spelling/use, conversion/packaging
and native lookup/save/reopen remain pending.

## Review inputs

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-138`,
branch `codex/d2-content-138`. The clean branch was brought forward to integration
commit `d3725aa325ed9ac7222c2be80527efb798508485` before authoring. Live ticket and
parent captures matched the supplied context exactly. Captures, editorial
selection, helpers, logs and review patches are retained at:

`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-138`

The archive’s `final-verification.json` identifies author and final commit hashes,
tested checkout/platform, integration ancestry and retained diff commands.
Full `author-package.diff` and separate `resource-test.diff` cover only
`docs/evidence/issue-138/**` and `scripts/test_core_batch_138.py`.

Review [the exact readable candidates](review-candidates.md) against
[original author proposals](authored/), [groups](groups.json),
[field/use/spelling proposals](field-review.json), [source excerpts](source-excerpts.json),
[context and contributors](source-context.json), [fixed capture inventory](source-captures.json),
[rights](rights.json), [selected rights](selected-rights.md),
[notices](NOTICES.md) and [legal text](COPYING).
No author check is an independent review. All 200 new fields need actual separate
language, meaning/construction and selected-source/rights decisions.

## Content and coverage

Exact spelling, POS subclass, row order and both frequency strings are preserved.
Each group has one complete FR/IPA/EN/ZH proposed sense and explicit excluded uses.
The fixed denominator stays **4,087**; reuse, substitutions, new exceptions and
new lookup aliases are all zero. CEFR remains unknown. English/Chinese are new
Lexique explanations based on selected French meanings, not bilingual quotes.

The package distinguishes edible `riz` from the plant, waking `réveil` from the
alarm device, `araignée` from the obsolete web sense, and material `plastique`
from the adjective and feminine nouns. `éternité` uses ordinary hyperbole;
`féminin` selects grammatical gender without equating it to women. Registers,
pronunciation variants, gender and construction evidence remain in the records.

Three specific review questions are recorded in [admission](admission.json):

- `u/NOM`: the exact lowercase French masculine invariant letter name, /y/;
  French letter section, nominal examples and name heading are retained.
- `halte/ONO`: the stopping-command interjection specifically directed at
  soldiers, with h aspiré. Review everyday eligibility at that restricted use.
- `signature/NOM`: selected handwritten-name definition, nominal IPA and
  unselected printing/cartography sense; retain the actual CFC declaration
  and institutional credit while reviewing selected-sentence origin.

`aïe/ONO` is a pain cry, not the obsolete horse command. No group is omitted or
granted an exception. There are no missing authored fields; all **200 fields
remain missing from admission** until their separate gates pass.

## Checks and reproduction

Run from the author checkout with PowerShell 7 and the project’s pinned Python:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_138.py
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_138.py --admitted Batch138Tests.test_exact_ticket_resource_groups
uv run --python 3.14.10 python -B -X utf8 target/issue-138/reproduce.py
pnpm test:docs
git diff --cached --check
```

The default resource checks pass for exact membership/frequencies, four fields,
pending gates, API/raw revision equality, complete contributor continuations,
French POS/IPA spans, actual special forms and retained rights authorities.
The focused `--admitted` check fails as expected: canonical content has none of
these 50 groups. Its RED log remains in the archive. It must become green only
after genuine review and coordinator assembly.

For reproduction, copy the complete archived directory into this checkout’s
ignored `target/issue-138` without discarding existing work, then run the owned
`reproduce.py`. It writes only the author package and compares the generated
JSON, notices and readable proposals byte-for-byte. Do not execute archived
helpers in place: their repository root is resolved from the owned target path.
Do not run author writers on accepted reviewer output.

Actual results are in [verification](verification.json). Existing canonical
resource inputs and learner data are unchanged. Native execution, converter
assembly, packaging, learner acceptance and release readiness are coordinator
gates; this package establishes author availability and fixed-source evidence.
