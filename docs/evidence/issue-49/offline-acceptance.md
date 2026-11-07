# Fresh offline Windows conjugation acceptance (#49)

Use the untouched [speech package](../../../target/issue-49/offline-package-20261107/Lexique-0.1.0-windows-x64-portable-speech.zip),
177,115,811 bytes, with its [transfer checksum](../../../target/issue-49/offline-package-20261107/Lexique-0.1.0-windows-x64-portable-speech.zip.sha256).
Package identity and included dependencies are recorded in [the inventory](package-inventory.json).
2026-11-07: the user reported that this package's offline acceptance passed.

## First launch

1. Use Windows 11 x64 with an installed Microsoft Edge WebView2 runtime. The ZIP includes the speech runtime, model, voices and encoder. If WebView2 is missing, follow the included `README.txt` and prepare its official standalone installer before disconnecting. This is an unsigned development package.
2. Block network access before the first launch: disconnect Wi-Fi and Ethernet, or use an equivalent outbound block. Keep it blocked through the restart and playback checks.
3. Extract the entire ZIP into a new writable folder. Confirm that it has no `Lexique-data/` before opening `Lexique.exe`. Keep `dictionary/`, `conjugation/` and `speech/` beside the executable. Start the app and choose English in the language selector.

## Lookup and display

Use Conjugation and check the following independent expectations. Matching expressions retain their underline as well as any chosen color.

| Queries | Expected result |
| --- | --- |
| `aimer`; `abéliser` | 23 blocks remain usable; `aimer` has English meanings including “love”; `abéliser` explicitly has no available meaning. |
| `sommes`; `suis` | All candidates remain selectable. `sommes`: `sommer` has 2 matching cells and `être` has 1. `suis`: `suivre` has 3 and `être` has 1. English meanings appear in choices and the selected header. |
| `paie` | `payer` retains separate `je paie` and `je paye` alternatives. |
| `avons aimé`; `aie aimé`; `en ayant aimé` | Complete ordinary compound/non-finite expressions resolve; `aie aimé` retains both matching interpretations. |
| `je suis allée`; `je suis monté`; `j’ai monté` | Gender alternatives and independently admitted auxiliary uses remain available. |
| `se laver`; `je me lave`; `lave-toi` | The separate pronominal identity, subject/reflexive expression and imperative placement are correct. Context-dependent agreement is explained. |
| `elles se sont abstenues`; `elles se sont parlé` | Subject-agreeing alternatives for `s’abstenir`; the invariant `parlé` reference for `se parler`, without invented `parlées`. |
| `elle s’est arrogé`; `nous nous sommes écrites`; `on s’est abstenus` | Context-dependent agreement and collective `on` remain distinct; the app does not invent an absent object. |
| `va-t’en` | `s’en aller` resolves as a whole expression with its own use and meaning. |
| `falloir`; `gésir` | Verified grammatical absences are labelled separately from unavailable data; all 23 reference blocks remain visible. |
| `SOMMES`; `S'ABSTENIR`; `VA-T'EN`, with extra surrounding spaces | Typography/case normalization preserves the typed query, original source spelling and valid interpretations. |
| `ete`; `J'AI AIME`; `JE SE LAVE`; `zzzinconnu` | No match; accents and whole-expression grammar are not discarded. |

For `nous avions lavé`, the three color modes select `avions lavé`, `lavé` and `é`, respectively. Seven priority blocks remain independent of the match and color mode. In mode 3, inspect the entire `aller` table: it has no colored endings and mode 3 stays selected. Inspect `finir` to confirm that second-group endings still work.

## Playback, navigation and restart

1. Play each action separately: `je paie`, `je paye`, `je suis allé`, `je suis allée`, `je me lave`, `lave-toi`, `elles se sont abstenues`, and `va-t’en`. Hear the complete expression, including its subject, pronouns and auxiliary. Confirm the machine-speech notice and that another action stops the previous player.
2. Change the speech preference to male/slow and play `je paie`; change to male/normal and replay it. Confirm the voice and speed behavior, then restore your preferred settings.
3. In Dictionary, look up `laver`, open its conjugation and return to its dictionary entry. From `suis`, select `être` and navigate both ways; the original query and candidate context survive the return. `se laver` explicitly has no corresponding admitted dictionary verb entry. Repeat the links with keyboard Enter/Space and in Chinese, French and English, using white, sepia and dark themes.
4. Move the past-perfect block (`PQP`) one position later, after simple past (`J`). Inspect `laver`, `falloir` and `se laver`: the order is shared and the priority set is unchanged. Exit fully and reopen the same binary; inspect different verbs and confirm the chosen color mode and custom order. Repeat restart with each color mode. Reset the block order, exit fully, restart and confirm the default order without changing the color preference.

## Preserve existing data in a second test copy

1. Close your normal Lexique app. Extract this ZIP into another new folder and, before opening it, copy your existing closed `Lexique-data/` into that test copy. Keep the original application and data as your backup.
2. Compare existing words, notes, selected audio, cards and progress before and after lookup, expression playback, navigation and restart. Open a recognition card, reveal and close it. Open a listening card, hear its selected word audio, reveal and skip without grading. Changing the global voice must leave the selected word audio and progress intact.
3. Use a saved dictionary verb for unsaved-edit navigation; explicitly save `laver` in this test copy if needed. “Keep editing” preserves the draft, “Discard changes” preserves the saved notes, and “Save and leave” commits only the intended edit. Existing records and cards keep their identities.
4. With this test copy closed, rename `conjugation/conjugation.sqlite3` to `conjugation.sqlite3.saved`. Reopen and query: the missing-resource error is visible and existing learning data remains available. Close, restore the resource from the ZIP, reopen and confirm recovery and preservation.
5. With the test copy closed, rename `speech/` to `speech.saved/`. Reopen and request an unplayed expression such as `en ayant parlé`: a real generation failure must be visible. Close, restore `speech/`, reopen and retry successfully. Perform resource changes only in this disposable test copy.

## Result

2026-11-07 user result: "offline test passed" for the package identified above.
The user did not supply Windows/WebView2 versions, the network-block method or
per-step logs. The result is recorded as user-reported acceptance alongside the
independent local native evidence. The accepted archive remains unchanged.

This acceptance covers Windows x64 and the admitted reference resource. English coverage is partial. Other-platform execution, missing-dependency clean-machine installation and M5 distribution remain separate gates.
