# Development tooling follow-ups

The review guidance below applies now. Native destination preflight, CI trigger
filtering and bounded output tooling remain pending following
[PR #180](https://github.com/zardonc/lexique/pull/180) and
[PR #192](https://github.com/zardonc/lexique/pull/192).
The existing lint, typecheck, test and documentation jobs remain in place.

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

Add a focused check against the destination guards in
[native-24-harness.mjs](../../apps/desktop/e2e/native-24-harness.mjs) and
[native-backup-state.mjs](../../apps/desktop/e2e/native-backup-state.mjs), wired
into the existing [tooling checks](../../package.json) before native packaging.
The merge verification exercised 110 checks, but its script is local evidence
under ignored `target/`, rather than a tracked regression.

Completion means both guards accept descendants of every explicitly approved
fixture directory, including `grammar-classification`, and reject the directory
root itself, sibling prefixes and paths resolving outside it. Each new content
batch needs its approved destinations covered before a native build. Preserve
the explicit ownership boundaries when implementing the check.

## CI triggers

The unfiltered push and pull-request triggers in
[ci.yml](../../.github/workflows/ci.yml) ran two full eleven-job matrices for
the same PR #180 head. Limit automatic full matrices to pull requests and
pushes to `main`; retain manual dispatch for branch verification.

Completion means pull-request required checks and post-merge validation still
run, manual branch dispatch works, and a branch push with an open PR starts
only the pull-request matrix. The separate ARM timing spike remains unexplained;
duplicate runs have not been established as its cause. Both
[PR CI](https://github.com/zardonc/lexique/actions/runs/44663885434) and
[push CI](https://github.com/zardonc/lexique/actions/runs/44663892957) passed
on `245e76b99c17703ed6d27c550a0e29724d89ed15`, with an ARM retry in push CI.

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
