# Issue tracker: GitHub

Issues and specs live in GitHub Issues for `zardonc/lexique`.
Use the `gh` CLI from this repository.

## Operations

- Create: `gh issue create --title "..." --body-file <file>`.
- Read: `gh issue view <number> --json number,title,body,labels,comments`.
- List: `gh issue list --state open --json number,title,body,labels,assignees`.
- Comment: `gh issue comment <number> --body-file <file>`.
- Add or remove labels: `gh issue edit <number> --add-label "..."`
  or `--remove-label "..."`.
- Close: `gh issue close <number>`.

Use UTF-8 files with actual newlines for multiline bodies and comments.
Infer the repository from the Git remote.

When a skill says "publish to the issue tracker", create a GitHub issue.
When it says "fetch the relevant ticket", read the GitHub issue.

## Pull requests as a triage surface

**PRs as a request surface: no.**

## Parent tickets and dependencies

Use GitHub sub-issues and native issue dependencies when available.
Dependency API calls require the issue's numeric database id, not its
issue number or node id.

If sub-issues are unavailable, add `Part of #<parent>` to the child body
and link the child in the parent's task list.
If dependencies are unavailable, add `Blocked by: #<number>` to the child.
A ticket is unblocked when all its blocking issues are closed.

## Wayfinding

Keep the map in one issue labelled `wayfinder:map`.
Link child issues to it and label each `wayfinder:<type>`, where type is
`research`, `prototype`, `grilling`, or `task`.

The frontier is the first open, unassigned, unblocked child in map order.
Claim it by assigning yourself. On resolution, record the answer, close
the child, and add a summary and link to the map's Decisions-so-far.
