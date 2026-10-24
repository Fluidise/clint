<p align="center">
  <img src="docs/assets/banner.png" alt="Fluidise-Clint — forêt prune et lavande ouverte sur une clairière" width="100%">
</p>

<h1 align="center">fluidise-clint</h1>

<p align="center">
  <strong>Standalone, deterministic commit-message validation designed for AI agents and automated workflows.</strong>
</p>

<p align="center">
    <a href="https://github.com/Fluidise/clint/releases">
        <img src="https://img.shields.io/github/v/release/Fluidise/clint?style=flat&labelColor=24141E&color=603840&logo=GitHub&logoColor=white"></a>
    <a href="https://www.rust-lang.org">
        <img src="https://img.shields.io/badge/rustc-1.99.1%2B-informational?style=flat&labelColor=24141E&color=603840&logo=Rust&logoColor=white"></a>
    <br>
    <a href="https://github.com/Fluidise/clint/actions/workflows/ci.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/Fluidise/clint/ci.yml?branch=main&style=flat&labelColor=24141E&color=603840&logo=GitHub%20Actions&logoColor=white"></a>
    <a href="LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-informational?style=flat&labelColor=24141E&color=603840"></a>
    <br>
</p>

`fluidise-clint` validates commit messages against conventional commit standards with strict operational guarantees: default structured JSON output, bounded resource consumption, deterministic exit codes, and zero side effects (no file modifications, no hook installations, no remote network calls).

## Table of Contents

- [Quick Start](#quick-start)
- [Key Features](#key-features)
- [Benchmarks at a Glance](#benchmarks-at-a-glance)
- [Installation](#installation)
- [CLI Usage](#cli-usage)
  - [Options](#options)
  - [Exit Codes](#exit-codes)
  - [JSON Output Schema](#json-output-schema)
  - [Human-Readable Output](#human-readable-output)
- [Rust Library](#rust-library)
- [Configuration](#configuration)
  - [Policy selection](#policy-selection)
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
printf 'feat: add search\n' | fluidise-clint --stdin --defaults
```

Output (`stdout`, exit code `0`):

```json
{"configuration":{"source":"defaults","unrecognized_sections":[],"used_sections":[]},"diagnostics":[],"diagnostics_complete":true,"schema_version":1,"status":"valid"}
```

When a commit message violates rules:

```sh
printf 'invalid message\n' | fluidise-clint --stdin --defaults
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

## Benchmarks at a Glance

Correctness comes before speed. The [benchmark protocol and audit](docs/benchmarks.md)
separate specification checks, project policy, operational limits and CLI latency.
The versioned oracle includes public commit messages and targeted synthetic cases;
its expected results are justified independently of the tools being measured.

Historical cross-tool rankings are withdrawn: their raw evidence was unavailable,
and newer local measurements used different workloads and revisions. A 1 MiB input
limit is not an RSS cap; a timeout does not establish ReDoS or an infinite loop.
CLI throughput does not establish an improvement in agent task completion.

The [suite](scripts/benchmark_suite.py) checks correctness before optional timing.
Only tools passing the full shared oracle may enter that comparison. Missing tools,
policy differences and failed checks remain visible rather than becoming speed wins.
Published results apply only to the recorded binaries, configurations and environment.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/) toolchain (MSRV 1.99.1, edition 2024).

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
fluidise-clint (--file <path> | --stdin) (--config <path> | --defaults) [--human]
```

### Options

| Option | Description |
| :--- | :--- |
| `--stdin` | Read the commit message from standard input until EOF. |
| `--file <path>` | Read the commit message from a regular file. Path `-` is treated as a literal file named `-`. Paths beginning with `-` require `--file=--name` or `--file ./--name`. |
| `--config <path>` | Read only the specified configuration file; no Git discovery. |
| `--defaults` | Use built-in rules without a configuration file or Git discovery. |
| `--human` | Opt-in plain English text output (status on `stdout`, diagnostics on `stderr`). Argument errors still return JSON. |
| `--help` | Print help information and exit. Must be used alone. |
| `--version` | Print version information and exit. Must be used alone. |

> [!NOTE]
> Exactly one input source (`--stdin` or `--file`) and exactly one policy source (`--config <path>` or `--defaults`) are required. Empty option values, short aliases (e.g., `-h`), repeated arguments, unknown options, and positional arguments are strictly rejected with an argument error.

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
printf 'feat: add search\n' | fluidise-clint --stdin --defaults --human
# stdout: valid (exit code 0)
```

```sh
printf 'invalid message\n' | fluidise-clint --stdin --defaults --human
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

### Policy selection

Every validation explicitly selects its rules:

- `--config <path>` reads that file, relative to the invocation working directory.
  Missing, unreadable or invalid files are errors; no fallback occurs.
- `--defaults` uses built-in rules without reading configuration files.

There is no Git-root discovery or parent/home configuration search, and the linter
never launches Git. Both or neither selection is an argument error (JSON, exit 2).
Help and version remain standalone commands.

**Breaking change:** previous calls without a policy selection must add
`--defaults` or the intended `--config <path>`. A root `fluidise.toml` is no longer
automatically loaded. See [ADR 0002](docs/adr/0002-explicit-policy-selection.md)
and the [throughput report](docs/agent-throughput.md).

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
fluidise-clint --file "$1" --defaults
```

> [!IMPORTANT]
> `fluidise-clint` checks the exact byte content provided. It does not strip Git comments (`#`), scissors lines (`# ------------------------ >8 ------------------------`), or diff comments generated by `git commit -v`. Because Git strips those sections *after* `commit-msg` succeeds, linting the raw file may produce false diagnostics if verbose diffs or comments are present.

## Documentation and License

- Additional local development, test commands, and rehearsal packaging notes are documented in [USAGE.md](USAGE.md).
- Comprehensive 5-competitor benchmarks in [docs/benchmarks.md](docs/benchmarks.md).
- Agent loop and throughput benchmarks in [docs/agent-throughput.md](docs/agent-throughput.md).
- Engine optimization benchmarks and methodology in [docs/benchmark-loop.md](docs/benchmark-loop.md).
- Architectural and design decisions in [docs/adr/](docs/adr/) (see [ADR 0001: Évaluation des Écarts Fonctionnels Face aux Concurrents et Pistes d'Évolution](docs/adr/0001-competitor-feature-gaps-and-evolution-tracks.md) and [ADR 0002: Sélection Explicite de la Politique CLI](docs/adr/0002-explicit-policy-selection.md)).
- Licensed under the [MIT License](LICENSE). Copyright (c) 2026 Acrazie.
