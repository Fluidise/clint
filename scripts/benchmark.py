#!/usr/bin/env python3
"""Compare two frozen release builds using only Python's standard library.

Each build directory contains fluidise-clint and the benchmark-engine driver.
No builds, dependency installation or repository mutation are performed here.
"""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


LIMIT = 1_048_576


def corpus():
    header = b"feat(core): implement bounded memory stream\n\n"
    cases = {
        "valid_small": (b"feat: improve validation\n", 0),
        "invalid_small": (b"fix something without type separator\n", 1),
        "multiline_2k": (header + b"- preserve original byte offsets\n" * 55, 0),
        "invalid_utf8": (b"feat: \xff", 2),
        "redos_100k": (b"feat(" + b"a" * 100_000, 1),
        "many_lines": (header + b"x\n" * 500_000, 0),
        "trailing_blank_lines": (header + b"\n" * 1_000_000, 0),
        "many_footers": (header + b"Refs #\n" * 140_000, 1),
        "merge_many_lines": (b"Merge branch 'topic'\n" + b"x\n" * 500_000, 0),
        "extreme_20mib": (header + b"a" * (20 * LIMIT - len(header)), 2),
    }
    for size in [LIMIT - 1, LIMIT, LIMIT + 1]:
        cases[f"boundary_{size}"] = (
            header + b"a" * (size - len(header)), 0 if size <= LIMIT else 2
        )
    return cases


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summary(values):
    median = statistics.median(values)
    return {"median": median, "mad": statistics.median(abs(x - median) for x in values)}


def require(condition, context):
    if not condition:
        raise RuntimeError(context)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--samples", type=int, default=15)
    parser.add_argument("--legacy-baseline", action="store_true", help="baseline uses implicit discovery")
    parser.add_argument("--legacy-candidate", action="store_true", help="candidate uses implicit discovery")
    args = parser.parse_args()
    if args.samples < 3:
        parser.error("at least 3 samples required")
    builds = {key: getattr(args, key).resolve() for key in ["baseline", "candidate"]}
    flags = {"baseline": [] if args.legacy_baseline else ["--defaults"], "candidate": [] if args.legacy_candidate else ["--defaults"]}
    protected = {path / name for path in builds.values() for name in ("fluidise-clint", "engine")}
    protected.add(Path(__file__).resolve())
    if args.output.resolve() in protected:
        parser.error("output must not overwrite frozen build or harness")
    results = {
        "complete": False,
        "environment": {"platform": platform.platform(), "python": platform.python_version()},
        "method": {
            "samples": args.samples, "warmups": 3, "engine_iterations": 100,
            "policy_flags": flags,
            "cli_unit": "ms including Python subprocess overhead; file input outside Git",
            "engine_unit": "ns per check; input and policy preparation excluded",
            "rss_unit": "bytes on macOS, collected separately with /usr/bin/time -l",
            "acceptance": "descriptive only; historical optimization targets are not statistical significance or general acceptance criteria",
        },
        "harness_sha256": digest(Path(__file__)),
        "builds": {key: {name: digest(path / name) for name in ["fluidise-clint", "engine"]}
                   for key, path in builds.items()},
        "cases": {},
    }
    for key, path in builds.items():
        manifest = path / "sources.json"
        if manifest.is_file():
            results["builds"][key]["sources"] = json.loads(manifest.read_text())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    with tempfile.TemporaryDirectory(prefix="clint-benchmark-") as temporary:
        cwd = Path(temporary)
        for name, (payload, expected) in corpus().items():
            message = cwd / "message"
            message.write_bytes(payload)
            row = {"bytes": len(payload), "sha256": digest(message), "expected_exit": expected}
            observations = {key: {"cli_ms": [], "engine_ns": [], "rss_bytes": []} for key in builds}
            comparable = {}
            for sample in range(-3, args.samples):
                for key in (["baseline", "candidate"] if sample % 2 else ["candidate", "baseline"]):
                    build = builds[key]
                    start = time.perf_counter_ns()
                    out = subprocess.run([str(build / "fluidise-clint"), "--file", str(message), *flags[key]],
                                         cwd=cwd, capture_output=True, timeout=10)
                    elapsed = (time.perf_counter_ns() - start) / 1_000_000
                    require(out.returncode == expected, (name, key, out))
                    require(not out.stderr, (name, key, out.stderr))
                    require(out.stdout.endswith(b"\n"), (name, key, "missing newline"))
                    json.loads(out.stdout)
                    identity = (out.returncode, out.stdout, out.stderr)
                    comparable.setdefault("cli", identity)
                    require(comparable["cli"] == identity, (name, key, "CLI behavior changed"))
                    if sample >= 0:
                        observations[key]["cli_ms"].append(elapsed)
                    if name != "invalid_utf8":
                        out = subprocess.run([str(build / "engine"), str(message), "100"],
                                             cwd=cwd, capture_output=True, timeout=20, check=True)
                        elapsed, identity = out.stdout.split(b"\n", 1)
                        comparable.setdefault("engine", identity)
                        require(comparable["engine"] == identity, (name, key, "engine behavior changed"))
                        if sample >= 0:
                            observations[key]["engine_ns"].append(int(elapsed) / 100)
            # RSS is a separate fresh-process observation, not cold caches or a memory cap.
            if platform.system() == "Darwin":
                for sample in range(5):
                    for key in (["baseline", "candidate"] if sample % 2 else ["candidate", "baseline"]):
                        out = subprocess.run(["/usr/bin/time", "-l", str(builds[key] / "fluidise-clint"),
                                              "--file", str(message), *flags[key]], cwd=cwd, capture_output=True, timeout=10)
                        require(out.returncode == expected, (name, key, "RSS invocation failed"))
                        for line in out.stderr.decode().splitlines():
                            if line.strip().endswith("maximum resident set size"):
                                observations[key]["rss_bytes"].append(int(line.split()[0]))
                        require(len(observations[key]["rss_bytes"]) == sample + 1, (name, key, "missing RSS"))
            for key, samples in observations.items():
                row[key] = {metric: {"samples": values, **summary(values)}
                            for metric, values in samples.items() if values}
            row["behavior_sha256"] = hashlib.sha256(comparable["cli"][1]).hexdigest()
            results["cases"][name] = row
            # Keep completed cases if a later case fails.
            args.output.write_text(json.dumps(results, indent=2) + "\n")
            before = row["baseline"]["cli_ms"]["median"]
            after = row["candidate"]["cli_ms"]["median"]
            print(f"{name}: CLI {before:.3f} -> {after:.3f} ms", flush=True)
    for key, path in builds.items():
        for name in ("fluidise-clint", "engine"):
            require(digest(path / name) == results["builds"][key][name],
                    (key, name, "binary changed during measurement"))
    results["complete"] = True
    args.output.write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    main()
