"""Mutation tests for the structural CI workflow checks.

The twelve cases this file used to carry -- `if: false`, `continue-on-error`,
an unpinned action tag, a swapped runner, a replaced gate command -- tested a
Python copy of the workflow that no longer exists. Each of those edits is
visible in a one-line diff of `rust.yml`, and the copy that caught them had to
be edited in the same change by the same author, so it detected nothing an
author had not already decided to do.

What remains is what a diff does not show: a second workflow file shadowing the
first, a symlink pointing outside version control, and a duplicate YAML key
that silently discards the block above it.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/rust.yml"
SPEC = importlib.util.spec_from_file_location("tiler_ci_gate", ROOT / "scripts/check_ci.py")
assert SPEC is not None and SPEC.loader is not None
ci = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = ci
SPEC.loader.exec_module(ci)


def test_checked_in_workflow_is_structurally_sound() -> None:
    assert ci.validate_repository(ROOT) == []


def test_ci_contract_rejects_duplicate_yaml_keys(tmp_path: Path) -> None:
    workflow_root = tmp_path / ".github/workflows"
    workflow_root.mkdir(parents=True)
    source = WORKFLOW.read_text(encoding="utf-8") + "\nname: duplicate\n"
    (workflow_root / "rust.yml").write_text(source, encoding="utf-8")
    assert any("malformed YAML" in error for error in ci.validate_repository(tmp_path))


def test_ci_contract_rejects_an_extra_workflow(tmp_path: Path) -> None:
    workflow_root = tmp_path / ".github/workflows"
    workflow_root.mkdir(parents=True)
    (workflow_root / "rust.yml").write_text(WORKFLOW.read_text(), encoding="utf-8")
    (workflow_root / "bypass.yml").write_text("jobs: {}\n", encoding="utf-8")
    assert any("workflow set" in error for error in ci.validate_repository(tmp_path))


def test_ci_contract_rejects_a_symlinked_workflow(tmp_path: Path) -> None:
    workflow_root = tmp_path / ".github/workflows"
    workflow_root.mkdir(parents=True)
    source = tmp_path / "outside.yml"
    source.write_text(WORKFLOW.read_text(), encoding="utf-8")
    (workflow_root / "rust.yml").symlink_to(source)
    assert any("non-symlink" in error for error in ci.validate_repository(tmp_path))
