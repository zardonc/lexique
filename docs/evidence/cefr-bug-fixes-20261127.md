# CEFR matching, import evidence and notebook verification

2026-11-27. Branch `codex/cefr-option2-and-core-gender-pilot`, base `136c43f`.

The optional CEFR overlay preserves exact headword matches, displays in the unsaved collection
editor, and batches notebook queries by visible page. Noun-gender evidence uses reproducible LF
bytes under the repository's Git attributes. Learning data and resource admission are unchanged.

## Resulting behavior

| Finding | Cause and result | Regression |
| --- | --- | --- |
| Resource catalog test | FLELex adds the fourteenth catalog entry. The test checks 14 entries and the exact UCLouvain download URL. | `dictionary_links::tests::only_catalogue_ids_select_https_pages` |
| LF-checkout checksum | Windows text output used CRLF bytes while Git requires LF. The capture script explicitly writes UTF-8/LF; the seven affected text files use LF, and the existing noun-evidence checksum pins those bytes. | Generated-file newline check and LF-checkout checksum in `test_complete_core_genders.py` |
| Exact spelling CEFR | Combining grades for the query, selected-source spelling and aliases discarded valid exact matches. Each candidate spelling is now queried separately in that order, within eligible POS tags. Conflicting levels within a matching spelling remain null. | Synthetic different-grade pairs for `enceinte/enceint`, `clef/clé`, `cieux/ciel`; source fallback, POS isolation, aliases and exclusions |
| Unsaved collection | The draft summary omitted its runtime adapter. It now receives the same adapter as saved summaries and displays imported grades before first save. | Unsaved collection editor E2E |
| Notebook request fan-out | Row-local hooks issued separate IPC requests and repeated errors. The visible page now shares queries and one error/retry banner, split at the existing 200-query boundary. Word lookup uses the existing BTreeMap's range search. | Two-row request/error/retry E2E and 220-query `[200, 20]` batches followed by page navigation |

The two-row development-mode reproduction issued four CEFR requests; the corrected page issues
one. A synthetic 14,236-row dataset took 939.2052 ms for 1,000 lookups before range lookup and
1.9363 ms afterward on this Windows debug-test build. This measures lookup work, not native GUI
latency. The temporary benchmark and runtime-omission probe were removed from application source.

## Validation

Commands use pinned Cargo with MSVC initialized and uv-managed Python. Full logs are retained
under ignored `target/cefr-bug-*` paths in the isolated worktree.

- `uv run --offline --no-project python -X utf8 scripts/test_complete_core_genders.py`: three tests pass,
  including generator output and the canonical LF checksum. The checksum reproduction was red
  before normalization; the unsaved-editor regression was also verified against runtime omission.
- `uv run --offline --no-project python -X utf8 scripts/complete_core_genders.py --output target/core-gender-completion`:
  2,134 noun rows, 187 original residuals, all 307 fixed-revision captures replayed, zero failures.
  Generated noun evidence is byte-identical to the version-controlled LF file.
- `uv run --offline --no-project python -X utf8 -m unittest discover -s scripts -p test_cefr_gender_pilot.py`:
  seven tests pass.
- `uv run --offline --no-project python -X utf8 scripts/test_core_pilot.py CorePilotPreparationTests.test_gender_evidence_keeps_pending_semantic_decisions_out_of_the_resource`:
  resource preparation passes with 1,940 admitted genders and 194 pending values absent.
- `pnpm lint`, `pnpm typecheck`, `pnpm build`: pass.
- `pnpm rust:test:build`, then `pnpm test:rust`: workspace compilation and all 262 reported checks pass;
  initial compilation remains separate from the runner's hard 60-second execution timeout.
- `cargo test --locked --release -p lexique-desktop --lib --no-run`, then
  `node scripts/test-rust.mjs --desktop-lib dictionary_links::tests:: --test-threads=1` and
  `node scripts/test-rust.mjs --desktop-lib cefr::tests:: --test-threads=1`: two catalog checks
  and the native filesystem import/replacement/restart-read check pass.
- `pnpm test:e2e` for `cefr-gender`, `core-pilot`, `core-contract`, `core-preservation`, `dictionary`,
  `dictionary-tabs`, `dictionary-resources`, `notebook`, `review`, `review-queue`, `selection`, `ocr`,
  with `--workers=4`: all 166 applicable cases pass in one run.
- `cargo fmt --all --check`, `scripts/check-docs.ps1 -RepositoryRoot .`, `git diff --check`:
  pass; 596 documents and 4,602 local links have no problems. Root checkout status is unchanged,
  dependency lockfiles are unchanged, task processes are stopped, and no FLELex data file is tracked.

## Limits

The [194 semantic candidates](noun-gender-residual-review.md) remain pending independent human
review. ADR 0012 records the device-local overlay and noun-gender boundaries. The full browser
suite, full desktop backup/restore subprocess suites, live Windows
WebView2/file-picker flow, other platforms, whole-machine offline behavior and release readiness
were not revalidated by these focused fixes.
