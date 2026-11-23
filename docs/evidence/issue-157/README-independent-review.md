# Independent #157 content review

The separate reviewer `independent-content-reviewer-157 / codex/d2-review-157`
reviewed all 200 fields and all 50 exact use/POS/spelling relations. Result:
**50 complete groups, 200 accepted fields, 197 unchanged, three corrected,
zero unresolved accepted fields**. French, English and Chinese decisions have
actual language-specific reasons; IPA is read from each applicable French head.

Author content commit: `2b46533`; final author report: `70b0b05`.
Review checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/review-157`.
Branch: `codex/d2-review-157`; initial review base: `79e6b44`, after merging the
current integration tip from the assigned `069d3ca` base. Final integration
tip merged before delivery: `e7db3a58473191f762b1ebd3ff72fb29a45687e3`.
The tested review revision is recorded in the execution archive.
Scope: committed author content plus this review’s owned evidence/test changes.
The review changes only `docs/evidence/issue-157/**` and
`scripts/test_core_batch_157.py`.

[Preserved author snapshot](authored/snapshot.json) and
[exact replacements](accepted-replacements.json) retain proposed and accepted
text/spans. The source audit/capture and established schema helpers were reused
from #151 and #150; their automation supplies no linguistic verdict. The
reviewer independently read fixed raw fields, support frames, contributor
continuations, original expressions, rights and modifications. The three
`crapaud` meanings are limited to the DAF8-corresponding frog-resemblance clause
and modern amphibian terminology; unverified Quillet additions are excluded.

## Verification

Platform: Windows, PowerShell 7, uv Python 3.14.10.

- Focused `crapaud` resource/source-boundary regression: failed on the authored
  selection, then passed on the accepted selection; `RED.log` and `GREEN.log`.
- `uv run --python 3.14.10 python -B -X utf8 target/issue-157-review/audit.py`:
  51 literal API/raw articles, 250 field/use spans, 980 supporting occurrences,
  ten complete contributor files; passed.
- `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_157.py`:
  exact frozen membership/frequency, source/IPA/POS, accepted field/use/rights,
  preserved author rejection and original-expression boundaries; results in
  `batch-tests.log`; all seven tests passed.
- `pnpm test:docs`: passed, 558 documents / 4,270 local links, zero problems;
  422 local evidence links are counted separately by the validator.
- `git diff --cached --check`: passed.

Retained reviewer helpers, original HTML/text, complete logs and scoped patch:
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/reviewer-evidence/issue-157`.
Author raw sources remain separately at
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-157`.

## Limitations

The original Quillet expression/terms for excluded `crapaud` additions were not
verified. Empty, redirected or 404 original URLs are recorded as such; applicable
POS-specific DAF8 captures are separate evidence. Article contributor registries
do not establish sentence-level authorship. This is a separate agent content
review, with no human learner signoff claimed.

Canonical/global resource changes, combined packaging, frontend/Rust/native
checks and learner acceptance were not run here; the coordinator owns them.
The whole #78 core and release gates remain incomplete.
