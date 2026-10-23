# Task contract: remove redundant commit-msg wrapper discovery

Status: implementation approved by the user's final "go" after Q11; locally implemented and validated. No hook installed and no Git delivery performed.

## Objective

Reduce hook validation latency without changing standalone clint behavior or adding
dependencies. This is a hook-wrapper optimization, not an improvement to the
standalone Startup & Lint Latency benchmark or competitive ranking.

## Approved decisions

- Prioritize practical hook latency over ranking alone (Q1).
- Preserve clint's public behavior, default JSON, diagnostics, exit codes,
  compatibility and bounded resource handling (Q2).
- Keep CLI validation latency distinct from hook validation latency (Q3, Q9).
- Retain clint's policy selection contract (explicit `--defaults` in the wrapper since #15, ignoring root configuration) and test ordinary Git repositories
  with and without configuration, for valid and invalid small messages (Q4).
- Require at least 20% and at least 1 ms lower median latency in each primary
  scenario, confirmed in two interleaved measurement rounds (Q6, applied to the
  selected hook-wrapper scope). Prototype observations were approximately 26–32%.
- Reject direct gix and git2 replacements because their behavior differs (Q7–Q8
  evaluations); do not add either dependency.
- Optimize the generated hook wrapper, not clint's discovery implementation (Q9).
- Support the wrapper as a Git-invoked non-bare commit-msg hook from the worktree
  root. Arbitrary manual invocation from subdirectories, outside Git, or bare
  directories is not supported; manual validation uses the clint CLI (Q10).
- Accept the wrapper's correction for worktree-root names ending in LF, which
  command substitution previously truncated; add a dedicated regression test (Q11).

## Implementation scope

1. In the generated hook body within scripts/install-hooks.sh, replace the
   redundant git rev-parse command substitution with the shell's working directory,
   and document the Git-provided root-cwd invariant in a concise comment.
2. Keep the installer's own discovery command and installation behavior unchanged.
   Fixing installer path handling or worktree installation is not part of this task.
3. Preserve release/debug selection, existing Cargo fallback, message-file handling,
   --human output and existing hook error handling.
4. Add focused automated tests using extracted wrapper scripts in disposable
   directories, without installing hooks. Cover ordinary root invocation,
   valid/invalid messages and configuration, release/debug selection, relative and
   quoted/native path handling, trailing-LF root, and applicable failure paths.
5. Document the root-cwd invocation boundary and measured evidence. Do not change
   historical competitive claims to imply a standalone CLI improvement.

## Acceptance evidence

- Shell syntax checks and focused automated wrapper tests pass.
- The four timing scenarios meet the stated gain threshold in both rounds when
  comparing frozen reference and final generated candidate together. Report raw
  observations and environment; investigate meaningful regressions before acceptance.
- Ordinary wrapper exit/stdout/stderr are unchanged. The approved trailing-LF
  correction is explicit, not presented as byte-identical behavior.
- Product Rust sources, Cargo manifest/lockfile and the unsafe prohibition remain
  unchanged. No dependency or public CLI option is introduced.
- Run repository quality gates: cargo fmt --all -- --check,
  cargo clippy --all-targets -- -D warnings, and cargo test --all-targets.
- Clearly distinguish direct wrapper tests from actual Git hook dispatch,
  cross-platform validation, and installed-hook behavior; do not claim unperformed
  validation. Existing ignored prototype artifacts remain local evidence only.

## Exclusions and separate approvals

No hook installation/replacement, Git commit creation, staging, commit, push, pull
request, merge, deployment, global configuration change, cleanup of prototypes,
dependency integration or unsafe Rust. No additional repository modernization or
unrelated installer fixes. No ADR is needed for this readily reversible change;
this contract records scope and approval, not a durable architecture decision.

## Delivery evidence

The generated wrapper assignment and its invariant comment are changed; installer
discovery is unchanged. USAGE.md documents the invocation boundary. Three new
Cargo-integrated hook tests pass, and both regression tests were observed failing
before the change. All 35 repository all-target tests pass; format, strict Clippy,
shell syntax and diff whitespace checks pass. The full test suite required a
sandbox exception for two pre-existing native-path/socket tests.

Final interleaved measurements meet both thresholds in all four cases and both
rounds: 39.1–42.7% lower median latency, about 6.7–7.6 ms (with both reference and
candidate using explicit `--defaults` as introduced in #15). Exact ordinary outputs
and fixture non-mutation checks passed. See hook-latency-prototype.md for detailed
measurements, provenance and limits. This is direct wrapper evidence, not actual
Git dispatch, installed-hook behavior or standalone CLI acceleration.

Only the approved generated-hook change, focused tests and documentation were
implemented. No dependency, product Rust implementation, installer installation
behavior, installed hook or Git delivery state was changed.
