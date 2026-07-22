#!/usr/bin/env python3
"""Run the stable shape-evidence measurements and derive retained summaries."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import platform
import re
import signal
import statistics
import subprocess
import sys
import time
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
RAW = SPIKE / "measurements" / "raw"
COUNTS = (1, 10, 100, 1000)
SPELLINGS = {
    "shapes": "open_descriptor",
    "family": "owned_family",
    "tuple": "dimension_tuple",
}
TOOLCHAIN = "1.89.0"
TIMEOUT_SECONDS = 300


def run(
    command: list[str], *, timed: bool = False, environment: dict[str, str] | None = None
) -> dict[str, object]:
    """Run one subprocess with a process-group deadline and retained output."""
    actual = command
    if timed:
        time_flag = "-lp" if sys.platform == "darwin" else "-v"
        actual = ["/usr/bin/time", time_flag, *command]
    started = time.monotonic()
    process = subprocess.Popen(
        actual,
        cwd=SPIKE,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired as error:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        raise RuntimeError(
            f"command exceeded {TIMEOUT_SECONDS}s deadline: {' '.join(command)}"
        ) from error
    elapsed = time.monotonic() - started
    if process.returncode != 0:
        raise RuntimeError(f"command failed ({process.returncode}): {' '.join(command)}\n{stderr}")
    peak_rss_bytes = None
    if timed:
        if sys.platform == "darwin":
            match = re.search(r"(\d+)\s+maximum resident set size", stderr)
            peak_rss_bytes = int(match.group(1)) if match else None
        else:
            match = re.search(r"Maximum resident set size \(kbytes\):\s+(\d+)", stderr)
            peak_rss_bytes = int(match.group(1)) * 1024 if match else None
        if peak_rss_bytes is None:
            raise RuntimeError("could not parse peak RSS from /usr/bin/time output")
    return {
        "command": command,
        "elapsed_seconds": round(elapsed, 6),
        "peak_rss_bytes": peak_rss_bytes,
        "stdout": stdout,
        "stderr": stderr,
    }


def retain(label: str, phase: str, sample: int, result: dict[str, object]) -> dict[str, object]:
    """Retain raw streams and return the normalized measurement fields."""
    destination = RAW / label
    destination.mkdir(parents=True, exist_ok=True)
    (destination / f"{phase}.{sample}.stdout").write_text(str(result.pop("stdout")))
    (destination / f"{phase}.{sample}.stderr").write_text(str(result.pop("stderr")))
    command = [part.replace(str(SPIKE), "<spike>") for part in result.pop("command")]
    (destination / f"{phase}.{sample}.command.json").write_text(
        json.dumps(command, indent=2) + "\n"
    )
    return result


def cargo(*arguments: str) -> list[str]:
    return ["cargo", f"+{TOOLCHAIN}", *arguments, "--manifest-path", str(SPIKE / "Cargo.toml")]


def provenance() -> dict[str, object]:
    """Capture the exact environment that bounds this run."""
    rustc = run(["rustc", f"+{TOOLCHAIN}", "--version", "--verbose"])["stdout"]
    cargo_version = run(["cargo", f"+{TOOLCHAIN}", "--version", "--verbose"])["stdout"]
    measured_sources = [
        SPIKE / "Cargo.lock",
        SPIKE / "Cargo.toml",
        SPIKE / "generate-workloads.sh",
        SPIKE / "measure.py",
        SPIKE / "src" / "lib.rs",
        *sorted((SPIKE / "src" / "bin").glob("*.rs")),
    ]
    digest = hashlib.sha256()
    for source in measured_sources:
        digest.update(str(source.relative_to(SPIKE)).encode())
        digest.update(b"\0")
        digest.update(source.read_bytes())
        digest.update(b"\0")
    repository_head = run(["git", "rev-parse", "HEAD"])["stdout"]
    return {
        "measured_at_utc": dt.datetime.now(dt.UTC).isoformat(timespec="seconds"),
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "python": platform.python_version(),
        },
        "toolchain": {
            "selector": TOOLCHAIN,
            "rustc_verbose": str(rustc).strip(),
            "cargo_verbose": str(cargo_version).strip(),
        },
        "source": {
            "repository_base_revision": str(repository_head).strip(),
            "measured_input_tree_sha256": digest.hexdigest(),
        },
        "timeout_seconds_per_subprocess_group": TIMEOUT_SECONDS,
        "timer": "/usr/bin/time -lp" if sys.platform == "darwin" else "/usr/bin/time -v",
        "binary_size_source": "Python Path.stat().st_size",
    }


def clean(target_dir: Path | None = None) -> None:
    command = cargo("clean", "-p", "shape-evidence-spike")
    if target_dir is not None:
        command.extend(["--target-dir", str(target_dir)])
    run(command)


def measure_baseline() -> dict[str, object]:
    """Measure one clean, incremental, and optimized build per scale."""
    records = []
    for count in COUNTS:
        binary_name = f"shapes_{count}"
        clean()
        check = cargo("check", "--bin", binary_name)
        cold = retain("baseline", f"{binary_name}.cold", 1, run(check, timed=True))
        (SPIKE / "src" / "bin" / f"{binary_name}.rs").touch()
        incremental = retain("baseline", f"{binary_name}.incremental", 1, run(check, timed=True))
        release_command = cargo("build", "--release", "--bin", binary_name)
        release = retain("baseline", f"{binary_name}.release", 1, run(release_command, timed=True))
        records.append(
            {
                "distinct_shapes": count,
                "cold_check": cold,
                "incremental_check": incremental,
                "optimized_build": release,
                "optimized_binary_bytes": (SPIKE / "target" / "release" / binary_name)
                .stat()
                .st_size,
            }
        )
    return {
        "schema": "tiler.shape-evidence-measurement/v2",
        **provenance(),
        "method": {
            "samples_per_case": 1,
            "workload_counts": list(COUNTS),
            "commands": {
                "cold_and_incremental_check": (
                    "cargo +1.89.0 check --bin <binary> --manifest-path <spike>/Cargo.toml"
                ),
                "optimized_build": (
                    "cargo +1.89.0 build --release --bin <binary> "
                    "--manifest-path <spike>/Cargo.toml"
                ),
            },
        },
        "results": records,
    }


def median(samples: list[dict[str, object]], field: str) -> float:
    return round(statistics.median(float(sample[field]) for sample in samples), 6)


def measure_spellings() -> dict[str, object]:
    """Measure all three public static-shape spellings with retained samples."""
    records = []
    for prefix, spelling in SPELLINGS.items():
        for count in COUNTS:
            binary_name = f"{prefix}_{count}"
            check_samples = []
            release_samples = []
            binary_sizes = []
            for sample in range(1, 6):
                check_target = RAW / "spellings" / "target-check"
                release_target = RAW / "spellings" / "target-release"
                clean(check_target)
                clean(release_target)
                check_command = cargo("check", "--bin", binary_name)
                check_command.extend(["--target-dir", str(check_target)])
                check_samples.append(
                    retain(
                        "spellings",
                        f"{binary_name}.check",
                        sample,
                        run(check_command, timed=True),
                    )
                )
                release_command = cargo("build", "--release", "--bin", binary_name)
                release_command.extend(["--target-dir", str(release_target)])
                release_samples.append(
                    retain(
                        "spellings",
                        f"{binary_name}.release",
                        sample,
                        run(release_command, timed=True),
                    )
                )
                binary_sizes.append((release_target / "release" / binary_name).stat().st_size)
            if len(set(binary_sizes)) != 1:
                raise RuntimeError(f"binary size varied across samples for {binary_name}")
            records.append(
                {
                    "spelling": spelling,
                    "distinct_shapes": count,
                    "source_bytes": (SPIKE / "src" / "bin" / f"{binary_name}.rs").stat().st_size,
                    "binary_bytes": binary_sizes[0],
                    "check_samples": check_samples,
                    "release_samples": release_samples,
                    "median_check_seconds": median(check_samples, "elapsed_seconds"),
                    "median_release_seconds": median(release_samples, "elapsed_seconds"),
                    "median_check_peak_rss_bytes": int(median(check_samples, "peak_rss_bytes")),
                    "median_release_peak_rss_bytes": int(median(release_samples, "peak_rss_bytes")),
                }
            )
    return {
        "schema": "tiler.shape-evidence-spelling-measurement/v2",
        **provenance(),
        "method": {
            "samples_per_case": 5,
            "reported_statistic": "median",
            "workload_counts": list(COUNTS),
            "commands": {
                "check": (
                    "cargo +1.89.0 check --bin <binary> --manifest-path "
                    "<spike>/Cargo.toml --target-dir <check-target>"
                ),
                "release": (
                    "cargo +1.89.0 build --release --bin <binary> --manifest-path "
                    "<spike>/Cargo.toml --target-dir <release-target>"
                ),
            },
            "spelling_names": {
                "open_descriptor": "one downstream StaticShapeSpec implementation per exact shape",
                "owned_family": "one library-owned Dims3<A, B, C> const-generic family",
                "dimension_tuple": "one tuple over the library-owned Dim<N> type",
            },
        },
        "results": records,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("suite", choices=("baseline", "spellings", "all"))
    args = parser.parse_args()
    if sys.platform not in {"darwin", "linux"}:
        raise SystemExit("measurement supports only macOS and Linux")
    if not Path("/usr/bin/time").is_file():
        raise SystemExit(
            "measurement requires /usr/bin/time (BSD time on macOS or GNU time on Linux)"
        )
    run([str(SPIKE / "generate-workloads.sh")])
    run(cargo("fetch", "--locked"))
    selections = ("baseline", "spellings") if args.suite == "all" else (args.suite,)
    for selection in selections:
        if selection == "baseline":
            summary = measure_baseline()
            destination = SPIKE / "measurements" / "summary.json"
        else:
            summary = measure_spellings()
            destination = SPIKE / "measurements" / "spelling-summary.json"
        destination.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        print(destination.relative_to(SPIKE))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
