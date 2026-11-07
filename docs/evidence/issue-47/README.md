# Dictionary and conjugation navigation with learning data preserved (#47)

2026-11-06: dictionary verb entries open the shared conjugation panel and its
selected table. Return navigation opens the corresponding dictionary entry.
This implements [#47](https://github.com/zardonc/lexique/issues/47), part of
[#38](https://github.com/zardonc/lexique/issues/38), after dependency
[#40](https://github.com/zardonc/lexique/issues/40) closed.

## Verified relationships and navigation

A dictionary entry needs an explicit `v` grammar tag and a complete lemma
match using #44's NFC, case, apostrophe and whitespace comparison. Accents and
ordinary/pronominal identities remain distinct. A noun such as `été`, an
unmatched verb phrase, or an absent pronominal entry cannot acquire a link
through an inferred base verb.

Dedicated lookup retains every candidate. Returning through a dictionary
entry and reopening conjugation rereads the original query, preserving its
selected verb and other candidates. A new dictionary search clears that
context. Query input and selection survive page changes during this run;
coloring and block order continue to use their independent saved preferences.

Return links use the originating dictionary spelling when one was chosen.
Dedicated lookup checks the lemma's straight/curly apostrophe spellings through
the existing exact dictionary adapter. Multiple verified entries require a
choice. Each choice retains its exact dictionary response and source ordinal;
case/apostrophe-distinct vocabulary identities remain separate. Missing
relationships and resource failures are explicit. Link reads are cancelled
when their view leaves, so late responses cannot open a destination.

All page and link transitions use the existing leave guard. Staying, discarding,
saving successfully, failed saves, duplicate records and conflicting revisions
retain the established rules. Destination headings receive keyboard focus.
The existing three languages and themes have native and browser coverage.

## Actual Windows results

[Native results](native-results.json) cover keyboard navigation in both
directions, the ambiguous `suis` candidates, selected `être` and query retention,
the typographic query ` S'ADAPTER ` and dictionary entry `s’adapter`, and honest
unavailable links for `se laver`, the noun `été`, and `faire l’amour`.
Both resources are separately removed, replaced with corrupt bytes, restored,
and reread. Errors stay visible and the restored resources work again.

Acceptance copies the six-record synthetic learning fixture from #24 and
explicitly saves one additional `laver` record without enrolling it. Subsequent
browsing and resource recovery preserve all seven records, their references,
twelve cards and progress records, speech preferences, selected audio, and
seven audio files. A rejecting update trigger proves failed save-and-leave
retains input and blocks navigation. Discard leaves stored content unchanged.
An explicit successful save changes only that record's notes, revision and
update time; native duplicate and stale-revision requests cannot overwrite it.

A normal close and restart restore the edited notes, unchanged learning/audio
state, `main` coloring, and the custom order with `PQP` after `J`. Browser
fixtures additionally cover multiple dictionary entries, distinct apostrophe
and case spellings, edit conflicts, and delayed responses. Those symbolic
dictionary entries do not extend the admitted native dictionary.

## Verification

Windows x64; uv Python 3.14.10, Node.js 24.22.1, pnpm 11.25.0 and Rust 1.99.1.
Tests use the already approved desktop boundary: browser adapter fixtures and
actual Windows IPC/UI. No Rust, IPC, schema, resource, dependency or packaging
contract changes are included.

| Command / boundary | Result |
| --- | --- |
| `pnpm lint`, `pnpm typecheck` | PASS |
| `pnpm desktop:build --no-bundle` | PASS; production frontend and matching format-6 Windows x64 reader |
| `pnpm test:e2e -- conjugation-navigation.spec.ts --workers=2` | PASS; 15 navigation checks |
| `pnpm test:e2e -- --workers=2` | PASS; all 133 browser tests |
| `node --check apps/desktop/e2e/native-conjugation-check.mjs` | PASS |
| Windows `native-conjugation-check.mjs --navigation` | PASS; both directions, failures, edit guards, restart and learning/audio preservation; hard 60-second execution limit |
| Obscura browser preview | PASS; lookup explicitly reports unavailable native dictionary access |
| `pnpm test:docs` | PASS; 119 Markdown documents and 711 local links |
| `git diff --cached --check`, `git diff --check` | PASS |
| Standards review against `ce28d1e` | PASS; 0 findings |
| Spec review against #47 | PASS; 0 findings |
| Backend/resource/CLI/speech-package suites | Not rerun; their implementations are unchanged, with prior validation retained in [#46](../issue-46/README.md) |

Logs are in `target/issue-47-20261106/`. The executed reader, native report and
screenshots are in `target/issue-47/native-navigation-20261106-2/`.

From the repository root, select uv Python before native fault checks:

```powershell
$issue47Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue47Python) + [IO.Path]::PathSeparator + $env:PATH
pnpm desktop:build --no-bundle
New-Item -ItemType Directory -Path target/issue-47 -Force | Out-Null
```

Then run from `apps/desktop` with a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/conjugation-format6-fresh-20261106-2/Lexique-data ../../target/issue-47/native-new --navigation
```

The existing harness's default checks remain available; `--navigation` runs
this slice within its unchanged hard timeout. It uses copied synthetic data.
Fresh combined package/offline acceptance remains #49. Individual conjugation
playback remains #48; other platforms and formal releases remain unverified.
