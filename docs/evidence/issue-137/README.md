Current result: [final admission and native verification](README-admission.md). The preparation record below retains its original scope.

# Issue #137 reviewed content

**50 exact frozen groups / 50 modern everyday selected uses / 200 fields accepted
by independent quality and selected-rights review.** There are **198 unchanged
fields, two Chinese replacements and zero unresolved fields**. Rows **2801–2850**
retain exact headwords, POS subclasses, frequencies and order from the
[scope manifest](../../../resources/core-pilot-79/scope-manifest.csv) and
[ticket #137](https://github.com/zardonc/lexique/issues/137). The denominator remains
**4,087 groups / 3,773 headwords**. Reuse, annotations, specialist exceptions and
new runtime aliases: **0**.

[Groups](groups.json) hold accepted content. [Field review](field-review.json)
records exact proposals, accepted text, fixed source spans, individual field/use
reasons and separate quality/rights decisions. Chinese explanations retain the
general land-area sense of `territoire` and both sides of the family for `mamie`.
[Replacement records](accepted-replacements.json) and
[authored snapshots](authored/snapshot.json) preserve the exact proposals.

[Sources](sources.json), [context](source-context.json), [excerpts](source-excerpts.json),
[captures](source-captures.json), [rights](rights.json), [selected rights](selected-rights.md),
[notices](NOTICES.md) and [COPYING](COPYING) retain the selected boundaries and
obligations. The [independent review](README-independent-review.md) identifies
the reviewer, bases, source audit and execution archive.

## Verification

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_137.py --review-only
uv run --python 3.14.10 python -B -X utf8 target/issue-137-review/audit.py
pnpm test:docs
git diff --cached --check
```

Six focused tests check exact membership, immutable proposals, accepted field/use
alignment, rights, fixed source frames, IPA substrings and complete contributor
pagination. The audit compares all **53 raw article bodies** to retained API
revision bytes and checks **200 field spans / 194 distinct supporting spans**.
Its ledger does not supply linguistic verdicts. Full outcomes are in
[verification](verification.json).

Omitting `--review-only` also checks coordinator canonical admission and exact
group equality; that gate remains pending while the 50 rows are absent.

## Limitations

All **200 independently accepted fields** await coordinator canonical assembly,
combined conversion/notices and native lookup/save/reopen/preservation. This shard
does not change the canonical resource or learner data. Learner acceptance,
other-platform execution, whole-machine offline checks and formal release remain
separate gates. Live primary statute retrieval returned HTTP 403; the original
French collective-edition assessment uses the recorded L123-3 basis.
