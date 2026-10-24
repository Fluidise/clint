# Candidate binary distribution

This repository prepares **test artifacts**, not a public installation channel.
The approved scope is recorded in [the task contract](distribution-contract.md).
GitHub Releases is the intended place to consider for future standalone downloads;
GitHub Packages does not provide a Cargo or generic binary registry. Neither is
published to by this implementation. Existing R2 assets and the Homebrew tap are
not changed or advertised as new candidate releases.

## Ownership

Clint owns its candidate builds, archive format, provenance and tests. The existing
Homebrew recipe remains the reference; no second editable formula is kept here.
The test workflow reads `Fluidise/homebrew-tap` at
`a59bbe0129e05f8e211647427fb19fbe5357c322`. Its recipe is a publication copy of
the maintainer's existing distribution recipe. A future recipe change requires
reviewing that ownership and updating the pinned reference explicitly.

No source build, APT/APK packaging, Homebrew Linux support or Windows ARM64 support
is added as a consumer installation channel.

## Workflow and trust boundary

`.github/workflows/distribution.yml` has only a manual `workflow_dispatch` trigger.
Adding it locally does not register or execute it on GitHub. The workflow must be
on the default branch before GitHub offers manual dispatch; delivery, merge and
dispatch each require separate approval.

It uses `contents: read`, does not persist checkout credentials and has no signing,
cloud or registry secrets. Each matrix job has a 40-minute timeout. Only candidates
whose archive execution and applicable Homebrew tests pass are retained as GitHub
Actions artifacts, for seven days. Artifacts are per-target and per-run attempt;
one successful job does not imply the whole matrix passed.

| Target | Native runner | Archive |
| --- | --- | --- |
| `aarch64-apple-darwin` | `macos-15` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-15-intel` | `.tar.gz` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-2022` | `.zip` |

Runner labels and action revisions were checked against upstream documentation
and official action refs during preparation. Availability on this repository's
plan, runtime provisioning, and the complete matrix still require an approved
remote run. Labels identify test environments, not minimum supported OS versions.

The workflow uses the repository's pinned Rust toolchain, its target standard
library, `rust-docs` for license inventories, Python 3.12's standard library, and
native platform build tools. Linux installs `musl-tools`; macOS uses the runner's
Homebrew. Those are build/test requirements, not requirements of the CLI consumer.

## Local rehearsal

Use Python 3.11 or newer. The target must match the actual OS and architecture;
cross-compilation or Rosetta is not substituted for native runtime evidence.
Install the declared Rust toolchain/target and `rust-docs` before preparing a
candidate. Linux additionally needs a native `musl-gcc`; Windows needs MSVC build
tools. Dependency/tool downloads require the usual environment permissions.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_distribution.py' -v

# Choose a new, nonexistent directory outside this source checkout.
python3 scripts/distribution.py prepare \
  --target aarch64-apple-darwin \
  --output /tmp/clint-candidate
```

Use `python` instead of `python3` where appropriate on Windows. The workflow uses
the interpreter selected by `actions/setup-python`.

Preparation builds `cargo build --locked --release --bin fluidise-clint --target
<target>`, executes the build, assembles the archive, validates its inventory,
extracts into a fresh temporary directory and executes the extracted binary. CLI
tests run outside Git with explicit `--defaults`; no repository configuration is discovered. Any failed
step fails the command; choose another output directory after inspecting a partial
failure. Existing candidates are never replaced.

For macOS, use the existing, reviewed recipe at the pinned revision:

```sh
python3 scripts/distribution.py homebrew \
  --distribution /tmp/clint-candidate \
  --reference /path/to/reviewed/homebrew-tap/Formula/fluidise-clint.rb \
  --output /tmp/clint-homebrew-rehearsal
