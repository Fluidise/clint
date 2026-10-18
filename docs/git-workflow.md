# Git Workflow & Delivery Policy

This document defines the authoritative Git and GitHub workflow for the `fluidise-clint` repository (`Fluidise/clint`).

> **Note on Platform Migration**: All development, branching, and delivery are performed directly in **GitHub**. The legacy `entire.io` platform is no longer in use. All pushes and pull requests target `github.com/Fluidise/clint`.

---

## 1. Branching Strategy

We follow a **Trunk-Based Development** model with short-lived feature branches:

- **Base branch**: `main` (production-ready baseline).
- **Work branches**:
  - `feat/<short-description>`: New capabilities or functional improvements.
  - `fix/<short-description>`: Bug fixes and issue resolutions.
  - `chore/<short-description>`: Maintenance, refactoring, dependencies, and tooling.
  - `docs/<short-description>`: Documentation updates.

### Branch Freshness

Before creating a new branch or worktree, ensure your local `main` is up to date:

```bash
git checkout main
git pull origin main
```

---

## 2. Commit Message Conventions

Commit messages must strictly follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```text
<type>(<optional scope>): <description>

[optional body]

[optional footer(s)]
```

### Allowed Types
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to our CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

### Local Dogfooding Validation

We dogfood our own tool `fluidise-clint` via a native Git `commit-msg` hook:

```bash
# Install local hooks (one-time setup)
./scripts/install-hooks.sh
```

Every commit is validated against `fluidise-clint` before Git records it.

---

## 3. Pre-Commit Quality Checks

Before committing or pushing, verify code quality locally:

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Linter check
cargo clippy --all-targets -- -D warnings

# 3. Test suite
cargo test --all-targets
```

---

## 4. Git Delivery Policy (Pushes & Pull Requests)

1. **Targeted Staging**: Review staged diffs before committing:
   ```bash
   git diff --cached --check
   git diff --cached --stat
   ```
2. **Push Destination**: Always push to GitHub (`origin`):
   ```bash
   git push -u origin <branch-name>
   ```
3. **Pull Requests**:
   - Open Pull Requests against `main`.
   - Ensure the CI pipeline passes on GitHub Actions (`.github/workflows/ci.yml`).
   - GitHub automatically deletes merged head branches (`delete_branch_on_merge: true`).

---

## 5. Parallel Work with Git Worktrees (Situational)

For working on parallel tasks or isolating bug fixes without switching working directories, situational worktrees are supported in `.worktrees/` (ignored by Git):

```bash
# Create a new worktree
./scripts/worktree.sh add feat/my-feature

# List active worktrees
./scripts/worktree.sh list

# Clean up worktree after merge
./scripts/worktree.sh remove feat/my-feature
```
