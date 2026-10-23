# Git discovery prototype evaluation, 2026-10-23

## Status: not eligible for integration

The user approved an isolated prototype, not a product dependency or source change.
The latency threshold was met in the primary fixtures, but behavioral equivalence
failed. Do not integrate this replacement under the approved unchanged-guarantees
contract.

## Candidate

A copy of the current source uses `gix = "=0.90.0"`, default features disabled,
with `sha1`. Automatic discovery retains the original ancestor-marker precheck,
then uses `gix::discover_opts`; `workdir()` or the bare repository directory locates
`fluidise.toml`. Existing message/config reads, validation and JSON are unchanged.

Opening uses isolated permissions, with repository-local includes re-enabled and
HOME interpolation permitted. `strict_config(true)` and `bail_if_untrusted(true)`
are enabled. No ambient discovery environment overrides or forced ownership trust
are used. There is no fallback to Git in this experiment.

Exact APIs were checked in downloaded gix 0.90.0 source after Context7 discovery.
All prototype dependencies, lockfile, source and builds remain under the ignored
`target/git-discovery-prototype/`. Product Cargo.toml, Cargo.lock and Rust sources
are unchanged. No hooks, commits, pushes, remote CI or installation occurred.

## Primary latency

Reference and candidate were interleaved in two rounds of 60 samples, with five
warmups per round and shuffled command order, using the baseline fixture protocol.
The clock includes Python subprocess overhead, captured output and waiting. Warm
filesystem caches, ordinary empty repositories, cwd at root, one macOS ARM64
machine only. Reference/candidate exit code, stdout and stderr were identical in
all four primary cases.

| Case | Round | Reference median ms | Candidate median ms | Reduction |
| --- | ---: | ---: | ---: | ---: |
| defaults/valid | 1 | 10.310 | 4.331 | 58.0% |
| defaults/valid | 2 | 10.602 | 4.213 | 60.3% |
| defaults/invalid | 1 | 10.706 | 4.327 | 59.6% |
| defaults/invalid | 2 | 10.219 | 4.241 | 58.5% |
| configured/valid | 1 | 10.569 | 4.349 | 58.9% |
| configured/valid | 2 | 10.258 | 4.310 | 58.0% |
| configured/invalid | 1 | 10.373 | 4.229 | 59.2% |
| configured/invalid | 2 | 10.491 | 4.255 | 59.4% |

Every primary case met both thresholds (at least 20% and 1 ms) in both rounds.
This is a prototype speed result, **not a deliverable product improvement**.

## Observed cost

- Stripped release executable: 503,264 bytes reference; 1,117,328 bytes
  candidate (2.22 times larger).
- Lockfile: 22 packages reference; 135 candidate, **113 additions**. This includes
  target-specific resolutions, not 113 crates necessarily compiled on this host.
- Normal dependency graph on this host: 98 unique package lines including the
  project, as recorded by `cargo tree`; no network feature was explicitly enabled.
- Fresh target release build, one observation each, downloads excluded: reference
  7.07 s; candidate 23.68 s. Different runs, possible load/cache differences; not a
  statistically controlled compilation benchmark. Final API-name-only rebuild took
  8.51 s and must not be confused with the fresh build.
- macOS `/usr/bin/time -l` reported median maximum resident set size across seven
  alternating observations: reference 7,405,568 bytes,
  candidate 2,457,600 bytes. Reference invokes Git;
  this metric is not a sum or time series of memory across a process tree, nor a
  memory guarantee. Initial sandbox measurement was blocked by `kern.clockrate`;
  the read-only resource measurement was rerun successfully outside that sandbox.

## Behavioral differential: blocking failures

25 executed cases: **21 identical, 4 different**. One additional non-UTF8 path
fixture was skipped because directory creation was refused by the environment.
Observed fixture files were unchanged before/after invocations.

1. **Git absent**: reference reports `config.read_failed` (exit 2); candidate
   validates successfully (exit 0). Removing that runtime dependency is an
   observable compatibility change, not approved by the unchanged-guarantees rule.
2. **Repository-local include defining core.worktree**: reference selects the
   root containing `.git`, uses default Fluidise rules and returns 1; candidate
   selects the included worktree path, loads a different Fluidise policy and
   returns 0. This changes validation semantics, not just diagnostics.
3. **Unknown repository extension with format version 1**: reference rejects
   discovery with exit 2; candidate accepts repository and returns message exit 1.
4. **HEAD containing `not-a-ref\n`**: reference rejects discovery with exit 2;
   candidate continues with default rules and returns message exit 1.

Passing fixtures include ordinary/default/subdirectory discovery, forged GIT_*
environment, ignored user configuration, direct absolute/relative core.worktree,
a conditional include case, malformed Git config, invalid core.bare, missing HEAD,
invalid gitfiles, bare and linked worktree, Unicode/newline paths, and tested
symlink cases. Passing these does not prove general Git equivalence.

Source audit also found a blocking ownership concern to investigate: gix-sec
0.16.0 accepts a foreign owner matching SUDO_UID without an explicit effective-root
check in that code path, unlike documented Git behavior. No foreign-ownership test
was performed; this is an unverified security compatibility risk, not an observed
exploit. Permissions failures, filesystem boundaries, concurrent mutations and
large Git metadata were not comprehensively covered. Fluidise input limits do not
bound all metadata loaded by either Git implementation.

## Artifacts

Local, ignored, not versioned:

- `target/git-discovery-prototype/compare.py` and `results.json`: raw latency,
  commands, output identities, environment and executable hashes.
- `target/git-discovery-prototype/differential.py` and
  `differential-results.json`: fixtures, byte-exact outputs and skipped case.
- `target/git-discovery-prototype/footprint.py` and `rss.json`: RSS observations.
- `target/git-discovery-prototype/build.log`, `baseline-build.log`, `rebuild.log`,
  `tree.txt`, source copies and Cargo.lock: build and dependency provenance.

The prototype is rejected as a direct replacement. Correcting known divergences
alone would not establish exhaustive Git equivalence. A different strategy or an
explicitly revised compatibility contract requires a further decision. No ADR
records adoption because no durable architecture has been selected.
