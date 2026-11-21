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

Historical execution checkout: `C:/Users/inthe/IdeaProjects/lexique-interchange-189`, branch
`codex/note-interchange-189`. Inherited integration is
`f92d3ff6b50acd1e601855e361c7df259dda16a3`, which includes all five prior slices
and the parent's test-only V03 diagnostics. It is not the ZIP-enabled source.

Prior evidence is retained at
`C:/Users/inthe/IdeaProjects/lexique-interchange-183/target/issue-183/worker-evidence/ticket-189`.
The `target/issue-189` paths below identify the original execution root; their
relative artifact paths now resolve within this archived root. Original paths
inside manifests remain execution history.

The archive retains `source-pre-native.patch`,
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

The parent's inherited f92d3ff diagnostic run `44824859852` passed all 11 jobs;
the duplicate PR run `44824853320` failed V03 before rating, with a 391.6065 ms
primary transaction and 252.9793 ms competing writer. Its cause remains unknown;
the export fixture convention change does not explain that failure. Diagnostics
are recorded in
`C:/Users/inthe/IdeaProjects/lexique-interchange-183-context/parent-v03-diagnostics.md`.
At integration `120ff887ea8c4fdaaf50b38d9f3952e9c449a2bd`, PR run `44832581524`
and push run `44832574034` both passed all 22 jobs without retry. The parent's
436 browser tests, lint, typecheck and build also passed at that integration.
The PR watcher's HTTP 502 annotation error was a client failure; authoritative
job results were successful. These are inherited results, separate from the
review-fix source checks below. No parent V03 diagnostic source was edited here.
macOS/Linux native
execution, clean-machine installation and whole-machine offline checks remain
unrun in this ticket. Parent owns final PR review, integration and cleanup.

## Review fixes at integration 120ff887

The shared Markdown filename helper prefixes Windows device basenames before
the first dot, case-insensitively, including numbered COM/LPT names and their
documented superscript digits. `CON.txt` now exports as `_CON.txt-<id>.md` in
both single Markdown and ZIP; `CON-<id>.md` remains usable without a prefix.
Body headwords and text, saved IDs, snapshot scope/count/order and uniqueness
are preserved. The 25-row export-completeness fixture uses parameterized
inserts in one transaction, committed before export.

Execution checkout:
`C:/Users/inthe/IdeaProjects/lexique-interchange-review-fixes`.
Evidence is under its ignored `target/review-fixes`; the parent's final archive
destination is
`C:/Users/inthe/IdeaProjects/lexique-interchange-183/target/issue-183/worker-evidence/review-fixes`.
Original execution paths in manifests remain historical after archival.

Fresh release source is `acf89234486c6d5bee32bf46d468aa4cd1e62422`, retained
in `source-built.{zip,json}`. Tested source is
`31bb1043d594ccd2f4e8ba4b099b3fe6e8271b9a`, retained in
`source-tested.{zip,json}`; `built-to-tested.patch` contains only one test-only
ZIP entry-name ownership line. Runtime and native driver inputs are identical.
The fresh executable, package and passing native executable share SHA-256
`7e61e3a6cf5a0cd15a381480faf98880fb2c12918b2f2db9c011a841aa898745`.

`verify.ps1` and `checks.jsonl` retain successful lint/typecheck/build, locked
workspace compilation, shared all-target Clippy and 218 separately precompiled
backend checks. `desktop-precompile-recovery.log` precedes
`desktop-tests-recovery.log` in the same pinned Cargo/MSVC/Python environment,
with no intervening source/resource preparation. All eight original 60-second
desktop partitions passed: 144 tests and two deliberately ignored child entry
points invoked by their interruption parents. Dotted device names and case
variants exercise actual single-file readback and ordinary-tool ZIP extraction.
Rust formatting, documentation links and diff checks passed.

`native-green-02` passed the `device-names` scenario using the existing native
driver/helper, six real Windows pickers and actual files/SQLite in 21.388 s,
with the hard 60-second limit unchanged. It observed `_CON.txt-1.md` and
`_con.TXT-2.md` as the picker defaults and saved both files. Four exported ZIP
entries extracted on Windows with exact body identity/text, unique names,
all/search counts/order and unchanged saved records on reimport.
`native-manifest.json`, per-run driver/helper copies, `results.json`, lifecycle
logs, screenshot, generated files and copied single-file regression artifacts
retain the evidence. `owned-processes-final.json` is empty.

`source-red-native.{zip,patch,json}` and `native-red-02` retain the real picker
default regression. Earlier probe logs, the initial native locator failure and
test compilation failure remain retained. `native-green-01` records a WebView2
renderer access violation before export assertions; its cause remains unknown.
The fresh native recovery establishes the affected behavior without attributing
that renderer failure. The unchanged frontend's full browser suite was not
rerun; other-platform, clean-machine and whole-machine offline acceptance
remain unrun.
