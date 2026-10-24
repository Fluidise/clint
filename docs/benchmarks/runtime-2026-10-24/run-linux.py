"""Run an offline, isolated Linux validation and preserve results and logs."""
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import time

output = Path('/results')
commands = [
    ('format', ['cargo', 'fmt', '--all', '--', '--check']),
    ('clippy', ['cargo', 'clippy', '--offline', '--locked', '--all-targets', '--', '-D', 'warnings']),
    ('rust_tests', ['cargo', 'test', '--offline', '--locked', '--all-targets']),
    ('doctests', ['cargo', 'test', '--offline', '--locked', '--doc']),
    ('benchmark_tests', ['python3', '-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_benchmark*.py', '-v']),
    ('oracle', ['python3', 'scripts/benchmark_suite.py', '--clint', 'target/release/fluidise-clint',
                '--commitlint', '/opt/commitlint/node_modules/.bin/commitlint',
                '--commitizen', '/opt/commitizen/bin/cz', '--timing', '--samples', '7',
                '--iterations', '1', '--output', '/results/correctness.json']),
]
report = {'complete': False, 'source_revision': '54e0ab15d694c684d6bb8c02249d507db9c64a36',
          'source_overrides': ['scripts/benchmark_suite.py', 'tests/test_benchmark_suite.py'],
          'platform': platform.platform(), 'machine': platform.machine(),
          'network': 'none during validation; dependencies fetched at image build time',
          'checks': [], 'versions': {}, 'sha256': {}}
for command in (['rustc', '--version'], ['cargo', '--version'], ['node', '--version'],
                ['npm', '--version'], ['python3', '--version'], ['git', '--version'], ['cz', 'version']):
    report['versions'][' '.join(command)] = subprocess.check_output(command, text=True).strip()
for path in ('scripts/benchmark_suite.py', 'scripts/benchmark.py', 'tests/benchmark-corpus.json',
             'tests/test_benchmark_suite.py', 'Cargo.lock', 'target/release/fluidise-clint', '/opt/commitlint/package-lock.json', '/opt/requirements.txt'):
    report['sha256'][path] = hashlib.sha256(Path(path).read_bytes()).hexdigest()
for name, command in commands:
    started = time.monotonic()
    with (output / (name + '.log')).open('wb') as log:
        completed = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=300)
    report['checks'].append({'name': name, 'command': command, 'exit': completed.returncode,
                             'seconds': time.monotonic()-started})
    (output / 'validation.json').write_text(json.dumps(report, indent=2)+'\n')
    print(name, completed.returncode, flush=True)
report['complete'] = True
report['passed'] = all(check['exit'] == 0 for check in report['checks'])
(output / 'validation.json').write_text(json.dumps(report, indent=2)+'\n')
raise SystemExit(0 if report['passed'] else 1)
