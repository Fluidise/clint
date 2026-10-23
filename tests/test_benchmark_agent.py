"""Regression tests for the independent-process benchmark protocol."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


scripts = Path(__file__).resolve().parents[1] / "scripts"
with patch.object(sys, "path", [str(scripts), *sys.path]):
    spec = importlib.util.spec_from_file_location("benchmark_agent", scripts / "benchmark-agent.py")
    agent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(agent)


class ProtocolTests(unittest.TestCase):
    def test_environment_removes_caller_routing_and_preserves_path(self):
        with patch.dict(agent.os.environ, {"PATH": "/bin", "GIT_DIR": "hostile",
                                         "CONVCO_CONFIG": "hostile", "COG_CONFIG": "hostile"}, clear=True):
            env = agent.environment(Path("/isolated"))
        self.assertEqual(env["PATH"], "/bin")
        self.assertEqual(env["HOME"], "/isolated")
        self.assertEqual(env["GIT_CONFIG_GLOBAL"], "/isolated/.gitconfig")
        for key in ("GIT_DIR", "CONVCO_CONFIG", "COG_CONFIG"):
            self.assertNotIn(key, env)

    def run_protocol(self, change_candidate=False, wrong_competitor=False, flags=()):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binaries = {}
            for name in ("baseline", "candidate", "cog", "convco"):
                binaries[name] = root / name
                binaries[name].write_bytes(name.encode())
            output = root / "results.json"
            argv = ["benchmark-agent.py", str(binaries["baseline"]), str(binaries["candidate"]),
                    "--cog", str(binaries["cog"]), "--convco", str(binaries["convco"]),
                    "--samples", "3", "--output", str(output), *flags]
            expected = {payload: code for payload, code in agent.corpus().values()}

            def fake_invoke(command, cwd, payload, env):
                tool = Path(command[0]).name
                code = expected[payload]
                if wrong_competitor and tool == "convco":
                    code = 2
                stdout = json.dumps({"exit": code}).encode() + b"\n"
                if change_candidate and tool == "candidate":
                    stdout += b" "
                return subprocess.CompletedProcess(command, code, stdout, b"")

            with patch.object(sys, "argv", argv), patch.object(agent, "invoke", side_effect=fake_invoke), \
                    patch.object(agent.subprocess, "check_output", return_value="test-version\n"), \
                    patch.object(agent.subprocess, "run"), \
                    patch.object(agent.platform, "platform", return_value="test-platform"), \
                    contextlib.redirect_stdout(io.StringIO()):
                agent.main()
            return json.loads(output.read_text())

    def test_complete_protocol_and_outside_scope(self):
        report = self.run_protocol()
        self.assertTrue(report["complete"])
        self.assertEqual(len(report["cases"]), 8)
        self.assertEqual(len(report["behavior_cases"]), 4 * len(agent.corpus()))
        self.assertEqual(set(report["cases"]["outside/valid_small"]["samples_ms"]),
                         {"baseline", "candidate"})
        self.assertEqual(len(report["cases"]["auto_absent/valid_small"]["samples_ms"]["cog"]), 3)

    def test_legacy_baseline_maps_to_explicit_candidate_policy(self):
        report = self.run_protocol(flags=("--legacy-baseline",))
        defaults = report["cases"]["auto_absent/valid_small"]["commands"]
        configured = report["cases"]["auto_present/valid_small"]["commands"]
        self.assertNotIn("--defaults", defaults["baseline"])
        self.assertIn("--defaults", defaults["candidate"])
        self.assertNotIn("--config", configured["baseline"])
        self.assertIn("--config", configured["candidate"])
        self.assertNotIn("--defaults", configured["candidate"])

    def test_rejects_candidate_byte_difference(self):
        with self.assertRaisesRegex(RuntimeError, "clint behavior changed"):
            self.run_protocol(change_candidate=True)

    def test_rejects_competitor_operational_failure(self):
        with self.assertRaises(RuntimeError):
            self.run_protocol(wrong_competitor=True)


if __name__ == "__main__":
    unittest.main()
