# Single grammar Markdown verification (#216)

The single-record exchange uses the saved grammar snapshot and the existing
preview and atomic import session. The [template and field semantics](grammar-markdown-exchange.md)
describe the content format.

The single-record save dialog uses the saved title and exchange identity for its
Windows-compatible suggested filename. Two same-title records were exported to
distinct files with a newly built packaged executable; the actual Win32 defaults,
source/binary lineage and full-suite results are recorded in the
[#212 review verification](grammar-exchange-review-verification.md).

Windows x64 native acceptance used the executable built from commit
`09769da0f9055942c81fd890ce6cb29072bdcb70` (SHA-256
`ADBEA55D846DB06BD7E6DCA1A6168611094A4A41F3C413BF9F4914864AD2E780`).
The native vocabulary Markdown compatibility driver was added at `b0fb692`;
it uses the same executable. All final scenarios used the resource package
created by that build and isolated portable learning databases.

| Scenario | Observed behavior |
| --- | --- |
| Grammar Markdown | Real import and save pickers; raw Unicode, whitespace, multiline text and backticks; stable identity after file and title changes; omitted and explicitly empty body; blank pair preserves the card; cancelled save; failed SQLite update rolls back; failed start remains visible and paused. |
| Validation and cancellation | Wrong marker, missing title and incomplete question/answer produce filename and reason; actual import-picker cancellation leaves learning data intact. |
| Reopen | A reopened copy of the portable learning database retains the identity, edited content and paused card, which remains outside the queue. |
| Vocabulary compatibility | Real vocabulary Markdown export and identical import preserve vocabulary and grammar learning state. A separate CSV scenario also passes. |

The browser checks cover the Markdown error and selected-update interfaces,
related grammar CSV and vocabulary Markdown flows, and the browser capability
boundary. They do not establish native persistence. The Rust backend and desktop
library suites, including the existing backup and restore tests, passed with
storage precompilation separate from the unchanged 60-second execution limit.
Frontend lint, typecheck and production build passed. Scenario cleanup reports
show no owned host, worker or helper processes.

Full command logs, failed attempts, native files, screenshots, SQLite readback,
source and binary identity, and cleanup records are archived in the local
`../evidence/ticket-216` directory alongside the integration worktrees.
The WebView was set offline for these checks; whole-machine offline operation,
other platforms, clean-machine installation and release acceptance remain
unverified.
