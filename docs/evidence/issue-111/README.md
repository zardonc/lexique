# Issue #111 independently reviewed content

**50 exact frozen groups / 200 FR/IPA/EN/ZH fields accepted unchanged;
zero unresolved groups or fields.** Rows 1501–1550 retain their headwords,
POS subclasses, frequencies and order against the 4,087-group manifest.
Independent selected-content and rights acceptance is complete. Canonical
conversion, actual combined notices, packaging and native admission are
parent-owned and pending.

The [ticket](https://github.com/zardonc/lexique/issues/111) and
[parent](https://github.com/zardonc/lexique/issues/78) define the scope.
[Independent review](independent-review.md) records semantic, construction,
spelling and verification decisions. [Selected-rights review](selected-rights-review.md)
records the actual original-work, contributor and adaptation boundaries.

| File | Content |
| --- | --- |
| [groups.json](groups.json) | 50 accepted exact uses and their four fields |
| [review-candidates.json](review-candidates.json) | Unmodified original author groups and pending statuses |
| [field-review.json](field-review.json) | 200 accepted/proposed texts, individual reasons, 50 use decisions and 50 binding-only spelling decisions |
| [sources.json](sources.json) | 43 fixed revisions, attribution, contributors and selected-span decisions |
| [source-excerpts.json](source-excerpts.json) | Exact primary field spans checked against the fixed raw captures |
| [source-context.json](source-context.json) | Grammar/construction support, original layers and independent capture comparisons |
| [source-captures.json](source-captures.json) | Owning raw/API paths, contributor pagination and independent comparison record |
| [rights.json](rights.json) | 200 separate selected-rights decisions and redistribution obligations |
| [NOTICES.md](NOTICES.md) / [COPYING](COPYING) | Attribution/modifications and full CC BY-SA 4.0 legal text |
| [source-research.md](source-research.md) | Author selection and preparation record |
| [admission.json](admission.json) / [coverage.json](coverage.json) | Review acceptance separated from pending canonical admission |
| [measurements.json](measurements.json) | Source/field sizes; parent runtime measurements pending |
| [reuse.json](reuse.json) | Empty reuse; no canonical content rewritten |

All spelling relationships are source bindings with `alias_admission=not_admitted`.
No runtime alias, owner annotation or specialist exception is added.
The unchanged canonical baseline has 1,436 groups / 5,744 fields; this review
does not add actual assembled coverage. The denominator remains 4,087 groups /
16,348 fields, and the parent recomputes combined coverage on admission.

## Verification

Windows x64, PowerShell 7, uv-pinned Python 3.14.10. Reviewed base:
`b3560857e61286a61cbf88a1aabf2072fe8ae11e`; author:
`9aca23191000a915a757ed9999646d9b24c55099`.
The [focused test](../../../scripts/test_core_batch_111.py) passes all four
tests, including the existing validator on an isolated resource seam and
rejection of altered text, source, use, rights and aliases.

Full reviewer logs and bounded five-group projections are retained under
`C:/Users/inthe/IdeaProjects/lexique-review-111/target/issue-111-review/`.
Raw sources remain under the supplied author evidence root; the
[review document](independent-review.md) records exact paths and commands.
