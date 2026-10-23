# Hook wrapper latency experiment, 2026-10-23

## Approved scope and current status

The user selected Q9 option 1: measure removal of redundant discovery in the
repository-provided commit-msg wrapper, without changing clint, adding a dependency,
or installing/replacing any hook. This experiment targets **hook validation
latency**, not the earlier standalone CLI latency metric or competitive ranking.

The primary timing target is met. The user subsequently approved limiting the
wrapper to Git's root-cwd invocation contract (Q10) and correcting the existing
trailing-newline path bug in that wrapper (Q11). The user then explicitly confirmed the bounded implementation contract with
"go". The generated wrapper is now updated in scripts/install-hooks.sh; installed
hooks remain untouched. Final validation is recorded below.

## Prototype

The generated hook body was extracted verbatim from scripts/install-hooks.sh into
an executable reference.sh under target/hook-startup-prototype/. The candidate.sh
copy changes only the generated wrapper's root assignment:

```diff
-REPO_ROOT="$(git rev-parse --show-toplevel)"
+REPO_ROOT="$PWD"
```

The installer's own discovery command is not part of this experiment. Binary
selection (release, debug, then Cargo build), --human output, validation and error
handling remain unchanged in the extracted wrappers. Both use the exact same
frozen clint release executable from the initial baseline.

Git 2.53.0's local githooks manual, cross-checked against Context7 /git/htmldocs,
states that Git enters the working-tree root before invoking a hook in a non-bare
repository. Bare repositories use GIT_DIR, and push hooks have other rules. This
supports removing redundant root discovery for a real non-bare commit-msg hook,
but does not cover arbitrary manual script invocation from another directory.

## Measurement protocol

Four scenarios: valid/invalid small messages, with/without fluidise.toml. For each,
two rounds of 60 timed samples follow five warmups per round. Reference, candidate
and a direct --human CLI control run in seeded shuffled order per sample. Warm
filesystem caches, fresh processes, Python subprocess launch/capture/wait overhead
included. Absolute command paths and fixture details are recorded in raw JSON.

Wrappers were executed directly from temporary repository roots, outside all
.git/hooks directories. No hook was installed or replaced; Git did not dispatch
these wrappers, and no commit was created. The frozen release executable was
symlinked into each fixture's target/release location. The primary timings exclude
compilation and debug fallback. Personal/system Git configuration was disabled in
the controlled timing environment. Fixture files were hashed before/after.

Both wrappers returned 0 for valid messages and 1 for invalid messages. Exit code,
stdout and stderr matched exactly in every primary case and all repetitions.
Outputs are those of the existing --human hook, not JSON. The standalone CLI
implementation and binary were not changed.

## Results

Median milliseconds, 60 samples per cell:

| Case | Round | Reference wrapper | Candidate wrapper | Reduction |
| --- | ---: | ---: | ---: | ---: |
| defaults/valid | 1 | 27.591 | 18.826 | 31.8% |
| defaults/valid | 2 | 27.147 | 19.847 | 26.9% |
| defaults/invalid | 1 | 25.534 | 18.540 | 27.4% |
| defaults/invalid | 2 | 27.558 | 19.588 | 28.9% |
| configured/valid | 1 | 27.020 | 19.870 | 26.5% |
| configured/valid | 2 | 27.295 | 19.183 | 29.7% |
| configured/invalid | 1 | 26.601 | 19.212 | 27.8% |
| configured/invalid | 2 | 26.459 | 19.250 | 27.2% |

Every primary scenario and round exceeded a 20% and 1 ms reduction. Observed gain
was approximately 26–32%, or 7–9 ms. Direct CLI control medians were roughly
11.4–12.1 ms in this run. Absolute numbers must not be compared with earlier
sessions because machine load and timing context vary.

These are direct wrapper invocation measurements under the documented root-cwd
condition, not a benchmark of git commit or proof of actual Git hook dispatch.
No standalone CLI performance or competitor-ranking improvement is claimed.

## Differential behavior and pending decisions

17 cases executed, 13 byte-identical overall; fixture files unchanged. Of 13 cases
classified as Git-root contexts, 12 matched. Passing cases include release/debug,
valid/invalid messages, restrictive and malformed Fluidise configuration, a relative
message path, exported Git context, symlink root, Unicode/internal-newline root,
linked worktree and an effective root specified through core.worktree.

The one Git-root divergence is a root name ending in LF. The reference's shell
command substitution strips trailing LF from git rev-parse output, truncates the
root, and fails with exit 1. The candidate preserves PWD and succeeds with exit 0.
This is a correction of an existing path bug, but remains an observable behavior
change requiring explicit scope approval under this interview's strict contract.

Three arbitrary manual contexts also differ:

- Subdirectory invocation: reference succeeds; candidate attempts its build fallback.
  A deliberately failing Cargo stub returns 91; no real build was run.
- Outside Git: reference returns 128; candidate can find the fixture binary and
  validate successfully. This must not be presented as equivalent behavior.
- Bare directory: the same 128 versus 0 difference for manual script invocation.

