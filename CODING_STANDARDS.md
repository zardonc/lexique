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

## Database fixtures

Batch bulk SQLite fixture writes in a transaction when assertions depend on
the resulting data. Commit setup before invoking the code under test. Keep
individual commits when commit boundaries, recovery, or durability are the
behavior being tested.

Review fixture setup cost as part of the existing backend execution budget.
