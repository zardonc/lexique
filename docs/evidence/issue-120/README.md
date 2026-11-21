# D2 #120 independently reviewed content

The package has **50 accepted exact groups, 50 selected senses and 200 fields** for
frozen rows **1951–2000**. Every selected sense has French definition, IPA,
English meaning and Chinese meaning for the same use. All **46 French source
articles** have fixed revisions, exact numbered spans and complete retained
contributor pagination: **7 responses, 3,332 named contributor memberships**.

[Groups](groups.json) and [review candidates](review-candidates.json) contain
the accepted and original four-field content respectively. [Field review](field-review.json)
is a separate independent review, with one accepted record per field and
specific semantic, rights, use, construction and spelling reasons.
[Admission](admission.json) retains the **4,087-group** global denominator.
[Reuse](reuse.json) compares the exact slice with admitted `core-119-v1`:
**0 reused groups / 0 reused fields**. No annotation or specialist exception is
admitted. [Independent review](independent-review.md) records **197 unchanged
fields and three exact `épée` replacements**, with zero unresolved decisions.
Original [admission](authored-admission.json), [field review](authored-field-review.json),
[rights](authored-rights.json), [sources](authored-sources.json) and
[context](authored-source-context.json) snapshots retain the author proposals.

## Fixed evidence

- [Sources](sources.json): article identity, revision, timestamp, fixed URL,
  complete history, contributor attribution, licence and passed selected-span frames.
- [Source captures](source-captures.json): retained first-party API responses,
  raw wikitext, request URLs, current contributor pagination and recapture entry.
- [Selected spans](source-excerpts.json): four field bindings per sense plus
  French language, exact POS, headword, grammar and construction spans.
- [Source context](source-context.json): contributors, anonymous counts,
  import/reference boundaries and 50 source spelling relations. The relations
  bind source evidence only; no lookup alias is proposed.
- [Reviewed rights](rights.json), [independent rights review](rights-review.md), [selected rights](selected-rights.md),
  [notices](NOTICES.md) and [CC BY-SA 4.0 legal text](COPYING): fixed policies,
  field-level modification/attribution records and scoped original-work layers.
- [Selected-use notes](source-research.md): register, construction and semantic
  boundaries accepted during separate review.

All **200 fields** pass independent semantic, rights and construction/spelling
review. Contributor pagination does not establish selected-sentence authorship.
Unselected quotations, media, etymologies and other-language sections supply no
content fields. This package does not admit the source dictionaries as a
whole.

## Reproduction and checks

The approved test seam is the resource/admission record. Membership is checked
against the self-contained ticket and frozen manifest. Checks verify every
accepted text, proposal, source/rights binding and use; subsequent canonical
overlap is allowed only for identical whole records. Retained
[RED](../../../target/issue-120-review/content-rights-red.log),
[GREEN](../../../target/issue-120-review/content-rights-green.log) and
[full focused results](../../../target/issue-120-review/green.log) cover the
approved resource/admission seam.

The independent offline [byte audit](../../../target/issue-120-review/fixed-source-byte-audit.json)
matches all **46 complete wikitext articles byte for byte** with retained
first-party revision and fixed-recapture responses, all 200 spans and complete
contributor pagination. No new upstream requests or 429 retries were made.

```powershell
uv run --python 3.14.10 scripts/test_core_batch_120.py -v
uv run --python 3.14.10 target/issue-120-review/audit_sources.py
pnpm test:docs
git diff --cached --check
```

The original author scripts, raw responses, requests and execution logs are
copied byte-for-byte into ignored `target/issue-120-review/author-evidence/`
for parent archival. The originals remain in the author and integration
worktrees. [Documentation checks](../../../target/issue-120-review/docs.log)
and [final verification](../../../target/issue-120-review/final-verification.json)
record the current review commands and statuses. Author reproduction scripts
describe the preserved proposals; they do not generate the accepted review.

Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-120`, branch
`codex/d2-review-120`; initial review base
`92c208d2154c262035c8781463e3218463fac04f`. The latest integration tip is merged
before handoff; final verification records the tested head and merged tip.

## Remaining gates

Canonical resource assembly and reproduction, combined redistribution
notices, native lookup/save/reopen, learner acceptance, platform/offline checks
and release readiness remain parent-owned. Application and native checks were
not run by this reviewer. The existing canonical resource and learner data are
unchanged.
