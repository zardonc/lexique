# D2 #120 author package

The package proposes **50 exact groups, 50 selected senses and 200 fields** for
frozen rows **1951–2000**. Every selected sense has French definition, IPA,
English meaning and Chinese meaning for the same use. All **46 French source
articles** have fixed revisions, exact numbered spans and complete retained
contributor pagination: **7 responses, 3,332 named contributor memberships**.

[Groups](groups.json) and [review candidates](review-candidates.json) contain
the four-field content. [Field review](field-review.json) contains one pending
review record per field, with selected use, construction and spelling evidence.
[Admission](admission.json) retains the **4,087-group** global denominator.
[Reuse](reuse.json) compares the exact slice with admitted `core-119-v1`:
**0 reused groups / 0 reused fields**. No annotation or specialist exception is
proposed.

## Fixed evidence

- [Sources](sources.json): article identity, revision, timestamp, fixed URL,
  complete history, contributor attribution and licence.
- [Source captures](source-captures.json): retained first-party API responses,
  raw wikitext, request URLs, current contributor pagination and recapture entry.
- [Selected spans](source-excerpts.json): four field bindings per sense plus
  French language, exact POS, headword, grammar and construction spans.
- [Source context](source-context.json): contributors, anonymous counts,
  import/reference boundaries and 50 source spelling relations. The relations
  bind source evidence only; no lookup alias is proposed.
- [Rights proposals](rights.json), [selected rights](selected-rights.md),
  [notices](NOTICES.md) and [CC BY-SA 4.0 legal text](COPYING): fixed policies,
  field-level modification/attribution records and scoped original-work layers.
- [Selected-use notes](source-research.md): register, construction and semantic
  boundaries requiring attention during separate review.

All **200 fields** await independent semantic, rights and construction/spelling
review. Independent acceptance is **zero**; contributor pagination and author
checks do not establish independent quality or selected-sentence authorship.
Unselected quotations, media, etymologies and other-language sections supply no
content fields. The author package does not admit the source dictionaries as a
whole.

## Reproduction and checks

The approved test seam is the resource/admission record. Membership is checked
against the self-contained ticket and frozen manifest; source checks compare
each proposed span with retained first-party revision content and verify
contributor continuation, field rights bindings and pending review status.
The two TDD slices and subsequent checks are retained in
[membership RED](../../../target/issue-120/membership-red.log),
[membership GREEN](../../../target/issue-120/membership-green.log),
[source/rights RED](../../../target/issue-120/source-rights-red.log) and
[final GREEN](../../../target/issue-120/green.log).

Live fixed-revision recapture matches all **46 complete wikitext articles byte
for byte**. Offline preparation reproduces all **10 JSON evidence files byte
for byte** and the article notices, using retained revisions and author selections.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-120/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-120/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-120/write_notices.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_120.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-120/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-120/recapture_fixed.py
pnpm test:docs
git diff --cached --check
```

The [selection](../../../target/issue-120/selection.py),
[preparation](../../../target/issue-120/prepare_candidates.py),
[rights preparation](../../../target/issue-120/finish_author_package.py),
[notice preparation](../../../target/issue-120/write_notices.py),
[reproduction](../../../target/issue-120/reproduce.py),
[fixed recapture](../../../target/issue-120/recapture_fixed.py), original raw
responses and request files remain under ignored `target/issue-120/` for parent
archival. See [recapture log](../../../target/issue-120/fixed-recapture.log),
[reproduction log](../../../target/issue-120/reproduction.log),
[documentation log](../../../target/issue-120/docs-check.log) and
[final verification](../../../target/issue-120/final-verification.json).

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-120`, branch
`codex/d2-content-120`; initial base
`b91c4729a0a1c93baa58f86ed666b173fb2fc253` descended from integration
`codex/d2-content-120-129`. The latest integration tip is merged before handoff;
the final verification record identifies the author and merged commits.

## Remaining gates

Independent review and canonical resource assembly, combined redistribution
notices, native lookup/save/reopen, learner acceptance, platform/offline checks
and release readiness remain parent-owned. Application and native checks were
not run by this author. The existing canonical resource and learner data are
unchanged.
