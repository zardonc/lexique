# M3 conjugation ticket breakdown

Source: [specification #38](https://github.com/zardonc/lexique/issues/38), [local specification](m3-conjugation-spec.md), and [confirmed scope](m3-conjugation-scope.md). Status: user-approved and published as 11 sub-issues, with 16 native blocking relationships. Publication read-back verified the issue bodies, titles, open states, labels, parent relationships, and blocking edges.

T01–T11 are breakdown identifiers; the delivery table maps them to GitHub issues. Each issue is labelled `ready-for-agent`, includes its acceptance criteria and the shared requirements below, and references real issue numbers. Blocking edges use native relationships.

## Delivery order

The [compact storage contract](conjugation-storage.md) precedes #44. Shared simple-form models and suffix/stem candidate lookup replace materialized tables; #38 records full format-5 equivalence and preservation requirements. #44 applies normalization at candidate lookup and complete-expression verification, #45 keeps linguistic ending spans independent of storage suffixes, and #49 packages the compact resource and measures current costs. #46–#48 retain their existing lookup/table interface and dependencies.

| Slice | GitHub issue | Title | Blocked by | Independently observable result |
| --- | --- | --- | --- | --- |
| T01 | [#39](https://github.com/zardonc/lexique/issues/39) | Validate ordinary verb groups and auxiliary metadata through the resource CLI | None | Reproducible real-resource queries return justified metadata and preserve all source matches. |
| T02 | [#40](https://github.com/zardonc/lexique/issues/40) | Search simple forms and browse real conjugation tables in the desktop | #39 | Native lookup, candidate selection, source tables, and every match highlight work. |
| T03 | [#41](https://github.com/zardonc/lexique/issues/41) | Complete ordinary active tables and reverse lookup across all 23 blocks | #40 | Ordinary compound and nonfinite expressions resolve to independently verified complete tables. |
| T04 | [#42](https://github.com/zardonc/lexique/issues/42) | Show licensed offline English meanings in verb headers and candidate choices | #40 | Present and missing English meanings are distinguishable without hiding usable verbs. |
| T05 | [#43](https://github.com/zardonc/lexique/issues/43) | Look up pronominal expressions and display validated reflexive agreement alternatives | #41 | Ordinary and pronominal uses retain distinct identities, pronouns, auxiliaries, and agreement. |
| T06 | [#44](https://github.com/zardonc/lexique/issues/44) | Accept typographic query variations while preserving accents and complete expression matches | #40 | Equivalent typing resolves consistently; unrelated tokens and accent changes do not create matches. |
| T07 | [#45](https://github.com/zardonc/lexique/issues/45) | Add three verb coloring modes with validated endings and restart persistence | #41 | Exact spans change by mode, group-three verbs stay uncolored in mode 3, and selection persists. |
| T08 | [#46](https://github.com/zardonc/lexique/issues/46) | Reorder tense blocks across verbs and restore the order after restarting | #40 | Keyboard-accessible ordering and reset persist without changing priority identity. |
| T09 | [#47](https://github.com/zardonc/lexique/issues/47) | Navigate between dictionary entries and the shared conjugation view without changing learning records | #40 | Both entry points open the same verb relationship and preserve editing/save behavior. |
| T10 | [#48](https://github.com/zardonc/lexique/issues/48) | Play each complete conjugated expression through the existing offline speech flow | #40 | Each displayed alternative has real on-demand playback with existing voice/speed behavior. |
| T11 | [#49](https://github.com/zardonc/lexique/issues/49) | Verify the complete conjugation flow in a fresh offline Windows package | #42, #43, #44, #45, #46, #47, #48 | The combined flow passes real first-launch, playback, restart, resource-coverage, and preservation checks. |

T01 is a complete public CLI capability at the user-approved resource test boundary. Subsequent feature tickets include resource changes, native access, UI, and boundary tests as needed; there are no separate generic database, IPC, UI, or test-only implementation tickets. Existing boundaries support this sequence without a prefactoring ticket.

T02 establishes the 23 block identities and honest missing-data states; T03 supplies the ordinary active constructions. The independently useful first desktop slice therefore does not claim complete M3 coverage. English meanings, tolerant queries, ordering, dictionary navigation, and speech can start after T02. Coloring waits for T03's actual compound expressions. Pronominal support also needs the validated ordinary construction rules. Speech consumes the addressable expression alternatives already established by T02, so it does not wait for every later paradigm extension. T11 verifies those combinations together.

## Shared acceptance requirements

Every ticket inherits #38's scope and the resource/learning-data and licensing ADRs. Its published body links #38; the criteria below describe its specific completion boundary.

- Preserve existing vocabulary identities, notes, review progress, selected word audio, and pre-existing edits. Lookup, browsing, and playback do not implicitly save records or enroll cards. Resources remain replaceable independently of learning data.
- Reuse the existing injected runtime adapter, validated structured IPC, resource-reading conventions, application preferences, and speech supervisor. Keep errors visible and render resource content as text. Validate supported inputs at entry points; native callers cannot choose arbitrary resource/model paths.
- Any new source or unfamiliar generation design needs compatible upstream evidence, pinned provenance, separate code/data license review, required notices, and independently justified linguistic expectations before use. An unresolved admission or correctness gate remains open with concrete evidence.
- UI work includes keyboard operation, non-color cues, and the existing three languages and themes. Browser fixtures establish presentation behavior; actual Windows execution establishes native resource access, persistence, or playback.
- Test through the two approved public boundaries: desktop user flow and the resource build/query CLI using the product's admitted representation. Extend existing checks rather than exposing a new API for tests. Record commands and passed/failed/skipped results. Run applicable frontend/Rust/resource/browser/native checks; precompile backend tests separately and keep their hard 60-second execution timeout.
- Update readiness evidence when a claim changes. Each capability is verified when it lands; T11 supplies combined fresh-package acceptance rather than deferring feature tests to the end.

## T01: validate ordinary verb groups and auxiliary metadata through the resource CLI

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

An implementer can reproducibly build and query the real conjugation resource to inspect ordinary verb identity, traditional group, applicable auxiliaries, usage labels, and original source forms. This extends #4's source validation into the metadata needed for generation, with explicit evidence for what is supported or still unknown.

### Acceptance criteria

- [ ] Reuse #4's pinned Lefff baseline; inspect compatible maintained upstream designs before selecting supplemental data/rules. Pin admitted inputs and conversion versions, verify integrity, retain applicable licenses/notices and modification provenance, and separate code permission from data permission.
- [ ] The public CLI exposes first-/second-/third-group classification and ordinary auxiliary/use metadata with traceable support. Cover regular -er, finir/finissons, aller, partir, avoir, être, a multiple-auxiliary verb, and a defective verb with independent expectations; suffix alone is insufficient.
- [ ] Every usable admitted verb remains queryable without a frequency cap. Report metadata coverage and exclusions across the resource, distinguishing justified values, grammatical inapplicability, and unconfirmed/missing data. Unknown dimensions are not expanded into invented values.
- [ ] Preserve source spelling variants, source references, and all original matches, including sommes across sommer/être and suis across suivre/être. Retain justified evidence for corrections or malformed-row exclusions.
- [ ] Reproducible build/query checks reject missing, corrupt, truncated, incompatible, and incomplete required inputs while preserving a previously valid output. Failures are distinguishable from an empty query result.
- [ ] Record original/generated/index sizes and cold/warm query measurements on a declared environment; derive initial implementation budgets from measurements. English glosses, complete compound generation, pronominal generation, and ending spans remain the responsibility of their feature tickets.

### Blocked by

None (can start immediately).

## T02: search simple forms and browse real conjugation tables in the desktop

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner uses the conjugation navigation entry to search a source-supported simple form, select its verb when ambiguous, and inspect a real native table with every corresponding cell highlighted.

### Acceptance criteria

- [ ] Load T01's real resource through the existing desktop runtime boundary with an application-controlled resource location, validated requests/responses, read-only parameterized access, and reproducible preparation/bundling for this slice. Browser preview reports unavailable native capability honestly.
- [ ] A sole candidate opens directly; multiple candidates remain selectable in stable French-lemma order with distinguishable use labels. Candidate counts and cell-match counts differ correctly, and other candidates remain accessible after selection.
- [ ] sommes and suis demonstrate every source interpretation. All corresponding cells, including nonpriority cells, receive identifiable highlighting; shared tokens do not highlight unrelated cells. Preserve all spelling alternatives in their cell with individually addressable complete display expressions.
- [ ] Display traditional group and auxiliary/use metadata. Missing English meanings are explicit until T04; missing conjugation data is distinct from verified defective/inapplicable forms.
- [ ] Render 23 distinct blocks using #38's matrix, with the exact seven priority blocks first and remaining family/entry order. Mark priorities independently of matches. Source-supported simple forms have appropriate subjects and imperative/nonfinite presentation; constructions not yet validated are explicitly unconfirmed, not fabricated. Explain the historical second conditional-past name without duplicate forms.
- [ ] Preserve the query for no-match, invalid input, missing/corrupt resource, and operational failures, with distinct visible outcomes. Native resource replacement does not rebuild or modify learning data.
- [ ] Browser user-flow checks cover selection, counts, variants, ordering, and failures; a real Windows native check exercises actual resource IPC and table results. Verify lookup/browsing preserve existing learning records and selected audio.

### Blocked by

T01.

## T03: complete ordinary active tables and reverse lookup across all 23 blocks

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner looks up an ordinary compound expression such as avons aimé, optionally with a subject, and sees its matching cells in the full agreed ordinary active paradigm, including literary and nonfinite constructions.

### Acceptance criteria

- [ ] Admit and validate the needed ordinary generation rules/data against compatible upstream implementations and independent grammar/form expectations. Extend the public build/query CLI and the resource consumed by the desktop together, preserving provenance and original variants.
- [ ] Populate all eight indicative, two conditional, four subjunctive, two imperative, two infinitive, three participle, and two gerund blocks where grammatically supported. Verify each block independently, including ayant aimé, en aimant, and en ayant aimé. Do not generate surcomposés or duplicate the historical second conditional-past form.
- [ ] Whole expressions with and without appropriate subjects reverse-resolve every valid candidate and cell. Include both auxiliaries, person/tense ambiguities, and labelled alternatives for auxiliary choice by use; reject an unmatched expression rather than substituting its component-token matches.
- [ ] Display validated gender/number alternatives together, including je suis allé / allée, and keep each complete pronounceable alternative addressable. Use independently supported past-participle forms; never infer agreement dimensions from absent source tags.
- [ ] Explain context-dependent ordinary agreement, including preceding-object agreement with avoir, beside a reference form rather than guessing sentence context. Distinguish genuine defective/inapplicable cells from unresolved resource gaps.
- [ ] Cover every usable admitted ordinary verb without a frequency cap. Produce resource-wide 23-block support/unavailability/missing/unconfirmed reports and actual independent sample results; close generation/metadata gaps needed for claimed support instead of claiming completeness from sample success.
- [ ] CLI checks cover correct forms, every interpretation, integrity/format failures, and reproducibility. Desktop/browser and real native checks prove compound lookup, table content, variants, and all match highlights. Update resource/index sizes and cold/warm costs against measured budgets.

### Blocked by

T02.

## T04: show licensed offline English meanings in verb headers and candidate choices

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner understands the verb and ambiguous candidates from short offline English meanings, with sense labels where necessary, and can still use a table when no meaning is available.

### Acceptance criteria

- [ ] Admit an actual English-meaning resource with pinned reproducible inputs, compatible content licensing, required attribution/notices, and justified lemma/use relationships. The existing French-Chinese dictionary is not treated as English data, and no online translation or credentials are required.
- [ ] Show short English meanings in both table headers and ambiguous candidate choices; use brief sense labels where needed without conflating ordinary and pronominal identities or overwriting their source references.
- [ ] An explicit unavailable-meaning state leaves conjugation lookup and tables usable. Report English-meaning coverage separately from conjugation coverage across the admitted verbs.
- [ ] Public CLI checks prove actual mapped present/missing meanings and rejected malformed resources; desktop/browser and native resource checks prove the same admitted content reaches headers and choices with existing counts and navigation preserved.

### Blocked by

T02.

## T05: look up pronominal expressions and display validated reflexive agreement alternatives

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner searches se laver or je me lave and sees the appropriate pronominal reference across the agreed matrix, with readable reflexive expressions and justified auxiliary/agreement behavior.

### Acceptance criteria

- [ ] Admit pronominal-use data/rules with independently supported examples; do not manufacture a pronominal paradigm for every ordinary lemma by prefixing se. Preserve distinct ordinary/pronominal candidate and learning-record identities with distinguishable use labels.
- [ ] The public CLI and real desktop lookup resolve pronominal infinitives, subject-bearing simple expressions, and compound expressions as whole expressions, retaining every valid interpretation and highlighting the intended cells/variants.
- [ ] Populate the agreed 23 blocks where applicable with correct reflexive/subject presentation, elision, imperative positioning, and validated pronominal auxiliary rules. Preserve verified spelling and gender/number alternatives as individually addressable expressions.
- [ ] Display applicable alternatives together and explain context-dependent pronominal participle agreement with reference forms rather than guessing an absent object or sentence context. Separate defective/inapplicable cells from missing/unconfirmed data.
- [ ] Include independent cases for straightforward agreeing pronominals and context-dependent agreement, with both singular/plural and applicable masculine/feminine forms. Extend resource-wide coverage/exclusion reports without arbitrary verb caps or silently excluding difficult uses.
- [ ] Resource CLI, browser flow, and real native checks prove source-backed content, identity separation, all matches, and failure reporting; ordinary lookup, admitted metadata/meanings where present, and learning data remain intact.

### Blocked by

T03.

## T06: accept typographic query variations while preserving accents and complete expression matches

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner types a supported form or complete expression with ordinary capitalization, Unicode, apostrophe, or spacing differences and gets the same valid interpretations without losing accent distinctions or original display spelling.

### Acceptance criteria

- [ ] Case differences, canonical Unicode equivalents, straight/curly apostrophes, and trimmed/repeated whitespace resolve consistently through both the public CLI and native desktop lookup. Keep original source spelling for display and retain the learner's entered query.
- [ ] Accents remain significant. Typo guessing, accent removal, and matches for individual tokens cannot be reported as an exact whole-expression match. Invalid/unusable input, no-match, and resource failure remain distinct.
- [ ] Normalization retains all candidate/cell/variant interpretations and stable candidate order, including collisions where equivalent query spellings reach several valid cells. It does not merge vocabulary identities, ordinary/pronominal uses, or rewrite existing references.
- [ ] Boundary tests cover simple and apostrophe/space-bearing expressions available at landing, with independently expected positive and negative cases. Apply the same lookup behavior to later compound/pronominal additions; T11 verifies the combined expression set.
- [ ] Apply normalization consistently to compact-model suffix/stem candidate lookup and complete generated-form/expression verification. Keep the format-6 table and identity contract; do not add materialized normalized expression tables.

### Blocked by

T02.

## T07: add three verb coloring modes with validated endings and restart persistence

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner directly chooses complete-verbal-part, main-verb-only, or main-inflection-only coloring; the choice applies across verbs and survives restarting the actual application.

### Acceptance criteria

- [ ] Expose all three modes directly, with mode 3 as the initial default. In nous avions lavé, mode 1 colors avions lavé, mode 2 colors lavé, and mode 3 colors only é. In lavais, mode 3 colors only ais. Pronouns remain ordinary text; mode 3 leaves auxiliaries ordinary text.
- [ ] Validate first-/second-group inflectional spans through the public resource CLI against independent linguistic expectations and traceable admitted data/rules. Preserve complete correct forms and variants; a string-difference guess is not sufficient evidence.
- [ ] Every third-group/irregular verb remains ordinary text in mode 3, even if an individual ending is recognizable. Do not switch modes or substitute whole-word coloring; modes 1 and 2 still work for those verbs.
- [ ] Priority marking and all match highlights remain independently visible in every mode, with readable non-color cues, keyboard selection, and the existing languages/themes.
- [ ] Persist mode across different verbs and real native restarts using existing application-preference conventions. Preference writes, unreadable/corrupt settings, and unavailable storage surface meaningful failures. Any needed incremental migration preserves existing data and rejects unsupported higher versions.
- [ ] CLI checks prove actual resource spans and whole-category exclusion; browser checks prove exact rendering/control behavior; actual Windows execution proves persistence. Include spelling/gender alternatives and protect existing learning data.
- [ ] Keep linguistic ending spans independent of the compact storage models. A model's cut/suffix describes compression and does not establish which characters are inflectional; validate coloring spans using the admitted linguistic rules and preserve runtime-generated forms.

### Blocked by

T03.

## T08: reorder tense blocks across verbs and restore the order after restarting

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner changes the order of all 23 blocks, sees that order on other verbs and after restarting, and can reset to the exact agreed priority-first layout.

### Acceptance criteria

- [ ] Reorder all blocks using keyboard-accessible controls. An order contains each block once; switching verbs, including defective verbs, preserves the order without discarding unavailable blocks.
- [ ] Persist custom order locally across actual application restarts. Reset restores #38's exact seven-block priority order followed by the remaining matrix family/entry order without duplicates.
- [ ] Priority identity stays attached to the same seven tenses after reordering, while search highlights follow their actual matching cells. Moving a block does not change either interpretation.
- [ ] Reuse application preferences; expose write and corrupt/unavailable-setting failures. Any incremental storage change preserves existing data and rejects unsupported higher versions; saving block order preserves the separately owned coloring preference.
- [ ] Browser checks cover reorder/reset, keyboard operation, and match/priority behavior; real Windows restart checks demonstrate cross-verb restoration and failure handling with learning records intact.

### Blocked by

T02.

## T09: navigate between dictionary entries and the shared conjugation view without changing learning records

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner opens conjugation from a corresponding dictionary verb and returns to the associated dictionary entry, using the same table and candidate relationships as dedicated conjugation lookup.

### Acceptance criteria

- [ ] Both navigation entry points open one shared conjugation view using verified dictionary/verb relationships. Preserve the selected verb/use identity and access to other lookup candidates where applicable.
- [ ] Return navigation opens the corresponding dictionary entry when one exists. Missing or ambiguous relationships are shown honestly and require an appropriate choice rather than inventing a meaning or arbitrarily choosing a lemma.
- [ ] Existing explicit-save, duplicate-record, and unsaved-edit handling still apply. Forgiving lookup does not merge case/apostrophe-distinct records or ordinary/pronominal identities.
- [ ] Navigation and resource replacement do not create vocabulary/cards, overwrite notes or references, reset progress, or change selected audio. Verify these invariants using existing records with edits and review state.
- [ ] Browser flow and real native checks cover both directions, verified/unavailable/ambiguous relationships, keyboard operation, and unsaved-edit protection without replacing the dictionary resource or adding dictionary features outside #38.

### Blocked by

T02.

## T10: play each complete conjugated expression through the existing offline speech flow

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A learner plays one displayed conjugation expression or alternative on demand using the existing offline French voice and normal/slow preferences.

### Acceptance criteria

- [ ] Every supported displayed expression has an individual playback action. Speak its complete text, including displayed subjects/reflexive pronouns; spelling and gender alternatives play separately rather than sending written slash notation to speech.
- [ ] Reuse existing voice/speed preferences, CPU speech supervision, cancellation, temporary-audio retention, and error reporting. Preserve machine-synthesized disclosure and applicable voice/model notices. Do not pre-generate a whole table or create new review work.
- [ ] Actual generation and playback failures remain visible and recoverable through existing behavior, including unavailable speech resources and cancellation. Check the displayed expressions against the existing speech-input boundary; handle supported expressions without silent truncation or simulated success.
- [ ] Lookup/table speech does not replace a vocabulary word's selected audio, change review progress, save notes, or enroll cards. Navigation/cancellation and concurrent existing speech requests retain established lifecycle behavior.
- [ ] Browser checks assert the complete requested text, selected alternative, preferences, and errors; a real Windows check generates and plays source-backed expressions using actual speech resources. The same expression contract supports later pronominal/compound alternatives, verified together in T11.

### Blocked by

T02.

## T11: verify the complete conjugation flow in a fresh offline Windows package

### Parent

[#38](https://github.com/zardonc/lexique/issues/38)

### What to build

A Windows x64 learner can start a fresh prepared package offline, look up and navigate the complete agreed conjugation resource, hear forms, and restart with display preferences restored, without losing existing learning data.

### Acceptance criteria

- [ ] Prepare the actual executable package with admitted conjugation/English resources, provenance and redistribution notices, required TTS assets, and declared system/runtime dependencies. Verify package resource discovery without developer-checkout paths or a first-launch download; retain replaceable-resource separation from learning data.
- [ ] Execute first launch under declared offline conditions, then simple, compound, pronominal, subject-bearing, ambiguous, normalized, accented-negative, and unmatched queries. Confirm all interpretations and highlights, all 23 blocks, independent priority/color states, present/missing meanings, auxiliaries, and justified agreement/absence states.
- [ ] Play complete ordinary and pronominal expressions and separate spelling/gender alternatives through real offline generation/playback. Verify voice/speed behavior and visible resource/generation/playback failures rather than accepting a mocked event or cached sample as proof of generation.
- [ ] Reorder blocks and change mode, exit fully, restart the executed binary, and inspect different verbs. Confirm restoration and reset, visible preference failures, keyboard operation, and existing themes/languages; color mode 3 remains disabled in effect for the entire third group without changing the chosen mode.
- [ ] Exercise both dictionary navigation directions and unsaved-edit handling. Verify existing vocabulary identities, notes, selected audio, cards, progress, and review behavior survive the combined flow and resource replacement; resource failures never trigger learning-data rebuilds.
- [ ] Run the final actual resource CLI corpus against independent expectations and review resource-wide 23-block, metadata, pronominal-use, ending-span, and separate English-gloss coverage reports. Preserve traceable corrections/exclusions and resolve gaps required for the claimed scope; sample success cannot stand in for full admitted-resource coverage.
- [ ] Record original/generated/index/package sizes and cold/warm costs against the budgets established during implementation. Run applicable frontend, locked Rust, resource, browser, and native checks, preserving separate backend precompilation and the hard execution timeout; tie evidence to the executed binary/resource versions and declare passed/failed/skipped results.
- [ ] Update readiness evidence for Windows x64 only after the actual flow passes. Track other-platform compilation/execution gaps and M5 distribution requirements separately; this acceptance does not close unrelated M3 modules or claim four-target release readiness.
- [ ] Package the compact model resource (format-6 baseline and subsequent admitted refinements), including all seven MIT converters, original source/licensing materials and provenance. Preserve exhaustive format-5 equivalence evidence for all 1,451,539 cells and 10,781 identities, native-runtime comparisons, and learning-data preservation. Record compact database/model/stem-index/package sizes and fresh/warm costs against the current budgets; the format-5 disk/index budgets are historical.

### Blocked by

T04, T05, T06, T07, T08, T09, T10. These include T01–T03 transitively.

## Specification coverage

| User stories in #38 | Delivery ownership |
| --- | --- |
| 1, 20, 23–25, 56 | T01 resource/metadata admission; T02 native resource access; T03/T05 full paradigms; T11 fresh bundled verification |
| 2, 5, 12–19 | T02 simple lookup, candidates, all highlights, variants, and visible failures |
| 6, 27–29, 35–39 | T03 ordinary compound/nonfinite tables and agreement; T05 pronominal extensions |
| 7–8, 26 | T05 pronominal and subject-bearing expressions, building on T03 |
| 9–11 | T06 query equivalence and accent distinctions |
| 15, 21–22 | T04 present/missing English meanings in headers and choices |
| 30–31 | T02 priority-first defaults; T07 independent color/match states; T08 priority identity after reordering |
| 32–34 | T08 order, native persistence, and reset |
| 40–46 | T07 all three modes, exact spans, exclusions, and native persistence |
| 3–4 | T09 shared dictionary/conjugation navigation |
| 47–51 | T10 complete-expression alternative playback and failures |
| 52–54 | Shared requirements in each feature; T11 combined preservation/accessibility verification |
| 55 | T11 actual fresh-package offline Windows x64 acceptance |

## Publication verification

Published using `gh issue create --body-file` with the `ready-for-agent` label and `gh api` for native sub-issues and dependencies. Read-back verification passed for all 11 issue titles and complete bodies, labels, open states, parent links, and the exact 16 blocking edges. Body comparisons normalize Windows/Unix line endings. Parent content and state are unchanged.

Only [#39](https://github.com/zardonc/lexique/issues/39) can start immediately. Implementation has not started. Documentation links and staged-diff checks validate this publication record, not the unimplemented feature.
