# Issue #89: sourced content draft

The handoff covers all **50 exact frozen groups in rows 401–450**. It adds
**48 selected-use candidates / 192 proposed fields** and reuses
**`fort/ADJ` and `fort/ADV` / eight previously accepted fields** byte-identically.
Every headword and original POS subclass is preserved. The draft adds no
independently accepted content: all new field and usage reviews are pending.

| Ticket result | Count |
| --- | ---: |
| Exact frozen identities represented | 50 |
| New candidate senses / fields | 48 / 192 |
| Unchanged reused groups / fields | 2 / 8 |
| Newly independently accepted fields | 0 |
| New fields pending quality / rights audit | 192 / 192 |
| Fixed Wiktionnaire captures / new context records | 41 / 41 |
| Fresh first-party API captures / byte-identical research copies | 13 / 28 |
| Source pages declaring DAF8 imports | 20 |
| Unique selected definition, IPA and contributor-support locations | 99 |
| Concrete modern-everyday eligibility gaps | 1 |
| Frozen global denominator | 4,087 |

**Row 449 `point/ADV` remains unresolved.** Wiktionnaire revision 39772006,
line 810 describes its negative use as literary or regional. The evidence
does not establish a specific modern everyday regional use. Its four proposed
texts preserve that restriction; no scope exception is assumed. The other
47 new senses are contemporary-use candidates awaiting independent acceptance.

[review-candidates.json](review-candidates.json) supplies exact proposed text,
sense IDs, source versions/lines, use restrictions and corroboration for the
fresh reviewer. [source-research.md](source-research.md) records source and
construction decisions. [source-captures.json](source-captures.json) locates
every fixed API/raw capture and copied-source origin.
[quality-review.md](quality-review.md) records the pending independent audit.

The existing monolithic resource files contain only new ticket records and
the permitted local checksum refresh. Prior content, source contexts, reviews,
spelling relations, global counters and content version are preserved. The
permitted one-line `ROWS` change includes this ticket’s added rows without
duplicating 422/423. Coordinator integration owns its combined row set, counters,
version, final reviews and resource/native validation. Learner data and runtime
are unchanged.

## Checks

Owning worktree: `C:\Users\inthe\IdeaProjects\lexique-issue-89`.
Evidence root: `target/issue-89/`. Python commands use uv Python **3.14.10**.
The source base is `42c720556fd1195cca39ba704761877d1c0df48a`.

| Command | Actual result / retained log |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_89.py -v` | Passed: three checks covering exact group membership, refusal to prepare pending content, preservation of all previous records/metadata and byte-identical reused `fort` records; `focused-tests.log` |
| `uv run --python 3.14.10 python -B -X utf8 target/issue-89/validate_sources.py` | Passed: 41 fixed API bindings, 28 byte-identical copies, 192 exact field spans and 99 selected locations; `source-validation.log`, `source-validation.json` |
| `uv run --python 3.14.10 python -B -X utf8 scripts/core_pilot.py prepare target/issue-89/prepared` | Failed as expected at the independent source-review gate: `Unreviewed field source or incompatible licence`; `conversion.log`. No admitted resource or native content result was produced. |
| Focused test development | Membership check failed for the 48 missing rows before drafting and passed after drafting; the permitted row-set change then made the pending-review rejection check pass. `tdd-membership-red.log`, `tdd-membership-green.log`, `tdd-conversion-red.log` |
| `pnpm test:docs` | Passed: 181 documents / 1,440 local links; `docs-check.log` |
| `git diff --cached --check` | Passed; `diff-check.log` |

The source/structure checks do not approve translations or underlying import
rights. Independent semantic/rights review is pending, including the selected
boundaries on the twenty pages declaring DAF8 imports. Native lookup/save/reopen,
full conversion/coverage/size and package validation are pending centrally.
Frontend, Rust, browser, speech and release checks were not run for this
content-only pending draft. No live learner database was opened or migrated.

The coordinator allocates fresh reviewers after this handoff. Integration must
retain parallel ticket records, resolve all review/source gaps and publish an
accurate combined artifact before this ticket can be considered admitted.
