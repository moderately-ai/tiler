"""Repository-gate coverage for optimization-safe numerical witnesses."""

from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CHECKER_PATH = REPOSITORY_ROOT / "spikes/numerics/check_witnesses.py"
SPEC = importlib.util.spec_from_file_location("numerical_witness_checker", CHECKER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load numerical witness checker")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def test_numerical_witnesses_match_optimized_mode_and_retained_results() -> None:
    """Run the governed witness aggregate as part of ordinary pytest."""
    subprocess.run(
        [sys.executable, "spikes/numerics/check_witnesses.py"],
        cwd=REPOSITORY_ROOT,
        check=True,
        timeout=120,
    )


def test_region_accuracy_retained_result_is_schema_checked_and_mutation_safe() -> None:
    """Reject schema, provenance, and numerical mutations of the retained result."""
    retained_path = Path("spikes/numerics/region_accuracy/results.json")
    retained = json.loads((REPOSITORY_ROOT / retained_path).read_text(encoding="utf-8"))
    witness_path = Path("spikes/numerics/region_accuracy_probe.py")
    CHECKER.compare_region_accuracy(witness_path, retained_path, retained, retained)

    mutations = []
    wrong_schema = copy.deepcopy(retained)
    wrong_schema["schema"] = "tiler-region-accuracy-observation/v2"
    mutations.append(wrong_schema)
    missing_provenance = copy.deepcopy(retained)
    del missing_provenance["provenance"]["source_sha256"]
    mutations.append(missing_provenance)
    changed_result = copy.deepcopy(retained)
    changed_result["materialization"]["boundary_elided"] = 0.0
    mutations.append(changed_result)

    for mutation in mutations:
        with pytest.raises(CHECKER.WitnessCheckFailure):
            CHECKER.compare_region_accuracy(witness_path, retained_path, mutation, retained)
