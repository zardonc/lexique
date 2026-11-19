# Issue #94: content handoff

All 50 frozen groups, rows 651–700, have a selected-use draft with FR/IPA/EN/ZH fields: 200 drafted fields, zero independently accepted fields and zero admitted groups. The fixed denominator remains 4,087 groups. No previously admitted row belongs to this ticket.

The [groups](groups.json), [sources](sources.json), [exact excerpts](source-excerpts.json), [selected context](source-context.json) and [author field checks](field-review.json) are additions-only shards. They retain pending independent acceptance. The [admission state](admission.json) records the outstanding gates; [reuse and preservation](reuse.json) identifies the unchanged baseline. The converter's shared inputs, runtime, global counters, notices and tests are unchanged.

Full captures and command logs are retained at `C:/Users/inthe/IdeaProjects/lexique-content-94/target/issue-94/`; [capture bindings](source-captures.json) identify the actual files and revisions. The same additions are supplied outside the repository at `C:/Users/inthe/.codex/tmp/lexique-spec-90-99/batch-94/` for central keyed integration.

The focused [membership check](../../../scripts/test_core_batch_94.py) verifies the existing admitted inputs plus this draft shard against the exact frozen CSV rows and four nonempty fields. It passed after the initial 50 missing-row failures. It checks a content handoff, not a converted or admitted SQLite artifact.

See [source boundaries and selected uses](source-research.md) and [validation](validation.md). A different reviewer must accept all exact fields, usage, spelling and selected-text rights before the parent can admit the content and run combined conversion, native lookup/save/reopen, preservation and packaging checks.
