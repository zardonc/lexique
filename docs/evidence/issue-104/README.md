# Issue #104 candidate handoff

Frozen rows **1151–1200** have **50 candidate groups / 200 candidate fields** across
46 source headwords. All FR/IPA/EN/ZH drafts have exact fixed-source bindings.
**Independent quality and rights review is pending; zero groups / zero fields
are admitted.** The permanent denominator remains **4,087 groups**.

The assigned checkout is `C:/Users/inthe/IdeaProjects/lexique-batch-104`, branch
`codex/d2-content-104`, based on `aae5273af846e57668a8d267fa6a6566c4fd6e91`.
Scope is [ticket #104](https://github.com/zardonc/lexique/issues/104) under
[spec #78](https://github.com/zardonc/lexique/issues/78). Only this evidence
directory and [the focused check](../../../scripts/test_core_batch_104.py)
are changed. Shared conversion, canonical resource and application files belong
to integration.

The seven-file handoff is [groups](groups.json), [sources](sources.json),
[field review](field-review.json), [source context](source-context.json),
[source excerpts](source-excerpts.json), [reuse](reuse.json) and
[admission](admission.json). `groups.json` and `sources.json` are arrays;
`source-context.json` uses `current_content`, and `source-excerpts.json` uses
`fields` plus separate supporting evidence. [Review candidates](review-candidates.json)
retain the same draft groups with their explicit pending status. Review records
contain proposed text and usage, pending quality/rights statuses and no reviewer
or claim of independence. All 200 fields remain unresolved at the admission gate.

## Source evidence

[Capture records](source-captures.json) identify the fixed revision, page ID,
timestamp, raw/API paths and article-history URL for every source. Original
captures and preparation inputs are retained at
`C:/Users/inthe/IdeaProjects/lexique-batch-104/target/issue-104/`.
The current first-party API response is `sources/batch-current-fixed.api.json`;
its actual returned revisions are used throughout. It is not a cutoff snapshot.
Each numbered span preserves the raw source's blank lines. IPA bindings include
the exact template substring and column; lexical bindings include the selected
definition and separately recorded French/POS/headword frames.

English and Chinese meanings are new explanatory translations of the selected
French use, credited to Lexique contributors and marked as adaptations.
They are not quotations from an upstream bilingual entry. [Source research](source-research.md)
records spelling, constructions and the original-author/licensing boundaries.

## Author checks

The approved resource seam checks exact membership against the frozen manifest,
four-field completeness and traceability across candidate groups, canonical
draft groups, proposed review text, source context, excerpts and original API
content. It detects missing groups/fields, substituted spelling/POS, mismatched
revision/span/text and premature independent-review or admission claims.

```powershell
uv run --python 3.14.10 python -X utf8 scripts/test_core_batch_104.py -v
```

The focused check recorded RED before the candidate package existed and GREEN
with the completed package. Full logs are [RED](../../../target/issue-104/red.log)
and [GREEN](../../../target/issue-104/green.log). Source acquisition and preparation
logs are retained beside them. Platform: Windows x64 / PowerShell 7 / Python
3.14.10 through uv. Documentation links and whitespace are checked before commit;
the final tested commit and retained patch are recorded in the same ignored
evidence root.

## Limitations

No unsupported frozen spelling or POS group was identified by the author.
There are no new scope exceptions or runtime aliases. Each spelling binding is
exact and is recorded as source evidence only. All 50 selected everyday-use
decisions, translations, IPA and original-text rights boundaries await a separate
reviewer. Current contributor names and author checks are not independent review.

Conversion, packaged notices, native lookup/save/reopen, learner acceptance,
offline execution and release checks are pending at integration's boundaries.
Application/frontend/Rust checks were not run for this source-only handoff.
