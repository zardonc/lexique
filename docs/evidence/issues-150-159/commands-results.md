# D2 batch verification commands

Windows x64, PowerShell 7, pnpm 11.25.0, Rust 1.99.1 and Python 3.14.10. Rust/native commands ran in the MSVC developer environment with pinned Cargo and Python first on PATH.

Tested runtime/resource commit: `5fcba9932f74e298d4a798e6eb195faad5f51507`. Logs, screenshots, lifecycle records and closed fixtures remain under `C:/Users/inthe/IdeaProjects/lexique-d2-150-159/integration/target/`. [Verification](verification-final.json) records executable/core identities, all native results and preservation checks.

| Command from the repository root | Result |
| --- | --- |
| `uv run --python 3.14.10 python -B -X utf8 scripts/core_pilot.py prepare target/desktop-core-pilot` | Passed: 3,927 groups / 15,708 fields, 18,268,160-byte SQLite |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_pilot.py -v` | 18 passed |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p 'test_core_batch_15[0-9].py' -v` | 75 passed |
| `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_package_portable.py -v` | 16 passed, including missing/changed FranceTerme licence |
| `pnpm lint` / `pnpm typecheck` / `pnpm build` | All passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo check --locked --workspace` | Passed |
| `pnpm rust:test:build` | Passed; separate compilation |
| `pnpm test:rust` | 242 passed in 44.61 seconds; hard 60-second limit retained |
| `pnpm test:native:fixtures` | Passed |
| `pnpm desktop:build --no-bundle` | Passed |
| `python -X utf8 scripts/package_portable.py --output-dir target/d2-150-159/final-package` | Passed: 49,921,060-byte ZIP |
| `pnpm test:docs` | Passed |
| `git diff --cached --check` | Passed |

Baseline preservation passed for all 3,431 groups / 13,724 fields, source/review/context prefixes, frozen manifest and compiled payloads. All 13 Git-stored core materials match the local bytes. The package carries the original FranceTerme PDF and `audition` producer/source/publication-date notice.

Native commands ran from `apps/desktop` with the packaged executable:

```powershell
$taskRoot = 'C:/Users/inthe/IdeaProjects/lexique-d2-150-159/integration'
$taskPackage = "$taskRoot/target/d2-150-159/final-package/Lexique-0.1.0-windows-x64-portable"
$taskBinary = "$taskPackage/Lexique.exe"
$taskConjugation = "$taskRoot/target/desktop-conjugation-v6-stats"
$taskFrequency = "$taskRoot/target/desktop-frequency-v1"
$taskCuration = "$taskRoot/resources/core-pilot-79/curated-inputs.json"
foreach ($taskIssue in 150..159) {
  node e2e/native-d2-pilot-check.mjs batch $taskPackage $taskBinary $taskConjugation $taskFrequency $taskCuration "$taskRoot/target/issue-$taskIssue/core159-final" $taskIssue
}
node e2e/native-d2-pilot-check.mjs upgrade $taskPackage $taskBinary $taskConjugation $taskFrequency $taskCuration "$taskRoot/target/issue-159/upgrade-from-core149" "$taskRoot/target/issue-149/baseline-core149-v1"
```

All ten batches and the upgrade passed on the `5fcba9932f74e298d4a798e6eb195faad5f51507` build. Each retains a hard 60-second limit, SQLite integrity checks, zero observed external WebView requests and zero remaining owned processes. Upgrade preserves references, digests, notes, examples, FSRS and cached/selected audio. Its baseline is a closed, seeded core-149 profile, separate from the prepared read-only artifact.

## Post-review fix verification

Post-review fixes at `66b12b514ed21cd1da4393b8c6465c57c7819e53` (based on `3e3aa4c71c8277522919f2ac395ad1b04c867158`) ran in an isolated git worktree (`target/pr-219-fixes/worktree`) using Windows x64, Visual Studio 2026 Developer Command Prompt v18.0 (`vcvars64.bat`), pnpm 11.25.0, Python 3.14.10 and Rust 1.99.1:

| Check | Result |
| --- | --- |
| `scripts/core_pilot.py prepare target/desktop-core-pilot` | Passed: 3,927 groups / 15,708 fields; SQLite SHA-256 `1240f403db18f130943000d37782857ca6695a92f60c0776466789a3ddbed2fc`; raw inputs 180,797,580 B; read-only SQLite 18,268,160 B; bundled materials 180,896,776 B; installed core excl. `report.json` 199,164,936 B; full prepared `dictionary/core-pilot-79/` 199,768,852 B (incl. 603,916 B `report.json`) |
| `unittest` `test_core_pilot.py` | 18 passed |
| `unittest` `test_core_batch_15[0-9].py` | 76 passed |
| `unittest` `test_core_batch_test_helpers.py` | 3 passed |
| `unittest` `test_package_portable.py` | 16 passed |
| `pnpm test:native:fixtures` | 2 passed |
| `pnpm lint` / `pnpm typecheck` / `pnpm build` | All passed |
| `cargo fmt --all -- --check` / `cargo check --locked --workspace` | Both passed |
| `pnpm rust:test:build` | Passed; separate precompilation |
| `pnpm test:rust` | 242 passed in 23.65 seconds; hard 60-second limit retained |
| `cargo test --locked -p lexique-desktop --lib dictionary_links` | 2 passed (including missing `LICENSE-FranceTerme.pdf` bundle rejection) |
| `pnpm desktop:build --no-bundle` | Passed (`Lexique.exe` SHA-256 `3ffc6cc987cdd752f0d74ee1e018495ab1477501918366b8ad067c3f3f999d54`) |
| `python -X utf8 scripts/package_portable.py --output-dir target/d2-150-159/post-review-package` | Passed: `49,926,877`-byte ZIP (SHA-256 `ed30cd6feac68c9085c479edc7d236a32659a6d67cf3afbdecfbb884a7295e6b`), `269,347,109` B `installed_bytes`, `199,768,852` B `core_installed_bytes`, `187` verified files |
| Native batches #150–#159 (`target/issue-150..159/core159-verified`) + upgrade (`target/issue-159/upgrade-from-core149-verified`) | All 11 passed; 60-second hard limit retained, SQLite integrity passed, zero external WebView requests, zero remaining owned hosts/workers/helpers |
| `scripts/check-docs.ps1` / `git diff --check` | Both passed |
| Preservation diff vs `3e3aa4c71c8277522919f2ac395ad1b04c867158` | Passed: all 3,431 prior groups (`1..3450`), `docs/evidence/issue-153/authored/*` and non-3613 fields unchanged except the 49 #153 IPA `modification` notices |

Learner acceptance, whole-machine offline behavior, other-platform native execution, full D2 / CEFR and formal release acceptance remain unrun. Upgrade/rollback use matching executable/core pairs and preserved learning-data directories/backups. Parent #78 remains open.
