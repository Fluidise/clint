# CLI validation latency: local baseline, 2026-10-23

## Approved measurement scope

The objective is lower real hook latency, not a ranking alone. Public behavior,
JSON defaults, diagnostics, exit codes, compatibility and bounded resource handling
must remain unchanged. The primary scenario is a small message in an ordinary Git
repository, with automatic configuration discovery, with and without configuration.
No optimization was implemented during this measurement phase.

CLI validation latency includes process launch through process completion after
output, including configuration discovery, message reading, validation and output.
The historical competitive numbers in `benchmarks.md` are not this baseline.

## Environment and provenance

- Source HEAD: `c742e55f7b2d935fd31ff0fb5efd4166d0f1a178`; no Rust source changes.
- platform: `macOS-26.7.1-arm64-arm-64bit-Mach-O`.
- machine: `arm64`.
- python: `3.14.3`.
- rustc: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- git: `git version 2.53.0`.
- Build: `cargo build --release --locked --offline`, existing release profile.
- Frozen copies of the installed competitors; no installations or updates.
- `fluidise-clint 0.1.0` SHA-256: `f23aea0682bd350468aa4190bbcc92839cf2497e3816fea12fda86b1a2f43d49`.
- `convco 0.7.2` SHA-256: `627f499a364caacb4dccd0b711c445197a22e0704d838fd1069c291836502653`.
- `cog 7.0.0` SHA-256: `406cb16a5f919b6114e0682d6de36b8b95204b811bab6cd50533d11ad59ec937`.

## Protocol

Two rounds of 60 samples per case and command, each preceded by five warmups.
Command order is shuffled deterministically for every sample. Every invocation is
a new process, but filesystem caches are warm: this is not a cold-cache benchmark.
The clock surrounds Python `subprocess.run`, including process creation, pipes,
input transfer, output capture and waiting. No shell or pipeline is timed.

Each fixture is an empty ordinary Git repository in a temporary directory, with
no commits or installed hooks. Commands run from its root. Personal/system Git
configuration is disabled; inherited `GIT_*`, `CONVCO_*` and `COG_*` variables are
removed. Outputs are captured with `NO_COLOR=1`, `TERM=dumb` and `LC_ALL=C`.
This is controlled direct invocation of the hook workload, not a timed `git commit`
or the user's actual installed hook wrapper.

Messages are `feat: improve validation\n` (25 bytes) and
`fix something without type separator\n` (37 bytes).
Configured fixtures contain `fluidise.toml` with `[commit]` and `version = 1`,
`.convco` with `{}`, and an empty `cog.toml`. These use default policies; they do
not establish equivalent configuration processing or validation coverage.

Commands:

- clint: frozen `fluidise-clint --file <message>`.
- convco: frozen `convco check --from-stdin`, bytes supplied by Python.
- cocogitto: frozen `cog verify --file <message>`.
- Explicit control: clint with an additional `--config <explicit-file>`.
- Outside-Git control: clint file validation from a non-repository directory.
- Git control: isolated `git -c core.fsmonitor=false -c core.hooksPath=/dev/null
  -c core.quotePath=false rev-parse --show-toplevel`, with equivalent Git environment.

The convco stdin interface differs from the native file interfaces of the other
tools. Results compare practical invocation costs, not identical feature sets or
output contracts. Installed-version help verified these commands; Context7 docs
confirmed single-message interfaces but were not guaranteed version-pinned.

All measured valid invocations returned 0; invalid invocations returned 1.
Clint output was decoded as JSON and status checked; stderr was empty. Repeated
outputs were checked for stability except cocogitto, whose text may contain time.
Configured discovery and explicit controls emitted exactly identical outputs.
Without configuration, the explicit empty file changes `configuration.source`
from `defaults` to `file`; outside-Git output was exactly identical to discovery.

## Results

Pooled median / median absolute deviation, milliseconds (120 samples per cell):

| Case | clint discovery | convco | cocogitto | clint explicit control | clint outside Git control |
| --- | ---: | ---: | ---: | ---: | ---: |
| defaults/valid | 10.451 / 0.736 | 6.810 / 0.526 | 9.824 / 0.537 | 3.920 / 0.402 | 3.841 / 0.301 |
| defaults/invalid | 10.934 / 0.493 | 7.051 / 0.457 | 9.985 / 0.403 | 4.079 / 0.287 | 4.099 / 0.293 |
| configured/valid | 9.529 / 0.385 | 6.309 / 0.204 | 9.299 / 0.254 | 3.500 / 0.159 | 3.499 / 0.164 |
| configured/invalid | 9.684 / 0.464 | 6.402 / 0.249 | 9.364 / 0.250 | 3.548 / 0.192 | 3.540 / 0.173 |

Both individual rounds show the same ordering in all four cases. Background load
and drift remain possible; configured and unconfigured cases were not interleaved,
so their difference must not be attributed to configuration parsing alone.

## Interpretation and limits

Automatic discovery costs several milliseconds in these fixtures. The configured
control is especially informative: the same message, same configuration policy,
and byte-identical output take roughly 3.5 ms with an explicit path versus 9.5–9.7 ms
with discovery. This identifies Git-root discovery as the priority for investigation,
not message validation or JSON emission. It does not yet prove a safe optimization.

The isolated Git control has a pooled median of approximately 6.9–7.8 ms across
cases. It is a separate measurement including Python/process overhead, not an
additive decomposition of clint's runtime. Do not subtract or sum those medians as
an exact profile.

The existing explicit-configuration interface already bypasses discovery, but is
not a transparent replacement for automatic discovery, especially without a config
file. Changing hook invocation would be a separate user choice.

No RSS measurement, cold-start claim, cross-platform validation, worktree/bare
performance measurement, hook installation, source optimization, commit, push or
remote CI was performed. Subdirectories, worktrees, bare/broken repositories,
native paths, errors and large inputs remain mandatory regression coverage for a
future implementation, not proven by these four benchmark cases.

## Local artifacts and reproduction

Artifacts are retained locally under the ignored directory
`target/startup-baseline-2026-10-23/`: `measure.py`, `results.json`, and frozen
executables in `bin/`. The JSON records raw samples, commands, fixture contents,
outputs, hashes and environment. These artifacts are **not versioned** and would
be lost if that directory is cleaned. The temporary fixtures were removed after
the run; the script reconstructs them.

From this unchanged checkout, while those frozen executables remain available:

```sh
python3 target/startup-baseline-2026-10-23/measure.py
```

The script overwrites its local `results.json` with a new run. Archive the previous
artifacts before rerunning if the original observations must be retained.

After reviewing this baseline, the user approved an improvement threshold of at
least 20% and at least 1 ms lower median latency in all four primary scenarios,
confirmed in two rounds. Public guarantees must remain unchanged; significant
regressions on other paths require investigation before acceptance. The reference
and candidate must be measured together, not compared across separate sessions.

The implementation strategy remains undecided. No ADR is warranted until a
durable trade-off is chosen.
