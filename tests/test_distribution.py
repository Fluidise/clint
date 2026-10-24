"""Standard-library regression tests for the candidate packaging boundary."""

import importlib.util
import io
import json
from pathlib import Path
import re
import stat
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile


spec = importlib.util.spec_from_file_location(
    "distribution", Path(__file__).resolve().parents[1] / "scripts/distribution.py"
)
distribution = importlib.util.module_from_spec(spec)
spec.loader.exec_module(distribution)


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)

    def candidate(self, windows=False):
        name = "fluidise-clint.exe" if windows else "fluidise-clint"
        target = "x86_64-pc-windows-msvc" if windows else "aarch64-apple-darwin"
        provenance = {"target": target, "archive_root": "candidate", "version": "0.1.0",
                      "binary_sha256": distribution.digest(b"executable")}
        files = {name: b"executable", "LICENSE": b"MIT", "USAGE.md": b"usage",
                 "THIRD-PARTY-NOTICES.md": b"pending runtime review",
                 "THIRD-PARTY/nested/LICENSE": b"terms",
                 "PROVENANCE.json": json.dumps(provenance).encode()}
        files["FILES.sha256"] = "".join(
            f"{distribution.digest(data)}  {path}\n" for path, data in sorted(files.items())
        ).encode()
        return files, name

    def archive(self, windows=False):
        files, binary = self.candidate(windows)
        path = self.base / ("candidate.zip" if windows else "candidate.tar.gz")
        distribution.write_archive(path, "candidate", files, binary)
        return path, files, binary

    def test_roundtrip_and_normalized_metadata(self):
        for windows in (False, True):
            with self.subTest(windows=windows):
                archive, files, binary = self.archive(windows)
                unpacked, _, actual_binary = distribution.read_archive(archive)
                self.assertEqual(files, unpacked)
                self.assertEqual(binary, actual_binary)
                destination = self.base / ("windows" if windows else "macos")
                _, installed = distribution.extract_verified(archive, destination)
                self.assertEqual(installed.read_bytes(), b"executable")
                self.assertTrue((destination / "THIRD-PARTY/nested/LICENSE").is_file())
                with self.assertRaises(FileExistsError):
                    distribution.extract_verified(archive, destination)
                if not windows:
                    with tarfile.open(archive) as stream:
                        self.assertTrue(all(i.mtime == 0 and i.uid == 0 and i.gid == 0 for i in stream))
                    self.assertEqual(installed.stat().st_mode & 0o777, 0o755)

    def test_same_input_produces_identical_archive(self):
        for windows in (False, True):
            archive, files, binary = self.archive(windows)
            other = self.base / ("second.zip" if windows else "second.tar.gz")
            distribution.write_archive(other, "candidate", files, binary)
            self.assertEqual(archive.read_bytes(), other.read_bytes())
            with self.assertRaises(FileExistsError):
                distribution.write_archive(other, "candidate", files, binary)

    def test_tampered_payload_rejected_before_extraction(self):
        files, binary = self.candidate()
        files[binary] = b"tampered"
        path = self.base / "bad.tar.gz"
        distribution.write_archive(path, "candidate", files, binary)
        destination = self.base / "must-not-exist"
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            distribution.extract_verified(path, destination)
        self.assertFalse(destination.exists())

    def test_tar_rejects_unsafe_entries(self):
        cases = ["../escape", "/absolute", "root/../escape", "root//file", "root/./file",
                 "root\\escape", "C:/escape", "root/file:stream"]
        for index, name in enumerate(cases):
            path = self.base / f"unsafe-{index}.tar.gz"
            with tarfile.open(path, "w:gz") as stream:
                info = tarfile.TarInfo(name)
                info.size = 1
                stream.addfile(info, io.BytesIO(b"x"))
            with self.subTest(name=name), self.assertRaises(ValueError):
                distribution.read_archive(path)

    def test_tar_rejects_links_and_special_files(self):
        for index, kind in enumerate((tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.DIRTYPE, tarfile.FIFOTYPE)):
            path = self.base / f"link-{index}.tar.gz"
            with tarfile.open(path, "w:gz") as stream:
                info = tarfile.TarInfo("root/link")
                info.type = kind
                info.linkname = "../../escape"
                stream.addfile(info)
            with self.assertRaises(ValueError):
                distribution.read_archive(path)

    def test_zip_rejects_symlink(self):
        path = self.base / "link.zip"
        with zipfile.ZipFile(path, "w") as stream:
            info = zipfile.ZipInfo("root/link")
            info.create_system = 3
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            stream.writestr(info, "../../escape")
        with self.assertRaisesRegex(ValueError, "unsafe"):
            distribution.read_archive(path)

    def test_zip_rejects_traversal(self):
        path = self.base / "traversal.zip"
        with zipfile.ZipFile(path, "w") as stream:
            info = zipfile.ZipInfo("root/../escape")
            info.external_attr = (stat.S_IFREG | 0o644) << 16
            stream.writestr(info, "unsafe")
        with self.assertRaisesRegex(ValueError, "unsafe"):
            distribution.read_archive(path)

    def test_missing_license_payload_rejected(self):
        files, binary = self.candidate()
        del files["LICENSE"]
        path = self.base / "missing.tar.gz"
        distribution.write_archive(path, "candidate", files, binary)
        with self.assertRaisesRegex(ValueError, "required archive payload"):
            distribution.read_archive(path)

    def test_case_collisions_and_multiple_roots_rejected(self):
        for names in (("root/file", "root/FILE"), ("first/file", "second/file")):
            path = self.base / "names.tar.gz"
            with tarfile.open(path, "w:gz") as stream:
                for name in names:
                    info = tarfile.TarInfo(name)
                    stream.addfile(info, io.BytesIO(b""))
            with self.assertRaises(ValueError):
                distribution.read_archive(path)

    def test_expansion_limit(self):
        archive, _, _ = self.archive()
        with patch.object(distribution, "MAX_ARCHIVE_BYTES", 1):
            with self.assertRaisesRegex(ValueError, "limits"):
                distribution.read_archive(archive)

    def test_native_runner_required(self):
        with patch.object(distribution.platform, "system", return_value="Linux"), \
                patch.object(distribution.platform, "machine", return_value="x86_64"):
            distribution.native_target("x86_64-unknown-linux-musl")
            with self.assertRaises(ValueError):
                distribution.native_target("aarch64-unknown-linux-musl")

    def test_regular_rejects_symlinks(self):
        if distribution.platform.system() == "Windows":
            self.skipTest("symlink creation requires Windows developer mode")
        source = self.base / "ordinary"
        source.write_text("text")
        link = self.base / "link"
        link.symlink_to(source)
        with self.assertRaises(ValueError):
            distribution.regular(link)

    def test_homebrew_adapts_reference_and_migrates_explicit_defaults(self):
        folder = self.base / "distribution"
        folder.mkdir()
        files, binary = self.candidate()
        archive = folder / "candidate.tar.gz"
        distribution.write_archive(archive, "candidate", files, binary)
        checksum = f"{distribution.digest(archive.read_bytes())}  {archive.name}\n"
        (folder / "SHA256SUMS").write_text(checksum)
        reference = self.base / "reference.rb"
        recipe = '  version "old"\n' + (
            '      url "https://old.example/archive"\n      sha256 "' + '0' * 64 + '"\n'
        ) * 2 + ('  pipe_output("#{bin}/fluidise-clint --stdin", "valid", 0)\n'
                 '  pipe_output("#{bin}/fluidise-clint --stdin", "invalid", 1)\n'
                 '# install method remains unchanged\n')
        reference.write_text(recipe, newline="\n")
        destination = self.base / "homebrew"
        with patch.object(distribution, "native_target"), patch.object(distribution, "run") as run, \
                patch.object(distribution, "smoke"):
            distribution.homebrew(folder, reference, destination)
        fixture = (destination / "fluidise-clint.rb").read_text()
        self.assertEqual(reference.read_text(), recipe)
        self.assertEqual(fixture.count(archive.resolve().as_uri()), 2)
        self.assertIn('  version "0.1.0"', fixture)
        self.assertTrue(fixture.endswith("# install method remains unchanged\n"))
        self.assertEqual(fixture.count("--stdin --defaults"), 2)
        self.assertEqual(run.call_args.args[:2], ("brew", "ruby"))
        # A changed reference recipe must fail before any test output is written.
        reference.write_text(recipe.replace('  version "old"', '  version :unknown'), newline="\n")
        failed = self.base / "failed-homebrew"
        with patch.object(distribution, "native_target"), self.assertRaisesRegex(ValueError, "reference changed"):
            distribution.homebrew(folder, reference, failed)
        self.assertFalse(failed.exists())
        # A changed outer checksum must also fail without executing Homebrew.
        (folder / "SHA256SUMS").write_text("wrong\n")
        with patch.object(distribution, "native_target"), self.assertRaisesRegex(ValueError, "outer archive"):
            distribution.homebrew(folder, reference, failed)

    def test_missing_cargo_license_fails_closed(self):
        package = self.base / "dependency"
        package.mkdir()
        metadata = {"resolve": {"root": "root", "nodes": [{"id": "dependency"}]},
                    "packages": [{"id": "dependency", "name": "dependency", "version": "1.0.0",
                                  "manifest_path": str(package / "Cargo.toml")}]}
        with self.assertRaisesRegex(ValueError, "missing license text"):
            distribution.notices(metadata, self.base / "sysroot")

    def release_archives(self, directory, change=None, omit=None):
        directory.mkdir()
        for target in distribution.TARGETS:
            if target == omit:
                continue
            files, binary = self.candidate(windows="windows" in target)
            provenance = json.loads(files["PROVENANCE.json"])
            provenance.update(target=target, source_commit="a" * 40, working_tree_dirty=False,
                              public_distribution_ready=True, publication_blockers=[],
                              publisher_signing_policy="unsigned")
            if change:
                change(provenance)
            files["PROVENANCE.json"] = json.dumps(provenance).encode()
            del files["FILES.sha256"]
            files["FILES.sha256"] = "".join(
                f"{distribution.digest(data)}  {path}\n" for path, data in sorted(files.items())
            ).encode()
            suffix = ".zip" if "windows" in target else ".tar.gz"
            distribution.write_archive(directory / (target + suffix), "candidate", files, binary)

    def test_release_requires_complete_eligible_exact_source(self):
        valid = self.base / "valid"
        self.release_archives(valid)
        distribution.verify_release(valid, "v0.1.0", "a" * 40)
        cases = [
            {"public_distribution_ready": False},
            {"public_distribution_ready": "true"},
            {"publication_blockers": ["runtime review"]},
            {"working_tree_dirty": True},
            {"source_commit": "b" * 40},
            {"version": "0.2.0"},
            {"publisher_signing_policy": "undecided"},
        ]
        for index, updates in enumerate(cases):
            directory = self.base / str(index)
            self.release_archives(directory, lambda provenance: provenance.update(updates))
            with self.subTest(updates=updates), self.assertRaises(ValueError):
                distribution.verify_release(directory, "v0.1.0", "a" * 40)
        for field in ("public_distribution_ready", "publication_blockers", "working_tree_dirty",
                      "publisher_signing_policy"):
            directory = self.base / field
            self.release_archives(directory, lambda provenance: provenance.pop(field))
            with self.subTest(missing=field), self.assertRaises(ValueError):
                distribution.verify_release(directory, "v0.1.0", "a" * 40)
        for tag, source in (("v0.2.0", "a" * 40), ("v0.1.0", "main")):
            with self.assertRaisesRegex(ValueError, "identity mismatch"):
                distribution.verify_release(valid, tag, source)

    def test_release_rejects_missing_duplicate_and_tampered_archives(self):
        directory = self.base / "matrix"
        self.release_archives(directory, omit="x86_64-pc-windows-msvc")
        with self.assertRaisesRegex(ValueError, "five supported targets"):
            distribution.verify_release(directory, "v0.1.0", "a" * 40)
        archive = next(directory.glob("*.tar.gz"))
        (directory / "duplicate.tar.gz").write_bytes(archive.read_bytes())
        with self.assertRaisesRegex(ValueError, "duplicate"):
            distribution.verify_release(directory, "v0.1.0", "a" * 40)
        (directory / "duplicate.tar.gz").unlink()
        files, provenance, binary = distribution.read_archive(archive)
        files[binary] = b"tampered"
        archive.unlink()
        distribution.write_archive(archive, provenance["archive_root"], files, binary)
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            distribution.verify_release(directory, "v0.1.0", "a" * 40)

    def test_release_workflow_checks_eligibility_before_publication(self):
        workflow = (distribution.ROOT / ".github/workflows/release.yml").read_text()
        self.assertLess(workflow.index("distribution.py verify-release"), workflow.index("gh release create"))
        self.assertEqual(workflow.count("ref: ${{ needs.source.outputs.sha }}"), 2)
        self.assertNotIn("--clobber", workflow)
        self.assertIn("--verify-tag", workflow)
        self.assertNotIn("${{ github.event.inputs.tag }}", workflow)

    def test_workflow_stays_manual_read_only_and_has_exactly_five_targets(self):
        # Guard the agreed boundary, not a substitute for actionlint/schema checks.
        workflow = (distribution.ROOT / ".github/workflows/distribution.yml").read_text()
        trigger = re.search(r"^on:\n(.*?)(?=^\S)", workflow, re.MULTILINE | re.DOTALL).group(1)
        permissions = re.search(r"^permissions:\n(.*?)(?=^\S)", workflow, re.MULTILINE | re.DOTALL).group(1)
        self.assertEqual(trigger.strip(), "workflow_dispatch:")
        self.assertEqual(permissions.strip(), "contents: read")
        self.assertEqual(set(re.findall(r"^          - target: (.+)$", workflow, re.MULTILINE)), set(distribution.TARGETS))
        self.assertNotIn("secrets.", workflow)
        self.assertNotIn("gh release", workflow)


if __name__ == "__main__":
    unittest.main()
