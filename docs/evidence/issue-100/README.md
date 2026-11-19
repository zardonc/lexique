# Issue #100 author handoff

[Ticket #100](https://github.com/zardonc/lexique/issues/100), under
[spec #78](https://github.com/zardonc/lexique/issues/78), retains all **50 exact
frozen groups at rows 951–1000**. Each has one selected candidate sense and four
nonempty FR/IPA/EN/ZH fields: **200 candidate fields, zero admitted fields**.
Independent content/rights review is pending for every field. The permanent
denominator is **4,087 groups / 3,773 headwords**. No annotation exception,
runtime alias or reuse is proposed.

The seven-file handoff comprises [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). [Review candidates](review-candidates.json) preserve
the author-stage envelope; [source research](source-research.md) describes the
selected uses and actual rights boundaries. [Source captures](source-captures.json)
locate the fixed original evidence. `independence: false`, null reviewer identities
and pending statuses describe an unperformed independent review; author checks
do not establish admission.

## Review checkout and evidence

- Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-100`.
- Branch: `codex/d2-content-100`.
- Starting integration base: `aae5273af846e57668a8d267fa6a6566c4fd6e91`.
- Integrated tip before author commit: `87d4b3124bdb06f2727c817fa18bbbcd6c5c5cc5`.
- Reviewed changes: staged author files only; `git diff --cached --binary`.
- Full retained patch: `target/issue-100/authored.patch`.
- Test-only patch: `target/issue-100/test.patch`.
- Original captures and logs: `C:/Users/inthe/IdeaProjects/lexique-batch-100/target/issue-100/`.
- Platform: Windows, PowerShell 7, uv-selected Python 3.14.10.

The [focused check](../../../scripts/test_core_batch_100.py) checks exact frozen
membership/frequencies, four-field bindings, the pending review contract and
original API/wikitext equality with one-based excerpt locations. Its original
snapshot test requires the owning evidence root; detached checkouts without
those captures explicitly skip that part.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_100.py -v
```

The first missing `connerie/NOM` candidate produced the retained RED in
`target/issue-100/red.log`; the first candidate passed in `green-first.log`.
The exact 50-group expansion produced RED in `red-membership.log`, then GREEN
in `green-membership.log`. Final author verification passed all three checks,
including all 200 locations against the 42 original API-backed articles, in
`green-final.log`. Scoped documentation/link and diff checks are retained in
`docs-check.log` and `diff-check.log`.

## Limitations

All **200 fields remain missing from admitted coverage** until a separate
reviewer accepts their exact wording, modern use, POS/IPA fit and source rights.
Priority boundaries are row 959's adjective sense descriptor, row 971's
interjection/ONO and same-spelling IPA mapping, and row 978's general abstract
noun. [Pending review](independent-review.md) identifies them. No unsupported
frozen identity or field-level absence was established during author work;
that finding does not settle independent quality review.

Shared canonical content, conversion, notices assembly and native
lookup/trusted save/reopen are integration responsibilities. This author work
does not establish resource admission, native execution, offline operation,
learner acceptance or release readiness. Application/native checks were not run.
