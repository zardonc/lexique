# Issue #93 content handoff

The slice contains all 50 exact frozen groups at rows 601–650, one selected use
per group, and 200 nonempty FR/IPA/EN/ZH fields. None of these rows was already
admitted at the integration base `3397219e8d159ef60db3ed5456bdcb2d6ff41b4a`.
These are author-validated drafts awaiting a separate independent reviewer:
**zero newly admitted groups or fields** are claimed by this handoff.

Parent spec: [#78](https://github.com/zardonc/lexique/issues/78).
Content ticket: [#93](https://github.com/zardonc/lexique/issues/93).
Membership remains the exact licensed [frozen manifest](../../../resources/core-pilot-79/scope-manifest.csv).

The additions-only files are [groups](groups.json), [sources](sources.json),
[source contexts](source-context.json), [selected excerpts](source-excerpts.json),
[spelling relation](spelling-relations.json), [draft field and usage review](field-review.json),
[reuse record](reuse.json), [capture registry](source-captures.json), and
[admission status](admission.json). Source and sense IDs use the `b93` namespace.
The same shards are supplied outside the repository at
`C:/Users/inthe/.codex/tmp/lexique-spec-90-99/batch-93/`.

All four fields derive from the selected fixed French Wiktionary definition and
its applicable French POS/IPA. English and Chinese are explicitly recorded
Lexique translations or scoped explanations. They are not attributed to an
unfetched English or Chinese dictionary. Each field retains its exact raw
excerpt, one-based line, source identity, fixed revision, contributor history,
applicable content licence and actual modification.

There are 40 fixed French content pages plus the fixed ASCII-apostrophe redirect
for `d'abord`. These are actual latest revisions returned by the retained API
responses, not October-cutoff revisions. Complete API/wikitext captures remain
under `C:/Users/inthe/IdeaProjects/lexique-content-93/target/issue-93/`.
Full pages, external quotations and media are not bundled in these shards.

Source boundaries and the 50 selected uses are in [source research](source-research.md).
The separate reviewer must accept the exact proposed fields, usage, spelling,
attribution and selected-text rights. `accepted_text_quality_status` and
`accepted_usage_quality_status` remain pending; no independent reviewer is
attributed. Source status cannot pass the existing admission gate as written.

Verification: [commands and results](checks.json). The focused
[artifact membership check](../../../scripts/test_core_batch_93.py) was red
before the content existed and passes with these shards. It checks the frozen
membership/frequencies, four-field shape and languages, nonempty content, fixed
revision URLs and matching selected excerpts. It does not establish linguistic
quality, rights eligibility, converter inclusion, native behaviour or release
readiness.

The parent owns independent acceptance, keyed integration into the current
monolithic resources, alias registration, global version/counters/notices,
checksums, combined conversion, native lookup/save/reopen, preservation and
packaging. Those checks have not been run in this worker. Shared resource,
runtime and global-test files remain unchanged.
