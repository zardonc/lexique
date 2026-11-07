# CLI validation latency and offline acceptance

2026-11-07: the public conjugation CLI retains full validation while reusing
sorted syntax slots in its existing per-load cache and parsing each verb's
meanings once. All six follow-up processes fit the original 1,700 ms budget.
The generated database is byte-identical to the preserved #49 speech package,
for which the user reported that offline acceptance passed.

## Measurements

The [machine-readable results](cli-latency.json) retain the failing baseline
at `d0dd143` and the current measurements. Both use the accepted format-6
database and the full public CLI query `elles se sont abstenues`, including
source validation and JSON serialization. Each process must return exactly
`s’abstenir`.

| Measure | Baseline | Optimized CLI |
| --- | ---: | ---: |
| First process | 3,871.236 ms | 965.009 ms |
| Subsequent-process median | 1,206.244 ms | 947.487 ms |
| Subsequent-process maximum | 1,263.053 ms | 980.102 ms |
| Six processes within 1,700 ms | No | Yes |
| Profiled resource validation | 1,421.207 ms | 952.921 ms |
| Profiled syntax slot sorts | 19,782 | 125 |
| Profiled syntax sorting time | 238.108 ms | 0.937 ms |
| Profiled JSON loads | 32,949 | 22,168 |

The final check against `target/desktop-conjugation-v6-cli/conjugation.sqlite3`
also passed all six processes: 1,100.073 ms first, 936.198 ms subsequent median
and 1,096.710 ms subsequent maximum.

The profile identifies repeated source reconstruction as a material CLI cost.
Sorting now occurs for each cached syntax pattern and manual entry rather than
each of the 19,782 source frames. Each of the 10,781 meaning payloads is parsed
once, then the same value is validated and returned. Provenance, source hashes,
archive/license checks, SQLite integrity/counts, every verb's meaning validation
and candidate metadata validation remain active. The cache lives within one
source load; it cannot retain data across queries or resource replacements.

The [timing command](../conjugation-storage/measure_cli.py) writes every sample
and exits 1 if any exceeds 1,700 ms. Its baseline invocation failed and its
optimized invocation passed. Original preparation measurements remain in
[measurements.json](measurements.json).

## Integration and checks

`pnpm conjugation:prepare` uses the new `target/desktop-conjugation-v6-cli/`
directory. Native preparation, release copying, Tauri resource configuration,
portable validation and test fixtures use that same directory. Earlier
generated directories and the accepted ZIP remain intact. A new build retains
format 6, 11,005,952 database bytes, 170 models, 24,307 stem references,
1,451,539 cells and 10,781 identities. Its database bytes equal the accepted
resource; the baseline and optimized query responses also compare equal.

| Check | Result |
| --- | --- |
| Baseline / optimized / final CLI budget commands | FAIL / PASS / PASS; every sample retained |
| Resource rebuild and baseline-query response comparison | PASS: byte-identical database and equal responses |
| `pnpm test:conjugation` | PASS: 36 public CLI tests, 324.138 seconds |
| `pnpm rust:fmt`; `pnpm rust:check` | PASS: formatting and locked workspace compilation |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS: 143 backend tests, separate precompilation and hard 60-second execution limit |
| `pnpm desktop:build --no-bundle` | PASS: production frontend and matching Windows x64 reader |
| `pnpm test:build-args`; portable unit suite | PASS: 2 build-argument and 8 package tests |
| `pnpm test:speech:portable`, deep task TEMP | FAIL: Windows path-length limit in temporary speech copy |
| `pnpm test:speech:portable`, short task TEMP | PASS: all 3 tests, including positive TTS packaging; no skips |
| `pnpm test:docs`; `git diff --cached --check` | PASS: 123 documents, links and staged whitespace |