```

This checks the archive checksum and creates a **native-only, non-publishable**
local fixture from the reference recipe. Both architecture branches intentionally
point to the current native test archive. No fixture belongs in the real tap.
The real Homebrew DSL loads the fixture through its contents loader and runs
its install/test methods with a temporary prefix; no tap is registered and no
global executable is linked. This uses Homebrew's internal Ruby API and fails
closed if the maintained recipe shape changes. It is **not** a complete
`brew install`/`brew audit` or public tap installation test. Final public URLs,
download behavior, supported Homebrew versions and consumer installation remain
separate publication gates. The output directory contains only local test state;
deleting that explicitly chosen directory later removes the rehearsal installation.

## Archive contract

Names include package version, target, and the first 12 digits of the source
commit; local dirty builds additionally end in `-dirty`. The full source commit,
dirty flag, compiler, Cargo version, Cargo.lock checksum, package inventory,
binary checksum and actual test environment are in `PROVENANCE.json`.

Dirty candidates are identifiable rehearsals, not reconstructible releases. Clean
committed source is required before any future publication. Archive metadata is
normalized for the same supplied bytes; reproducible Rust binaries across machines
or toolchain installations are not promised.

Each archive contains exactly one named top-level directory with the executable,
`LICENSE`, `USAGE.md`, `PROVENANCE.json`, `FILES.sha256`, `THIRD-PARTY-NOTICES.md`
and `THIRD-PARTY/` license materials. Windows uses `fluidise-clint.exe`.
`FILES.sha256` covers every other file. The sibling `SHA256SUMS` covers the complete
archive. Keep each target's checksum file with its own archive when downloading
multiple Actions artifacts; these are deliberately not merged into one directory.

The verifier checks ordinary files, one root, exact inventory and hashes before
writing extracted bytes. It rejects absolute/traversing paths, backslashes,
colon-containing names, links, case-insensitive duplicates, more than 4,096 files,
and over 128 MiB of expanded contents. Extraction requires a new destination.
These limits concern the archive rehearsal, not the CLI's input or memory limits.

SHA-256 detects modification relative to a trusted expected checksum. A checksum
served beside a compromised archive does not authenticate its publisher. No
publisher signing, notarization, Gatekeeper bypass or automatic installer is
included. Do not present unsigned test artifacts as production-ready downloads.

## License and publication gates

Packaging copies license/notice files from the actual target-resolved Cargo
package graph, plus installed Rust copyright inventories and license texts. This
is a conservative superset, not proof of exact linkage or legal completeness.
Missing expected license texts fail packaging.

Every candidate explicitly records `public_distribution_ready: false`. Before
public distribution, review the actual linked musl/compiler runtimes, Apple
SDK/runtime obligations or Windows MSVC/UCRT redistribution, as applicable. Resolve
all required notices and terms against the exact candidate; old Rust 1.94.1
rehearsal notices are not sufficient. Also decide publisher signing policy, test
public download URLs/checksums and verify public Homebrew installation separately.

No public-release eligibility gate is bypassed by a successful test-artifact upload.

### Release 0.1.0 preparation

The approved publisher policy for version 0.1.0 is **unsigned**, with no publisher
signature or Apple notarization. SHA-256 checksums are provided for integrity, not
publisher authentication. No operating-system security bypass is supported. This
decision does not approve the policy for later versions.

The release workflow resolves an existing version-matched tag to a commit SHA and
builds all five targets from that SHA. Before publication, `verify-release` checks
archive inventories, source/version identity, a clean source tree, the complete
target matrix, the approved signing policy and public-distribution eligibility.
Existing releases are not overwritten. Current candidates remain ineligible until
the runtime/SDK licensing review is completed; choosing unsigned binaries does not
waive that review. Local packaging success is not proof of the remote matrix.

## Local preparation evidence (2026-10-20)

The local rehearsal on macOS 26.7.1 ARM64 with Rust 1.99.1 produced a candidate
archive, verified its complete inventory and executed its extracted binary.
The existing Homebrew recipe's real install/test methods also passed against a
temporary prefix, followed by direct CLI checks of the installed executable.
No real tap or installed formula was modified. The Homebrew loader uses an
in-memory tap namespace to avoid implicit discovery/installation of homebrew/core.

The 15 packaging/boundary tests passed, as did Rust formatting, strict Clippy and
28 all-target Rust tests. Two existing native-file/FIFO tests needed the approved
unsandboxed rerun after macOS denied their filesystem operations; that rerun passed.
Actionlint 1.7.11 accepted the new workflow. No ShellCheck executable was available,
so shell-command linting is not claimed. These are local/static checks only.

The remote workflow and other four native targets have **not** been executed.
There is no public release, complete Homebrew consumer-installation proof,
independent review or certification of platform license coverage. The local
archive is explicitly marked dirty and not ready for public distribution.

## References

- [GitHub release binaries](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [GitHub Packages registries](https://docs.github.com/en/packages/working-with-a-github-packages-registry)
- [Manual workflow events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_dispatch)
- [Runner image labels](https://github.com/actions/runner-images#available-images)
- [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
