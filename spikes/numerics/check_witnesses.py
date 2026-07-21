#!/usr/bin/env python3
"""Run every Python numerical witness with and without optimization.

Python's ``-O`` mode removes ``assert`` statements. Byte-identical output from
both modes is therefore a repository-level acceptance check that published
verdicts do not depend on removable assertions.
"""

from __future__ import annotations

import ast
import copy
import json
import subprocess
import sys
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WITNESSES = (
    Path("spikes/cost-model/bootstrap_model.py"),
    Path("spikes/reference/reference_evaluator.py"),
    Path("spikes/region-search/exhaustive_oracle.py"),
    Path("spikes/numerics/reduction_contract_probe.py"),
    Path("spikes/numerics/region_accuracy_probe.py"),
    Path("spikes/numerics/sound_accuracy/observe.py"),
)
TIMEOUT_SECONDS = 60
RETAINED_OUTPUTS = {
    Path("spikes/numerics/sound_accuracy/observe.py"): Path(
        "spikes/numerics/sound_accuracy/observations.json"
    ),
}


class WitnessCheckFailure(RuntimeError):
    """A witness failed, timed out, or changed under optimized Python."""


def reject_removable_asserts(path: Path) -> None:
    """Reject executable ``assert`` statements in governed witness programs."""
    source_path = REPOSITORY_ROOT / path
    tree = ast.parse(source_path.read_text(encoding="utf-8"), filename=str(path))
    assertions = [node.lineno for node in ast.walk(tree) if isinstance(node, ast.Assert)]
    if assertions:
        lines = ", ".join(str(line) for line in assertions)
        raise WitnessCheckFailure(f"{path} contains removable assert statements at lines {lines}")


def run_witness(path: Path, optimized: bool) -> subprocess.CompletedProcess[bytes]:
    command = [sys.executable]
    if optimized:
        command.append("-O")
    command.append(str(path))
    try:
        return subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            check=False,
            capture_output=True,
            timeout=TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        mode = "optimized" if optimized else "ordinary"
        raise WitnessCheckFailure(f"{path} exceeded {TIMEOUT_SECONDS}s in {mode} Python") from error


def require_success(path: Path, result: subprocess.CompletedProcess[bytes], mode: str) -> None:
    if result.returncode != 0:
        stderr = result.stderr.decode("utf-8", errors="replace").strip()
        raise WitnessCheckFailure(
            f"{path} failed in {mode} Python with status {result.returncode}: {stderr}"
        )


def require_retained_observation(
    witness_path: Path, retained_path: Path, current_bytes: bytes
) -> None:
    """Validate portable corpus identity and exact same-environment replay."""
    try:
        current = json.loads(current_bytes)
        retained = json.loads((REPOSITORY_ROOT / retained_path).read_bytes())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise WitnessCheckFailure(
            f"cannot compare retained output {retained_path}: {error}"
        ) from error

    current_environment = current["provenance"]["interpreter"], current["provenance"]["host"]
    retained_environment = retained["provenance"]["interpreter"], retained["provenance"]["host"]
    if current_environment == retained_environment:
        if current != retained:
            raise WitnessCheckFailure(
                f"{witness_path} does not exactly reproduce retained output {retained_path}"
            )
        return

    current_portable = copy.deepcopy(current)
    retained_portable = copy.deepcopy(retained)
    for output in (current_portable, retained_portable):
        del output["provenance"]["interpreter"]
        del output["provenance"]["host"]
        del output["observed_max_absolute_error"]
        del output["max_witnesses"]
    if current_portable != retained_portable:
        raise WitnessCheckFailure(
            f"{witness_path} corpus identity differs from retained output {retained_path}"
        )
    print(f"retained result not replayed on this interpreter/host: {retained_path}")


def main() -> None:
    for relative_path in WITNESSES:
        reject_removable_asserts(relative_path)
        ordinary = run_witness(relative_path, optimized=False)
        optimized = run_witness(relative_path, optimized=True)
        require_success(relative_path, ordinary, "ordinary")
        require_success(relative_path, optimized, "optimized")
        if ordinary.stdout != optimized.stdout or ordinary.stderr != optimized.stderr:
            raise WitnessCheckFailure(
                f"{relative_path} produced different output under optimized Python"
            )
        retained_path = RETAINED_OUTPUTS.get(relative_path)
        if retained_path is not None:
            require_retained_observation(relative_path, retained_path, ordinary.stdout)
        print(f"checked {relative_path}")
    print("numerical witness optimization checks passed")


if __name__ == "__main__":
    main()
