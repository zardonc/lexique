# Issue #125 independently reviewed content

**50 selected uses / 200 fields accepted: 196 unchanged, four replaced, zero
pending or missing.** All frozen rows **2201–2250**, exact headwords/POS
subclasses, frequencies and ordering remain. No annotation exception, reused
group or runtime alias is admitted. The denominator remains **4,087 groups**.
Canonical assembly and application acceptance remain parent-owned.

Ticket [#125](https://github.com/zardonc/lexique/issues/125),
[PR #193](https://github.com/zardonc/lexique/pull/193),
parent [#78](https://github.com/zardonc/lexique/issues/78).
Reviewed author head: `9cb84bab4e7f41f15195c6fd359037b91c320580`.
Reviewer: **Independent semantic and selected-rights reviewer #125**, separate
from the author and parent integration owner. This is an agent review;
learner acceptance remains a separate gate. The supplied spec's Testing
Decisions approve the resource/admission seam.

| File | Result |
| --- | --- |
| [groups.json](groups.json) | Exact frozen groups and accepted fields/use |
| [field-review.json](field-review.json) | `separate_independent_field_review`; 200 specific field/rights reasons, 50 uses and spelling decisions |
| [review-candidates.json](review-candidates.json) | Complete unchanged original author group objects |
| [admission-author-proposal.json](admission-author-proposal.json) | Original author admission/count ledger, byte for byte |
| [admission.json](admission.json) | Actual 50/200 acceptance, zero pending/missing and separate parent gates |
| [sources.json](sources.json), [source-excerpts.json](source-excerpts.json), [source-context.json](source-context.json) | 51 fixed revisions, selected fields and reviewed French language/POS/form/construction frames, complete contributors |
| [source-captures.json](source-captures.json) | Original API/raw capture and pagination pointers |
| [rights.json](rights.json), [selected-rights.md](selected-rights.md) | Accepted selected-span and original-work rights, original proposals retained |
| [NOTICES.md](NOTICES.md), [COPYING](COPYING) | Attribution, changes and full CC BY-SA 4.0 text |
| [independent-review.md](independent-review.md) | Independent result, exact replacements and verification boundaries |
| [source-research.md](source-research.md), [selection.py](selection.py) | Retained authored selection/research, not the live acceptance ledger |

`gay/ADJ` retains the selected source's **exclusive** same-sex or same-gender
sexual/romantic attraction in FR/EN/ZH. `noyer/VER` Chinese retains asphyxiation
by immersion in **liquid**. `vous-même` remains polite singular; `grand-chose`
remains an invariable negative-construction pronoun. Exact plural cardinal
`cents` has its own adjective IPA **/sɑ̃/** and reviewed multiplied-hundreds,
completing-numeral and ordinal notes. No source spelling becomes a runtime alias.

## Verification and retained evidence

Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-125`, branch
`codex/d2-review-125`. Author raw evidence:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-125/`;
review copy and complete logs:
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/`.

The [focused check](../../../scripts/test_core_batch_125.py) passes **five tests**.
It verifies exact accepted fields against the retained proposals and four
literal replacements, fixed sources, independent selected rights and source
frames, live counters and original snapshots. Identical subsequent canonical
overlap is supported; overlapping content must match the reviewed record.

The independent [byte/source audit](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/source-audit.json)
passes **51 fixed API/raw files**, **200 field spans**, **812 field-support
spans** and all frozen identities. Seven main contributor response parts and
one `cent` part terminate without continuation; **3,004 + 121 named article
credits** and **460 + 25 anonymous contributor counts** match the retained
registries. Counts are per article, not unique people. Registries may include
later contributors and do not identify individual sentence authors.

Retained logs: [acceptance RED](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/acceptance-red.log),
[acceptance GREEN](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/acceptance-green.log),
[gay RED](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/gay-red.log),
[gay GREEN](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/gay-green.log),
[noyer RED](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/noyer-red.log),
[noyer GREEN](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/noyer-green.log),
[full focused GREEN](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/green.log),
[source bytes](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/source-byte-audit-reviewed.log),
[documentation](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/docs-check.log),
[owned links](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/owned-links.json) and
[final verification](../../../target/implement-120-129/reviewer-evidence/issue-125/issue-125-review/final-verification.json).
PowerShell 7 and uv Python **3.14.10** are used. All complete author logs/raw
inputs, acquisition scripts and original tracked snapshots are retained in
`author-evidence/` and `authored-snapshots/` under the review evidence root.
No upstream requests or hot retries were made.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_125.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-125-review/source_audit.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

The retained author preparation/reproduction scripts regenerate **author
proposals** and must be run against the author baseline with its original
`target/issue-125/` inputs. The reviewed content has separate accepted records,
snapshots and independent audit; re-running author generation in this review
checkout would overwrite those decisions.

Retained patch command:
`git diff 9cb84bab4e7f41f15195c6fd359037b91c320580 HEAD -- docs/evidence/issue-125 scripts/test_core_batch_125.py`.
The owned patch and separate test patch are `reviewer.patch` and
`reviewer-test.patch` in the review evidence root. The latest committed
integration tip is merged before handoff; exact SHAs and results are in
`final-verification.json`. Only the evidence directory and focused test are
committed by this reviewer.

## Remaining gates

Zero unresolved field/use/spelling/source-frame/selected-rights decisions
remain. Rights authorities and original-work bases use retained evidence;
no live statute refresh is claimed. Canonical conversion, actual combined
notices, global coverage/size, native lookup/save/reopen, learner acceptance
and offline/platform/release checks remain parent-owned and unrun here.
