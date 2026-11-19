# Issue #105 candidate handoff

All **50 frozen groups, rows 1201–1250**, retain their exact headword, Lexique POS
subclass and both frequency strings. **46 selected senses / 184 fields** have
fixed-source candidates. Rows **1208, 1218, 1219 and 1220** retain **16 missing
candidate fields**. No field has independent acceptance or resource admission.
The global denominator remains **4,087 groups**.

[Candidate inventory](review-candidates.json) includes the four blockers;
[groups](groups.json) contains only the 46 proposed senses. The seven-file
handoff also contains [sources](sources.json), [field review](field-review.json),
[source context](source-context.json), [exact excerpts](source-excerpts.json),
[reuse](reuse.json) and [admission](admission.json). Their pending statuses are
author-stage records for a separate reviewer, not passed acceptance records.

## Source and review evidence

[Source research](source-research.md) records the selected meanings, source frames,
rights layers, construction limits and unresolved exact-POS/spelling evidence.
[Captures](source-captures.json) gives fixed revision IDs, timestamps, history
URLs and absolute API/raw paths for 43 articles. Eight paginated contributor
responses retain 3,229 article-associated contributor names. The registry is
current and broader than the frozen revisions; it does not prove sentence
authorship. The author and the future independent reviewer have separate roles,
as recorded in [the review handoff](independent-review.md).

The raw source root is
`C:/Users/inthe/IdeaProjects/lexique-batch-105/target/issue-105/sources/`.
The ignored `target/issue-105/prepare_candidates.py` reproduces the JSON handoff
from `selection.py` and the fixed captures. It checks complete API/wikitext
agreement, French sections, source POS frames, IPA and exact scope identities.

## Verification

The approved resource-boundary seam is exercised with:

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_105.py
```

RED failed on the absent candidate handoff; GREEN passed one focused check.
Logs are retained at `target/issue-105/red.log` and `target/issue-105/green.log`.
Candidate generation passed at `target/issue-105/prepare.log`.
The final documentation/link result is `target/issue-105/docs-check.log`;
the whitespace diff result is `target/issue-105/diff-check.log`.

## Limitations

Row 1208, `quiconque/PRO:int`, lacks supported modern interrogative use in the
fixed sources. Bare `qu` at rows 1218–1220 lacks its exact conjunction/pronoun
and spelling evidence. These remain ordinary incomplete groups; no new
annotation exception, specialist sense or query alias is admitted.

Every draft field and use still requires independent quality and selected-text
rights review, including original imported wording. Combined notices and actual
redistribution obligations remain an integration admission gate. The author
did not change the aggregate resource, converter, common documents or learning
data. Conversion, packaging and real native lookup/save/reopen checks belong to
integration and were not run here. Frontend, Rust, whole-machine offline and
other-platform checks were not run for this content-only handoff.
