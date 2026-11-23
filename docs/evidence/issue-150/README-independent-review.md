# #150 independent field and selected-rights review

**200 fields and 50 everyday uses accepted; zero unresolved.** Of these,
197 fields and 49 uses are unchanged; `publicité` has three replacement meaning
fields and one revised use. Exact frozen rows **3451–3500**, headword/POS
membership, frequencies and order are preserved.

Reviewer: **Independent semantic and selected-rights reviewer #150; separate
from author and integration owner**. This is agent review, with learner
acceptance a separate gate. Author commit:
`9df866ce2f4508b3ec50afd9b9405c3dae4d9e0b`; author head:
`ade84d1ad87da42084deb8d08e9bc812948f91ab`. Initial review base:
`e7bbe0afe7ab156024f8c2bee61fd57a0e2057bc`.
Checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/review-150`, branch
`codex/d2-review-150`. Only this evidence directory and
`scripts/test_core_batch_150.py` are owned review changes.

The exact ticket and parent spec were read from the sibling `context/`
captures. [Authored snapshots](authored/snapshot.json) retain the initial files.
[Field review](field-review.json) supplies each field's actual linguistic reason,
exact proposed/accepted text, source and separate rights decision; all 50 use
and spelling decisions have individual reasons. [Original-expression review](original-expression-review.json)
records each of the 54 sources and its original/support boundary.

`publicité` selects a commercial advertisement at DAF8-backed line 26;
[replacements](accepted-replacements.json) retain the exact FR/EN/ZH texts and
source spans. `tailler`'s inherited wording has original DAF8 credit.
`tranquillement` retains DAF8/Littré original credits and separately fixed calm
adjective support. `demeure` preserves formal register; `baby` the informal
game abbreviation; `mark` the former-currency/euro-replacement condition.
`se fiancer` and `s’effondrer` preserve pronominal grammar and bare-infinitive IPA.

## Verification

Four focused resource tests pass: frozen membership/four-field completeness,
the exact `publicité` boundary, selected source/rights consistency and exact
proposal/replacement/use/original-credit alignment. The focused `publicité`
regression failed against the authored scope before the accepted correction.
The source audit passes 54 raw/API byte-equal articles, 200 field spans and
871 selected/supporting occurrences across 258 unique locations. Eleven
contributor responses complete six chains, with 3,232 named article memberships
and 480 anonymous counts. Policy raw captures match API content after CRLF-to-LF
normalization; article byte equality is literal.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_150.py
uv run --python 3.14.10 python -B -X utf8 target/issue-150-review/audit.py
pnpm test:docs
git diff --check
```

Author evidence: `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/author-evidence/issue-150/`.
Independent helper/proof/log archive:
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/reviewer-evidence/issue-150/issue-150-review/`.
The sibling `final-verification.json` records tested source, integration merge,
commands and results. Helpers consume actual review decisions rather than
assigning linguistic verdicts.

## Limitations

Canonical assembly, converted artifact/combined notices, native all-group
lookup/save/reopen, learner acceptance and wider release gates remain
coordinator-owned and unrun. Independent acceptance admits zero canonical fields
by itself. The `délire` original-edition noun comparator was unavailable; the
selected nonmedical wiki definition is independently assessed without claiming
an original-edition textual match.
