# D2 frozen batches #120–#129

`core-129-v1` contains **2,435 accepted groups / 9,740 fields**, including **500 new groups / 2,000 fields** at frozen rows 1951–2450. The denominator remains **4,087 groups / 3,773 headwords**: 1,652 groups are unaccepted; 2,433 everyday groups are complete and 1,654 are incomplete. The two previously approved specialist senses and 15 zero-field annotations / 60 missing fields remain separately counted.

All ten batches passed separate content and selected-rights review. **1,973 fields remain as proposed; 27 exact replacements** preserve the selected meaning, grammar or construction. Original proposals and independent-review snapshots are retained in each batch.

| Batch | Frozen rows | Accepted groups / fields | Replaced fields |
| --- | --- | --- | --- |
| [#120](../issue-120/README.md) | 1951–2000 | 50 / 200 | 3 |
| [#121](../issue-121/README.md) | 2001–2050 | 50 / 200 | 3 |
| [#122](../issue-122/README.md) | 2051–2100 | 50 / 200 | 7 |
| [#123](../issue-123/README.md) | 2101–2150 | 50 / 200 | 4 |
| [#124](../issue-124/README.md) | 2151–2200 | 50 / 200 | 0 |
| [#125](../issue-125/README.md) | 2201–2250 | 50 / 200 | 4 |
| [#126](../issue-126/README.md) | 2251–2300 | 50 / 200 | 1 |
| [#127](../issue-127/README.md) | 2301–2350 | 50 / 200 | 1 |
| [#128](../issue-128/README.md) | 2351–2400 | 50 / 200 | 3 |
| [#129](../issue-129/README.md) | 2401–2450 | 50 / 200 | 1 |

## Resource admission

The [canonical resource](../../../resources/core-pilot-79/README.md) converts without network access and includes [combined notices](../../../resources/core-pilot-79/NOTICES.md), fixed source versions, contributors, selected import/original-work boundaries and [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING). The format-1 SQLite artifact is **11,350,016 bytes**. All 1,935 prior raw groups and compiled payloads, 15 spelling relationships / 11 aliases, and 15 approved zero-field annotations are preserved.

[Conversion](../../../target/implement-120-129/core-prepare-final.log), [coverage](../../../target/desktop-core-pilot/coverage.json), [preservation](../../../target/implement-120-129/preservation.json), [author archives](../../../target/implement-120-129/author-archive.json) and [reviewer archives](../../../target/implement-120-129/reviewer-archive.json) retain the measured outputs and byte-preservation evidence. Fresh author recapture for #124 encountered HTTP 429; independent retained-source validation and offline reproduction passed.

## Verification

Backend formatting, locked precompilation and **210 bounded backend tests** passed after the character-budget fixture was restricted to its selected headword. Production size limits remain unchanged. **135 core/cohort tests**, **16 portable-package tests**, documentation links and diff checks passed. The production build, ten native batch runs and closed-baseline upgrade are pending.

## Remaining gates

Whole-machine offline denial, other-platform native execution, learner acceptance and formal release readiness remain unverified. CEFR remains unknown. [PR #193](https://github.com/zardonc/lexique/pull/193) implements these ten batches; #78 and the complete D2 target remain open.
