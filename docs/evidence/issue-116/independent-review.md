# Issue #116 separate reviewer handoff

All **50 frozen groups / 200 authored fields** are prepared. No independent
reviewer or acceptance is recorded. The parent arranges a separate reviewer
for every row **1751–1800**. Author checks are not independent review.

| Input | Location |
| --- | --- |
| Issue / parent | [#116](https://github.com/zardonc/lexique/issues/116) / [#78](https://github.com/zardonc/lexique/issues/78) |
| Checkout / branch | `C:/Users/inthe/IdeaProjects/lexique-batch-116` / `codex/d2-content-116` |
| Initial diff base | `32b5d2b273afa0ae6964e05e56a60ce59f0bb15e` |
| Changes | Committed package under `docs/evidence/issue-116/` and `scripts/test_core_batch_116.py` |
| Full execution evidence | `C:/Users/inthe/IdeaProjects/lexique-batch-116/target/issue-116/` |
| Final head / integration / tested platform | [final-verification.json](../../../target/issue-116/final-verification.json) |
| Retained full / test-only patches | [authored.patch](../../../target/issue-116/authored.patch) / [test.patch](../../../target/issue-116/test.patch) |

The full patches use the fixed initial base, restricted to owned paths; the
evidence record includes the exact commands. Imported sibling commits are
outside this review. Full source and contributor responses stay in the owning
checkout and are not available in a detached tracked-only checkout.

## Review inputs

Compare the actual text in [groups.json](groups.json) and
[field-review.json](field-review.json) with the exact selected lines and support
in [source-excerpts.json](source-excerpts.json). All four fields must describe
the same contemporary POS/use. Inspect the grammar/flexion interpretation and
restrictions for each row, with particular attention to `tien`, `sauvage/NOM`,
`franc`’s two POS rows, `oeuf`, `espagnol/NOM`, `disputer`, `miss` and
`commun/NOM`.

Use [source-context.json](source-context.json) for full contributor attribution,
French/POS frames, exact import/reference context and construction evidence.
[rights.json](rights.json) records every field’s revision/span, contribution
and original-work layers, modification and selected boundary. Confirm those
actual boundaries and obligations independently; a page-wide licence or import
marker alone does not resolve a selected sentence’s origin.

All spelling/component relationships are source bindings with
`alias_admission=not_admitted`. Frozen spellings, subclasses, frequencies and
the 4,087 denominator remain unchanged. No owner exception is proposed.

Record the actual reviewer, field/use/spelling/rights decisions and any
corrections or unresolved rows. Independent acceptance is still pending for all
200 fields. Canonical admission, conversion/notices, native lookup/save/reopen,
learner acceptance and release checks remain parent-owned separate gates.
