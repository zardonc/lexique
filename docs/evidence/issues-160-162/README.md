# D2 frozen batches #160–#162

`core-162-v1` admits **135 new groups / 540 fields** at frozen rows 3951–4087, bringing the core to **4,062 groups / 16,248 fields**. The denominator remains 4,087 groups / 3,773 headwords. The 25 unaccepted groups are owner-approved zero-field annotations with 100 missing fields. Literal everyday coverage is 4,059 complete / 28 incomplete groups; the same three specialist senses are counted separately.

| Batch | Frozen rows | Admitted groups / fields | New annotations |
| --- | --- | --- | --- |
| [#160](../issue-160/README.md) | 3951–4000 | 49 / 196 | [rex/NOM, row 3953](../issue-160/owner-approval.json) |
| [#161](../issue-161/README.md) | 4001–4050 | 50 / 200 | 0 |
| [#162](../issue-162/README.md) | 4051–4087 | 36 / 144 | [ais/NOM, row 4057](../issue-162/owner-approval.json) |

Each admitted field and selected construction passed separate independent quality and selected-expression rights review. Authored proposals, exact replacements, fixed source versions/spans, contributor pagination and applicable original-work rights remain retained. The rejected rex/ais proposals remain unadmitted; owner approval does not resolve rex's selected-expression rights or admit a specialist meaning. Sir's French title binding is accepted without a new runtime alias. All prior 3,927 groups / 15,708 fields, source/review/context arrays, frozen manifest and SQLite payloads compare exactly; the 15 spelling relations, 11 aliases and three specialist senses remain intact.

The [canonical resource](../../../resources/core-pilot-79/README.md), [notices](../../../resources/core-pilot-79/NOTICES.md), [CC BY-SA terms](../../../resources/core-pilot-79/COPYING) and [FranceTerme Licence Ouverte original](../../../resources/core-pilot-79/LICENSE-FranceTerme.pdf) accompany the matching package. `pnpm core:prepare` reuses the existing converter and fixed local inputs. Archived source audits additionally require the retained source captures identified in the shard manifests; they do not fetch substitutes.

## Actual Windows x64 verification

The [receipt](verification.json) binds the runtime checks to source `14212a3d16e06758caf38b7d6bd9af4b5ee10020`, executable SHA-256 `36506f2508b233e24a4961560a20aefed1a2ff0bd3e1944da5624e03f07438b2` and SQLite SHA-256 `d81e1acda7a0575271ae86968d20aa00510d8844a401a2499acb915e1ee78603`. Three real native batch scenarios verified every admitted group, its four fields/use and provenance, trusted saves and normal reopen. Annotation lookups contain no admitted matching POS. The closed core-159-v1 baseline upgraded from schema 20 to 21 with its two old records, examples, FSRS and selected/protected audio intact; new first/last batch records saved successfully. All four scenarios retained the existing 60-second hard limit, zero external WebView requests and zero remaining host/worker/helper processes.

| Native evidence | Lookup | Normal reopen |
| --- | --- | --- |
| #160 | [Screenshot](batch-160-lookup.png) | [Screenshot](batch-160-reopened.png) |
| #161 | [Screenshot](batch-161-lookup.png) | [Screenshot](batch-161-reopened.png) |
| #162 | [Screenshot](batch-162-lookup.png) | [Screenshot](batch-162-reopened.png) |

Decision 15 measurements: 188,036,913 bytes raw inputs; 18,894,848 bytes read-only SQLite, within the unchanged 20,000,000-byte native bound; 188,097,811 bytes bundled materials; 206,992,659 bytes installed core excluding report; 207,619,275 bytes complete core directory; 277,304,539 bytes total portable installation; ZIP 50,515,371 bytes, SHA-256 `349992f829adf5442182d87323e9dcbfc307736d768bde1fbc0a804e9ba3f6a6`. The packager verified 187 files and excludes learner data.

Checks passed: 18 core tests, 25 batch tests, three shared-helper tests, 16 portable tests, two native directory fixtures, frontend lint/typecheck/build, rustfmt, locked workspace compilation, separately precompiled backend tests and the four native scenarios. Full commands/logs and original run results remain in the task checkout's `target/implement-160-162/` and short `target/issue-160/n`, `target/issue-161/n`, `target/issue-162/n`, `target/issue-162/u` paths; the receipt records absolute locations. Source archives and author/reviewer/merge proof remain under `C:/Users/inthe/IdeaProjects/lexique-d2-160-162/context/`.

Initial integration checks found stale coverage prose/row-bound assertions and relocated original-document links; final corrections retain the established coverage report contract, full last-batch boundary and byte-preserved author snapshots. A superseded review and an interrupted core run remain historical evidence, not acceptance. The first workspace compile lacked Python on PATH; the corrected pinned MSVC/Cargo/Python check passed.

## Release boundaries

The [PR](https://github.com/zardonc/lexique/pull/220) closes #160–#162 and relates to #78. Learner acceptance, other-platform native execution, clean-machine/whole-machine offline and formal release remain unrun; CEFR remains unknown. The annotated fields remain missing. #78 stays open.

Resource rollback restores the previous read-only core; saved learner digests retain their original resource versions. This package uses schema 21, while the old core-159-v1 executable uses schema 20: binary rollback needs a compatible pre-upgrade backup or its separately preserved old data directory. Do not open the upgraded learner database with the old executable.
