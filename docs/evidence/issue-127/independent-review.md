# Issue #127 independent semantic and selected-rights review

**50 groups / 200 fields accepted: 199 unchanged and one exact replacement.
Zero unresolved or pending independent fields.** All frozen rows 2301–2350,
headwords, POS subclasses, frequency strings and order remain. Reuse is zero
at this review boundary; the global denominator remains 4,087 groups. There
are zero new annotations, specialist exceptions or runtime aliases.

Reviewer: Independent semantic and selected-rights reviewer #127, separate
from author and parent integration owner. This is an agent review; human
learner acceptance is a separate gate. Author commit:
`1a2825fbe070384d45e39b91f8675c8a112c9053`, [PR #193](https://github.com/zardonc/lexique/pull/193).
Review worktree: `C:/Users/inthe/IdeaProjects/lexique-review-127`,
branch `codex/d2-review-127`. Review diff base:
`55971e6301c930cb381a6ef67349eece0de35269` (the full initial integration SHA is retained in reviewer
verification metadata). Changes cover committed author content and this
reviewer’s owned evidence/test edits.

The full integration spec and issue capture, project instructions, admission
and domain guidance were read. The existing spec authorizes the resource/
admission seam. The separate [#119 review](../issue-119/independent-review.md)
supplies the accepted review schema; accepted
[#123 selected rights](../issue-123/selected-rights.md) and retained primary
legal responses were read for the actual selected boundaries.

## Exact decisions

Every proposed FR/IPA/EN/ZH text was independently read against its frozen
definition, neighbouring sense boundaries, French language/POS/headword frame
and applicable pronunciation/construction. [field-review.json](field-review.json)
is `separate_independent_field_review`, with specific field/use reasons,
accepted texts, original proposals and 50 passed spelling bindings.

| Field | Accepted text | Fixed basis |
| --- | --- | --- |
| Row 2322 monseigneur/NOM, English | An honorific used when addressing certain people of high birth or dignity, especially bishops. | Revision 39641302, definition line 10, noun/headword/plural frames lines 7–9 |

The removed “Monsignor” equivalent could imply a distinct specific church title.
The selected French source supports the complete address-form explanation.
Other fields for this use remain unchanged. All proposals and the original
pending admission are retained in [review-candidates.json](review-candidates.json),
[admission-author-proposal.json](admission-author-proposal.json) and
[authored-review-proposals.json](authored-review-proposals.json).

`copie`’s structured import context now records both actual DAF8 line 203 and
Littré line 204. Selected source-frame statuses pass at their lexical/grammar
boundaries, without attributing a sentence to an original dictionary.
[selected-rights.md](selected-rights.md) records the separate historical,
wiki-contribution and Lexique-adaptation layers.

`pub`’s feminine advertisement /pyb/, both `technique` sections,
vulgar `bite`, honorific `monseigneur`, spatial `au-delà`,
masculine business-partner `associé`, and `jeudi`’s ISO-excluding substring
retain their exact selected restrictions. `hop` has aspirated h with the
recorded /ɔp/ variant; `débarquer` is intransitive, `embarquer` is transitive.
No field or difficult identity is dropped.

## Verification and reproduction

Retained reviewer execution root:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-127/issue-127-review/`.
Platform: Windows x64, PowerShell 7, uv Python 3.14.10.

| Check | Result and full log |
| --- | --- |
| Exact honorific explanation, RED then GREEN | `red-translation.log`, `green-translation.log` |
| Independent acceptance boundary, RED then GREEN | `red-admission.log`, `green-admission.log`; four focused tests pass |
| Fixed-source byte/span/import audit, RED then GREEN | `red-source-audit.log`, `green-source-audit.log`, `source-audit.json` |
| Identical canonical overlap and changed-content rejection | `overlap-green.log`, `overlap-red.log` in isolated ignored fixtures |
| Documentation and owned local links | `docs.log`, `owned-links.json` |
| Diff and owned scope | `diff-check.log`, `owned-scope.json` |
| Final SHA, integration and result metadata | `final-verification.json` |
| Full reviewer and focused-test patches | `reviewer.patch`, `reviewer-test.patch` |

The independent source audit matches all **49 original wikitext files** to
original API UTF-8 bytes and integration copies, all **200 field and 200 rights
spans**, selected construction spans, all contributor registries/anonymous
counts and **nine responses / five terminal chains**. It verifies retained
legal-authority bytes and complete CC BY-SA 4.0 text. No network request was made.

Run from the review checkout:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_127.py -v
uv run --python 3.14.10 python -B -X utf8 target/issue-127-review/source_audit.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff 55971e6 HEAD --check
```

The focused checks compare exact accepted texts/rights to the independent
review, require passed live field/source frames, preserve proposals and check
identical canonical overlaps when present. They do not freeze the author’s
pending counts or assert that these reviewed identities remain absent from
future canonical inputs.

Raw author evidence:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-127/`.
Integration copies:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/issue-127/`.
The reviewer’s audit, inspection and decision scripts stay in the ignored
review evidence root. Author preparation scripts are retained at their original
root; they reproduce the author proposals, not the reviewed replacements.

## Remaining gates

Canonical assembly, actual combined package notices/source delivery,
native lookup/save/reopen, learner acceptance and offline/platform/release
checks remain parent-owned and unrun. The legal authorities were reused and
read at the selected boundaries; no live statute refresh is claimed.
