#!/usr/bin/env python3
"""Measure independent CLI validations, not in-process engine throughput.

Consumes frozen binaries; creates only temporary, isolated Git fixtures.
Uses native config/output contracts, never pretends competitor output is JSON.
"""
import argparse
import json
import os
import platform
import random
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

from benchmark import corpus, digest, summary


def environment(home):
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(("GIT_", "CONVCO_", "COG_"))}
    env.update(HOME=str(home), XDG_CONFIG_HOME=str(home),
               GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=str(home / ".gitconfig"),
               GIT_CONFIG_COUNT="0", GIT_OPTIONAL_LOCKS="0", NO_COLOR="1")
    return env


def invoke(command, cwd, payload, env):
    return subprocess.run(command, cwd=cwd, input=payload, env=env,
                          capture_output=True, timeout=10)


def identity(out):
    return out.returncode, out.stdout, out.stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--cog", type=Path, required=True)
    parser.add_argument("--convco", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=7)
    parser.add_argument("--legacy-baseline", action="store_true", help="baseline uses implicit discovery")
    parser.add_argument("--legacy-candidate", action="store_true", help="candidate uses implicit discovery")
    args = parser.parse_args()
    if args.samples < 3:
        parser.error("at least three measured samples required")
    binaries = {k: getattr(args, k).resolve()
                for k in ("baseline", "candidate", "cog", "convco")}
    for path in binaries.values():
        if not path.is_file():
            parser.error(f"missing binary: {path}")
    report = {
        "complete": False,
        "environment": {"platform": platform.platform(), "python": platform.python_version(),
                        "git": subprocess.check_output(["git", "--version"], text=True).strip(),
                        "git_path": shutil.which("git")},
        "method": {"iterations": 100, "samples": args.samples, "warmups": 2,
                   "order_seed": 20261023, "unit": "ms per 100 independent CLI processes",
                   "input": "identical stdin bytes; stdout/stderr captured, validation outside timing",
                   "scope": "local warm-cache serial processes, Python/OS spawn overhead included",
                   "config": "native default-equivalent files; current clint uses explicit policy selection",
                   "legacy_baseline": args.legacy_baseline, "legacy_candidate": args.legacy_candidate,
                   "comparison": "native outputs/configs differ; shared accept/reject subset only",
                   "json_cost": "clint JSON vs --human is an output-mode delta, not serialization-only CPU time"},
        "binaries": {k: {"path": str(p), "sha256": digest(p),
                          "version": subprocess.check_output([str(p), "--version"], text=True).strip()}
                     for k, p in binaries.items()},
        "harness_sha256": digest(Path(__file__)),
        "cases": {}, "behavior_cases": {},
    }
    for tool in ("baseline", "candidate"):
        manifest = binaries[tool].parent / "sources.json"
        if manifest.is_file():
            report["binaries"][tool]["sources"] = json.loads(manifest.read_text())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="clint-agent-") as temporary:
        root = Path(temporary)
        home = root / "home"
        home.mkdir()
        (home / ".gitconfig").write_text("[user]\nname = Benchmark\nemail = benchmark@example.invalid\n")
        env = environment(home)
        directories = {}
        for mode in ("auto_absent", "auto_present", "explicit", "outside"):
            cwd = root / mode
            cwd.mkdir()
            if mode != "outside":
                subprocess.run(["git", "-c", "init.defaultBranch=main", "init", "--quiet",
                                "--template=", str(cwd)], env=env, check=True, capture_output=True)
            if mode in ("auto_present", "explicit"):
                (cwd / "fluidise.toml").write_text("[commit]\nversion=1\n")
                (cwd / "cog.toml").write_text("")
                (cwd / ".convco").write_text("{}\n")
            directories[mode] = cwd

        def command(tool, mode, human=False):
            cwd = directories[mode]
            if tool in ("baseline", "candidate"):
                cmd = [str(binaries[tool]), "--stdin"]
                legacy = getattr(args, f"legacy_{tool}")
                if mode == "explicit" or (mode == "auto_present" and not legacy):
                    cmd += ["--config", str(cwd / "fluidise.toml")]
                elif not legacy:
                    cmd += ["--defaults"]
                if human:
                    cmd += ["--human"]
                return cmd
            if tool == "cog":
                cmd = [str(binaries[tool])]
                if mode == "explicit":
                    cmd += ["--config", str(cwd / "cog.toml")]
                return cmd + ["verify", "--file", "-"]
            cmd = [str(binaries[tool]), "check", "--from-stdin"]
            if mode == "explicit":
                cmd += ["--config", str(cwd / ".convco")]
            return cmd

        for mode, cwd in directories.items():
            for name in ("valid_small", "invalid_small"):
                payload, expected = corpus()[name]
                tools = ["baseline", "candidate"] if mode == "outside" else list(binaries)
                references = {}
                for tool in tools:
                    out = invoke(command(tool, mode), cwd, payload, env)
                    if out.returncode != expected:
                        raise RuntimeError((mode, name, tool, identity(out)))
                    references[tool] = identity(out)
                if references["baseline"] != references["candidate"]:
                    raise RuntimeError((mode, name, "clint behavior changed"))
                row = {"bytes": len(payload), "payload_hex": payload.hex(),
                       "expected_exit": expected,
                       "commands": {t: command(t, mode) for t in tools},
                       "samples_ms": {t: [] for t in tools},
                       "outputs": {t: {"exit": r[0], "stdout": r[1].decode(), "stderr": r[2].decode()}
                                   for t, r in references.items()}}
                rng = random.Random(20261023)
                for sample in range(-2, args.samples):
                    order = tools.copy()
                    rng.shuffle(order)
                    for tool in order:
                        cmd = command(tool, mode)
                        start = time.perf_counter_ns()
                        outputs = [invoke(cmd, cwd, payload, env) for _ in range(100)]
                        elapsed = (time.perf_counter_ns() - start) / 1_000_000
                        if any(out.returncode != expected or (tool in ("baseline", "candidate")
                               and identity(out) != references[tool]) for out in outputs):
                            raise RuntimeError((mode, name, tool, "unstable behavior"))
                        if sample >= 0:
                            row["samples_ms"][tool].append(elapsed)
                row["summary_ms"] = {t: summary(v) for t, v in row["samples_ms"].items()}
                report["cases"][f"{mode}/{name}"] = row
                args.output.write_text(json.dumps(report, indent=2) + "\n")
                print(mode, name, {t: round(v["median"], 2) for t, v in row["summary_ms"].items()}, flush=True)

        # Same corpus as engine/CLI benchmark: exact JSON, codes and stderr,
        # including operational errors and limit boundaries, in each config mode.
        for mode, cwd in directories.items():
            for name, (payload, expected) in corpus().items():
                outputs = [invoke(command(t, mode), cwd, payload, env)
                           for t in ("baseline", "candidate")]
                if identity(outputs[0]) != identity(outputs[1]) or outputs[0].returncode != expected:
                    raise RuntimeError((mode, name, "behavior regression"))
                json.loads(outputs[0].stdout)
                report["behavior_cases"][f"{mode}/{name}"] = {"exit": expected, "bytes": len(payload)}

        # Isolate the native JSON/human output-mode delta without removing JSON
        # from the headline result. This is not an equivalent competitor contract.
        payload, _ = corpus()["valid_small"]
        report["output_mode_ms"] = {}
        for tool in ("baseline", "candidate"):
            values = {"json": [], "human": []}
            rng = random.Random(20261023)
            for sample in range(-2, args.samples):
                order = list(values)
                rng.shuffle(order)
                for mode in order:
                    cmd = command(tool, "auto_absent", human=mode == "human")
                    start = time.perf_counter_ns()
                    outputs = [invoke(cmd, directories["auto_absent"], payload, env) for _ in range(100)]
                    elapsed = (time.perf_counter_ns() - start) / 1_000_000
                    if any(o.returncode != 0 for o in outputs):
                        raise RuntimeError("output mode failed")
                    if sample >= 0:
                        values[mode].append(elapsed)
            report["output_mode_ms"][tool] = {k: {"samples": v, **summary(v)} for k, v in values.items()}
    for tool, path in binaries.items():
        if digest(path) != report["binaries"][tool]["sha256"]:
            raise RuntimeError(f"binary changed during measurement: {tool}")
    report["complete"] = True
    args.output.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
