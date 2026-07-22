"""Mutation tests for the exact supported GitHub Actions contract."""

from __future__ import annotations

import importlib.util
import sys
from collections.abc import Callable
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/rust.yml"
SPEC = importlib.util.spec_from_file_location("tiler_ci_gate", ROOT / "scripts/check_ci.py")
assert SPEC is not None and SPEC.loader is not None
ci = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = ci
SPEC.loader.exec_module(ci)


def replace_once(old: str, new: str) -> Callable[[str], str]:
    """Construct a mutation that must affect exactly one expected token."""

    def mutate(source: str) -> str:
        assert source.count(old) == 1
        return source.replace(old, new, 1)

    return mutate


def append(text: str) -> Callable[[str], str]:
    """Construct an append-only mutation."""
    return lambda source: source + text


def test_checked_in_workflow_satisfies_supported_profile_contract() -> None:
    assert ci.validate_repository(ROOT) == []


@pytest.mark.parametrize(
    "mutation",
    (
        replace_once("jobs:\n", "jobs\n"),
        replace_once("jobs:\n", "jobs:\n  check:\n    if: false\n"),
        replace_once(
            "      - name: Run the complete repository gate\n",
            "      - name: Run the complete repository gate\n        continue-on-error: true\n",
        ),
        replace_once(
            "      - name: Run the complete repository gate\n",
            "      - name: Run the complete repository gate\n        if: false\n",
        ),
        replace_once("runner: macos-15", "runner: macos-latest"),
        replace_once("runner: ubuntu-24.04", "runner: macos-15"),
        replace_once(
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
            "actions/checkout@v4",
        ),
        replace_once(
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
            "example/unknown@11bd71901bbe5b1630ceea73d27597364c9af683",
        ),
        replace_once(
            "uv run --locked python scripts/check_repository.py",
            "uv run --locked python -c 'pass'",
        ),
        replace_once("permissions:\n  contents: read", "permissions:\n  contents: write"),
        replace_once("pull_request:\n", "pull_request:\n    paths-ignore: ['docs/**']\n"),
        append("\njobs:\n  bypass:\n    runs-on: ubuntu-24.04\n    steps: []\n"),
    ),
)
def test_ci_contract_rejects_disabling_mutations(mutation: Callable[[str], str]) -> None:
    assert ci.validate_source(mutation(WORKFLOW.read_text(encoding="utf-8")))


def test_ci_contract_rejects_duplicate_yaml_keys() -> None:
    source = WORKFLOW.read_text(encoding="utf-8") + "\nname: duplicate\n"
    assert any("malformed YAML" in error for error in ci.validate_source(source))


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
