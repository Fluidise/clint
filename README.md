# fluidise-clint

Standalone, deterministic commit-message validation designed for AI agents and automated workflows.

`fluidise-clint` validates commit messages against conventional commit standards with strict operational guarantees: default structured JSON output, bounded resource consumption, deterministic exit codes, and zero side effects (no file modifications, no hook installations, no remote network calls).

## Table of Contents

- [Quick Start](#quick-start)
- [Key Features](#key-features)
- [Installation](#installation)
- [CLI Usage](#cli-usage)
  - [Options](#options)
  - [Exit Codes](#exit-codes)
  - [JSON Output Schema](#json-output-schema)
  - [Human-Readable Output](#human-readable-output)
- [Rust Library](#rust-library)
- [Configuration](#configuration)
  - [Discovery](#discovery)
  - [Schema (`fluidise.toml`)](#schema-fluidisetoml)
  - [Default Types](#default-types)
- [Operational Guarantees and Boundaries](#operational-guarantees-and-boundaries)
  - [Strict Resource Limits](#strict-resource-limits)
  - [Non-Mutation Invariant](#non-mutation-invariant)
  - [Git Hook Considerations](#git-hook-considerations)
- [Documentation and License](#documentation-and-license)

## Quick Start

Pass a commit message via standard input to receive structured JSON output on standard output:

```sh
printf 'feat: add search\n' | fluidise-clint --stdin
```

Output (`stdout`, exit code `0`):

```json
{"configuration":{"source":"defaults","unrecognized_sections":[],"used_sections":[]},"diagnostics":[],"diagnostics_complete":true,"schema_version":1,"status":"valid"}
```

When a commit message violates rules:

```sh
printf 'invalid message\n' | fluidise-clint --stdin
```

Output (`stdout`, exit code `1`):

```json
{"configuration":{"source":"defaults","unrecognized_sections":[],"used_sections":[]},"diagnostics":[{"code":"commit.header.invalid","message":"Expected type, optional scope/!, colon, space and a nonblank description.","source":"message","span":{"end":15,"start":0}}],"diagnostics_complete":true,"schema_version":1,"status":"invalid"}
```

## Key Features

- **Agent-First Design**: Emits machine-readable JSON (`schema_version: 1`) by default for automated parsing and programmatic agent decisions.
- **Deterministic and Safe**: Strict memory and output buffers. Never modifies files, never installs hooks, never stages or commits changes.
- **Zero Network or Shell Subprocesses**: Operates entirely in-process without network access, external services, or shell execution.
- **Dual Interface**: Available as a standalone CLI executable and as an embeddable Rust crate.
- **Configurable Rules**: Optional `fluidise.toml` for project-specific types, scopes, and title length limits.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/) toolchain (MSRV 1.85, edition 2024).

### Building from Source

Clone the repository and build using Cargo:

```sh
# Build optimized release binary
cargo build --release --locked

# The executable will be at target/release/fluidise-clint
```

To install directly into Cargo's binary path (`~/.cargo/bin`):

```sh
cargo install --path . --locked
```

No external runtime dependencies (such as Node.js or Python) are required to execute the compiled binary.

## CLI Usage

```text
fluidise-clint (--file <path> | --stdin) [--config <path>] [--human]
```

### Options

| Option | Description |
| :--- | :--- |
| `--stdin` | Read the commit message from standard input until EOF. |
| `--file <path>` | Read the commit message from a regular file. Path `-` is treated as a literal file named `-`. Paths beginning with `-` require `--file=--name` or `--file ./--name`. |
| `--config <path>` | Use the specified configuration file instead of discovering `fluidise.toml` at the Git root. |
| `--human` | Opt-in plain English text output (status on `stdout`, diagnostics on `stderr`). Argument errors still return JSON. |
| `--help` | Print help information and exit. Must be used alone. |
| `--version` | Print version information and exit. Must be used alone. |

> [!NOTE]
> Exactly one input source (`--stdin` or `--file`) is required. Empty option values, short aliases (e.g., `-h`), repeated arguments, unknown options, and positional arguments are strictly rejected with an argument error.

### Exit Codes

Automated callers must inspect process exit codes and decode the JSON payload:

| Exit Code | Status | Meaning |
| :---: | :--- | :--- |
| `0` | `valid` / `ignored` | The commit message is valid according to conventional rules, or matches an ignored pattern (e.g. merge/revert). |
| `1` | `invalid` | The commit message violates formatting, type, scope, or length rules. |
| `2` | `error` | Operational failure: invalid arguments, file read failure, configuration parse error, or exceeded size limits. |

### JSON Output Schema

JSON output adheres to `schema_version: 1`:

```json
{
  "schema_version": 1,
  "status": "valid",
  "diagnostics_complete": true,
  "diagnostics": [],
  "configuration": {
    "source": "defaults",
    "used_sections": [],
    "unrecognized_sections": []
  }
}
```

When a message is ignored, a `"reason"` field is included (e.g. `"merge"` or `"revert"`).

Each diagnostic entry contains:
- `code`: Stable string identifier (e.g., `commit.header.invalid`, `commit.title.too_long`, `commit.temporary.not_allowed`).
- `message`: Diagnostic explanation (maximum 512 code points).
- `source`: Category (`message`, `configuration`, `arguments`, or `tool`).
- `span`: Optional `{ "start": <usize>, "end": <usize> }` zero-based, half-open byte range within the original message.
- `field`: Optional path array targeting a configuration key.

### Human-Readable Output

When `--human` is provided, output is delivered as plain text without ANSI color codes:

```sh
printf 'feat: add search\n' | fluidise-clint --stdin --human
# stdout: valid (exit code 0)
```

```sh
printf 'invalid message\n' | fluidise-clint --stdin --human
# stdout: invalid (exit code 1)
# stderr: commit.header.invalid [0..15): Expected type, optional scope/!, colon, space and a nonblank description.
```

## Rust Library

The `fluidise-clint` crate exposes a pure, in-memory validation engine without filesystem access, process exits, or JSON serialization overhead.

Add the dependency in `Cargo.toml`:

```toml
[dependencies]
fluidise-clint = { path = "." }
```

Example usage:

```rust
use fluidise_clint::{check_message, prepare_policy, Status};

fn main() {
    // Compile validation policy (None uses defaults; Some(str) parses TOML)
    let policy = prepare_policy(None).expect("default policy is valid");

    // Check a commit message
    let result = check_message("feat(cli): add quiet mode", &policy);

    match result.status {
        Status::Valid => println!("Commit message is valid!"),
        Status::Ignored => println!("Ignored: {:?}", result.reason),
        Status::Invalid => {
            eprintln!("Invalid commit message ({} diagnostics):", result.diagnostics.len());
            for d in &result.diagnostics {
                eprintln!("  [{}] {}", d.code, d.message);
            }
        }
        Status::Error => eprintln!("Operational error occurred"),
    }
}
```

`prepare_policy` produces an immutable `Policy`. Mutating rules after preparation is prevented by Rust's type system.

## Configuration

### Discovery

When `--config` is omitted:
1. `fluidise-clint` searches for `fluidise.toml` at the root of the current Git repository (navigating upward from the current working directory).
2. If executed outside a Git repository, no parent or user home directories are searched; defaults are used.

When `--config <path>` is specified:
- The path is resolved relative to the invocation working directory and replaces discovery.

### Schema (`fluidise.toml`)

Only the `[commit]` section is semantically evaluated; unrecognized sections are ignored but reported in `configuration.unrecognized_sections`.

```toml
[commit]
version = 1
allowed-types = ["feat", "fix", "docs", "chore"]
allowed-scopes = ["cli", "engine", "parser"]
max-title-length = 72
ignore-merges = true
ignore-reverts = true
```

- `version` *(integer, required)*: Must be `1`.
- `allowed-types` *(array of strings, optional)*: Replaces (does not extend) the default types list. Types must start with an ASCII lowercase letter and contain only lowercase ASCII letters, digits, or hyphens.
- `allowed-scopes` *(array of strings, optional)*: Exact, case-sensitive scope strings. When set, only listed scopes are permitted.
- `max-title-length` *(positive integer, optional)*: Maximum allowed title line length, counted in Unicode code points.
- `ignore-merges` *(boolean, optional, default: `true`)*: Ignores exact standard Git merge commit messages.
- `ignore-reverts` *(boolean, optional, default: `true`)*: Ignores exact standard Git revert commit messages.

Temporary commit messages beginning with `fixup!` or `squash!` are always rejected with `commit.temporary.not_allowed`.

### Default Types

When `allowed-types` is not specified, the following conventional types are accepted:

`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.

## Operational Guarantees and Boundaries

### Strict Resource Limits

To ensure safe execution within autonomous agent runtimes, resource boundaries are strictly enforced:

- **Input Cap**: Maximum `1,048,576` bytes (1 MiB) for both commit message input and configuration file content. Reads terminate after one probe byte beyond the limit.
- **Output Cap**: Maximum `262,144` bytes (256 KiB) for generated JSON output. If exceeded, the payload is replaced with an operational error (`tool.output.limit_exceeded`).
- **Diagnostic Cap**: At most 20 diagnostics are collected. If additional violations exist, collection halts and `diagnostics_complete` is set to `false`.
- **Diagnostic Message Length**: Capped at 512 Unicode code points.

### Non-Mutation Invariant

`fluidise-clint` adheres to a strict read-only model:
- Never writes, alters, or stages files.
- Never mutates commit messages.
- Never installs or configures Git hooks.
- Never triggers Git commits.

### Git Hook Considerations

`fluidise-clint` can be invoked inside a `commit-msg` hook by passing the path of the message file:

```sh
fluidise-clint --file "$1"
```

> [!IMPORTANT]
> `fluidise-clint` checks the exact byte content provided. It does not strip Git comments (`#`), scissors lines (`# ------------------------ >8 ------------------------`), or diff comments generated by `git commit -v`. Because Git strips those sections *after* `commit-msg` succeeds, linting the raw file may produce false diagnostics if verbose diffs or comments are present.

## Documentation and License

- Additional local development, test commands, and rehearsal packaging notes are documented in [USAGE.md](USAGE.md).
- Licensed under the [MIT License](LICENSE). Copyright (c) 2026 Acrazie.
