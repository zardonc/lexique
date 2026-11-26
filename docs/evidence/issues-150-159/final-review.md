# Final review and reuse verification

The independent standards and spec reviews compared `72685c57285c86c89ae828dd21d72afaf3413f27` against base `03a27e42c261afda7ffd4a927a4d2810f8221e8a` for issues #150–#159.

## Standards

Zero confirmed documented-standard violations. Both nonblocking maintenance suggestions were resolved in `cd66816ce71bab03f460a5ceff6d6e7ea9eb4378`: contributor aggregation and pagination checks now reuse [the existing helper](../../../scripts/core_batch_test_helpers.py), and both native harnesses share the D2 directory list from [the existing backup-state module](../../../apps/desktop/e2e/native-backup-state.mjs). Batch-specific assertions and each harness's additional roots and containment predicate are preserved.

All 75 existing batch checks and two focused contributor tests passed. Three pagination mutations and four ownership mutations were rejected. Both native fixtures, lint, contributor/membership comparisons and the acyclic import check passed. The fresh merger confirmed exactly the intended 14 code/test files changed and a clean integration checkout.

## Spec

Zero confirmed findings for the amended subset specifications: no missing or partial requested requirements, scope creep, or incorrect implemented behavior established. The compiled core includes all 496 new groups / 1,984 fields; all 3,431 prior payloads and the frozen 4,087-group denominator remain unchanged. The four approved annotations retain zero senses and four missing fields each. Retained publisher captures and independent source-span review support `by/NOM` and the ordinary letter-name `g/NOM`, `/ʒe/`.

## Post-review native execution

Windows x64 executed the revised harness at `cd66816ce71bab03f460a5ceff6d6e7ea9eb4378` against the unchanged package built at `5fcba9932f74e298d4a798e6eb195faad5f51507`. Batch #159 passed in 26.53 seconds and the core-149 upgrade passed in 15.16 seconds, using fresh destinations `target/issue-159/core159-post-review` and `target/issue-159/upgrade-from-core149-post-review`. Both retained the hard 60-second limit, matched the recorded executable, observed zero external WebView requests and left zero owned hosts, workers or helpers.

The earlier CI run [45268222587](https://github.com/zardonc/lexique/actions/runs/45268222587) at reviewed head `72685c57285c86c89ae828dd21d72afaf3413f27` passed Windows x64 and hit the hard 60-second limit in Windows ARM64 `test:desktop:lib`; its original cause is unresolved. The final commit's CI status is attached to [PR #219](https://github.com/zardonc/lexique/pull/219).

## Follow-up review fixes

Review follow-up fixes in `66b12b514ed21cd1da4393b8c6465c57c7819e53` (from `3e3aa4c71c8277522919f2ac395ad1b04c867158`) address all confirmed Spec and Standards findings:

- **Spec fixes**: removed the erroneous `vôtres` clause from the 49 non-3613 batch #153 IPA `modification` notices across [issue-153 groups](../issue-153/groups.json), [issue-153 rights](../issue-153/rights.json) and [curated-inputs.json](../../../resources/core-pilot-79/curated-inputs.json) while leaving row 3613 and `docs/evidence/issue-153/authored/*` untouched; added `LICENSE-FranceTerme.pdf` / Licence Ouverte 1.0 attribution and bundle checks across [dictionary_links.rs](../../../apps/desktop/src-tauri/src/dictionary_links.rs), [core_pilot.py](../../../scripts/core_pilot.py), [package_portable.py](../../../scripts/package_portable.py) and [the canonical README](../../../resources/core-pilot-79/README.md); added Decision 15 raw-input and uncompressed installed-contribution size measurements in `core_pilot.py`, `package_portable.py` and the evidence records; updated root [README.md](../../../README.md) and the seven stale batch READMEs (#150, #152, #153, #155, #156, #158 and #159).
- **Standards fixes**: moved `ContributorCaptureTests` out of [test_core_batch_150.py](../../../scripts/test_core_batch_150.py) into `scripts/test_core_batch_test_helpers.py` using `tempfile.TemporaryDirectory()`, replaced the `approved_on` conditional ladder in `core_pilot.py` with explicit 5-tuple dates in `ANNOTATIONS`, deduplicated annotation row lists in [native-d2-pilot-check.mjs](../../../apps/desktop/e2e/native-d2-pilot-check.mjs) and extracted `assert_core_load_inputs_rejected` in [core_batch_test_helpers.py](../../../scripts/core_batch_test_helpers.py) for batches #151, #152, #154, #156, #157, #158 and #159.
- **Verification status**: `core_pilot.py prepare`, 18 core tests, 76 batch tests, 3 helper tests, 16 portable tests, `pnpm test:native:fixtures` (2 passed), `pnpm lint`, `pnpm typecheck`, `pnpm build`, `cargo fmt --all -- --check`, `cargo check --locked --workspace`, `pnpm rust:test:build`, `pnpm test:rust` (242 passed in 23.65s within the hard 60s limit), `cargo test --locked -p lexique-desktop --lib dictionary_links` (2 passed), `pnpm desktop:build --no-bundle` (`Lexique.exe` SHA-256 `3ffc6cc987cdd752f0d74ee1e018495ab1477501918366b8ad067c3f3f999d54`), `package_portable.py` (`49,926,877`-byte ZIP, `269,347,109` B `installed_bytes`, `199,768,852` B `core_installed_bytes`, `187` verified files), all ten post-review native batch runs (`target/issue-150..159/core159-verified`), the `core-149-v1` upgrade run (`target/issue-159/upgrade-from-core149-verified`), `scripts/check-docs.ps1` and preservation checks all passed with zero observed external WebView requests and zero remaining owned hosts, workers or helpers.

Learner acceptance, whole-machine offline behavior, other-platform native application execution, full D2 / CEFR and formal release acceptance remain unrun. Parent #78 stays open.
