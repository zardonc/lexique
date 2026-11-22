# #146 author package

All 50 frozen groups in rows **3251–3300** have one scoped selected-sense
proposal and **200 FR/IPA/EN/ZH fields**. `divers/ADJ` and `divers/ADJ:ind`
remain separate groups. [Readable proposals](review-candidates.md),
[fixed source excerpts](source-excerpts.json), [context](source-context.json),
[rights](selected-rights.md) and [notices](NOTICES.md) retain the exact content
and source boundaries.

The author branch is `codex/d2-content-146`, based on
`9fa4f264509c953dba801d18ea6eaab84b2aafb2`, in
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/author-146`.
Its only owned tracked changes are `docs/evidence/issue-146/**` and
`scripts/test_core_batch_146.py`. [verification.json](verification.json)
records commands/results and the integration merge. The fixed denominator
remains 4,087; canonical baseline is `core-139-v1`, 2,932 groups/11,728 fields.
No admitted group is reused or changed by these proposals.

## Sources and original proposals

The coordinator’s 49 unique-title captures, source index, API bytes and complete
contributor/request chain are reused verbatim from
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/source-preparation/issue-146`.
The original index is retained as `source-index.original.json`; relocated records
preserve their original paths and fixed URLs. Five supporting titles complete
meaning, inflection or pronunciation evidence: `plupart`, `celui-là`, `même`,
`le même` and `la`. [source-captures.json](source-captures.json) records the
actual copy paths and archive fallback. Académie ninth-edition HTML supplies
read-only grammatical corroboration of `la plupart`; its prose stays outside
the content package.

Raw captures, request URLs, original index, rights authorities, helpers,
execution logs, original proposals and patches are preserved outside the repo at:

`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-146`.

The tracked [authored snapshot](authored/snapshot.json) and external
`original-proposals/` preserve the original text and pending statuses for
separate review. The initial rights proposals remain unchanged in that snapshot;
current root rights/context add source-specific original-text and territorial
assessments without changing any of the 200 learner fields. Original proposals
are not replaced when reviewers accept different wording. Current root records
remain author proposals.

## Resource checks

The pre-approved resource/admission seam first failed because #146 had no
admitted groups or author package. The subsequent author checks verify all
50 literal memberships/frequencies, 200 fixed-source fields, French/POS frames,
IPA components, contributor pagination, source exclusions and pending rights.
No mock service or learner profile is involved.

Run the author check from this worktree:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_146.py
uv run --python 3.14.10 python -B -X utf8 target/issue-146/reproduce.py
pnpm test:docs
git diff --check
```

The canonical gate uses the same test with `--admitted`; it remains failing
until independently accepted records enter the canonical resource. It is skipped
explicitly in author mode. RED/GREEN, reproduction, docs and diff logs are in the
external archive. Reproduction uses retained bytes only and compares all nine
author JSON outputs. Restore helpers to this worktree’s `target/issue-146/`
before executing archived helpers; their repository root is relative to that
location. Verification also exercises the test against the external cache.

## Remaining gates

Zero fields are independently accepted or newly admitted. There are no missing
authored fields or author-unresolved content rows; **200 fields remain missing
from admission**, pending independent quality and selected-span rights review,
including original-expression originality and territorial scope. The 32
DAF8-marked sources each have a selected-boundary author assessment; the
historical French collective-work expiry supplies no worldwide permission.
No annotation or specialist exception is proposed. Modern-use, pronominal
source-label interpretation and composed IPA remain explicit reviewer decisions.

Combined conversion/packaging/notices, actual native lookup/save/reopen,
learner acceptance and release/platform gates are integration-owned and unrun
by this author. The wider #78 core remains incomplete. Frontend/Rust checks
are unrun because this author package changes no application or Rust source.
