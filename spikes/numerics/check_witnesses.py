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
import re
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
    Path("spikes/numerics/region_accuracy_probe.py"): Path(
        "spikes/numerics/region_accuracy/results.json"
    ),
    Path("spikes/numerics/sound_accuracy/observe.py"): Path(
        "spikes/numerics/sound_accuracy/observations.json"
    ),
}
REGION_ACCURACY_SCHEMA = "tiler-region-accuracy-observation/v1"
SOUND_ACCURACY_SCHEMA = "tiler-observation/v1"
SHA256 = re.compile(r"[0-9a-f]{64}")


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


def compare_sound_accuracy(
    witness_path: Path, retained_path: Path, current: dict, retained: dict
) -> None:
    """Compare the governed sound-accuracy observation schema."""
    if current.get("schema") != SOUND_ACCURACY_SCHEMA:
        raise WitnessCheckFailure(f"{witness_path} emitted an unsupported observation schema")
    if retained.get("schema") != SOUND_ACCURACY_SCHEMA:
        raise WitnessCheckFailure(f"{retained_path} has an unsupported observation schema")
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


def validate_region_accuracy(value: dict, label: Path) -> None:
    """Validate the closed producer and environment fields of a region record."""
    if value.get("schema") != REGION_ACCURACY_SCHEMA:
        raise WitnessCheckFailure(f"{label} has an unsupported region-accuracy schema")
    if set(value) != {
        "schema",
        "evidence",
        "materialization",
        "reference_choice",
        "reduction_topology",
        "provenance",
    }:
        raise WitnessCheckFailure(f"{label} has an unexpected region-accuracy record shape")
    provenance = value.get("provenance")
    if not isinstance(provenance, dict) or set(provenance) != {
        "algorithm",
        "host",
        "mpmath",
        "python",
        "source_sha256",
    }:
        raise WitnessCheckFailure(f"{label} has malformed region-accuracy provenance")
    if provenance.get("algorithm") != "tiler-region-accuracy-probe/v1":
        raise WitnessCheckFailure(f"{label} has an unsupported region-accuracy algorithm")
    source_digest = provenance.get("source_sha256")
    if not isinstance(source_digest, str) or SHA256.fullmatch(source_digest) is None:
        raise WitnessCheckFailure(f"{label} has a malformed source digest")
    if not isinstance(provenance.get("host"), dict) or not isinstance(
        provenance.get("python"), dict
    ):
        raise WitnessCheckFailure(f"{label} lacks recorded host or Python fields")
    if provenance.get("mpmath") != {"decimal_precision_digits": 100, "version": "1.3.0"}:
        raise WitnessCheckFailure(f"{label} has an unsupported mpmath oracle profile")
    if value.get("evidence") != {
        "claim": "witnesses only; not a worst-case certificate",
        "class": "empirical-adversarial",
        "oracle": "mpmath-1.3.0-100-decimal-digits",
    }:
        raise WitnessCheckFailure(f"{label} has an unsupported evidence boundary")
    for section in ("materialization", "reference_choice", "reduction_topology"):
        if not isinstance(value.get(section), dict):
            raise WitnessCheckFailure(f"{label} lacks numerical section {section}")


def compare_region_accuracy(
    witness_path: Path, retained_path: Path, current: dict, retained: dict
) -> None:
    """Require exact replay locally and portable equality across environments."""
    validate_region_accuracy(current, witness_path)
    validate_region_accuracy(retained, retained_path)
    current_environment = current["provenance"]["python"], current["provenance"]["host"]
    retained_environment = retained["provenance"]["python"], retained["provenance"]["host"]
    if current_environment == retained_environment:
        if current != retained:
            raise WitnessCheckFailure(
                f"{witness_path} does not exactly reproduce retained output {retained_path}"
            )
        return

    current_portable = copy.deepcopy(current)
    retained_portable = copy.deepcopy(retained)
    for output in (current_portable, retained_portable):
        del output["provenance"]["python"]
        del output["provenance"]["host"]
    if current_portable != retained_portable:
        raise WitnessCheckFailure(
            f"{witness_path} portable result differs from retained output {retained_path}"
        )
    print(f"retained result not replayed on recorded Python/host fields: {retained_path}")


def require_retained_observation(
    witness_path: Path, retained_path: Path, current_bytes: bytes
) -> None:
    """Validate one known schema and exact or portable retained replay."""
    try:
        current = json.loads(current_bytes)
        retained = json.loads((REPOSITORY_ROOT / retained_path).read_bytes())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise WitnessCheckFailure(
            f"cannot compare retained output {retained_path}: {error}"
        ) from error
    if not isinstance(current, dict) or not isinstance(retained, dict):
        raise WitnessCheckFailure(f"retained output {retained_path} is not a JSON object")
    schema = retained.get("schema")
    if schema == REGION_ACCURACY_SCHEMA:
        compare_region_accuracy(witness_path, retained_path, current, retained)
    elif schema == SOUND_ACCURACY_SCHEMA:
        compare_sound_accuracy(witness_path, retained_path, current, retained)
    else:
        raise WitnessCheckFailure(f"{retained_path} has an unsupported retained schema")


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
