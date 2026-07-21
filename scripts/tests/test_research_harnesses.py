"""Fast canonical checks for retained research-harness evidence."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LEGACY_EMBEDDING = Path("docs/research/embedding/measurements/2026-07-20-macos-arm64")
MACRO_RESULTS = Path("spikes/macro-environment/results")


@pytest.mark.parametrize(
    "command",
    (
        (
            "spikes/embedding/measure.py",
            "--verify-retained",
            str(LEGACY_EMBEDDING),
        ),
        (
            "spikes/macro-environment/probe.py",
            "verify",
            str(MACRO_RESULTS / "native-2026-07-21.json"),
        ),
        (
            "spikes/macro-environment/probe.py",
            "verify",
            str(MACRO_RESULTS / "family-cfg-2026-07-21.json"),
        ),
        ("spikes/extensions/run.py", "--self-test"),
    ),
)
def test_retained_research_harness_contracts(command: tuple[str, ...]) -> None:
    """Keep fast semantic verifiers in the ordinary repository pytest gate."""
    subprocess.run(
        [sys.executable, *command],
        cwd=REPOSITORY_ROOT,
        check=True,
        timeout=120,
    )
