# D2 #128 independent content and rights review

**50/50 selected uses and 200/200 fields pass independent review:** 197 fields
unchanged and three construction replacements for Québec `sacrer/VER`.
There are zero unresolved or pending independent fields, zero pending source
frames, and no annotation exception, specialist exception or runtime alias.
The frozen rows **2351–2400**, exact headwords/POS/frequencies and **4,087-group**
denominator are preserved. Canonical admission remains **0 new fields / 200
pending fields** until the parent assembles and tests the resource.

The [independent report](independent-review.md), [200 field decisions](field-review.json),
[selected rights](selected-rights.md) and [three exact replacements](accepted-replacements.json)
record the review. [Groups](groups.json) contain accepted wording;
[original candidates](review-candidates.json), [authored admission](admission-author-proposal.json),
[authored README](README-author-proposal.md) and [authored rights narrative](selected-rights-author-proposal.md)
preserve proposals and the original pending state. The [live ledger](admission.json)
reports the actual independent result separately from parent gates.

All **47 fixed French articles**, **200 selected field bindings** and **189
language/POS/form/construction frames** were checked against retained bytes.
Six complete contributor responses record **2,749 named memberships**, anonymous
counts and full-history attribution; that registry is not selected-sentence
authorship proof. [Sources](sources.json), [captures](source-captures.json),
[spans](source-excerpts.json), [context](source-context.json), [rights](rights.json),
[notices](NOTICES.md) and [legal text](COPYING) retain those boundaries.

Verification in `C:/Users/inthe/IdeaProjects/lexique-review-128`, branch
`codex/d2-review-128`:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_128.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128-review/audit.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128-review/check_review_mutations.py
pnpm test:docs
git diff --cached --check
```

The [complete test log](../../../target/issue-128-review/full-green.log),
[fixed-source byte audit](../../../target/issue-128-review/fixed-source-byte-audit.log),
[canonical overlap and negative checks](../../../target/issue-128-review/overlap-and-mutations.log)
and [verification record](../../../target/issue-128-review/verification.json)
are retained under ignored `target/issue-128-review/`, including RED/GREEN logs.
Author raw/current/fixed API captures, contributor continuations and legal
authorities remain at `C:/Users/inthe/IdeaProjects/lexique-batch-128/target/issue-128`;
the integration archive is the alternate retained root. The independent audit
used retained captures and made no upstream requests.

Canonical conversion, combined packaged notices, native lookup/save/reopen,
learner acceptance, offline/platform and release checks remain parent-owned.
This review does not assert application or canonical admission.
