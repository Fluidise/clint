#!/usr/bin/env bash
set -euo pipefail

# install-hooks.sh - Installs local Git hooks dogfooding fluidise-clint

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "Error: .git/hooks directory not found at $HOOKS_DIR" >&2
    exit 1
fi

COMMIT_MSG_HOOK="$HOOKS_DIR/commit-msg"

cat > "$COMMIT_MSG_HOOK" << 'EOF'
#!/usr/bin/env bash
# Git commit-msg hook dogfooding fluidise-clint
set -euo pipefail

COMMIT_MSG_FILE="$1"
# Git invokes commit-msg from the non-bare worktree root; manual use requires the CLI.
REPO_ROOT="$PWD"

# Prefer compiled release binary for maximum speed, fall back to debug or cargo run
if [ -x "$REPO_ROOT/target/release/fluidise-clint" ]; then
    LINTER="$REPO_ROOT/target/release/fluidise-clint"
elif [ -x "$REPO_ROOT/target/debug/fluidise-clint" ]; then
    LINTER="$REPO_ROOT/target/debug/fluidise-clint"
else
    # Build debug binary quietly if not present
    (cd "$REPO_ROOT" && cargo build --quiet --bin fluidise-clint)
    LINTER="$REPO_ROOT/target/debug/fluidise-clint"
fi

if ! "$LINTER" --file "$COMMIT_MSG_FILE" --defaults --human; then
    echo "" >&2
    echo "❌ Commit rejected by fluidise-clint." >&2
    echo "Please update your commit message to adhere to Conventional Commits." >&2
    exit 1
fi
EOF

chmod +x "$COMMIT_MSG_HOOK"
echo "✅ Git commit-msg hook installed successfully at $COMMIT_MSG_HOOK"
