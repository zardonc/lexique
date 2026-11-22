# D2 #132 author package

Frozen rows **2551–2600** retain **50 exact groups, 50 candidate senses and
200 sourced fields** for [ticket #132](https://github.com/zardonc/lexique/issues/132)
under [parent #78](https://github.com/zardonc/lexique/issues/78). There are
**49 modern-use proposals and one unresolved noun group, `oublie/NOM`**.
The denominator remains **4,087 groups / 3,773 headwords**. No group, POS
subclass or spelling is replaced, and no exception or query alias is admitted.

## Reviewer package

[Groups](groups.json) contain the proposed FR/IPA/EN/ZH text and selected uses.
[Original review candidates](review-candidates.json) retain the author wording
for comparison with any later accepted replacements. [Field review](field-review.json)
contains 200 pending field decisions, 50 use records and 50 spelling relations.
[Admission](admission.json) reports zero newly admitted or independently accepted
fields, all 200 fields pending separate quality and rights review, and the
four fields held by `oublie`’s unresolved everyday eligibility.

[Sources](sources.json), [captures](source-captures.json),
[context](source-context.json) and [excerpts](source-excerpts.json) identify
50 fixed French Wiktionnaire articles, their revisions, numbered field spans,
grammar/construction evidence and complete contributor attribution. Seven
paginated contributor responses retain 3,113 named contributor memberships,
returned anonymous counts and linked complete histories. The current registry
does not establish authorship of an individual selected sentence.

[Selected-use notes](source-research.md), [rights proposals](rights.json),
[selected rights](selected-rights.md), [notices](NOTICES.md) and [legal text](COPYING)
provide the selected import/original-work boundaries, licence terms and
modification obligations. [Reuse](reuse.json) records no overlap with the
admitted `core-129-v1` inputs; existing resources and saved learner content are
outside this package’s write scope.

## Reproduction and author checks

The retained archive is
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-132`.
It contains the exact [ticket JSON](../../../target/issue-132/issue-132.json),
[parent spec](../../../target/issue-132/spec.md), explicit
[selection](../../../target/issue-132/selection.py), preparation scripts,
API request/response files, full raw articles, contributor pagination,
unchanged authority captures and check logs. The working copy is
`target/issue-132`; [capture records](source-captures.json) name both locations.
To reproduce in another checkout, restore the archive to `target/issue-132`.

```powershell
uv run --no-project python -B -X utf8 target/issue-132/reproduce.py
uv run --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_132.py -v
uv run --no-project python -B -X utf8 target/issue-132/recapture_fixed.py
pnpm test:docs
git diff --cached --check
```

The focused test observes the parent-approved resource/admission seam: frozen
membership, complete candidate fields, exact source/POS/IPA bindings,
attribution/rights proposals, selected restrictions and honest pending states.
Its raw-source test requires the local or archived captures and explicitly
skips when neither is available. It does not certify semantics independently
or verify native lookup and storage.

Author checks passed: four focused tests with no skips, offline byte-for-byte
reproduction of ten JSON evidence files and article notices, and live
fixed-revision recapture of all 50 complete articles. Initial membership and
unresolved-use failures are retained under ignored paths. See
[check report](author-checks.json), [resource RED](../../../target/issue-132/canonical-admission-red.log),
[membership RED](../../../target/issue-132/membership-red.log),
[eligibility RED](../../../target/issue-132/blocker-red.log),
[GREEN](../../../target/issue-132/green.log),
[reproduction](../../../target/issue-132/reproduction.log) and
[fixed recapture](../../../target/issue-132/fixed-recapture-retry.log).

## Remaining gates

`oublie/NOM` at row 2560 has a sourced old-fashioned wafer sense, four candidate
fields and its own pronunciation. Both the fixed Wiktionnaire noun definition
and Académie française ninth-edition entry as rendered by CNRTL mark it
old-fashioned. A supported modern everyday noun sense remains unestablished.
The separate reviewer must assess this evidence; any exemption requires the
project owner’s actual approval. The row remains in the ordinary denominator.

All 200 proposed fields require an actual separate independent quality and
selected-span rights decision. Combined conversion/notices, native lookup for
all exact groups, trusted representative save/restart/reopen and preservation
checks belong to the coordinator. Learner acceptance and release gates remain
pending. This author package does not complete #132 or native admission.
