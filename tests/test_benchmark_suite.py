"""Tests of the benchmark's oracle and admission gates, not speed assertions."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'
sys.path.insert(0, str(SCRIPTS))
import benchmark_suite as suite
import benchmark as legacy


class OracleTests(unittest.TestCase):
    def test_corpus_has_provenance_and_separates_contracts(self):
        cases = suite.load_cases()
        self.assertTrue(any(c['origin']['kind'] == 'public' for c in cases))
        self.assertTrue(any(c['expected']['status'] == 'invalid' and c['shared'] for c in cases))
        self.assertTrue(any(c['group'] == 'configuration' for c in cases))
        for c in cases:
            if 'pad_to_bytes' in c:
                self.assertEqual(len(suite.payload(c)), c['pad_to_bytes'])
        self.assertIn(b'\xff', suite.payload(next(c for c in cases if c['id'] == 'invalid_utf8')))

    def test_duplicate_oracle_ids_are_rejected(self):
        case = suite.load_cases()[0]
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'corpus.json'
            path.write_text(json.dumps({'schema_version': 1, 'cases': [case, case]}))
            with self.assertRaisesRegex(ValueError, 'duplicate'):
                suite.load_cases(path)

    def test_exit_codes_do_not_conflate_operational_failures(self):
        case = suite.load_cases()[8]  # no_type
        self.assertEqual(case['expected']['status'], 'invalid')
        for tool, code in [('commitizen', 19), ('commitlint', 9), ('cog', 101), ('convco', 2)]:
            out = subprocess.CompletedProcess([], code, b'', b'failed to configure')
            self.assertEqual(suite.judge(tool, out, case)['outcome'], 'operational')
        self.assertEqual(suite.judge('clint', None, case)['outcome'], 'timeout')
        for tool, code in [('commitizen', 14), ('commitlint', 3)]:
            self.assertTrue(suite.judge(tool, subprocess.CompletedProcess([], code, b'', b''), case)['passed'])

    def test_json_status_and_diagnostics_are_not_inferred_from_exit(self):
        case = suite.load_cases()[0]
        for output in (b'not json', b'{}', b'{"status":"invalid"}',
                       b'{"schema_version":1,"status":"valid","diagnostics":[{"code":"unexpected"}]}'):
            self.assertFalse(suite.judge('clint', subprocess.CompletedProcess([], 0, output, b''), case)['passed'])

    def test_environment_does_not_inherit_runtime_or_config_injection(self):
        with patch.dict(suite.os.environ, {'PATH': '/bin', 'NODE_OPTIONS': '--require=evil',
                                          'PYTHONPATH': '/evil', 'GIT_DIR': '/evil',
                                          'CZ_CONFIG': '/evil'}, clear=True):
            env = suite.environment(Path('/isolated'))
        self.assertEqual(env['PATH'], '/bin')
        for key in ('NODE_OPTIONS', 'PYTHONPATH', 'GIT_DIR', 'CZ_CONFIG'):
            self.assertNotIn(key, env)

    def test_one_shared_disagreement_excludes_tool_from_all_timings(self):
        cases = suite.load_cases()[:2]
        references = {c['id']: {'clint': {'passed': True, 'elapsed_ms': 1},
                               'cog': {'passed': c is cases[0], 'elapsed_ms': 2}} for c in cases}
        good = subprocess.CompletedProcess([], 0, json.dumps({'schema_version': 1, 'status': 'valid',
                                          'diagnostics': [], 'diagnostics_complete': True}).encode()+b'\n', b'')
        with patch.object(suite, 'invoke', return_value=(good, 1.0)) as invoke:
            result = suite.measure({'clint': Path('/clint'), 'cog': Path('/cog')}, cases, references,
                                   Path('/repo'), {}, 3, 1)
        self.assertEqual(result['eligible'], ['clint'])
        self.assertIn('cog', result['excluded'])
        self.assertTrue(all(call.args[0][0] == '/clint' for call in invoke.call_args_list))
        self.assertEqual(len(result['cases'][cases[0]['id']]['clint']['samples_ms']), 3)

    def test_unstable_timing_aborts_instead_of_reporting_fast_failure(self):
        case = suite.load_cases()[0]
        refs = {case['id']: {'clint': {'passed': True, 'elapsed_ms': 1}}}
        with patch.object(suite, 'invoke', return_value=(None, None)):
            with self.assertRaisesRegex(RuntimeError, 'unstable'):
                suite.measure({'clint': Path('/clint')}, [case], refs, Path('/repo'), {}, 3, 1)

    def test_legacy_guards_survive_optimized_python(self):
        out = subprocess.run([sys.executable, '-O', '-c',
                              'import sys; sys.path.insert(0, sys.argv[1]); '
                              'from benchmark import require; require(False, "invalid measurement")', str(SCRIPTS)],
                             capture_output=True, timeout=10)
        self.assertNotEqual(out.returncode, 0)
        self.assertIn(b'RuntimeError: invalid measurement', out.stderr)

    def test_legacy_harness_has_no_optimizable_asserts(self):
        import ast
        tree = ast.parse(Path(legacy.__file__).read_text())
        self.assertFalse(any(isinstance(node, ast.Assert) for node in ast.walk(tree)))

    def test_oracle_checks_survive_python_optimization(self):
        # The new harness uses explicit checks, not optimizable assert statements.
        import ast
        tree = ast.parse(Path(suite.__file__).read_text())
        self.assertFalse(any(isinstance(node, ast.Assert) for node in ast.walk(tree)))


if __name__ == '__main__':
    unittest.main()
