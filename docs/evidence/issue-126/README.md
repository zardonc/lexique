# D2 issue #126 author candidate package

[Ticket #126](https://github.com/zardonc/lexique/issues/126) / [parent spec #78](https://github.com/zardonc/lexique/issues/78).

50 exact frozen groups (2251–2300), 50 selected uses and 200 authored fields. No reused exact group exists in the admitted core-119-v1 baseline; no alias, specialist exception or annotation is proposed. The fixed denominator remains 4,087 groups. All 200 independent semantic and rights decisions are pending; newly admitted fields: zero.

[groups.json](groups.json) and [review-candidates.json](review-candidates.json) contain the four fields, exact POS/use, exclusions, recorded IPA and field provenance. [source-research.md](source-research.md) lists every selected use. [source-excerpts.json](source-excerpts.json), [sources.json](sources.json) and [source-context.json](source-context.json) retain fixed spans, French/POS frames, complete contributor attribution and import/reference boundaries. [rights.json](rights.json), [selected-rights.md](selected-rights.md), [NOTICES.md](NOTICES.md) and [COPYING](COPYING) provide selected rights proposals and candidate obligations. [field-review.json](field-review.json) is an author-prepared review input, not independent acceptance. [admission.json](admission.json) and [reuse.json](reuse.json) separate coverage from admission.

There are 49 fixed French articles, six fully paginated contributor-registry responses and 7,902 public revisions across complete fixed-revision histories. Exact source bytes are recaptured by revision ID, and all owned evidence files are replayed from retained source/selection inputs. Source markers identify imported or referenced original layers; selected-sentence origin is not inferred solely from a bibliography.

## Verification and retained evidence

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

Raw API responses, fixed UTF-8 Wiki source, complete contributor and revision pagination, legal authorities, selection and preparation/replay scripts remain in `C:/Users/inthe/IdeaProjects/lexique-batch-126/target/issue-126/` for parent archival. [source-captures.json](source-captures.json) names the exact retained inputs. Reproduction uses `uv run --python 3.14.10 python -B -X utf8 target/issue-126/reproduce_package.py`; the focused check uses `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_126.py`.

## Limitations

Independent field/construction/rights review, canonical conversion and combined notices, native lookup/save/reopen and learner/release acceptance belong to the parent and are not run or asserted here. This is a complete authored candidate package, not admitted runtime content.
