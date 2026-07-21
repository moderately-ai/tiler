"""Repository-gate coverage for optimization-safe numerical witnesses."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def test_numerical_witnesses_match_optimized_mode_and_retained_results() -> None:
    """Run the governed witness aggregate as part of ordinary pytest."""
    subprocess.run(
        [sys.executable, "spikes/numerics/check_witnesses.py"],
        cwd=REPOSITORY_ROOT,
        check=True,
        timeout=120,
    )
