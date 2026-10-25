# Agent guidance

## Scope and decisions

- Make the smallest complete change for the requested behavior; reuse existing modules before adding dependencies or abstractions.
- Preserve pre-existing edits and user learning data. Before discarding work, rewriting Git history, or applying destructive data migrations, explain the impact and obtain explicit approval.
- Before feature work, read `docs/product.md` and `docs/roadmap.md`. Before adding resources or service integrations, read `docs/development-readiness.md` for admission and milestone gates.
- Before changing module boundaries, persistence, IPC, or credential handling, read `docs/architecture.md`.
- **unfamiliar features**: don't AI-generate from scratch. Reference active, well-licensed GitHub projects for proven designs and known pitfalls. Never copy unverified or incompatible code.

## Code and safety

- Follow the existing TypeScript ES modules and Rust conventions. Treat ESLint, TypeScript configuration, and rustfmt as the authority for enforced style and complexity limits.
- Prefer `readonly` interfaces and immutable React state updates; allow local mutation where Rust ownership or a transaction makes it appropriate.
- Inject platform, storage, and network implementations at existing boundaries so business logic remains testable and the shared core remains portable.
- Preserve error context and fix the cause. Keep failures visible; do not downgrade diagnostics or return simulated success for unavailable capabilities.
- Validate untrusted files, IPC payloads, and service responses at entry points. Use parameterized SQL and isolate external dictionary HTML from native privileges.
- Keep secrets, tokens, personal learning data, and private service settings out of tracked files, frontend bundles, and logs. Build and test configuration belongs in version control; `VITE_*` values are public frontend data.

## Development workflow

- Use the pnpm version in `package.json` and the Rust toolchain in `rust-toolchain.toml`. Install with `pnpm install --frozen-lockfile`; use locked Cargo commands and change lockfiles only for intentional dependency changes.
- On Windows, use PowerShell 7. Prefer the native file-editing / `apply_patch` tool; if a safe fallback is needed, verify the resulting diff.
- Use `rg` for text and file searches. For symbol or call-path exploration, use the existing CodeGraph index when available, then verify findings against current source; creating or rebuilding an index requires a user request.
- Keep the diff within the task. When asked to commit, use `<type>: <description>`; PR descriptions state the resulting behavior and relevant validation.
- Stop services and test processes started for the task after verification; preserve unrelated running processes.

## Verification and delivery

- Test changed behavior, boundary cases, and failure paths. For a bug fix, first reproduce the failure with a focused regression check. Choose unit, integration, or E2E tests according to the boundary being verified; avoid redundant tests and excessive mocking.
- Use the checks documented in `README.md`: frontend changes require lint, typecheck, and build; Rust changes require formatting, compilation, and relevant tests. UI flows need the applicable browser or native check. Documentation-only changes need diff and link checks.
- Precompile the storage tests with `pnpm rust:test:build`, then run `pnpm test:rust`. Preserve the runner's hard 60-second execution timeout when extending backend tests; initial compilation is separate.
- Record commands and passed, failed, or skipped results. Browser preview does not prove native storage; compilation does not prove platform execution, offline behavior, or release readiness. Maintain `docs/verification.md` when those readiness claims change.
- Deliver the result concisely with material limitations and unrun checks.

## Agent skills

### Issue tracker

Issues and specs live in GitHub Issues for `zardonc/lexique`.
Read `docs/agents/issue-tracker.md` before tracker operations.

### Triage labels

Use the five canonical triage labels.
Read `docs/agents/triage-labels.md` before triaging or changing labels.

### Domain docs

Use a single-context layout.
Read `docs/agents/domain.md` before exploring domain concepts or decisions.