```powershell
$taskPython = (& uv python find).Trim()
$env:PATH = (Split-Path -Parent $taskPython) + ';' + $env:PATH
& $taskPython -X utf8 docs/evidence/conjugation-storage/measure_cli.py target/issue-49/package-extracted-20261107/Lexique-0.1.0-windows-x64-portable-speech/conjugation/conjugation.sqlite3 target/issue-49/cli-diagnosis-20261107/budget-after.json
pnpm conjugation:prepare
pnpm test:conjugation
pnpm rust:fmt
pnpm rust:check
pnpm rust:test:build
pnpm test:rust
pnpm desktop:build --no-bundle
pnpm test:build-args
python -X utf8 -m unittest discover -s scripts -p test_package_portable.py -v
$env:LEXIQUE_SPEECH_BUNDLE = '.tooling/issue-14/supertonic-evidence-v16-revised'
$env:LEXIQUE_OPUS_ENCODER = '.tooling/issue-15-opus/speech-stage/opusenc.dll'
$env:LEXIQUE_OPUS_SOURCES = '.tooling/issue-15-opus'
$env:TEMP = (Join-Path (Get-Location) 'target/i49tts-1107')
$env:TMP = $env:TEMP
pnpm test:speech:portable
pnpm test:docs
git diff --cached --check
```

## Review basis

Independent Standards and Spec reviews cover the committed #49 preparation
and staged converter/build follow-up against the merge-base of `88d628e`
and `HEAD`. The complete comparison is:

```powershell
$taskBase = (git merge-base 88d628e HEAD).Trim()
git diff --cached $taskBase
git log 88d628e..HEAD --oneline
```

## Standards

PASS: zero documented violations. Current callers only read the sorted cached
slot lists or copy them into sets. Validation, separate resource preparation,
matching build/package paths, learning-data preservation and the hard
60-second execution limits remain intact.

One optional P3 judgement: possible duplicated code in the
[native acceptance harness](../../../apps/desktop/e2e/native-conjugation-check.mjs).
The fresh-package flow at lines 716–722 repeats the speech flow's slow/normal
playback, audio version/byte equality and preference restoration at lines
683–689, including:

```javascript
const normal = await playNativeExpression(page, 'je paie');
assert.equal(normal.version, slow.version);
```

A small shared assertion helper could keep these expectations together.
This is nonblocking maintenance feedback; no correctness risk was identified.

## Spec

PASS: zero actionable findings. No missing or partial requirement, scope creep
or incorrect behavior was identified. The CLI preserves sorted slot contents
and full source validation, and validates and returns each meaning from one
parse. Preparation, build copying, Tauri packaging and fixtures consistently
use the new directory. The unchanged budget fails on any miss. Evidence ties
the user's offline pass to the preserved ZIP and records the rebuilt tooling
reader separately.

Standards: 0 documented violations, 1 optional P3 duplication (worst issue).
Spec: 0 findings, no worst issue. Both reviews were read-only and did not
repeat tests or native/offline acceptance.

## Offline acceptance and limits

The user reported "offline test passed" for the preserved
[speech package](../../../target/issue-49/offline-package-20261107/Lexique-0.1.0-windows-x64-portable-speech.zip).
Its identity is unchanged; the rebuilt tooling reader is recorded separately.
The user did not supply machine/runtime versions, network-block details or
per-step logs. Existing actual Windows checks cover the accepted reader's
generation, playback, display preferences, navigation and learning data.

These are fresh/subsequent process timings on Windows x64 with Python 3.14.10;
OS caches were not flushed. The earlier first-process spike is larger than the
saved CPU work, so these passing samples do not identify all startup variability
or establish a cold-disk bound. Frontend lint/typecheck, browser/native flows and
dictionary-only tests retain the original #49 evidence; they were not repeated
for this converter/path change. Positive speech packaging needs a short
task-owned TEMP because Windows rejects the nested runtime tree under the
longer test path. Other-platform execution, missing-WebView2
clean-machine setup and M5 distribution remain separate gates.
