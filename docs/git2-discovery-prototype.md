# git2/libgit2 discovery prototype evaluation, 2026-10-23

## Status: not eligible for integration

The user approved a second isolated prototype (Q8), not integration, a new product
dependency, unsafe Rust, hook installation or Git delivery. The latency target is
met in ordinary fixtures, but the unchanged-guarantees requirement fails.

## Exact candidate and isolation boundary

A source copy uses git2 0.21.0 with default features disabled and vendored-libgit2
enabled. The lockfile resolves libgit2-sys 0.18.8+1.9.7, including libgit2 1.9.7.
There are no HTTPS or SSH features requested. The existing ancestor-marker check
is retained. The prototype uses safe `Repository::open_ext` with empty flags and
no ceiling directories; it selects `workdir()` or the bare repository `path()`.
Message/configuration reading, validation and output are unchanged.

The safe opening API has no per-open configuration-source isolation option found
in this evaluation. Without FROM_ENV, repository-routing and configuration GIT_*
overrides are ignored, but global, system, XDG and ProgramData configuration can
still be loaded by normal search. `git2::opts::set_search_path` is explicitly unsafe
because it mutates global, non-thread-safe state. It was **not called**. The
product and prototype keep `unsafe_code = "forbid"`; no wrapper crate, environment
mutation or weakened lint was used to hide that obligation. `set_config` after
opening cannot undo worktree/trust decisions already made during opening.

Ownership validation remains enabled. Source inspection confirms SUDO_UID is
considered only for effective root, but worktree/config loading occurs before
ownership validation, and the safe.directory allowlist may come from personal
configuration. This is not equivalent to the reference's isolated trust boundary.

Context7 docs for /rust-lang/git2-rs and /libgit2/libgit2 were followed by inspection
of the downloaded exact versions. No version equivalence was inferred from latest
API docs alone.

## Primary latency

Same four primary scenarios, two rounds of 60 samples, five warmups, shuffled order,
reference and candidate measured together. Warm filesystem caches; separate CLI
processes; Python subprocess, pipes, capture and waiting included. Outputs, exit
codes and stderr were byte-identical in the primary cases. Ordinary empty Git
repositories and one macOS ARM64 machine only.

The first attempt stopped during warmup: cocogitto panicked when empty HOME lacked
user.name. No results from that attempt were accepted. The completed run gives all
tools the same disposable HOME containing only fixture user.name/user.email and
an empty XDG path. No user configuration was modified. System configuration is not
neutralized for libgit2 by the opening API, a known isolation limitation.

| Case | Round | Reference median ms | Candidate median ms | Reduction |
| --- | ---: | ---: | ---: | ---: |
| defaults/valid | 1 | 12.083 | 6.884 | 43.0% |
| defaults/valid | 2 | 12.679 | 7.252 | 42.8% |
| defaults/invalid | 1 | 12.702 | 6.948 | 45.3% |
| defaults/invalid | 2 | 12.924 | 7.430 | 42.5% |
| configured/valid | 1 | 12.376 | 6.676 | 46.1% |
| configured/valid | 2 | 12.352 | 7.005 | 43.3% |
| configured/invalid | 1 | 12.568 | 6.895 | 45.1% |
| configured/invalid | 2 | 12.939 | 7.044 | 45.6% |

Both thresholds (20% and 1 ms) passed in every primary case and round. Machine load
was higher than during the earlier gix experiment; absolute numbers must not be
compared across those separate sessions as a library ranking.

## Costs observed

- Release executable: 503,264 bytes reference, 1,069,680 bytes candidate
  (2.13 times larger).
- Lockfile: 22 packages reference, 37 candidate, **15 additions**, including
  target-specific/build dependencies. Normal graph: 18 unique package lines
  including the project. This is not the count of native libraries or source files.
- Fresh-target release build: one observation, downloads excluded, 15.97 seconds.
  Earlier reference observation was 7.07 seconds in another run. These are not
  statistically controlled build-time comparisons.
- macOS `/usr/bin/time -l`, seven alternating observations: median maximum resident
  set size 7,421,952 bytes reference and
  6,438,912 bytes candidate. Reference invokes Git;
  this is neither a sum over the process tree nor a memory guarantee.
- `otool -L` shows only macOS system dependencies (libiconv, Security,
  CoreFoundation, libz and libSystem), no Homebrew libgit2 runtime dependency.
  Native compilation/toolchain and other-platform distribution remain extra costs.

## Behavioral differential: blocking failures

27 executed cases: **22 identical, 5 different**; observed fixture files unchanged.
An additional non-UTF8 path fixture was skipped after environment refusal to create
its directory. No ownership-foreign-user, filesystem-boundary, concurrent-mutation
or complete permission-denial matrix was executed.

1. Git absent: reference operational error 2; candidate success 0.
2. Hostile HOME configuration: reference valid 0 with file policy; candidate
   invalid 1 with defaults because global configuration redirected the worktree.
3. Hostile XDG configuration: the same policy/exit-code divergence.
4. Repository-local include defining core.worktree: reference invalid 1/defaults;
   candidate valid 0/file. Different validation policy is selected.
5. Invalid HEAD: reference discovery error 2; candidate message failure 1/defaults.

The unknown-extension case now matches reference (both reject). Hostile
GIT_CONFIG_* overrides match reference when FROM_ENV is absent. These successes
do not compensate for the broken configuration/trust isolation or establish full
Git equivalence.

## Artifacts and disposition

Local ignored artifacts, not versioned, live under `target/git2-discovery-prototype/`:
`Cargo.toml`, `Cargo.lock`, `src/`, `cargo-home/`, `build.log`, `tree.txt`,
`compare.py`, `results.json`, `differential.py`, `differential-results.json`,
`footprint.py`, `rss.json`, `measurement-notes.txt` and the executable in
`target/release/`. JSON includes raw observations, output identities and hashes.
These artifacts would be lost if target/ is cleaned; they are not remote evidence.

The product Cargo.toml, Cargo.lock, Rust sources and hooks remain unchanged.
The unsafe lint remains intact. No full product test-suite or cross-platform run
was performed: the differential already disqualifies this prototype. No installation,
commit, push, remote CI or product improvement is claimed.

Both direct-library replacements are rejected under the current contract. A
possible separate scope is the repository-owned hook wrapper: its generated hook
already calls git rev-parse before invoking clint, which performs discovery again.
That is a distinct end-to-end hook optimization, not an improvement of the CLI
latency metric defined in CONTEXT.md. It requires an explicit scope decision and
has not been changed or measured here. No adoption ADR is warranted yet.

## Follow-up fact: hook working directory

Context7's Git hook documentation states that Git enters the working-tree root
before invoking commit-msg in a non-bare repository (bare hooks use GIT_DIR; push
hooks have separate rules). The generated hook in scripts/install-hooks.sh still
runs git rev-parse --show-toplevel to locate the binary. Reusing the guaranteed
working directory could remove that wrapper's redundant discovery without changing
clint's own discovery or adding a Git library. This remains an unmeasured separate
scope; it would not improve the standalone CLI metric or its competitive ranking.
