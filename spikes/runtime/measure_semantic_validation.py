#!/usr/bin/env python3
"""Measure the bounded CPU semantic-validation model with full provenance."""

from __future__ import annotations

import contextlib
import csv
import datetime as dt
import hashlib
import io
import json
import os
import platform
import signal
import statistics
import subprocess
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "spikes" / "runtime" / "semantic_validation_enforcement.rs"
SUMMARY = ROOT / "spikes" / "runtime" / "measurements" / "semantic-validation.json"
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


def run(command: list[str]) -> str:
    """Run a subprocess under an overall process-group deadline."""
    process = subprocess.Popen(
        command,
        cwd=ROOT,
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
    if process.returncode != 0:
        raise RuntimeError(f"command failed ({process.returncode}): {' '.join(command)}\n{stderr}")
    return stdout


def toolchain() -> str:
    """Read the repository-owned exact compiler selection."""
    with (ROOT / "rust-toolchain.toml").open("rb") as file:
        return str(tomllib.load(file)["toolchain"]["channel"])


def git(*arguments: str) -> str:
    return run(["git", *arguments]).strip()


def main() -> int:
    selected_toolchain = toolchain()
    source_bytes = SOURCE.read_bytes()
    with tempfile.TemporaryDirectory(prefix="tiler-semantic-validation-") as directory:
        binary = Path(directory) / "semantic-validation"
        compile_command = [
            "rustc",
            f"+{selected_toolchain}",
            "-O",
            "--edition",
            "2021",
            str(SOURCE),
            "-o",
            str(binary),
        ]
        run(compile_command)
        output = run([str(binary)])

    samples = []
    for row in csv.DictReader(io.StringIO(output)):
        samples.append(
            {
                "elements": int(row["elements"]),
                "strategy": row["strategy"],
                "sample_index": int(row["sample_index"]),
                "elapsed_ns": int(row["elapsed_ns"]),
                "validation_elements": int(row["validation_elements"]),
                "input_bytes": int(row["input_bytes"]),
                "private_bytes": int(row["private_bytes"]),
                "dispatches": int(row["dispatches"]),
                "observations": int(row["observations"]),
            }
        )
    expected_samples = (
        sum(9 if elements < 1_000_000 else 5 for elements in (65_536, 1_048_576, 8_388_608)) * 4
    )
    if len(samples) != expected_samples:
        raise RuntimeError(f"expected {expected_samples} samples, found {len(samples)}")

    grouped: dict[tuple[int, str], list[int]] = {}
    for sample in samples:
        key = (int(sample["elements"]), str(sample["strategy"]))
        grouped.setdefault(key, []).append(int(sample["elapsed_ns"]))
    medians = [
        {
            "elements": elements,
            "strategy": strategy,
            "median_elapsed_ns": int(statistics.median(values)),
        }
        for (elements, strategy), values in sorted(grouped.items())
    ]
    summary = {
        "schema": "tiler.runtime-semantic-validation-measurement/v1",
        "measured_at_utc": dt.datetime.now(dt.UTC).isoformat(timespec="seconds"),
        "measurement_boundary": (
            "optimized dependency-free CPU control/accounting model; not GPU performance"
        ),
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "python": platform.python_version(),
        },
        "repository": {
            "head": git("rev-parse", "HEAD"),
            "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
            "source_dirty": bool(git("status", "--short", "--", str(SOURCE.relative_to(ROOT)))),
        },
        "toolchain": {
            "selector": selected_toolchain,
            "rustc_verbose": run(
                ["rustc", f"+{selected_toolchain}", "--version", "--verbose"]
            ).strip(),
        },
        "timeout_seconds_per_subprocess_group": TIMEOUT_SECONDS,
        "compile_command": [
            "rustc",
            f"+{selected_toolchain}",
            "-O",
            "--edition",
            "2021",
            "spikes/runtime/semantic_validation_enforcement.rs",
            "-o",
            "<temporary-binary>",
        ],
        "samples": samples,
        "derived_medians": medians,
    }
    SUMMARY.parent.mkdir(parents=True, exist_ok=True)
    SUMMARY.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(SUMMARY.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
