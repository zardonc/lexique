# D2 issue #126 independently reviewed content

[Ticket #126](https://github.com/zardonc/lexique/issues/126) / [parent spec #78](https://github.com/zardonc/lexique/issues/78).

50 exact frozen groups (2251–2300), 50 everyday selected uses and 200 independently accepted fields. No reused exact group exists in the admitted core-119-v1 baseline; no alias, specialist exception or annotation is admitted. The fixed denominator remains 4,087 groups. Zero content/rights fields remain missing or pending. [independent-review.md](independent-review.md) records the separate review, one Chinese wording replacement, corrected collective-work statute, source frames and RED/GREEN/audit logs. Canonical resource and native admission remain parent-owned.

[groups.json](groups.json) contains the accepted fields. [review-candidates.json](review-candidates.json) and the six `*-author-proposal.json` snapshots retain original author proposals byte-identical to commit `60805fc`, including [admission-author-proposal.json](admission-author-proposal.json). [source-research.md](source-research.md) lists every selected use. [source-excerpts.json](source-excerpts.json), [sources.json](sources.json) and [source-context.json](source-context.json) retain reviewed fixed spans, French/POS frames, complete contributor attribution and import/reference boundaries. [rights.json](rights.json), [selected-rights.md](selected-rights.md), [NOTICES.md](NOTICES.md) and [COPYING](COPYING) provide independently reviewed selected rights and batch obligations. [field-review.json](field-review.json) contains all 200 specific independent field decisions and 50 use/spelling decisions. [admission.json](admission.json) and [reuse.json](reuse.json) separate coverage from final resource admission.

There are 49 fixed French articles, six fully paginated contributor-registry responses and 7,902 public revisions across complete fixed-revision histories. Exact source bytes are recaptured by revision ID, and all owned evidence files are replayed from retained source/selection inputs. Source markers identify imported or referenced original layers; selected-sentence origin is not inferred solely from a bibliography.

## Author verification and retained evidence

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-126`, branch `codex/d2-content-126`; initial clean baseline `b91c4729a0a1c93baa58f86ed666b173fb2fc253`, integration merged before authoring. Scope: `docs/evidence/issue-126/**` and `scripts/test_core_batch_126.py`.

Approved seam: conversion output/admission record in spec Testing Decision 2. The focused resource/admission check verifies exact membership and frequencies, all four scoped fields, source-byte reproduction, contributor/history completion, precise rights records and pending independent decisions.

| Check | Result | Retained log |
| --- | --- | --- |
| Exact membership TDD red | Expected absent-package failure | `target/issue-126/tdd-red.log` |
| Membership tracer green | Passed | `target/issue-126/tdd-green-membership.log` |
| Provenance TDD red | Expected absent-rights failure | `target/issue-126/tdd-red-provenance.log` |
| Focused admission candidates | Passed | `target/issue-126/tdd-green.log` |
| Fixed source byte recapture | 49 articles reproduced | `target/issue-126/source-reproduction.log` |
| Complete candidate byte replay | Passed | `target/issue-126/package-reproduction.log` |
| Diff / local document links | Passed | `target/issue-126/diff-check.log`, `target/issue-126/link-check.log` |

Raw API responses, fixed UTF-8 Wiki source, complete contributor and revision pagination, legal authorities, selection and preparation/replay scripts remain in `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-126/` for parent archival. [source-captures.json](source-captures.json) names the exact retained inputs. Reproduction uses `uv run --python 3.14.10 python -B -X utf8 target/issue-126/reproduce_package.py`; the focused check uses `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_126.py`.

## Independent verification and limitations

The separate reviewer passes 50 groups, 200 fields and 186 selected source frames; all 49 fixed source bytes, contributor registries and 7,902 public revisions are audited. Five focused checks pass. Canonical overlap of 50 identical records passes; a changed adjective IPA is rejected. Full review logs and scripts remain in `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-126/issue-126-review/`. Canonical conversion and actual combined notices, native lookup/save/reopen and learner/release acceptance remain parent-owned and unrun here.
