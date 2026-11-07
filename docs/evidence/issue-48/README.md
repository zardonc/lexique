# Complete conjugation expression playback through the existing offline speech flow (#48)

2026-11-06: every displayed conjugation expression has its own keyboard-accessible
playback action. This implements [#48](https://github.com/zardonc/lexique/issues/48),
part of [#38](https://github.com/zardonc/lexique/issues/38).

## Complete text and existing speech behavior

The shared conjugation table reuses `SpeechButton` and `RuntimeAdapter.previewSpeech`.
It sends exactly one complete displayed expression, including subjects, reflexive
pronouns and auxiliaries. Spelling and gender/number alternatives are independent
actions; a written slash list is never synthesized. Confirmed source words retained
with an unavailable construction keep their existing warning and speak the displayed
word without inventing an expression.

The existing CPU speech supervisor reads the saved French female/male voice. Playback
reads normal/slow preferences and uses 1.0/0.6 times the same normal-speed Opus audio.
Generation, resource, preference and decoding failures remain visible and retryable.
Machine-speech disclosure and Supertonic 3 are visible; voice/model notices remain
with the admitted speech resources. French expression text and interface-language
controls have separate language attributes. All three languages and themes retain
keyboard operation and 44-pixel playback targets.

Selecting another expression cancels the previous player and any pending playback
response on the same table. Selecting a different verb remounts its players, including when two candidates share
an expression such as `je suis`. A new lookup or page departure also releases audio
and invalidates pending playback responses. Already submitted shared CPU work retains
the established lifecycle; completed unowned audio remains temporary. A late table
response cannot start playback or interrupt a concurrent dictionary request.

The speech-input boundary remains nonempty NFC text, at most 200 Unicode characters
and no control characters. No text is truncated. Native acceptance checks all 1,458
displayed expressions in its five source-backed tables: the maximum is 32 characters,
with no control characters or slash notation. This count covers the exercised tables.
Lookup does not generate a whole table, save learning records, enroll cards, change
review progress or replace selected word audio. Coloring and ordering remain independent.
There are no Rust, IPC, schema, resource, dependency or packaging-contract changes.

## Actual Windows results

[Native results](native-results.json) record actual generation, Opus decoding and
advancing playback time for `je paie`, `je paye`, `je suis allé`, `je suis allée`,
`je me lave`, `lave-toi`, `elles se sont abstenues` and `va-t’en`. Each literal is
checked against the native source table and the published synthesis specification.
The supplied audio bytes are observed through the existing Blob and media APIs;
the production IPC and resource boundaries remain unchanged.

Male/slow and male/normal `je paie` playback uses the same exact bytes and content
version at 0.6 and 1.0. Original speech preferences are explicitly restored afterward.
A pending `nous avions lavé` request completes after navigation while its playback is
cancelled; the existing dictionary `école` playback succeeds and remains the only
player started in that phase.
The native observer loops real media to keep clips active, then verifies that selecting
another expression or replaying one pauses the previous clip. Real playback is forwarded
through the existing media API.

The isolated speech bundle is removed while the app is closed. The real `en ayant parlé`
generation fails visibly, creates no player and records a speech-cache error. After
restoring the bundle and restarting, the same action generates and plays successfully.
All eleven unowned generated content versions have an absolute expiry of generation
completion plus 604,800 seconds, with no vocabulary ownership or pending publication.

Six existing synthetic records, their notes and references, twelve cards and progress
records, current/previous selected audio identities, original speech preferences,
coloring, block order and seven original audio files remain unchanged. New expression
files are temporary additions. The script keeps its hard 60-second execution limit,
uses copied synthetic data and closes every task-owned native process normally.

## Verification

Windows x64; Node.js 24.22.1, pnpm 11.25.0, uv Python 3.14.10 and Rust 1.99.1.
The tests use the already approved desktop boundary: browser adapter fixtures and
actual Windows UI/IPC. Browser checks use a real admitted Opus sample for decoding;
native checks synthesize the requested expressions with the actual admitted resources.

Logs are in `target/issue-48-20261106/`. The executed reader, full report and screenshots
are in `target/issue-48/native-speech-20261106-5/`.

| Command | Result |
| --- | --- |
| `pnpm lint` | Passed |
| `pnpm typecheck` | Passed; final production build also typechecks |
| `pnpm test:e2e conjugation-speech.spec.ts speech-button.spec.ts --workers=2` | 14 passed: 13 expression cases and the existing word case |
| `pnpm test:e2e --workers=2` | 146 passed (1.5 minutes) |
| `pnpm desktop:build --no-bundle` | Passed: production frontend and matching format-6 Windows x64 reader |
| `node --check apps/desktop/e2e/native-conjugation-check.mjs` | Passed |
| Native `--speech` command below | Passed within the hard 60-second limit |
| `pnpm test:docs` | Passed: 120 documents, 716 local links checked; 68 local links skipped by the existing validator |
| `git diff --cached --check` | Passed |

Browser checks set `PLAYWRIGHT_BROWSERS_PATH` to `target/playwright-browsers/`.
Final logs are `lint-final-2.log`, `typecheck-final.log`, `speech-final-2.log`,
`e2e-final-2.log`, `desktop-build-final-2.log` and `native-final-5.log` in the log directory.
The task-owned test server and native application are stopped.

Select uv Python before building and running native checks:

```powershell
$issue48Python = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $issue48Python) + [IO.Path]::PathSeparator + $env:PATH
pnpm desktop:build --no-bundle
New-Item -ItemType Directory -Path target/issue-48 -Force | Out-Null
```

Then run from `apps/desktop` with a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/conjugation-format6-fresh-20261106-2/Lexique-data ../../target/issue-48/native-new --speech
```

The four paths retain the existing harness contract; `--speech` additionally copies
the admitted `speech/` folder beside the supplied `Lexique-data/` fixture. The reader
and dictionary/conjugation resources come from the rebuilt release directory.

## Standards

No Standards findings. The staged changes reuse the existing speech component and
injected runtime, retain readonly props and visible errors, and preserve the native
harness’s 60-second timeout and synthetic-data boundary. No baseline smell warrants
an in-scope refactor.

## Spec

No Spec findings. The staged change satisfies #48’s complete-expression and
separate-alternative playback, shared speech lifecycle, visible/retryable errors,
input validation and learning-data preservation requirements. Browser and real
Windows evidence cover the required behavior; no scope creep found.

Two fresh independent reviews used `git diff --cached d01f9a4b8ea4c157b47e9971463077b17c4c7fd7`,
the user-selected baseline for this uncommitted change. Review totals: Standards 0
findings; Spec 0 findings.

Fresh combined package/offline acceptance remains #49. Other platforms, additional
voice-quality admission and formal releases remain unverified. Backend/resource/CLI
and speech-package suites are unchanged and retain their [#46 validation](../issue-46/README.md).
