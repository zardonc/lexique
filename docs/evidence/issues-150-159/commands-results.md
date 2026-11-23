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

All ten batches and the upgrade passed. Each retains a hard 60-second limit, SQLite integrity checks, zero observed external WebView requests and zero remaining owned processes. Upgrade preserves references, digests, notes, examples, FSRS and cached/selected audio. Its baseline is a closed, seeded core-149 profile, separate from the prepared read-only artifact.

Learner acceptance, whole-machine offline behavior, other-platform native execution, full D2 / CEFR and formal release acceptance remain unrun. Upgrade/rollback use matching executable/core pairs and preserved learning-data directories/backups. Parent #78 remains open.
