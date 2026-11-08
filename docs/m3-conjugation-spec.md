# M3: offline conjugation, complete tables, expression lookup, coloring, and pronunciation

2026-11-08: the shared playback target is [80–240 WPM](speech-preferences.md),
in 20 WPM steps with 180 as the default; it is pending implementation for
word, example and conjugation pronunciation. [Vocabulary frequency colors](word-frequency-colors.md)
have their own statistical standard. [System selection and screen OCR](translation-inputs.md)
are required in the first M3 delivery; the [updated plan](lookup-notes-speech-plan.md)
preserves the accepted Windows conjugation evidence below.

2026-11-07: the agreed Windows x64 conjugation scope is accepted through
#39–#49 and the user's fresh offline package pass. The
[closeout report](evidence/issue-38/README.md) maps all 56 stories to the
executed evidence and retains the separate platform, resource and M5 gates.
The specification below records the confirmed scope and implementation baseline.

## Problem Statement

French learners need to identify a conjugated form, understand every valid interpretation, inspect the corresponding verb's complete agreed paradigm, and hear its forms without a network connection. A dictionary hit or an incomplete source table cannot establish that a compound or pronominal expression is supported.

The desktop currently has a conjugation navigation entry but no product lookup or table flow. The completed M0 validation in [#4](https://github.com/zardonc/lexique/issues/4) proves reproducible Lefff source conversion, exact reverse lookup, multiple matches, and source-table reading. It does not establish the full M3 paradigm, English meanings, group and auxiliary metadata, compound/pronominal generation, or ending segmentation.

This specification delivers the confirmed conjugation portion of M3, under [the overall specification #1](https://github.com/zardonc/lexique/issues/1). It preserves existing vocabulary records, notes, review progress, and speech behavior.

## Solution

Provide one offline conjugation view reachable from both dictionary and conjugation navigation. A learner enters a verb form or expression, sees every matching verb and interpretation, and opens a complete reference table with every matching cell highlighted. Each verb shows English meanings, its traditional conjugation group, and the applicable auxiliary or auxiliaries.

Each agreed tense or nonfinite construction has its own block. Seven priority tenses appear first. The learner can reorder blocks, choose one of three verb-coloring modes, and play each displayed form separately. Display preferences survive a real application restart on the current device. Windows x64 is the first native acceptance target.

## User Stories

1. As an offline learner, I want conjugation resources supplied with the application, so that lookup does not depend on a first-launch download.
2. As a learner, I want to search from the dedicated conjugation entry, so that I can investigate a form directly.
3. As a dictionary user, I want to open the same conjugation view from a matching verb, so that I can move between meaning and conjugation.
4. As a conjugation user, I want to return to the corresponding dictionary entry, so that I can inspect its meaning without losing the verb relationship.
5. As a learner, I want to look up a simple form such as sommes, so that I can identify its possible verbs.
6. As a learner, I want to look up a compound expression such as avons aimé, so that I can identify the whole expression.
7. As a learner, I want to look up a pronominal infinitive such as se laver, so that I can inspect its pronominal paradigm.
8. As a learner, I want to look up a verb expression with a subject such as je me lave, so that I can search a readable conjugation.
9. As a learner, I want capitalization differences accepted, so that ordinary typing does not prevent a match.
10. As a learner, I want equivalent Unicode spellings, straight/curly apostrophes, and surrounding or repeated spaces accepted, so that typographic variations do not prevent a match.
11. As a learner, I want accents to remain meaningful, so that a different spelling is not silently treated as an exact match.
12. As a learner, I want an unmatched expression reported explicitly while retaining my input, so that I can correct it.
13. As a learner, I want resource failures distinguished from an absent form, so that I know when lookup could not be completed.
14. As a learner, I want one matching verb to open directly, so that an unambiguous lookup takes fewer steps.
15. As a learner, I want multiple matching verbs shown with English meanings and match counts, so that I can select the intended interpretation.
16. As a learner, I want the other candidates to remain accessible after selecting one, so that ambiguity is not hidden.
17. As a learner, I want every matching person and tense highlighted in the selected table, so that I can understand all interpretations of the input.
18. As a learner, I want candidate counts distinguished from match counts, so that several cells of one verb do not appear to be several verbs.
19. As a learner, I want all supported spelling variants retained in the same cell, so that one legitimate spelling does not overwrite another.
20. As a learner, I want all usable admitted verbs available, so that coverage is not arbitrarily limited to a small high-frequency list.
21. As a learner, I want short English meanings and sense labels where needed, so that I can understand the selected verb and ambiguous candidates.
22. As a learner, I want missing English meanings identified while available conjugation remains usable, so that missing metadata does not hide a verb.
23. As a learner, I want the traditional first, second, or third/irregular group displayed, so that I can recognize its conjugation family.
24. As a learner, I want the appropriate auxiliary displayed, so that I can understand compound forms.
25. As a learner, I want multiple auxiliaries and their uses explained, so that one choice is not presented as universally correct.
26. As a learner, I want ordinary active and pronominal forms supported, so that both laver and se laver have an appropriate reference.
27. As a learner, I want each agreed tense or construction in its own block, so that I can scan the table by tense.
28. As a learner, I want the full agreed simple and compound paradigm, including literary forms, so that the table is a reference beyond the most common tenses.
29. As a learner, I want infinitive, participle, and gerund constructions included, so that nonfinite forms are covered explicitly.
30. As a learner, I want the seven specified priority tenses first and visibly marked, so that my main study targets stand out.
31. As a learner, I want priority marking and search highlighting to remain distinct, so that a match in another tense is still obvious.
32. As a learner, I want to reorder blocks across all verbs, so that the view follows my study order.
33. As a returning learner, I want my block order restored after restarting, so that I do not repeat the adjustment.
34. As a learner, I want to reset block order to its default, so that I can recover the agreed priority-first layout.
35. As a learner, I want subject and reflexive pronouns shown where appropriate, so that forms are readable expressions.
36. As a learner, I want applicable feminine and masculine alternatives shown together, so that agreement is visible.
37. As a learner, I want validated past-participle gender/number forms, so that unsupported combinations are not invented.
38. As a learner, I want context-dependent agreement explained with a reference form, so that the table does not pretend to infer an absent sentence context.
39. As a learner, I want defective or inapplicable cells distinguished from missing resource data, so that I can tell grammatical absence from incomplete coverage.
40. As a learner, I want a directly selectable three-mode coloring control, so that I can change the learning emphasis.
41. As a learner, I want mode 1 to color the complete verbal part including auxiliaries, so that it stands apart from pronouns.
42. As a learner, I want mode 2 to color only the main verb, so that I can distinguish it from its auxiliary.
43. As a learner, I want mode 3 to color only the main verb's inflectional characters, so that I can focus on endings.
44. As a learner, I want mode 3 to leave every third-group/irregular verb uncolored, so that it does not suggest a regular ending analysis.
45. As a learner, I want mode 3 as the default and my selected mode remembered across verbs and restarts, so that the view keeps my preferred emphasis.
46. As a learner, I want match highlighting to remain visible in every coloring mode, so that switching emphasis does not hide search results.
47. As a learner, I want every displayed form individually playable on demand, so that I can hear the form I am studying.
48. As a learner, I want the complete displayed expression pronounced, including its pronouns, so that I can hear the readable form.
49. As a learner, I want gender and spelling alternatives individually playable, so that a written slash is not read as one expression.
50. As a learner, I want playback to use my existing voice and normal/slow preference, so that pronunciation follows my settings.
51. As a learner, I want speech generation and playback failures shown explicitly, so that unavailable audio is not reported as success.
52. As a learner, I want lookup, browsing, and playback to preserve my vocabulary, notes, and review progress, so that exploring conjugation does not modify learning records.
53. As a keyboard user, I want the controls and block ordering operable without a pointer, so that I can use the full view.
54. As a learner using any existing theme or interface language, I want readable labels and distinguishable states, so that the view remains usable without relying on color alone.
55. As a Windows x64 user, I want a fresh prepared package to support the whole flow offline, so that development-preview success is backed by native behavior.
56. As a user or redistributor, I want resource origins, licenses, modification notices, and coverage limitations retained, so that I can assess and redistribute the delivered data correctly.

## Implementation Decisions

### Agreed paradigm and order

Each entry in the following matrix is a separate block, for 23 blocks in total. A defective verb can have explicitly unavailable forms; the number of blocks is not a promise to invent forms for it.

| Family | Blocks |
| --- | --- |
| Indicatif | Présent; passé composé; imparfait; plus-que-parfait; passé simple; passé antérieur; futur simple; futur antérieur |
| Conditionnel | Présent; passé |
| Subjonctif | Présent; passé; imparfait; plus-que-parfait |
| Impératif | Présent; passé |
| Infinitif | Présent; passé |
| Participe | Présent (aimant); passé (aimé); composé (ayant aimé) |
| Gérondif | Présent (en aimant); passé (en ayant aimé) |

The default first seven blocks, in order, are Présent de l'indicatif, Passé composé, Imparfait, Futur simple, Conditionnel présent, Subjonctif présent, and Plus-que-parfait. Remaining blocks follow the matrix's family and entry order, without duplicating the first seven. Priority identity remains attached to the tense after reordering.

The historical name conditionnel passé deuxième forme is explained beside the corresponding subjunctive construction rather than duplicating its morphology. The actual construction anchors nonfinite labels where grammar references use different terminology.

### Verb information, variants, and agreement

1. The header and ambiguous candidates show short English meanings, with brief sense labels where necessary. Missing meanings are explicitly unavailable and do not suppress usable conjugation.
2. Traditional groups mean regular first-group -er verbs, second-group -ir verbs of the finir/finissons pattern, and the remaining third-group/irregular verbs, including aller and partir. Infinitive suffix alone does not establish group membership.
3. Show all applicable auxiliaries and brief usage explanations. Compound blocks preserve labelled alternatives when auxiliary choice depends on use. A pronominal use follows its own validated auxiliary and agreement rules.
4. Show subject pronouns for finite forms and reflexive pronouns for pronominal forms. Imperative and nonfinite constructions use their appropriate presentation.
5. Preserve every valid spelling alternative, person, tense, number, gender, and use distinction. Applicable alternatives such as je suis allé / allée appear together; each pronounceable alternative remains individually addressable.
6. Past-participle forms and inflectional spans require verified linguistic support. Sentence-dependent agreement, including a preceding direct object with avoir, uses a reference form and an explanation rather than a guessed agreement. A missing tag is not evidence that every gender/number combination is valid.
7. Represent verified grammatical absence/inapplicability separately from missing or unconfirmed source data. Malformed-source exclusions and corrections retain evidence and traceability; unconfirmed cells do not masquerade as a complete supported table.

### Expression lookup and match identity

1. Recognize single forms and complete verb expressions, optionally with subject pronouns, within the agreed active/pronominal paradigm. Examples include sommes, avons aimé, se laver, and je me lave. Whole-expression matches are not replaced by unrelated matches for individual tokens.
2. Accept capitalization differences, canonically equivalent Unicode, straight/curly apostrophes, and surrounding/repeated whitespace. Preserve accent distinctions and original correct display spelling. Typo guessing or accent removal cannot create an exact match.
3. Open a sole matching candidate directly. With multiple candidates, show a chooser with meanings and match counts. Keep alternate candidates accessible after opening a table. Stable candidate ordering follows the French lemma, with distinguishable labels for uses of the same lemma.
4. Distinguish candidate identities from cell matches. Preserve all interpretations; selecting a candidate controls the visible table without deleting the other candidates' matches. Every corresponding cell in the selected table receives match highlighting.
5. Matching identifies the intended tense, person, and applicable variants, rather than coloring every cell containing a shared token. A normalized query does not rewrite source forms or merge learning-record identities.
6. Keep no-match, invalid/unusable input, unavailable resource, and lookup failure observable at the application boundary. Retain the entered query when reporting failure. Browser preview continues to report capability limits explicitly.

### Coloring and local preferences

The three modes are directly selectable. In nous avions lavé, mode 1 colors avions lavé, mode 2 colors lavé, and mode 3 colors only the é in lavé. Mode 3 leaves the auxiliary avions in ordinary text; in lavais it colors only ais.

Mode 3 excludes the entire third-group/irregular category, even when a particular form has an identifiable ending. It leaves that verbal text uncolored rather than silently changing mode. Modes 1 and 2 remain available, and match highlighting works independently of all three modes.

Mode 3 is the default. The selected mode and custom block order apply across verbs and survive restart on the current device. Resetting block order restores the specified default order. Store these as application preferences using existing persistence conventions; no cross-device preference synchronization or fixed new schema version is introduced by this specification. Any needed incremental migration must preserve existing data and reject unsupported higher versions.

Priority marking, verb-coloring spans, and match highlighting are independent presentation properties. Provide readable non-color cues, keyboard operation, and compatibility with the existing three interface languages and themes. Exact colors and responsive arrangement follow the existing interface conventions.

### Resources and admission

1. Cover every usable verb in the admitted resource, without an arbitrary high-frequency cap. Retain explicit coverage and exclusions. English gloss availability and conjugation completeness are measured separately.
2. Reuse the pinned Lefff validation as the baseline, then validate the additional group, auxiliary, English meaning, pronominal, compound, and regular-ending information needed by this specification. No particular additional source, generation library, or complete M3 resource is deemed admitted merely by naming this scope.
3. Before committing to unfamiliar generation or parsing designs, inspect active, compatible, well-licensed upstream implementations and their known pitfalls. Evaluate data and code licenses separately, retain source versions and hashes, preserve attribution/license/warranty notices, and document conversion changes. Derived linguistic data retains its applicable license.
4. Preserve original forms, source references, and all matches in generated resources. Provide reproducible build/query entry points, independently justified expected outputs, actual results, and coverage reports for the 23-block matrix. A source tag expansion or a plausible generated string is not independently verified language correctness.
5. Verify source integrity, generated-resource integrity and format, and required metadata before relying on them. Missing/corrupt resources fail visibly. Never replace a bad resource with fabricated success or rebuild learning data as a repair.
6. Measure original and transformed resource sizes, reverse indexes, and cold/warm lookup costs on a declared dataset and environment. Set implementation budgets from those measurements; this specification does not assert unmeasured latency or package-size limits.

### Compact conjugation storage

Store shared simple-form conjugation models with per-verb model references, source-line anchors, compressed metadata and lexical flags. Pronominal identities refer to their ordinary base. Construct compounds, subjects, reflexives, gerunds and agreement alternatives at runtime using the admitted rules; keep spelling variants and source exceptions. Do not materialize the complete tables or an expression index.

Reverse lookup strips compatible model suffixes, finds candidate stems, then verifies the complete generated form or expression. Preserve all identities, interpretations, source fields, duplicates, ordered alternatives and stable table order through the existing lookup interface. Before continuing #44, regenerate and compare all 1,451,539 cells and 10,781 identities with the preserved format-5 database, and verify the native runtime through that same interface. Keep the old resource and all learning data.

The format-6 baseline is prepared in `target/desktop-conjugation-v6/`; its contract is recorded in `docs/conjugation-storage.md`. Storage cuts and suffixes are compression artifacts, not linguistic ending spans for #45. Measure the compact database, models, stem index, package and cold/warm costs; earlier format-5 budgets are historical.

### Application boundaries, speech, and data preservation

1. Extend the existing injected runtime adapter for conjugation lookup, selected-table access, and display preferences. Keep platform file access, resource paths, speech scheduling, and native storage at the established desktop boundary. Reuse resource-reading modules and persistence conventions before introducing another abstraction or dependency.
2. Native and frontend boundaries validate requests/responses, preserve meaningful errors, and expose structured data rendered as text. Use parameterized resource queries, least-privilege IPC, and bounded supported inputs. Frontend callers do not supply arbitrary resource or model paths. No new credentials or online service is needed for the confirmed flow.
3. Keep replaceable conjugation resources separate from learning data. Search normalization is an aid to lookup, not authority to merge case/apostrophe-distinct vocabulary records, conflate ordinary and pronominal records, or overwrite existing references. Dictionary navigation reuses verified candidate relationships and existing explicit-save rules.
4. Lookup, browsing, and playback do not automatically create vocabulary records, edit notes, enroll cards, reset progress, or change selected word audio. Resource replacement cannot rewrite the learner's records.
5. Play the complete selected displayed expression on demand, including its displayed pronouns. Individually play gender/spelling alternatives rather than sending slash notation as spoken text. Reuse the established voice preferences, normal/slow playback, native speech supervisor, temporary-audio retention, error reporting, and cancellation boundaries. A verb table does not pre-generate every form's audio or create review work.
6. First native acceptance targets Windows x64. Prepare a fresh package with the declared dependencies and required offline speech resources, then verify first offline launch, lookup, tables, real playback, and preference restoration. Other-platform execution and formal four-target distribution remain separately tracked gates.

## Testing Decisions

The user confirmed two public test boundaries for this specification. Tests observe externally visible behavior and independently expected language results, not private helper calls, internal SQL layout, or snapshots generated by the converter under test.

### Boundary 1: desktop user flow through the existing runtime adapter

Observe lookup, candidate selection, table content and all match highlights, color modes, ordering, dictionary navigation, playback outcomes, and saved preferences. Reuse the established browser E2E adapter fixtures for UI scenarios and the existing real Windows native harness pattern for actual IPC, resource access, restart persistence, and playback. Fixtures do not establish native storage, offline speech, or resource correctness.

Required coverage:

- Simple, compound, pronominal, subject-bearing, ambiguous, and unmatched queries; equivalent Unicode/case/apostrophe/space inputs; accents kept distinct; no token-only substitution for whole-expression matching.
- One candidate opening directly; multiple candidates selectable with meanings/counts; switching candidates preserves access to all interpretations; highlight every matching cell including nonpriority blocks.
- All 23 blocks in the agreed order, seven distinct priority markers, reorder/reset behavior, and no duplicate blocks.
- English meaning present/missing; traditional group; ordinary and pronominal auxiliaries; labelled multiple-auxiliary alternatives; gender/number variants; context-dependent agreement explanations; distinct defective and unknown states.
- Exact color spans for the agreed lavais and nous avions lavé examples in all modes; whole-category third-group exclusion in mode 3; pronouns and auxiliary behavior; independently visible priority and match states.
- Real restart restoring mode and order across different verbs, with write failures and corrupt/unavailable settings remaining visible and existing learning data preserved.
- Individual alternative playback, full spoken text, existing voice/speed preferences, explicit generation/playback errors, and reuse of native speech boundaries without automatic review enrollment or selected-audio replacement.
- Keyboard operation, readable cues beyond color, existing themes/languages, and navigation that preserves existing explicit-save and unsaved-edit handling.
- A fresh prepared Windows x64 package's first offline run through actual lookup, table display, audio generation/playback, shutdown, and restart. Tie evidence to the executed binary and resource versions; retain declared offline conditions and any unavailable dependencies.

### Boundary 2: public resource build/query CLI

Extend the real-source CLI behavior proven by #4. Validate the actual admitted resource, runtime-generated tables, suffix candidate lookup, metadata, and supported ending spans through public inputs/outputs. Maintain independent literal or source-cited expected results rather than deriving expected forms with the same generation logic. Product and CLI outputs must exercise the admitted representation/generation decisions; an unrelated successful demo is insufficient.

Required coverage:

- The known Lefff sommes matches for sommer (P2s and S2s) and être (P1p), and suis across suivre and être, plus independently verified expectations for newly supported expressions.
- First-/second-group regular verbs, third-group irregular verbs, both auxiliaries, ordinary/pronominal uses, defective verbs, multiple auxiliaries, spelling variants, person/tense ambiguity, and validated gender/number alternatives.
- Every block in the confirmed matrix has representative independently checked support, while resource-wide reports distinguish supported, grammatically unavailable, missing, and unconfirmed coverage. Sample success alone cannot claim full-resource coverage.
- First-/second-group ending boundaries, complete third-group exclusion from mode-3 coloring, preservation of full correct forms, and no invented spans from textual similarity.
- Compound/pronominal reverse lookup preserving all valid interpretations and agreement/usage labels; unknown source dimensions remain unknown until justified by validated data/rules.
- Source provenance/licensing materials, reproducible builds and query outputs, resource and index-size accounting, and declared cold/warm performance measurements.
- Missing, corrupt, truncated, incompatible, or incomplete source/generated resources; rejected unsafe input; outputs that preserve prior valid resources and learning data. Never report an unreadable resource as an empty result.

Prior art is the existing conjugation and dictionary public CLI resource tests, dictionary and speech browser E2E flows using injected runtime adapters, and M1/M2 native restart/playback evidence. Keep tests at these boundaries rather than introducing a third public API solely for tests. Run the repository's relevant frontend, Rust, resource, browser, and native checks for changed code. Precompile backend tests separately and retain the runner's hard 60-second execution timeout.

Record commands with passed/failed/skipped outcomes and maintain readiness evidence. No tests of this feature have passed merely because this specification is published.

## Out of Scope

- Surcomposés, such as j'ai eu aimé, beyond the agreed 23 blocks.
- Passive-voice, negative-sentence, question/inversion generation, and arbitrary sentence analysis.
- Automatically inferring absent sentence context for participle agreement or inventing unattested defective forms.
- Silent accent removal, typo correction presented as an exact match, or vocabulary-identity merging based on forgiving search.
- New conjugation flashcard types, conjugation drills, automatic note/card creation, or changes to M2 scoring and progress rules.
- Cloud synchronization of these preferences, account requirements, online translation/AI, or new service credentials.
- Full dictionary replacement, general nonverb inflection lookup, custom MDX/MDD integration, and other independently tracked M3 modules.
- Browser-host persistence/offline-resource parity, other-platform native acceptance, signing, installers, and full four-target final distribution as conditions of this first Windows x64 milestone.

## Further Notes

- The user confirmed the product scope through Q23 and approved the two test boundaries during specification synthesis. This is a multi-session specification to be decomposed into dependency-linked, independently verifiable tickets. Publication does not start implementation or automatically publish child tickets.
- #4 is closed and its implementation PR [#37](https://github.com/zardonc/lexique/pull/37) is merged. This is the implementation baseline, not proof that the full M3 resource has been admitted.
- Resource admission and generation validation gate complete-table product claims. If a proposed resource cannot meet the confirmed scope, report the concrete gap and resolve it before claiming completion; do not silently shrink the agreed paradigm or hide unsupported cells.
- The first Windows x64 acceptance does not imply the rest of M3 or formal desktop release is complete. Existing platform, resource, model-license, and distribution gates remain applicable.
- Grammar references for the agreed constructions and agreement constraints: [Académie française's aimer table](https://www.dictionnaire-academie.fr/conjuguer/A9A1016), [Usito's tense/construction tables](https://examen-francais-quebec.usito.usherbrooke.ca/articles/th%C3%A9matiques/contant_1), [OQLF on the historical second conditional-past form](https://vitrinelinguistique.oqlf.gouv.qc.ca/24141/la-grammaire/le-verbe/temps-grammaticaux/conditionnel/conditionnel-passe-et-plus-que-parfait-du-subjonctif), [OQLF's simple/compound comparison](https://vitrinelinguistique.oqlf.gouv.qc.ca/24658/la-grammaire/le-verbe/temps-grammaticaux/temps-simples-et-temps-composes), and [Government of Canada on agreement with avoir](https://nos-langues.canada.ca/fr/cles-de-la-redaction/participe-passe-avec-avoir). These establish grammatical reference points, not the quality of a particular admitted dataset.
