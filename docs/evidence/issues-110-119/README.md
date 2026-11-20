# D2 content batches #110–#119

`core-119-v1` contains **1,935 independently accepted groups / 7,740 fields**, including **1,933 everyday groups** and the two existing specialist senses. Frozen rows **1451–1950** contain **500 complete groups / 2,000 fields**: **499 new groups / 1,996 new fields**, plus the unchanged `côte/NOM` group at row 1911. The fixed denominator remains **4,087 groups / 3,773 headwords**; **2,152 groups remain unaccepted** and **2,154 remain incomplete for everyday use**.

| Independent review | Frozen rows | Complete groups | Fields | Reused groups |
| --- | --- | ---: | ---: | ---: |
| [#110](../issue-110/independent-review.md) | 1451–1500 | 50 | 200 | 0 |
| [#111](../issue-111/independent-review.md) | 1501–1550 | 50 | 200 | 0 |
| [#112](../issue-112/independent-review.md) | 1551–1600 | 50 | 200 | 0 |
| [#113](../issue-113/independent-review.md) | 1601–1650 | 50 | 200 | 0 |
| [#114](../issue-114/independent-review.md) | 1651–1700 | 50 | 200 | 0 |
| [#115](../issue-115/independent-review.md) | 1701–1750 | 50 | 200 | 0 |
| [#116](../issue-116/independent-review.md) | 1751–1800 | 50 | 200 | 0 |
| [#117](../issue-117/independent-review.md) | 1801–1850 | 50 | 200 | 0 |
| [#118](../issue-118/independent-review.md) | 1851–1900 | 50 | 200 | 0 |
| [#119](../issue-119/independent-review.md) | 1901–1950 | 50 | 200 | 1 |

All **1,436 previous raw records and SQLite payloads**, their field reviews and source contexts, **15 spelling relations / 11 runtime aliases**, and **15 zero-field annotations / 60 missing fields** remain unchanged. No group is dropped or substituted, and no new exception, runtime alias or CEFR assignment is introduced. Source records total **1,968**, including **493 new sources**. [Preservation evidence](../../../target/implement-110-119/preservation.json) checks the previous compiled payloads and all existing aliases byte-for-byte.

Every new selected FR/IPA/EN/ZH field and modern use passed independent review. [Bundled notices](../../../resources/core-pilot-79/NOTICES.md) retain each fixed source and its applicable credit. The directly selected `tien/NOM` wording comes from the identified DAF8 1935 original, tome 2 page 659; its original public-domain boundary and Lexique's CC BY-SA 4.0 adaptations remain distinct. The converter retains its printed edition identifier and capture-line provenance. It rejects a missing edition, unreviewed publication or incompatible adaptation licence. All existing Wikimedia revision validation and payloads remain unchanged.

Original author and reviewer logs are byte-preserved at `target/implement-110-119/author-evidence/issue-N` and `target/implement-110-119/reviewer-evidence/issue-N`; the [author archive](../../../target/implement-110-119/author-archive.json) and [reviewer archive](../../../target/implement-110-119/reviewer-archive.json) record their original and retained roots.

Final admission records report zero missing batch fields and the accepted canonical counts. Original pre-admission statuses and counters remain byte-preserved at `C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/pre-final-admissions/issue-N.json`. Cohort checks require the exact final admission status and preserve independent reviewer identity, decisions and original proposal history.

## Verification

The packaged native build is from `00ea06228926e1b9a4e8644a267bce46316f1a39` on Windows 11 x64. Its executable SHA-256 is `53517529b48a639db9a7f2e7606565bcc81e688c1e05dfdd09b1535b6a80decc`. The format-1 core resource is **9,039,872 bytes**, SHA-256 `5afa6c9c3d80f386e9a3e034895fabedc6e62dc0bba051cbfe855df605341ed4`. The portable ZIP passed **183-file** verification. [Build metadata](../../../target/implement-110-119/final-build.json), [package results](../../../target/implement-110-119/final-package.json), [resource report](../../../target/desktop-core-pilot/report.json) and [coverage](../../../target/desktop-core-pilot/coverage.json) retain the exact inputs and output identities.

Final admission metadata and all **90 converter/cohort tests** passed at clean source `c0c09ff3137ab3064706e71d0819bfa93d556836`, using Python 3.14.10 via uv, pnpm 11.25.0 and PowerShell 7 on Windows 11 x64. The owning evidence root is `C:/Users/inthe/IdeaProjects/lexique-final-fixes-110-119/target/final-review-fixes`; [execution metadata](../../../target/final-review-fixes/core-check.json) records the command, exact source and clean-tree status. `pnpm test:docs` and `git diff --check` passed. Native, runtime and canonical resource inputs remain unchanged from `00ea062`; its eleven native scenarios retain that build identity.

| Check | Result | Evidence |
| --- | --- | --- |
| Frontend lint, typecheck and build | Passed | [lint](../../../target/implement-110-119/lint-final.log), [types](../../../target/implement-110-119/typecheck.log), [build](../../../target/implement-110-119/build.log) |
| Browser smoke and core flows | 1 + 13 passed | [smoke](../../../target/implement-110-119/browser-smoke.log), [core](../../../target/implement-110-119/browser-core.log) |
| Converter and content cohorts, final metadata | 90 passed at `c0c09ff3137ab3064706e71d0819bfa93d556836` | [tests](../../../target/final-review-fixes/core-all.log) |
| Documentation | Passed; 318 documents, no problems | [check](../../../target/final-review-fixes/docs-clean-final-source.log) |
| Portable package | 16 passed | [tests](../../../target/implement-110-119/package-119-green.log) |
| Rust formatting and locked precompilation | Passed | [format](../../../target/implement-110-119/rust-fmt.log), [precompile](../../../target/implement-110-119/rust-precompile-final.log) |
| Backend execution | 210 passed; hard 60-second execution limit retained | [tests](../../../target/implement-110-119/rust-tests.log) |
| Actual packaged native batches | All ten passed: every exact group/field matches; every headword saved and reopened | [native execution](../../../target/implement-110-119/native-execution-verification.json) |
| Old-resource upgrade | Passed; notes, examples, FSRS and protected Opus identities preserved; closed baseline files byte-identical | [upgrade](../../../target/issue-119/native-upgrade/results.json) |

Every native scenario uses the recorded executable, retains the hard 60-second limit, reports **zero external WebView requests**, and leaves **zero owned hosts, workers or helpers**. The `tien/NOM` source panel displays the published edition and all four field sources with their versions and spans.

Broad checks ran during integration. The final publication attribution and native source-detail assertions were followed by focused publication validation, lint, resource preparation and prior-payload preservation. The final portable checks and native build/scenarios include the package-version update. Full logs and this distinction are retained in [aggregate verification](../../../target/implement-110-119/verification-final.json).

## Limits

Whole-machine offline execution, other-platform native flows, learner acceptance and formal release readiness remain unverified. The full D2 target remains incomplete; CEFR grading is outside this frozen batch. [PR #191](https://github.com/zardonc/lexique/pull/191) closes #110–#119 when merged; #78 and #9 remain open.
