# Development tooling follow-ups

These changes are pending implementation following review of
[PR #180](https://github.com/zardonc/lexique/pull/180).
The existing lint, typecheck, test and documentation jobs remain in place.

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
display within 6,000 characters. An oversized selection stays available for a
smaller read without repeating the command. Verify successful commands,
failures and oversized results through the same path. Keep the implementation
in existing development tooling and use its example in the local environment
guide rather than adding another prose warning.
