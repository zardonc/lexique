# Issue #141 content-author package

[Ticket #141](https://github.com/zardonc/lexique/issues/141), under
[spec #78](https://github.com/zardonc/lexique/issues/78), fixes rows **3001–3050**.
This package supplies **50 exact groups, 50 selected everyday senses and 200 complete
FR/IPA/EN/ZH proposals**. No group is dropped, substituted, merged or exempted.
All **200 fields remain unadmitted**, pending separate quality and selected-span rights
review. Author checks do not establish independent acceptance.

[Readable proposals](review-candidates.md) retain every field and selected/excluded use.
[groups.json](groups.json), [field-review.json](field-review.json),
[source-excerpts.json](source-excerpts.json), [source-context.json](source-context.json)
and [rights.json](rights.json) bind exact text to its fixed source/POS/use.
[authored/](authored/snapshot.json) preserves original proposals for separate review.
[admission.json](admission.json) separates proposed, missing-admitted and pending counts.

## Scope and source evidence

The initial baseline is `ff8aae2bdaf36096c6600d5d29342745b97c0eb7`,
`core-139-v1`: **2,932 admitted groups / 11,728 fields**, with 18 zero-field annotations.
All #141 exact pairs are absent from that baseline; reused groups/fields are **0/0**.
The denominator remains **4,087 groups**. Shared resources and learner data are unchanged.
The 200 proposed fields contain 6,783 characters; the longest field is 138 characters.
These are author-package measurements, not converted resource or release size claims.

50 real first-party API captures retain fixed article revisions and literal raw text.
Seven completed contributor responses retain **3,177 named article credits** and returned
anonymous counts totaling **711**. Those totals are per-article credits, not distinct people
or proof of sentence authorship. Complete histories preserve anonymous/edit attribution.
The author audit verifies **50 byte-identical article/API revisions, 200 selected field
spans and 193 supporting-span occurrences**, including French language/POS frames.
[Capture metadata](source-captures.json) points to the retained request/response chains.

Raw captures, rights authorities, helpers, source frames, audit results and full command
logs are archived outside the repository at:
`C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-141/`.
Working logs also remain under this checkout’s ignored `target/issue-141/`.
Article requests were sequential; all succeeded. Existing fixed rights-authority captures
were reused with their original dates. The capture helper retains success files and HTTP
failures, honors Retry-After and applies exponential backoff; no HTTP 429 occurred here.
Reproduction uses only the fixed files and makes zero network requests.

## Rights and source restrictions

The fixed Wiktionnaire contribution layer and new Lexique explanations/adaptations retain
**CC BY-SA 4.0**. [NOTICES.md](NOTICES.md), [COPYING](COPYING) and
[selected-rights.md](selected-rights.md) retain source/history credits, actual modifications
and redistribution obligations. 29 articles declare DAF8 imports. The established original
collective-work basis applies only to Académie française’s eighth edition, 1932–1935,
conservatively expired **2006-01-01**; modern editorial/transcription work has separate rights.
That historical decision does not review any new field. Referenced publishers’ prose,
literary/news examples, etymologies and media are outside the proposed fields.

Exact nominal/adjectival/verb frames remain distinct. `came` retains its slang drug sense;
`bien-aimé` is a masculine noun; `basket` is the masculine sport; `accoucher` is intransitive.
`do` uses fixed-do C/ut correspondence at actual supporting lines 22 and 39.
`couvent` and `dater` use the selected nominal/lexical pronunciation rather than other frames.
`las`, `pâle` and `pâte` select recorded variants; `villa` retains the recorded /vil.la/ variant.
No source binding creates a lookup alias. `adjudant`’s questionable source NATO code is
excluded from all fields; only the France-specific rank relationship is proposed.

## Actual author checks

Commands run from `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/author-141`:

| Check | Command | Result and boundary |
| --- | --- | --- |
| Initial RED | `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_141.py` | Failed on the absent groups package, after verifying all 50 ticket memberships against the frozen manifest. |
| Rights RED | Same script with `Batch141Tests.test_author_fields_have_rights_and_remain_pending_separate_review` | Failed on missing rights.json. |
| GREEN | `uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_141.py` | 2 resource/admission-record checks passed; proposals remain pending. |
| Fixed-source audit | Same pinned Python invocation of external `audit.py` | Passed: raw/API identity, selected spans, French/POS frames, full contributor continuation and retained policy captures. |
| Offline reproduction | Same pinned Python invocation of external `reproduce.py` | 20 author files byte-identical; zero network requests. |
| Documentation links | `pnpm test:docs` | Passed, including notices and proposal links. Skipped external/capture links are outside this local-link check. |
| Existing core | `pnpm test:core`, with the uv-resolved Python directory first on process PATH | Passed on the unchanged canonical baseline; does not admit #141. Initial default-PATH attempt could not resolve Python and reached no assertions; both logs retained. |
| Whitespace/scope | `git diff --check` and scoped staged-path inspection | Passed; author changes confined to docs/evidence/issue-141 and scripts/test_core_batch_141.py. |

## Pending gates

There are **0 missing authored fields** and no unresolved author-content row.
Separate review is pending for **200 language fields, 200 selected-span rights fields,
50 selected uses and 50 spelling/construction relations**, including the noted source anomaly.
Reviewer is null; independently accepted/admitted new fields are zero.
Combined conversion/package obligations, all-group native lookup/save/reopen, preservation
through resource updates, and learner acceptance are coordinator-owned and **not run here**.
Frontend/Rust/native builds are not run for this content-only author scope.
The ticket and whole-core release remain incomplete until their own gates pass.