A forged PWD case matched: Bash recomputed PWD to correspond to the actual cwd.
Differential scripts invoke /bin/bash directly; timed scripts use their existing
/usr/bin/env bash shebang. No broad Bash-version or cross-platform proof is claimed.
Foreign ownership, complete read-failure coverage, non-UTF8 paths, concurrent
filesystem mutations and real Git dispatch were not tested here. No claim is made
that arbitrary exported Git routing or inconsistent cwd contexts are equivalent.

## Evidence boundary of the initial experiment

Local, ignored artifacts (not versioned):

- target/hook-startup-prototype/reference.sh and candidate.sh
- target/hook-startup-prototype/provenance.json
- target/hook-startup-prototype/measure.py and results.json
- target/hook-startup-prototype/differential.py and differential-results.json

Provenance/JSON retain hashes, raw samples, output bytes, commands and environment.
Temporary fixtures were removed. Cleaning target/ would remove these artifacts.
At the experiment stage, product Rust sources, Cargo files, installer and installed
hooks were unchanged. Shell syntax and Python syntax checks passed at that stage;
full product gates were deferred until implementation. No commit, push, installation
or deployment was authorized or performed.

No adoption ADR is necessary for this reversible one-line optimization. Manual
invocation coverage and the trailing-LF wrapper correction were settled by Q10
and Q11. The implementation contract is separate from any hook installation
approval; the installer's own root discovery remains outside this optimization.

## Final implementation validation after explicit approval

The generated wrapper now uses PWD, with a comment explaining Git's non-bare
root-cwd contract. The installer's own root-discovery command and installation
behavior are unchanged, including their existing limitations. USAGE.md states the
Git-only wrapper invocation boundary; manual callers use the clint CLI.

Three Cargo-integrated tests were added to tests/cli.rs using the existing
fixture machinery and the actual installer heredoc body. No installer is run.
They cover:

- Exact validation stdout/stderr and hook exit mapping for default, restrictive
  and malformed configuration; valid/invalid messages, missing file, and a relative
  message path with quotes, Unicode and an internal newline; message/config files
  unchanged and no hook or index created.
- A root with quotes, Unicode, internal and terminal LF, using the real test-built
  clint executable. The approved correction now returns valid successfully.
- Release preference, debug fallback, successful simulated Cargo build and its
  exact arguments/root, build failure propagation, and absence of wrapper Git
  calls (a failing Git stub makes any redundant discovery fail). The Cargo stub
  avoids actual compilation inside a fixture.

Before the wrapper change, the two regression tests failed on the truncated root
and unwanted Git command; validation-equivalence test passed. After the change,
all three pass. Tests invoke extracted shell code directly, not an installed hook
or a Git-created commit.

Final checks on macOS ARM64:

- `bash -n scripts/install-hooks.sh` and both final extracted scripts: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo test --locked --offline --all-targets`: **33 passed** (5 binary unit,
  16 CLI integration including the 3 new hook tests, 12 engine integration).
  Initial sandbox run blocked two existing tests (Unix socket and native-byte
  filename); the same full suite passed when rerun outside that sandbox.
- `git diff --check`: passed. Product src/, Cargo.toml and Cargo.lock unchanged.

### Final generated-wrapper measurement

The frozen original reference is compared with a fresh extraction from the final
installer source. Its SHA-256 was verified against the measurement provenance.
Both use the same frozen release clint executable; there was no Rust binary change.
Four scenarios, two rounds of 60 samples after five warmups, shuffled interleaving
and exact output/code checks use the same direct-wrapper protocol as above.

| Case | Round | Reference median ms | Final wrapper median ms | Reduction |
| --- | ---: | ---: | ---: | ---: |
| defaults/valid | 1 | 23.704 | 16.812 | 29.1% |
| defaults/valid | 2 | 23.114 | 16.693 | 27.8% |
| defaults/invalid | 1 | 22.857 | 16.077 | 29.7% |
| defaults/invalid | 2 | 21.451 | 15.525 | 27.6% |
| configured/valid | 1 | 21.362 | 15.064 | 29.5% |
| configured/valid | 2 | 21.504 | 15.396 | 28.4% |
| configured/invalid | 1 | 22.483 | 15.939 | 29.1% |
| configured/invalid | 2 | 21.875 | 15.643 | 28.5% |

All cases pass the at-least-20% and at-least-1-ms criteria in both rounds. Final
observed improvement is 27.6–29.7%, approximately 5.9–6.9 ms per wrapper call.
Measured fixture files are unchanged. These observations do not measure Git's
pre-hook work, actual hook dispatch, a cold-cache workload or another platform.

Final local artifacts, ignored and not versioned, are in
`target/hook-latency-final/`: reference.sh, candidate.sh, measure.py, results.json
and provenance.json. The JSON contains raw samples, environment, commands, output
bytes, executable/source hashes and file non-mutation checks. The temporary
fixtures were removed; cleaning target/ would remove the remaining evidence.

No dependency, unsafe code, public CLI change, installed-hook modification, staging,
commit, push, remote CI or deployment occurred. Existing installed wrappers must
not be described as updated: activating this generated wrapper requires a separate
installation approval. Standalone CLI latency and competitive ranking are unchanged.
