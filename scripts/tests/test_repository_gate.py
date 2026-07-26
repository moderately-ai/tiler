"""Mutation tests for the non-Cargo repository validation envelope."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "tiler_repository_gate", ROOT / "scripts/check_repository.py"
)
assert SPEC is not None and SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = gate
SPEC.loader.exec_module(gate)


def test_pytest_conftest_cannot_skip_governed_failures(tmp_path: Path) -> None:
    (tmp_path / "conftest.py").write_text(
        "def pytest_collection_modifyitems(items):\n"
        "    for item in items:\n"
        "        item.add_marker('skip')\n",
        encoding="utf-8",
    )
    test = tmp_path / "test_failure.py"
    test.write_text("def test_failure():\n    assert False\n", encoding="utf-8")

    result = subprocess.run(
        [sys.executable, "-m", "pytest", "-q", "-c", str(gate.PYPROJECT), str(test)],
        cwd=gate.ROOT,
        env=gate.sanitized_environment(),
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 1
    assert "failed" in result.stdout


def test_hostile_environment_is_removed_without_losing_ordinary_values() -> None:
    source = {
        "PATH": "/bin",
        "UV_NO_PROJECT": "1",
        "UV_PROJECT": "/wrong",
        "PYTEST_ADDOPTS": "--collect-only",
        "PYTHONPATH": "/wrong",
        "RUSTFLAGS": "--cap-lints allow",
        "CARGO_TARGET_X_RUNNER": "true",
        "SHELLCHECK_OPTS": "--severity=error",
    }
    result = gate.sanitized_environment(source)
    assert result["PATH"] != "/bin"
    assert result["PATH"].endswith("/usr/bin:/bin")
    assert result["PYTEST_DISABLE_PLUGIN_AUTOLOAD"] == "1"
    assert not (set(source) - {"PATH"}) & set(result)
