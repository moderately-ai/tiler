#!/usr/bin/env python3
"""Generate and measure dependent-static-shape compiler workloads."""

from __future__ import annotations

import json
import platform
import re
import subprocess
import sys
import time
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
RAW = SPIKE / "measurements" / "raw"
SUMMARY = SPIKE / "measurements" / "summary.json"
COUNTS = (1, 10, 100, 1000)
TOOLCHAINS = ("nightly-2026-07-19", "nightly-2026-07-20")
TIMEOUT_SECONDS = 300


def run(command: list[str], *, timed: bool = False) -> dict[str, object]:
    """Run one command and return normalized timing and output."""
    actual = command
    if timed:
        actual = ["/usr/bin/time", "-lp" if sys.platform == "darwin" else "-v", *command]
    started = time.monotonic()
    result = subprocess.run(
        actual,
        cwd=SPIKE,
        capture_output=True,
        text=True,
        timeout=TIMEOUT_SECONDS,
        check=False,
    )
    elapsed = time.monotonic() - started
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr}"
        )
    rss_bytes = None
    if timed:
        if sys.platform == "darwin":
            match = re.search(r"(\d+)\s+maximum resident set size", result.stderr)
            rss_bytes = int(match.group(1)) if match else None
        else:
            match = re.search(r"Maximum resident set size \(kbytes\):\s+(\d+)", result.stderr)
            rss_bytes = int(match.group(1)) * 1024 if match else None
    return {
        "command": command,
        "elapsed_seconds": round(elapsed, 6),
        "peak_rss_bytes": rss_bytes,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def generate(count: int) -> Path:
    """Generate one deterministic workload with `count` distinct shape types."""
    destination = SPIKE / "workload" / "src" / "bin" / f"generated_{count}.rs"
    destination.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "//! Generated dependent-static-shape compile workload.",
        "",
        "use nightly_shape_api::StaticShape;",
        "use nightly_shape_workload::touch;",
        "",
        "#[allow(clippy::too_many_lines)]",
        "fn main() {",
    ]
    for index in range(count):
        lines.append(f"    touch::<StaticShape<2, {{ [{index + 1}, {index + 2}] }}>>();")
    lines.append("}")
    destination.write_text("\n".join(lines) + "\n")
    return destination


def toolchain_version(toolchain: str) -> str:
    """Return exact verbose rustc provenance for one toolchain."""
    return run(["rustc", f"+{toolchain}", "--version", "--verbose"])["stdout"].strip()


def measure(toolchain: str, count: int) -> dict[str, object]:
    """Measure clean, warm, incremental, and release compilation."""
    source = generate(count)
    binary = SPIKE / "target" / "release" / f"generated_{count}"
    run(["cargo", f"+{toolchain}", "clean", "-p", "nightly-shape-workload"])
    base = [
        "cargo",
        f"+{toolchain}",
        "check",
        "-p",
        "nightly-shape-workload",
        "--bin",
        f"generated_{count}",
    ]
    clean = run(base, timed=True)
    warm = run(base, timed=True)
    source.touch()
    incremental = run(base, timed=True)
    release = run(
        [
            "cargo",
            f"+{toolchain}",
            "build",
            "--release",
            "-p",
            "nightly-shape-workload",
            "--bin",
            f"generated_{count}",
        ],
        timed=True,
    )
    symbols = run(["nm", "-g", str(binary)])["stdout"].splitlines()
    label = f"{toolchain}_{count}"
    for phase, result in (
        ("clean", clean),
        ("warm", warm),
        ("incremental", incremental),
        ("release", release),
    ):
        (RAW / f"{label}_{phase}.stdout").write_text(str(result.pop("stdout")))
        (RAW / f"{label}_{phase}.stderr").write_text(str(result.pop("stderr")))
    return {
        "toolchain": toolchain,
        "shape_count": count,
        "clean_check": clean,
        "warm_check": warm,
        "incremental_check": incremental,
        "release_build": release,
        "release_binary_bytes": binary.stat().st_size,
        "global_symbol_count": sum(bool(line.strip()) for line in symbols),
    }


def main() -> int:
    """Run the bounded matrix and write its compact reproducibility record."""
    RAW.mkdir(parents=True, exist_ok=True)
    records = [measure(toolchain, count) for toolchain in TOOLCHAINS for count in COUNTS]
    summary = {
        "schema_version": 1,
        "measurement_date": "2026-07-20",
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        },
        "timeout_seconds_per_command": TIMEOUT_SECONDS,
        "toolchains": {toolchain: toolchain_version(toolchain) for toolchain in TOOLCHAINS},
        "measurements": records,
    }
    SUMMARY.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(SUMMARY.relative_to(SPIKE))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
