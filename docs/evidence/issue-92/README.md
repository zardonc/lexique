# Issue #92 content handoff

This additions-only candidate artifact covers all **50 exact frozen groups,
rows 551–600**, with one selected use and four FR/IPA/EN/ZH fields per group.
There are **200 proposed fields**, **0 independently accepted new fields** and
**0 reused groups**. All source and field acceptance remains pending separate
independent review.

[Candidate records](review-candidates.json), [exact selected excerpts](source-excerpts.json),
[source contexts](source-context.json) and [review drafts](field-review-draft.json)
are ready for central review. [Capture locations](source-captures.json) identify
the 41 real fixed Wiktionnaire revisions. The source API capture occurred on
2026-11-19 UTC; these are the returned fixed revisions, not an asserted October
cutoff snapshot. Whole pages and retrieval/error logs stay under
[the ignored evidence root](../../../target/issue-92/).

The focused [membership check](../../../scripts/test_core_batch_92.py) failed
before content was added, then passed against the candidate artifact. It checks
all exact ticket identities, nonempty four-field content and fixed excerpt
bindings. It does not claim conversion or independent quality acceptance.
The shared converter still accepts only the parent's reviewed inputs.

[Admission status](admission.json) records the outstanding gates.
`nouvelle/ADJ` has its own feminine pronunciation and an explicit relationship
to `nouveau`; `soeur/NOM` retains its frozen spelling with evidence for the
typographic variant `sœur`. Neither relation is admitted as a runtime alias by
this worker. `prochain/NOM` selects the ordinary moral/religious use for a fellow
human; independent everyday-use acceptance remains pending. `con` retains
its vulgar, derogatory and insulting register in both grammatical uses.

Imported prose retains its original-author boundary. Source contexts retain
the actual DAF8 import/reference notices where present and identify the
Académie française, eighth edition (1932–1935), separately from Wiki changes
and Lexique translations/adaptations. Absence of an import notice does not
establish original authorship. No underlying public-domain or imported-prose
right is independently accepted here. Contributor histories are linked;
full-history API retrieval was not completed because attempted requests returned
HTTP 429. No contributor count or last editor is treated as sentence authorship.

The proposed Wiki selections and new Lexique translations/adaptations use the
[CC BY-SA 4.0 route](../../../resources/core-pilot-79/COPYING), subject to the
separate original-prose boundary decisions. The existing legal texts and notices
are retained for the parent to assemble against independently accepted fields.
`pnpm test:docs` and the staged diff check passed; complete logs are in the
ignored evidence root.

The parent owns independent field/rights review, the combined obligations,
converter row list, counters, checksums, notices, native lookup/save/reopen,
preservation checks and packaging. Native GUI builds, shared Cargo checks,
learner acceptance and release validation were not run in this worktree.
