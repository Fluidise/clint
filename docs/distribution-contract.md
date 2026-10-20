# Candidate distribution task contract

Status: approved for local implementation on 2026-10-20. This is a task contract,
not an ADR or authorization to publish.

## Outcome and scope

Prepare precompiled CLI archives for local and CI users without requiring Rust at
runtime. A separate, manually dispatched GitHub Actions workflow builds and tests:

- macOS Apple Silicon and Intel;
- Linux ARM64 and x86_64, using musl;
- Windows x86_64, using MSVC.

Keep the CLI, existing CI, Cargo publication policy and repository settings
unchanged. Archives include version, target, source commit, checksums and license
materials. Reuse the existing Fluidise Homebrew recipe for macOS-only temporary
installation tests; do not introduce a second maintained recipe or modify the tap.

## Acceptance criteria

1. Only `workflow_dispatch` can start the new workflow. Its repository permission
   is `contents: read`; no publisher credentials or release/upload commands exist
   except the explicitly approved GitHub Actions test-artifact retention step.
2. Five native runners build locked release candidates. Each candidate's extracted
   executable passes version, valid-message and invalid-message checks, including
   JSON status and exit code. Compilation alone is not execution evidence.
3. Archives have a single named root, ordinary files only, internal inventory and
   outer SHA-256 checksum. Extraction refuses traversal, links, duplicate/colliding
   names, oversized contents and existing destinations. Packaging does not silently
   overwrite a previous candidate.
4. The pinned existing Homebrew recipe is adapted only into a native-only local
   fixture with candidate version, local archive URL and actual SHA-256. Its real
   Homebrew install/test methods run against a temporary prefix, without registering
   a tap, linking globally or changing the user's installed formula.
5. Notices come from the candidate's resolved Cargo packages and installed Rust
   toolchain, not old rehearsal archives. Incomplete platform-runtime/SDK coverage
   explicitly blocks public distribution; no legal completeness is claimed.
6. Local packaging tests, Rust formatting, strict Clippy and all-target tests are
   run. Static workflow validation, local native execution and remote matrix
   execution are reported separately. An unexecuted remote matrix is not accepted
   as cross-platform proof.

## Boundaries

Artifacts have no publisher signature or notarization. SHA-256 checks integrity,
not publisher authenticity. No Gatekeeper bypass or security-setting changes.
Compatibility claims cover only actually tested environments, never every OS
version or Linux distribution.

No commit, push, PR, tag, release, R2 upload, registry publication, remote workflow
dispatch, certificate/secret installation, repository-setting change or tap update
is authorized. The clean detached worktree was separately authorized to advance
by fast-forward to `origin/main` before implementation, at source commit
`6a00c1b4e50b200e9e9662d88a42a6adeefe9c7e`.

See [the preparation procedure](distribution.md) for operation and remaining gates.
