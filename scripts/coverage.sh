#!/usr/bin/env bash
set -euo pipefail

# coverage.sh - Local source-based code coverage runner using cargo-llvm-cov

REPO_ROOT="$(git rev-parse --show-toplevel)"
OUTPUT_DIR="$REPO_ROOT/target/llvm-cov"

usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --html       Generate HTML report and place in target/llvm-cov/html"
    echo "  --lcov       Generate lcov.info file in target/llvm-cov/lcov.info"
    echo "  --open       Generate HTML report and open it in the default browser"
    echo "  --summary    Display summary text in terminal (default)"
    echo "  --help       Show this help message"
    exit 0
}

for arg in "$@"; do
    if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
        usage
    fi
done

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "❌ cargo-llvm-cov is not installed on this system." >&2
    echo "" >&2
    echo "To install cargo-llvm-cov:" >&2
    echo "  cargo install cargo-llvm-cov --locked" >&2
    echo "  # or on macOS via Homebrew:" >&2
    echo "  brew install cargo-llvm-cov" >&2
    exit 1
fi

MODE="summary"

while [ $# -gt 0 ]; do
    case "$1" in
        --html)
            MODE="html"
            shift
            ;;
        --lcov)
            MODE="lcov"
            shift
            ;;
        --open)
            MODE="open"
            shift
            ;;
        --summary)
            MODE="summary"
            shift
            ;;
        --help|-h)
            usage
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            ;;
    esac
done

mkdir -p "$OUTPUT_DIR"

case "$MODE" in
    html)
        echo "Generating HTML coverage report..."
        cargo llvm-cov --all-targets --html --output-dir "$OUTPUT_DIR/html"
        echo "✅ HTML coverage report written to: $OUTPUT_DIR/html/index.html"
        ;;
    open)
        echo "Generating and opening HTML coverage report..."
        cargo llvm-cov --all-targets --open
        ;;
    lcov)
        echo "Generating LCOV coverage report..."
        cargo llvm-cov --all-targets --lcov --output-path "$OUTPUT_DIR/lcov.info"
        echo "✅ LCOV report written to: $OUTPUT_DIR/lcov.info"
        ;;
    summary)
        echo "Running test suite with source coverage..."
        cargo llvm-cov --all-targets
        ;;
esac
