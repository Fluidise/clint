#!/usr/bin/env python3
"""Check a documented oracle before optional, gated CLI latency measurements.

No installation, builds or network calls. Creates temporary fixtures and an
--output report. Competitors are opt-in local executables, never download wrappers.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import shutil
import subprocess
import tempfile
import time

from benchmark import digest, summary

CORPUS = Path(__file__).resolve().parents[1] / 'tests' / 'benchmark-corpus.json'
TOOLS = ('clint', 'cog', 'convco', 'commitlint', 'commitizen')
EXIT_CODES = {
    'clint': {0: 'accept', 1: 'reject', 2: 'operational'},
    'cog': {0: 'accept', 1: 'reject'},
    'convco': {0: 'accept', 1: 'reject'},
    'commitlint': {0: 'accept', 3: 'reject'},  # --strict, no warning rules
    'commitizen': {0: 'accept', 14: 'reject', 32: 'reject'},
}
CONFIGS = {
    'cog.toml': '',
    '.convco': '{}\n',
    '.cz.toml': '[tool.commitizen]\nname = "cz_conventional_commits"\n',
    # Explicit limited policy, not the package's more opinionated defaults.
    'commitlint.config.cjs': r'''module.exports = {
  defaultIgnores: false,
  parserPreset: {parserOpts: {
    headerPattern: /^(\w+)(?:\(([^)]+)\))?!?: (.+)$/,
    headerCorrespondence: ['type', 'scope', 'subject'],
    noteKeywords: ['BREAKING CHANGE', 'BREAKING-CHANGE']
  }},
  rules: {
    'type-empty': [2, 'never'], 'subject-empty': [2, 'never'],
    'type-enum': [2, 'always', ['feat','fix','docs','style','refactor','perf','test','build','ci','chore','revert']],
    'body-leading-blank': [2, 'always']
  }
};
''',
}


def payload(case):
    data = bytes.fromhex(case['hex']) if 'hex' in case else case['message'].encode()
    if 'suffix_repeat' in case:
        repeat = case['suffix_repeat']
        data += repeat['text'].encode() * repeat['count']
    if 'pad_to_bytes' in case:
        data += b'a' * (case['pad_to_bytes'] - len(data))
    return data


def load_cases(path=CORPUS):
    document = json.loads(path.read_text())
    cases = document['cases']
    if document['schema_version'] != 1 or not cases:
        raise ValueError('unsupported or empty oracle')
    ids = set()
    for case in cases:
        if case['id'] in ids:
            raise ValueError('duplicate case ID')
        ids.add(case['id'])
        if not case['basis'] or not case['rationale'] or not case['origin']['kind']:
            raise ValueError('missing oracle provenance')
        if case['expected']['status'] not in ('valid', 'invalid', 'ignored', 'error'):
            raise ValueError('invalid oracle status')
        if case['shared'] and ('config' in case or case['expected']['status'] not in ('valid', 'invalid')):
            raise ValueError('policy/operational case cannot define shared timing contract')
        if 'pad_to_bytes' in case and len(payload(case)) != case['pad_to_bytes']:
            raise ValueError('padding target smaller than message')
        payload(case)
    return cases


def environment(home):
    # Allowlist prevents user-specific Git, tool, Python and Node routing options.
    env = {key: os.environ[key] for key in ('PATH', 'SYSTEMROOT', 'WINDIR', 'TMPDIR', 'TEMP', 'TMP')
           if key in os.environ}
    env.update(HOME=str(home), XDG_CONFIG_HOME=str(home), LANG='C.UTF-8', TZ='UTC',
               GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=str(home / '.gitconfig'),
               GIT_CONFIG_COUNT='0', GIT_OPTIONAL_LOCKS='0', NO_COLOR='1')
    return env


def version_command(tool, binary):
    # Commitizen exposes a subcommand, not a top-level --version flag.
    return [str(binary), 'version' if tool == 'commitizen' else '--version']


def command(tool, binary, cwd, case):
    if tool == 'clint':
        selection = ['--config', str(cwd / 'fluidise.toml')] if 'config' in case else ['--defaults']
        return [str(binary), '--stdin', *selection]
    if tool == 'cog':
        return [str(binary), '--config', str(cwd / 'cog.toml'), 'verify', '--file', '-']
    if tool == 'convco':
        return [str(binary), 'check', '--config', str(cwd / '.convco'), '--from-stdin']
    if tool == 'commitlint':
        return [str(binary), '--config', str(cwd / 'commitlint.config.cjs'), '--strict']
    return [str(binary), '--config', str(cwd / '.cz.toml'), 'check']


def invoke(cmd, cwd, data, env):
    start = time.perf_counter_ns()
    try:
        out = subprocess.run(cmd, cwd=cwd, input=data, env=env, capture_output=True, timeout=10)
        return out, (time.perf_counter_ns() - start) / 1_000_000
    except subprocess.TimeoutExpired:
        return None, None


def judge(tool, out, case):
    expected = case['expected']
    if out is None:
        return {'outcome': 'timeout', 'passed': False}
    classification = EXIT_CODES[tool].get(out.returncode, 'operational')
    observation = {'exit': out.returncode, 'classification': classification,
                   'stdout': out.stdout.decode('utf-8', errors='replace'),
                   'stderr': out.stderr.decode('utf-8', errors='replace')}
    target = 'accept' if expected['status'] in ('valid', 'ignored') else 'reject'
    if tool == 'clint':
        try:
            report = json.loads(out.stdout)
            codes = [d['code'] for d in report['diagnostics']]
            passed = (out.returncode == {'valid': 0, 'ignored': 0, 'invalid': 1, 'error': 2}[expected['status']]
                      and report['status'] == expected['status'] and report['schema_version'] == 1
                      and report['diagnostics_complete'] is True
                      and codes == expected['diagnostic_codes'] and not out.stderr
                      and out.stdout.endswith(b'\n') and len(out.stdout) <= 262144)
        except (ValueError, KeyError, TypeError):
            passed = False
    else:
        passed = classification == target
    observation.update(passed=passed, outcome='pass' if passed else
                       ('operational' if classification == 'operational' else
                        'false_accept' if classification == 'accept' and target == 'reject' else
                        'false_reject' if classification == 'reject' and target == 'accept' else
                        'contract_mismatch'))
    return observation


def measure(binaries, cases, references, cwd, env, samples, iterations):
    """Only tools passing ALL shared cases may enter shared latency measurements."""
    shared = [c for c in cases if c['shared']]
    eligible = [t for t in binaries if all(references[c['id']][t]['passed'] for c in shared)]
    result = {'eligible': eligible, 'excluded': {t: 'failed shared oracle; inspect correctness rows'
              for t in binaries if t not in eligible}, 'cases': {}}
    rng = random.Random(20261023)
    for case in shared:
        # Multiple distinct shapes; do not multiply near-identical cases by a headline score.
        row = {tool: {'samples_ms': [], 'first_case_observation_ms': references[case['id']][tool]['elapsed_ms']}
               for tool in eligible}
        for sample in range(-2, samples):
            order = eligible.copy()
            rng.shuffle(order)
            for tool in order:
                total = 0
                for _ in range(iterations):
                    out, elapsed = invoke(command(tool, binaries[tool], cwd, case), cwd, payload(case), env)
                    if not judge(tool, out, case)['passed']:
                        raise RuntimeError(f'unstable timing validation: {tool}/{case["id"]}')
                    total += elapsed
                if sample >= 0:
                    row[tool]['samples_ms'].append(total / iterations)
        for values in row.values():
            values.update(summary(values['samples_ms']))
        result['cases'][case['id']] = row
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for tool in TOOLS:
        parser.add_argument('--' + tool, type=Path, required=tool == 'clint')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timing', action='store_true')
    parser.add_argument('--samples', type=int, default=7)
    parser.add_argument('--iterations', type=int, default=10)
    args = parser.parse_args()
    if args.samples < 3 or args.iterations < 1:
        parser.error('at least three samples and one iteration required')
    binaries = {t: getattr(args, t).resolve() for t in TOOLS if getattr(args, t)}
    if any(not p.is_file() for p in binaries.values()):
        parser.error('every selected executable must exist')
    if args.output.resolve() in binaries.values() or args.output.resolve() in (CORPUS, Path(__file__).resolve(), Path(__file__).with_name('benchmark.py').resolve()):
        parser.error('output must not overwrite executable, harness or corpus')
    cases = load_cases()
    report = {'complete': False, 'schema_version': 1,
              'corpus_sha256': digest(CORPUS), 'harness_sha256': digest(Path(__file__)),
              'support_sha256': digest(Path(__file__).with_name('benchmark.py')),
              'environment': {'platform': platform.platform(), 'machine': platform.machine(),
                              'python': platform.python_version(), 'python_path': os.sys.executable},
              'method': {'samples': args.samples, 'iterations': args.iterations, 'warmups': 2,
                         'order_seed': 20261023,
                         'unit': 'ms per independent stdin CLI process; native output captured',
                         'cache': 'uncontrolled OS caches; first observation is NOT cold startup',
                         'gate': 'all shared oracle cases must pass before timing a tool',
                         'scope': 'serial CLI latency, not hooks, agents or in-process engine',
                         'threshold': 'descriptive median/MAD only; no significance or speed target'},
              'excluded': {t: 'local executable not supplied; not installed or measured' for t in TOOLS if t not in binaries},
              'configs': CONFIGS, 'tools': {}, 'correctness': {}}
    args.output.parent.mkdir(parents=True, exist_ok=True)

    def save():
        args.output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')

    save()  # Stale complete results cannot survive a failed run.
    with tempfile.TemporaryDirectory(prefix='clint-suite-') as temporary:
        root = Path(temporary)
        home = root / 'home'
        home.mkdir()
        (home / '.gitconfig').write_text('[user]\nname=Benchmark\nemail=benchmark@example.invalid\n')
        env = environment(home)
        cwd = root / 'repository'
        subprocess.run(['git', '-c', 'init.defaultBranch=main', 'init', '--quiet', '--template=', str(cwd)],
                       env=env, check=True, capture_output=True, timeout=10)
        report['environment']['git'] = subprocess.check_output(['git', '--version'], env=env, text=True).strip()
        report['environment']['git_path'] = shutil.which('git', path=env.get('PATH'))
        for name, content in CONFIGS.items():
            (cwd / name).write_text(content)
        for tool, binary in binaries.items():
            report['tools'][tool] = {'path': str(binary), 'sha256': digest(binary),
                                    'version': subprocess.check_output(version_command(tool, binary),
                                    cwd=cwd, env=env, timeout=10, text=True).strip()}
        save()
        for case in cases:
            if 'config' in case:
                (cwd / 'fluidise.toml').write_text(case['config'])
            row = {'group': case['group'], 'shared': case['shared'], 'basis': case['basis'],
                   'rationale': case['rationale'], 'origin': case['origin'], 'expected': case['expected'],
                   'bytes': len(payload(case)), 'sha256': hashlib.sha256(payload(case)).hexdigest()}
            for tool, binary in binaries.items():
                if tool != 'clint' and not case['shared']:
                    row[tool] = {'outcome': 'excluded', 'reason': 'clint-specific policy or operational contract'}
                    continue
                cmd = command(tool, binary, cwd, case)
                out, elapsed = invoke(cmd, cwd, payload(case), env)
                row[tool] = {**judge(tool, out, case), 'command': cmd, 'elapsed_ms': elapsed}
            report['correctness'][case['id']] = row
            save()
        if args.timing:
            # Defaults selection ignores any fluidise.toml left by configuration tests.
            report['timing'] = measure(binaries, cases, report['correctness'], cwd, env,
                                       args.samples, args.iterations)
        for tool, binary in binaries.items():
            if digest(binary) != report['tools'][tool]['sha256']:
                raise RuntimeError(f'executable changed during run: {tool}')
        if (digest(Path(__file__)) != report['harness_sha256'] or
                digest(Path(__file__).with_name('benchmark.py')) != report['support_sha256']):
            raise RuntimeError('harness changed during run')
        if digest(CORPUS) != report['corpus_sha256']:
            raise RuntimeError('oracle changed during run')
    report['summary_by_group'] = {}
    for row in report['correctness'].values():
        group = report['summary_by_group'].setdefault(row['group'], {})
        for tool in binaries:
            counts = group.setdefault(tool, {})
            outcome = row[tool]['outcome']
            counts[outcome] = counts.get(outcome, 0) + 1
    report['complete'] = True
    report['clint_passed'] = all(row['clint']['passed'] for row in report['correctness'].values())
    save()
    print(json.dumps({'complete': True, 'clint_passed': report['clint_passed'],
                      'cases': len(cases), 'excluded': report['excluded'],
                      'clint_failures': [name for name, row in report['correctness'].items()
                                         if not row['clint']['passed']]}))
    return 0 if report['clint_passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
