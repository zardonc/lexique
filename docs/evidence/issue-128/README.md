# D2 #128 author package

This package proposes **50 exact groups, 50 selected senses and 200 fields** for
frozen rows **2351–2400** of [ticket #128](https://github.com/zardonc/lexique/issues/128),
under the captured [D2 spec](../../../target/issue-128/spec.md).
Every sense has a French definition, isolated IPA, English meaning and Chinese
meaning for the same selected POS/use. All **47 French articles** have fixed
revisions and numbered spans. Complete contributor pagination is retained in
**six responses**, covering **2,749 named contributor memberships**, plus the
returned anonymous counts and full linked histories.

[Groups](groups.json) and [review candidates](review-candidates.json) contain the
four fields. [Field review](field-review.json) has **200 pending field records**,
50 usage records and 50 spelling relations, with precise source grammar,
construction and selected-use interpretation. [Admission](admission.json)
preserves the **4,087-group** denominator. [Reuse](reuse.json) records **zero
reused groups and fields** against the admitted `core-119-v1` inputs. There is
no annotation exception, specialist exception or proposed query alias.

## Evidence

- [Sources](sources.json): article identity, revision, timestamp, fixed URL,
  full linked history, attribution, contribution licence and original layers.
- [Captures](source-captures.json): first-party API/raw paths, request files,
  complete contributor pagination and fixed-revision recapture entry.
- [Selected spans](source-excerpts.json): all 200 field bindings, IPA variant
  columns and selected language/POS/spelling/construction support.
- [Context](source-context.json): named contributors, anonymous counts,
  original-work/import/reference boundaries and the 50 spelling relations.
- [Rights](rights.json), [selected rights](selected-rights.md), [notices](NOTICES.md)
  and [CC BY-SA 4.0 legal text](COPYING): field-specific attribution,
  selected boundaries, modification records and actual authority captures.
- [Selected-use notes](source-research.md): register, constructions and
  homograph distinctions for separate review.

The records are author proposals. **Independent acceptance is zero**; all
200 fields await separate semantic, selected-span rights and construction/
spelling review. The contributor registry is a complete current attribution
registry, not proof of selected-sentence authorship. Source dictionaries are
not admitted as a whole.

## Reproduction and checks

The parent-approved seam is the resource/admission record. The focused test
compares membership with the self-contained ticket and frozen manifest,
verifies four-field bindings, source rights, selected restrictions and honest
pending review state. With the retained author archive present, it also checks
raw/API byte equality and every contributor continuation. Tracked records
remain checkable in a checkout without ignored captures; live/raw reproduction
requires the retained archive.

The TDD evidence is [membership RED](../../../target/issue-128/membership-red.log),
[membership GREEN](../../../target/issue-128/membership-green.log),
[source/rights RED](../../../target/issue-128/source-rights-red.log) and
[GREEN](../../../target/issue-128/green.log).

Live fixed-revision recapture matches all **47 complete articles byte for byte**.
Offline preparation reproduces the **ten JSON evidence files** and article
notices byte for byte from retained sources and the explicit author selections.
The five established rights authority records are reused unchanged, retaining
original capture dates. The complete canonical legal text is retained with
terminal blank lines normalized.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128/write_notices.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_128.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-128/recapture_fixed.py
pnpm test:docs
git diff --cached --check
```

See [recapture](../../../target/issue-128/fixed-recapture.log),
[reproduction](../../../target/issue-128/reproduction.log),
[documentation](../../../target/issue-128/docs-check.log) and
[final verification](../../../target/issue-128/final-verification.json).
The [selection](../../../target/issue-128/selection.py),
[preparation](../../../target/issue-128/prepare_candidates.py),
[rights preparation](../../../target/issue-128/finish_author_package.py),
[notice preparation](../../../target/issue-128/write_notices.py),
[reproduction script](../../../target/issue-128/reproduce.py),
[recapture script](../../../target/issue-128/recapture_fixed.py), full API
responses/raw articles and request files remain under ignored `target/issue-128/`
for parent archival before cleanup. No dependency or shared converter change
is required by this author package.

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-128`, branch
`codex/d2-content-128`. Initial clean HEAD was
`b91c4729a0a1c93baa58f86ed666b173fb2fc253`; integration `a27f410` was merged
before authoring. The latest `codex/d2-content-120-129` is merged before final
handoff. The final verification record identifies the commits and log root.

## Remaining gates

Separate independent review, canonical assembly, combined resource/notices,
native lookup/save/reopen, learner acceptance and remaining platform/offline/
release checks are parent-owned. No source/content blocker is recorded by the
author; this does not resolve those acceptance gates. Application, canonical
conversion and native checks were not run. Shared resources, other author
work and learner data are unchanged.
