# Issue #107 candidate handoff

Frozen rows **1301–1350** have **50 candidate groups and 200 candidate fields**
from 47 fixed Wiktionnaire articles. Every group retains its exact spelling,
Lexique POS subclass and both original frequency strings. The denominator is
**4,087 groups**. **Zero groups and zero fields are admitted**: all selected
content and rights decisions await a separate reviewer.

Scope: [ticket #107](https://github.com/zardonc/lexique/issues/107) under
[spec #78](https://github.com/zardonc/lexique/issues/78). Author checkout:
`C:/Users/inthe/IdeaProjects/lexique-batch-107`, branch `codex/d2-content-107`.
The confirmed common base with integration `codex/d2-content-100-109` is
`aae5273af846e57668a8d267fa6a6566c4fd6e91`. The integration harness and other
authors' disjoint packages are preserved. This handoff changes only this evidence
directory and [the focused candidate check](../../../scripts/test_core_batch_107.py).

The seven-file handoff is [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). [Review candidates](review-candidates.json) contain
the identical draft groups. Review records hold proposed text and use, explicit
pending quality/rights statuses, no reviewer and no claim of independence.
No reused groups, scope exceptions or runtime aliases are added.

## Source captures

[Capture records](source-captures.json) supply each page ID, actual revision,
timestamp, fixed `oldid` URL, history URL and original raw/API paths. The complete
author evidence root is
`C:/Users/inthe/IdeaProjects/lexique-batch-107/target/issue-107/`.
Its `sources/batch-current-fixed.api.json` is a first-party current API response;
the actual returned revisions are used, rather than a claimed cutoff revision.
The raw `.wiki` files preserve blank lines. Every selected line and IPA template
has a one-based line and column binding.

Seven paginated contributor responses cover all 47 articles and retain 3,305
page-associated contributor names. The final response has no continuation.
The current registry and linked history support attribution; they do not prove
the author of each selected sentence at its fixed revision.

[Source research](source-research.md) records constructions, original-author
boundaries and focused review questions. English and Chinese fields are new
Lexique explanatory translations of the selected French uses. Source records
retain attribution, CC BY-SA 4.0 obligations, modification notices and actual
bibliographic/import markers. Le Robert captures supply grammatical facts for
`e/NOM` and `censé`; their definition prose is not included in candidate fields.

## Author validation

The parent-approved resource boundary compares membership and frequencies with
the frozen [manifest](../../../resources/core-pilot-79/scope-manifest.csv), then
reconciles candidate fields, proposed review records, context, excerpts and
original revision bodies. It checks exact French/POS/spelling frames and the
separate `e/NOM` bridge and `téléphoner` IPA frame. It detects missing content,
substituted identity, mismatched source/span/text and premature admission claims.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_107.py -v
```

Focused RED and GREEN results are retained in
[red.log](../../../target/issue-107/red.log) and
[green.log](../../../target/issue-107/green.log). Windows x64, PowerShell 7 and
Python 3.14.10 through uv were used. Acquisition, contributor, grammar and
preparation logs remain in the same ignored evidence root. The reviewed diff,
tested commits, local-link results and final integration merge are recorded there
for the reviewer.

## Limitations

All 200 fields remain unresolved at the independent admission gate. Review must
assess the actual translations, modern selected use, applicable IPA, construction
fit and original-text rights. The `e` letter-to-noun mapping requires that review
despite its explicit corroboration. Bibliographic references to other dictionaries
do not establish that their wording was imported or licensed for this package.
No incompatible source or unsupported exact spelling was established by the
author; the selected-text decisions remain pending.

Conversion, final notices, package reproduction, native lookup/save/reopen,
learner acceptance and release evidence belong to integration and the separate
reviewer. Frontend, Rust and native checks were not run for this candidate handoff.
