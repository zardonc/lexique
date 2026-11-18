# Coding standards

## Review inputs

Include these inputs in each reviewer handoff:

- Issue/spec reference and PR URL when available.
- Absolute path to the reviewed checkout, plus diff base and head SHA.
- Changes covered: committed, staged, unstaged, or an explicit combination.
- For staged or unstaged changes, exact diff command and absolute path to the
  retained patch.
- Absolute root of retained execution evidence, plus its tested SHA and platform.

Ignored logs and snapshots stay in their owning worktree; a detached review
checkout contains tracked files. Locate evidence at the supplied root and
match its SHA and platform to the verification claim.

When curated resource data dominates the diff, supply a separate patch for
runtime code and tests alongside the full retained diff. Include structural
preservation and source-review evidence at its owning worktree path.

## Persisted values and rollback

For changes to persisted shapes or allowed values, trace stored data through
notebook lists, review cards and snapshot import/restore, including any other
affected readers. Describe whether the previous reader accepts new records
and whether rejecting one record fails a whole response. Record supported
resource and application rollback paths in the PR, including reader-version
and backup requirements.

## Database fixtures

Batch bulk SQLite fixture writes in a transaction when assertions depend on
the resulting data. Commit setup before invoking the code under test. Keep
individual commits when commit boundaries, recovery, or durability are the
behavior being tested.

Review fixture setup cost as part of the existing backend execution budget.

## Timing and contention tests

Define the operation each timer measures and its clock boundaries. Keep fixture
and worker readiness outside the measured operation. For contention tests,
require observable lock contention for every counted sample. Enforce the
specified timing limits on every executed attempt, including attempts without
contention.
