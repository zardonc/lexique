# Development tooling follow-ups

Native destination preflight, CI trigger filtering and desktop test
precompilation are implemented. The review guidance below applies now;
bounded output tooling remains pending. The existing lint, typecheck, test
and documentation jobs remain in place.

## Desktop test compilation

The [desktop library test command](../../package.json) compiles the release
test binary before starting its eight timed partitions. A compilation failure
stops the command before execution. Initial compilation stays outside the
runner's hard 60-second limit; each test partition retains that limit.
CI keeps its explicit precompilation steps for separate build logs.

## Native crash diagnosis

For a WebView2 `ProcessFailed` access violation, locate the host lifecycle log
and the retained profile's `Crashpad/reports/*.dmp` files. Native fixtures keep
the profile below `Lexique-data/webview/EBWebView/`; the runtime can add a nested
`EBWebView` directory. Application event logs and system WER directories may
have no corresponding report.

Use a debugger or minidump parser to map the faulting thread's exception address
to the loaded module ranges. Record the exception code, module and offset; use
the stack or a controlled comparison when the address alone leaves attribution
unclear. An injected module's presence establishes injection, not causation.
Keep full dumps in their owning ignored evidence directories and publish only
the diagnostic facts needed for the verification claim.

The [two RTSS crash reports](../note-interchange-verification.md#rtss-crash-attribution)
both faulted in `RTSSHooks64.dll + 0x1490AF`. The owner excluded those two
environment failures from PR #192 blockers. That disposition covers those
reports; it does not classify other renderer failures or V03 timing overruns.

## Reviewer availability

When a rate limit or tool failure terminates a reviewer, close that reviewer
and complete its axis locally against the same fixed diff, standards and spec.
Keep an unaffected reviewer running. Retain separate Standards and Spec
findings, and state which axes were completed locally. Passing CI is test
evidence, not a replacement for an unfinished source review. Recheck the PR
head before delivering the combined result.

## Native destination preflight

The [tracked fixture tests](../../scripts/test-native-d2-fixtures.mjs) exercise
the destination guards in
[native-24-harness.mjs](../../apps/desktop/e2e/native-24-harness.mjs) and
[native-backup-state.mjs](../../apps/desktop/e2e/native-backup-state.mjs).
They run through the existing [tooling checks](../../package.json) and before
portable packaging, without starting the application.

The tests cover approved fixture descendants, including
`grammar-classification`, and reject directory roots, sibling prefixes and
paths resolving outside them. Each new content batch needs its approved
destinations covered before a native build.

## CI triggers

[ci.yml](../../.github/workflows/ci.yml) runs automatic full matrices for
pull requests and pushes to `main`. Manual dispatch remains available for
branch verification; a branch push with an open PR starts only the
pull-request matrix.

The separate ARM timing spike remains unexplained; duplicate runs have not
been established as its cause. A passing rerun establishes recovery without
identifying the cause of a timing overrun.

## Bounded output tooling

Prepare a reusable capture-and-selection helper for command and tool results.
Existing local guidance already requires bounded output; oversized batches
still recurred during the session.

Completion means the helper retains complete responses, includes each exit
status, selects fields or line ranges before display, and keeps the combined
serialized display, including labels and wrappers, within 6,000 characters.
Capture complete CLI output under ignored `target/` paths before parsing;
per-command token limits do not bound a combined tool batch.
An oversized selection stays available for a
smaller read without repeating the command. Verify successful commands,
failures and oversized results through the same path. Keep the implementation
in existing development tooling and use its example in the local environment
guide rather than adding another prose warning.
