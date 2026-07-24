#!/usr/bin/env python3
"""Generate and measure dependent-static-shape compiler workloads."""

from __future__ import annotations

import contextlib
import datetime as dt
import hashlib
import json
import os
import platform
import re
import signal
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
CLEANUP_REAP_SECONDS = 5.0


def kill_process_group(process: subprocess.Popen[str]) -> None:
    """Stop a command tree and reap its leader on a best-effort basis.

    Signalling a group can fail for reasons that are not measurement failures: a
    group whose only member is an exited-but-unreaped child answers `EPERM`, and
    a sandboxed execution context can refuse the syscall outright. A harness must
    not fail the run it is tidying up after, so tolerate both and fall back to the
    child this process directly owns. Bound the reap as well, so an undeliverable
    signal cannot turn a bounded failure into an unbounded wait, and so a caller
    asserting that the deadline was enforced observes the child's own state rather
    than the harness having waited out its natural exit. The reap goes through
    `communicate` because this harness's capture pipes are drained there.
    """
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        with contextlib.suppress(ProcessLookupError, PermissionError):
            process.kill()
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.communicate(timeout=CLEANUP_REAP_SECONDS)


def run(command: list[str], *, timed: bool = False) -> dict[str, object]:
    """Run one command and return normalized timing and output."""
    actual = command
    if timed:
        actual = ["/usr/bin/time", "-lp" if sys.platform == "darwin" else "-v", *command]
    started = time.monotonic()
    process = subprocess.Popen(
        actual,
        cwd=SPIKE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired as error:
        kill_process_group(process)
        raise RuntimeError(
            f"command exceeded {TIMEOUT_SECONDS}s deadline: {' '.join(command)}"
        ) from error
    elapsed = time.monotonic() - started
    if process.returncode != 0:
        raise RuntimeError(f"command failed ({process.returncode}): {' '.join(command)}\n{stderr}")
    rss_bytes = None
    if timed:
        if sys.platform == "darwin":
            match = re.search(r"(\d+)\s+maximum resident set size", stderr)
            rss_bytes = int(match.group(1)) if match else None
        else:
            match = re.search(r"Maximum resident set size \(kbytes\):\s+(\d+)", stderr)
            rss_bytes = int(match.group(1)) * 1024 if match else None
        if rss_bytes is None:
            raise RuntimeError("could not parse peak RSS from /usr/bin/time output")
    return {
        "command": command,
        "elapsed_seconds": round(elapsed, 6),
        "peak_rss_bytes": rss_bytes,
        "stdout": stdout,
        "stderr": stderr,
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
    if sys.platform not in {"darwin", "linux"}:
        raise SystemExit("measurement supports only macOS and Linux")
    if not Path("/usr/bin/time").is_file():
        raise SystemExit(
            "measurement requires /usr/bin/time (BSD time on macOS or GNU time on Linux)"
        )
    RAW.mkdir(parents=True, exist_ok=True)
    records = [measure(toolchain, count) for toolchain in TOOLCHAINS for count in COUNTS]
    measured_sources = [
        SPIKE / "Cargo.lock",
        SPIKE / "Cargo.toml",
        SPIKE / "measure.py",
        *(sorted((SPIKE / "api" / "src").rglob("*.rs"))),
        *(sorted((SPIKE / "workload" / "src").rglob("*.rs"))),
    ]
    digest = hashlib.sha256()
    for source in measured_sources:
        digest.update(str(source.relative_to(SPIKE)).encode())
        digest.update(b"\0")
        digest.update(source.read_bytes())
        digest.update(b"\0")
    summary = {
        "schema_version": 1,
        "measurement_date": dt.datetime.now(dt.UTC).date().isoformat(),
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        },
        "timeout_seconds_per_subprocess_group": TIMEOUT_SECONDS,
        "measured_input_tree_sha256": digest.hexdigest(),
        "repository_base_revision": run(["git", "rev-parse", "HEAD"])["stdout"].strip(),
        "toolchains": {toolchain: toolchain_version(toolchain) for toolchain in TOOLCHAINS},
        "measurements": records,
    }
    SUMMARY.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(SUMMARY.relative_to(SPIKE))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
