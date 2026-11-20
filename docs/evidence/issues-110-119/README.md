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

## Verification

Frontend lint, typecheck, build, the browser smoke test and all **13 core browser checks** passed on Windows 11 x64. Converter/cohort, portable-package, backend and final native checks are being recorded at `target/implement-110-119/`.

## Limits

Whole-machine offline execution, other-platform native flows, learner acceptance and formal release readiness remain unverified. The full D2 target remains incomplete; CEFR grading is outside this frozen batch. [PR #191](https://github.com/zardonc/lexique/pull/191) closes #110–#119 when merged; #78 and #9 remain open.
