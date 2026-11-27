# Issue #161 independent review

**50 groups / 200 fields accepted: 199 unchanged fields, one replacement,
50 unchanged uses, no unresolved blockers.** Every text and precise selected
French POS/use was read independently of the author proposals.
Exact frozen rows **4001–4050** and both frequency strings remain unchanged.

Reviewer: Independent field/use and selected-rights reviewer #161, separate
from author and integration owner. Spec captures: `issue-161.json` and
`issue-78.json` under `C:/Users/inthe/IdeaProjects/lexique-d2-160-162/context`.
Author branch: `t3/implement-issue-161`; author SHA:
`10a62d9c4d0ced4515ba45dfb1922e4ec1c065e8`. It was merged into the prepared
review checkout before review. Integration base:
`0963287ea5ed593cc84215377d62e6db1a072566`.
Review branch: `t3/review-issue161-shard`; checkout:
`C:/Users/inthe/.t3/worktrees/lexique/t3-05e83437-6abb18de8d947ccf`.
Only the issue161 shard and `scripts/test_core_batch_161.py` are changed.

[Field decisions](field-review.json) retain all 200 exact proposed/accepted
texts and individual semantic reasons, 50 everyday/POS/use reasons and
50 exact-spelling decisions. [Rights decisions](rights.json) and
[original-expression decisions](original-expression-review.json) retain the
separate expression-level reasons and original-work credits.
[Authored snapshots](authored/README.md) are untouched.

The [single replacement](accepted-replacements.json) changes `vaurien` English
from `(Informal, less severe than an insult) A rascal or scamp.` to
`(Informal, in a milder sense) A rascal or scamp.` Fixed wiki line 17 says
`dans un sens moins sévère`, comparing senses without defining an insult
threshold. The same definition/IPA/source boundaries are retained; use and
the other three fields are unchanged.

## Evidence and checks

All **50 raw articles** equal their retained first-party API revision bytes.
All **200 selected fields** and **196 supporting records** match source
locations. The recursive consistency audit checks **4,210 recorded span
occurrences / 428 unique source lines**, including source/context/rights
copies. Eight contributor responses form five complete chains with matched
continuation requests, **2,784 named article-contributor credits** and
**450 anonymous counts**. These are article credits, not sentence authorship.
Policy/template raw captures equal API content after the recorded CRLF
normalization; lexical article equality is literal.

Six focused resource tests pass: frozen membership/four scoped fields,
fixed bytes/French POS/IPA spans/contributor pagination, independent decisions
and proposal preservation, restrictions/excluded CFC content, original
credits/authored preservation, and the exact `vaurien` boundary. The
`vaurien` check failed against authored English and passed after replacement.
The independent audit also passes source-copy equality, notices/licence,
all authored tracked files against the author SHA, and changed-path scope.
Documentation/link and whitespace results are in [verification](review-verification.json).

Retained evidence:
`C:/Users/inthe/IdeaProjects/lexique-d2-160-162/context/reviewer-evidence/issue-161`.
It contains audit/application helpers, explicit decisions, source and legal
bytes, 41 original-edition comparison captures, full logs and the final
SHA-bound delivery record. Source tests resolve this recorded owning root.

## Limitations

Canonical assembly, converted artifact/combined notices, native lookup/save/reopen,
learner acceptance and release checks remain coordinator-owned and unrun.
No canonical resources, shared code or learner data are changed. The DAF8 `uni`
adjective comparator has no entry content; conservative original credit is
retained without claiming an original-edition sentence match.
