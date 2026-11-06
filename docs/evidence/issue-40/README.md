# Desktop simple-form conjugation lookup

[Issue #40](https://github.com/zardonc/lexique/issues/40) connects the admitted [#39 resource](../issue-39/README.md) to the desktop through `RuntimeAdapter`. Exact source forms resolve to all matching verbs. One verb opens automatically; ambiguous results retain French-sorted, keyboard-accessible choices after selection. Candidate counts and logical matching-cell counts are separate; each candidate also shows its own matching-cell count. Every matching cell has a checkmark, text label, border and underlined matching expressions, including nonpriority tenses. Spelling alternatives stay together without marking unrelated forms.

Each verb has 23 separate blocks. The seven confirmed priority tenses come first, followed by the remaining mood/tense matrix order. Headers retain group, auxiliary/use metadata and explicit unavailable English meaning. Source absence is unconfirmed, independently of verified ordinary-use inapplicability. Compound/pronominal constructions, h/y subject elision without supporting information, unspecified source identities and raw anomalies remain unconfirmed. The historical second conditional-past name refers to the subjunctive block without duplicating forms.

The implementation reuses the admitted SQLite layout, parameterized exact queries and the dictionary's verified-byte read-only snapshot pattern. The native command accepts a bounded word only; its resource path is application-controlled and it never opens learning storage. Responses are validated for structure, requested query, candidate identity and cell ownership. Browser preview explicitly cannot read the resource.

The pinned [Grammalecte decoder reference](../../../resources/grammalecte-2.3.0/conj.py) provides the existing design evidence for separating subject-bearing finite forms, imperative forms and nonfinite forms; its code is not executed or copied into the MIT application. Original data, LGPL-LR/GPL notices, provenance, corrections, decoder, MIT conversion source and rebuild instructions accompany the resource. No dependency or learning-data migration was added.

## Verification

Windows x64, 2026-11-05, managed Python 3.14.10, Node 24.22.1 and pnpm 11.25.0. Python's managed directory was prepended to process PATH; the test TEMP/TMP directory was `target/issue-40-temp`.

| Check | Result |
| --- | --- |
| `pnpm lint`, `pnpm typecheck`, `pnpm desktop:build --no-bundle` | PASS; includes frontend production build and native release build |
| `pnpm test:e2e --workers=2` | PASS, 104 browser tests; real data fixture, simulated IPC |
| `pnpm test:conjugation` | PASS, 15 public CLI tests; each CLI subprocess retains its 60-second timeout |
| `pnpm test:resources` | PASS, 7 dictionary CLI regressions |
| `pnpm test:tooling` | PASS, 2 build-command checks, 8 portable checks, 2 speech preflight checks; speech-source integration test skipped without its inputs |
| `cargo fmt --all --check`, `cargo check --locked --workspace` | PASS |
| `pnpm test:docs`, `pnpm test:docs:validator` | PASS, 106 documents / 638 local links and 13 validator cases |
| `cargo clippy --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings` | PASS |
| `pnpm rust:test:build`, then `pnpm test:rust` | PASS; compilation separate, execution hard limit remains 60 seconds |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run`, then `pnpm test:speech:cache`, `pnpm test:desktop:lib` | PASS, 10 cache and 3 desktop tests; existing explicit subprocess fixture ignored; separate 60-second execution limits |
| Real Windows WebView2 through `native-conjugation-check.mjs` | PASS, actual IPC and rendered table; normal close/restart; hard 60-second execution limit |
| `python -X utf8 scripts/package_portable.py --output-dir target/issue-40/portable-reviewed` | PASS, 18,984,076-byte ZIP, 26 verified files, no learning data |

The [native results](native-results.json) retain all source interpretations for the tested forms. The native check verifies `sommes`, `suis`, `paie`, `falloir`, `enfuir`, `voilà`, exact-case/no-match and SQL-literal queries, rejects blank/control/oversized inputs, renders the real `suivre` table with its per-candidate count, and checks missing/corrupt/operational failures. One existing synthetic vocabulary record, two cards, their progress, saved speech preferences, selected audio bytes and three audio files remain unchanged before/after lookup, resource replacement and a normal restart. Only task-owned copies are used; started processes are stopped.

Native reproduction from `apps/desktop`, with the existing synthetic learning fixture and a new destination:

```powershell
node e2e/native-conjugation-check.mjs ../../target/release ../../target/release/lexique-desktop.exe ../../target/issue-24/verified-native-selected-audio-27/Lexique-data ../../target/issue-40/native-new
```

The source fixture contains `data/lexique.sqlite3` and `temp/speech/`; the harness refuses an existing destination. A fresh synthetic fixture with selected audio can be supplied instead. [Native table screenshot](desktop.png). Portable SHA-256: `e71224e45717d65c0df2e823068f5a76e8f95b73be17cf8a044bc4de9db1fb3a`.

Independent reviews used the pre-task commit `6bf6d7e20462d33f9e61c468c297be098686c8de` and staged diff. Standards review: no findings. Specification review: one missing per-candidate match-count finding, resolved and independently rechecked. The focused browser checks, lint, typecheck, native build, actual Windows restart/preservation check and portable verification passed after that change.

## Remaining gates

This slice provides source-supported simple forms. Ordinary compound generation (#41), English meanings (#42), pronominal constructions (#43), expression normalization (#44), colors (#45), saved ordering (#46), dictionary navigation (#47) and conjugation playback (#48) retain their own scope. The native check does not impose network blocking or prove a fresh complete offline package; #49 retains that combined acceptance. ARM64/macOS execution and formal release readiness were not tested here.
