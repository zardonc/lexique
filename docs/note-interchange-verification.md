# Note interchange acceptance (#183 / #189)

2026-11-20, Windows x64. This closes the combined CSV, single Markdown and
Markdown ZIP execution gate for the six slices in the
[specification](note-interchange-spec.md). It does not establish other-platform
native acceptance, whole-machine offline operation or release readiness.

## Delivered contracts

- ZIP export uses the retained SQLite `NoteExports` snapshot shared with CSV
  and single Markdown: saved scope, count and order, one readable Windows-safe
  headword-plus-record-ID filename per note. Only exchange fields are included.
- ZIP import reads entries directly with zip 9.5.1, including ordinary Deflate
  archives. Entry names never become extraction/write targets or note identity.
  UTF-8, template, field and 101-example validation remain in the existing
  Markdown parser; 102 examples are rejected. Invalid entries show filename and
  reason and require exclusion. Container, header and CRC errors are fatal.
- NFC-equivalent identical content and field presence deduplicate. Any
  conflicting provided field/content invalidates the entire headword group.
  Shared enrichment, explicit resource-error consent, revision revalidation,
  atomic selected-batch saving and response-loss replay remain authoritative.
  New records retain actual container filename/format provenance; updates retain
  their original provenance. Import never automatically enrolls cards or audio.
- The shared file reader no longer rejects valid application exports at 32 MiB.
  A real saved SQLite → CSV → preview/reimport regression covers six records with
  101 examples and 20,000-character ASCII fields, and three records with Chinese
  notes/example sides. Field/group limits, UTF-8 and contextual I/O errors remain.

ZIP provenance adds `zip` to `NoteFormat`, while SQLite schema 20 and snapshot
v4 remain unchanged. Earlier readers without that enum variant cannot read ZIP
provenance; downgrading such data is unsupported.

## Source and artifact lineage

Owning checkout: `C:/Users/inthe/IdeaProjects/lexique-interchange-189`, branch
`codex/note-interchange-189`. Inherited integration is
`f92d3ff6b50acd1e601855e361c7df259dda16a3`, which includes all five prior slices
and the parent's test-only V03 diagnostics. It is not the ZIP-enabled source.

The ignored `target/issue-189` evidence root retains `source-pre-native.patch`,
`source-pre-native.zip` and metadata for source tree
`d44b0575200eafdb79a8ddc167ae65a45928afd5`. The release build used its runtime
inputs; the record-equality test assertion was corrected after that build and
before desktop test compilation. That assertion and the native driver are not
embedded executable inputs. Final tested code tree
`ce6af3245365c7975fc94865b33d9283b8f64f35` is retained as
`source-tested.patch`, `source-tested.zip` and `source-tested.json`; it differs
from the earlier retained tree only in the native acceptance driver.

`verify.ps1` and `checks.jsonl` record frontend/resource preparation, fresh
`pnpm desktop:build --no-bundle`, portable packaging, and release desktop test
precompile. Runtime binary `target/release/lexique-desktop.exe` and packaged
`target/issue-189/package/Lexique-0.1.0-windows-x64-portable/Lexique.exe` both have
SHA-256 `5685e6b09ce9c975aed295893bb15b71307b07a96e49423dcfcafec944925c89`.
The supplied preparation binary was not used for acceptance.

## Checks and native cases

Frontend lint/typecheck/build, locked workspace compilation, rustfmt, separately
precompiled backend checks and exact shared all-target Clippy passed:
`cargo clippy --locked -p lexique-core -p lexique-storage -p lexique-dictionary
--all-targets -- -D warnings`. `RUST_TEST_NOCAPTURE=1` retains V03 samples;
100/250 ms assertions and each runner's hard 60-second bound are unchanged.
`desktop-recovery.ps1` performs separate release library precompile immediately
before all eight timed `pnpm test:desktop:lib` partitions in the same MSVC,
pinned Cargo 1.99.1 and Python 3.14.10 environment. All partitions passed,
including all six real ZIP/SQLite tests and the large-CSV regression. Browser
interchange coverage passes all 27 tests; fixtures establish UI behavior only.

Native command, from `apps/desktop`, with the managed Python first on PATH:

```text
node e2e/native-interchange-check.mjs <portable-package> <fresh-executable> <new-destination> <scenario> [source-destination]
```

Every destination is disposable and beneath the owning `target/issue-189`.
The driver uses real WebView2/CDP, real Windows open/save pickers via
`native-input-helper.ps1`, native IPC, generated files and SQLite. No picker or
IPC success substitution was needed. All scenario runs retain the original
hard 60-second limit, perform owned cleanup, and retain per-run driver copies.

| Passing destination | Scenario and observed acceptance |
| --- | --- |
| `native-seed-03` | Core-rich, empty ordinary and unlinked notes; grammar/category, rated cards, pause and selected real Ogg audio |
| `native-csv-03` | Semicolon picker/manual mapping; mixed updates/create/identical/invalid exclusion; exact text; no automatic cards/audio; restart and retained state |
| `native-markdown-01` | Single saved Markdown export/import, exact text and identical skip |
| `native-zip-01` | All/search counts; unique filenames; Deflate/nested/traversal entries; identical dedupe; invalid entry exclusion; corrupt container; cancellation; exact CSV/Markdown/ZIP/CSV text |
| `native-consent-02` | Real missing dictionary resource; explicit consent, cancel/no writes and consent reset; actual ZIP provenance; no new cards/audio tasks |
| `native-enroll-02` | External note editing after restart; explicit single and batch recognition/listening cards; duplicate prevention, progress/pause and selected audio retained |
| `native-backup-02` | Real full backup/restore and controlled restart; external CSV/Markdown/ZIP provenance, ordinary/core/grammar state, categories, learning state and selected audio bytes |

The linked main chain uses `seed-03 → csv-03 → markdown-01 → zip-01 →
consent-01 → enroll-02 → backup-02`. `consent-02` independently extends the same
ZIP source with the no-jobs assertion. Each passing root contains `results.json`,
`lifecycle.jsonl`, real `Lexique-data/data/lexique.sqlite3`, and generated files;
non-seed runs also retain `native-interchange.png`. The seed retains source audio
hashes. Backup retains `interchange.lexique-backup`. Logs and matching driver
copies are adjacent to their roots.

## Retained failures and boundaries

TDD red/green logs are retained for the CSV reader, ZIP export/import, Deflate,
header corruption and browser slice. `zip-corrupt-green.log` retains a hard
runner timeout while recompiling; final separate-precompile recovery passed.
`desktop-precompile.log` retains the invalid whole-record equality assertion;
`desktop-precompile-recovery.log` and `desktop-tests.log` prove its recovery.

Native failed seed, CSV, enrollment and backup attempts remain in numbered roots
and logs. They include incorrect driver argument/card-kind/provenance/default-row
assumptions, plus `native-csv-01`'s real WebView2 renderer failure before the
mapping assertion. No cause is attributed to that renderer failure. Backup
baseline preferences are persisted through actual native commands before the
raw SQLite comparison because restore materializes defaults.

The parent's initial/retry Markdown CI failures and later successful diagnostic
CI are separate inherited evidence; recovery does not prove the DatabaseBusy
cause. No parent V03 diagnostic source was edited here. macOS/Linux native
execution, clean-machine installation and whole-machine offline checks remain
unrun in this ticket. Parent owns final PR review, integration and cleanup.
