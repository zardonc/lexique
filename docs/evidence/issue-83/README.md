# D2 content batch 003 (#83)

[Issue #83](https://github.com/zardonc/lexique/issues/83) completes all fifty exact
frozen word/POS groups in rows 0101–0150: **50 groups / 200 reviewed fields**,
with 49 everyday uses and the approved golf exception. No group is reused.
Each selected use has a French definition, applicable IPA, English meaning and
Chinese meaning. The [source audit](source-research.md) fixes the selected
versions and spans; the [independent review](quality-review.md) checks all four
fields and their construction, domain and redistribution boundaries.

## Approved scope amendment

On 2026-11-17 (America/Vancouver; 2026-11-18 UTC), the project owner answered
**“Approve the specific golf exception”** to the proposal for frozen row 128,
`par/NOM`. The approved use is the conventional number of strokes required to
complete a single golf hole, from
[Wiktionnaire revision 39871373](https://fr.wiktionary.org/w/index.php?title=par&oldid=39871373),
definition line 295 and IPA line 294.

The batch therefore admits **49 everyday uses and one specialist use**.
The golf domain, masculine noun and single-hole standard-stroke-count restriction
remain visible in the meanings and usage. The actual achieved score,
Georgian-letter name and preposition are excluded from this noun sense.
The exact frozen row, headword, POS and permanent denominator remain unchanged.
This decision does not extend the earlier `ça/NOM` or `quoi/ADJ` exceptions.
Everyday coverage and approved exception acceptance must be reported separately.

## Content and redistribution

The existing conversion, read-only native core lookup, UI, trusted vocabulary
save and frozen-digest paths are reused. Frozen `ONO` is accepted by the shared
core and frontend validators, with interjection/onomatopoeia labels in English,
French and Chinese. No runtime dependency or command is added. The development
resource is `core-83-v1`, resource format 1, with **154 groups / 616 fields**.
The three-language preview reports those 154 accepted groups.

The [curation](../../../resources/core-pilot-79/curated-inputs.json),
[independent field and usage review](../../../resources/core-pilot-79/field-review.json)
and [selected source context](../../../resources/core-pilot-79/selected-source-context.json)
retain the fixed versions, spans, contributors, rights and actual modifications.
All 200 new fields and 50 usages passed independent review. The 37 fixed captures
include the supplemented `lui` source and the specific `son` consonant boundary.
New Alexandria/CFC and Reverso context retains section markers and URLs without
their prose. Contributor histories, source versions, selected excerpts, actual
CC BY-SA 4.0 legal text, Lexique membership notices, prior import credits and the
MIT converter remain in the package; the
[resource notices](../../../resources/core-pilot-79/NOTICES.md) own these obligations.

`bon/ADV` remains literal `sentir bon`; `petit/ADV` remains `écrire petit`.
Frozen `pourquoi/CON` and `comment/CON` explicitly describe indirect-subordination
uses despite the sources' adverb terminology. `même/PRO:ind` uses the clean
`le même` pronoun source. `son/ADJ:pos` retains its restricted possessive
construction. Golf `par/NOM` retains the single-hole standard stroke count.

| Coverage | Count |
| --- | ---: |
| Frozen target groups / headwords | 4,087 / 3,773 |
| Accepted groups / fields | 154 / 616 |
| Unaccepted groups | 3,933 |
| Everyday-complete / everyday-incomplete groups | 152 / 3,935 |
| Approved specialist groups | 2 (`ça/NOM`, `par/NOM`) |
| Row 87 annotation exception | 1, with zero admitted fields |

## Preservation

All 104 prior groups, 416 fields and their provenance, 80 source records,
416 field reviews, 104 usage reviews and six query aliases remain unchanged.
The original review order, prior source context and spelling relationships are
preserved. All **104 original SQLite sense payloads are byte-identical**.
Row 87, `quoi/ADJ`, keeps its original incomplete coverage records and zero
admitted fields; its separate approved annotation exception does not add content.

## Verification

Checks ran on 2026-11-18 UTC, Windows x64, in
`C:\Users\inthe\IdeaProjects\lexique-83`, branch `codex/d2-content-003-83`,
based on `13b7fec87ec2c5a99b10be2cd03dc20ed0b5c978` plus the batch changes.
Complete logs and artifacts are retained under `target/issue-83/`; the reviewed
diff is `review-final.diff`. PowerShell 7, pnpm 11.25.0, uv-resolved Python
3.14.10 and pinned Rust/Cargo 1.99.1 with MSVC were used.

| Check | Result and retained evidence |
| --- | --- |
| `pnpm core:prepare` | Passed; exact membership, 50 batch groups / 200 fields and fixed global denominator; `core-prepare-delivery.log` |
| `pnpm test:core` | 11 passed; field/usage acceptance, specialist exceptions and reproducible conversion; `core-tests-delivery.log` |
| Preservation comparison | Passed; all 104 prior payloads and prior source/review bindings preserved; `preservation.json`, `preservation-delivery.log` |
| `pnpm test:tooling` | Passed; `tooling-tests.log` |
| `pnpm lint`, `pnpm typecheck`, `pnpm build` | Passed; `lint-delivery.log`, `typecheck-delivery.log`, `build-delivery.log` |
| Rust formatting and compilation | Passed; `rust-fmt-final.log`, `rust-check-final.log` |
| `pnpm rust:test:build`, then `pnpm test:rust` | Passed; 194 backend tests, precompilation separate, hard 60-second execution limit retained; `rust-test-build-delivery.log`, `rust-tests-delivery.log` |
| Localized browser coverage check | 3 passed; `coverage-browser-delivery.log` |
| `pnpm test:e2e` | 391 passed with build/native processes stopped; unchanged test timeouts; `e2e-delivery-isolated.log` |
| `node scripts/build-desktop.mjs --no-bundle` | Passed; fresh release executable, locked Cargo, two existing dead-code warnings; `native-build-delivery.log` |
| `python -B -X utf8 scripts/package_portable.py --output-dir target/issue-83/packages-delivery` | Passed; 183 verified files; `package-delivery.log`, `package-delivery-metadata.json` |
| `pnpm test:docs`, `git diff --cached --check` | Passed; 178 documents / 1,427 local links, including all three batch evidence files; `docs-delivery.log` |

Additional retained browser execution records are `e2e-delivery.log` (389 passed,
two 30-second conjugation timeouts during concurrent compilation) and
`e2e-timeout-focused.log` (both affected tests passed). The timeout cause was not
independently established; no test timeout or assertion was relaxed.

The prepared SQLite is **741,376 bytes**, SHA-256
`fa2b8744ec1021faba76a54b68d0de032ddb04896b035f77cfe6979c52161227`.
Its report accepts all fifty #83 groups and their 200 fields within the approved
scope. Global everyday completeness remains incomplete at 152 / 4,087 groups.

The portable ZIP is **35,348,235 bytes**, SHA-256
`c99cd4e3c9138665f03898dc65264fab35eb9367db44208a093a4102d42d499f`.
The extracted package contains **183 files / 71,544,988 bytes**. The freshly
built executable is **29,781,504 bytes**, SHA-256
`8903453007ff1e0a330ff6c4e848f6e55e507f445d13cdce7c4302f32aca7613`;
the packaged executable has the same hash.

## Native lookup, save and upgrade

From `apps/desktop`, with pinned Python on `PATH`:

```powershell
node e2e/native-d2-pilot-check.mjs no-speech ../../target/issue-83/packages-delivery/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../resources/core-pilot-79/curated-inputs.json ../../target/issue-83/native/no-speech-02
node e2e/native-d2-pilot-check.mjs upgrade ../../target/issue-83/packages-delivery/Lexique-0.1.0-windows-x64-portable ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../resources/core-pilot-79/curated-inputs.json ../../target/issue-83/native/upgrade-02 ../../target/issue-83/upgrade-seed
```

Both scenarios passed using the fresh executable above. The native lookup/save
run checks every admitted group, including all fifty #83 groups, and saves 99
distinct headwords through the trusted provider. Normal restart preserves the
selected fields and provenance. Saved `bon` covers its four separate POS groups;
`bon`, `petit` and `par` construction restrictions remain readable. English,
French and Chinese `ONO` labels and the prior `mon/ADJ:pos` labels pass. The
retained `bon` locale screenshots and `petit`/`par` lookup screenshots were
visually inspected. The model-absent scenario exposes the speech failure and
rejects client-supplied digests and references.

The upgrade starts from the closed, copied #82 profile with **65 records** and
two valid current/previous Opus fixture versions. Lookup exposes `core-83-v1`
while older references, original digests, notes, examples, cards, FSRS state,
committed save replay and audio relationships remain preserved. Cached audio
bytes are readable. New `bon` uses the new core; the final profile has 66 records.
The original #82 fixture and the upgrade seed remain unchanged by execution.

`native/no-speech-02/` and `native/upgrade-02/` contain `results.json`,
`run-metadata.json`, lifecycle and cleanup records. Both report zero external
WebView requests and empty owned-process cleanup sets. The native harness retains
its hard 60-second limit. External WebView routes are denied after startup;
whole-machine network denial was not run.

## Limitations

The permanent scope is 4,087 groups / 3,773 headwords. The complete D2 resource,
CEFR, learner acceptance, whole-machine offline execution and other-platform
hands-on release gates remain open.
