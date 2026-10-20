# AI Agent Instructions for `fluidise-clint`

## Project Overview

`fluidise-clint` is an agent-first, deterministic commit-message linter written in Rust. It enforces Conventional Commits with bounded memory consumption, strict JSON output by default, and zero external side effects.

- **Stack**: Rust 2024 edition (MSRV 1.99.1), Cargo.
- **Remote & Delivery**: All work, branches, and pushes go directly to **GitHub** (`Fluidise/clint`). The repository has fully transitioned away from `entire.io`.

## Authoritative Git Policy

Follow the rules in [docs/git-workflow.md](docs/git-workflow.md):
- **Branching**: Trunk-Based (`main` base; short-lived branches `feat/*`, `fix/*`, `chore/*`, `docs/*`).
- **Commits**: Conventional Commits convention enforced. Dogfooding via `fluidise-clint`.
- **Co-authorship**: Never add `Co-authored-by: Claude` or similar co-author mentions in commit messages.
- **Verification before commit/push**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --all-targets`

## Developer Tooling

- Local Git hook setup: `./scripts/install-hooks.sh`
- Situational worktrees: `./scripts/worktree.sh {add|remove|list}`
