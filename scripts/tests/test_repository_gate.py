"""Mutation tests for the non-Cargo repository validation envelope."""

from __future__ import annotations

import copy
import importlib.util
import subprocess
import sys
import tomllib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "tiler_repository_gate", ROOT / "scripts/check_repository.py"
)
assert SPEC is not None and SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = gate
SPEC.loader.exec_module(gate)


def project() -> dict[str, object]:
    return tomllib.loads((ROOT / "pyproject.toml").read_text())


def test_python_version_authority_is_exact() -> None:
    gate.validate_python_version_authority("3.11\n")
    with pytest.raises(gate.GateFailure, match="exact supported"):
        gate.validate_python_version_authority("3.12\n")


def test_ticketsplease_revision_receipt_is_exact(tmp_path: Path) -> None:
    receipt = tmp_path / "ticketsplease-revision"
    receipt.write_text("0" * 40 + "\n", encoding="utf-8")
    with pytest.raises(gate.GateFailure, match="does not match"):
        gate.validate_ticketsplease_receipt("1" * 40, receipt)
    receipt.write_text("1" * 40 + "\n", encoding="utf-8")
    gate.validate_ticketsplease_receipt("1" * 40, receipt)


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


@pytest.mark.parametrize(
    ("mutate", "message"),
    (
        (
            lambda value: value["tool"]["pytest"]["ini_options"].update(addopts=["--collect-only"]),
            "pytest configuration",
        ),
        (
            lambda value: value["tool"]["pytest"]["ini_options"].update(testpaths=["nowhere"]),
            "pytest configuration",
        ),
        (
            lambda value: value["tool"]["pytest"]["ini_options"].update(
                python_files=["test_docs.py"]
            ),
            "pytest configuration",
        ),
        (
            lambda value: value["tool"]["ruff"].update(include=["scripts/**/*.py"]),
            "Ruff configuration",
        ),
        (lambda value: value["tool"]["ruff"]["lint"].update(select=[]), "Ruff configuration"),
        (lambda value: value["tool"]["ruff"]["lint"].update(ignore=["F"]), "Ruff configuration"),
        (
            lambda value: value["tool"]["uv"].update(required_version=">=0.11.28"),
            "uv project",
        ),
    ),
)
def test_config_mutations_fail_for_their_typed_reason(mutate, message: str) -> None:
    changed = copy.deepcopy(project())
    mutate(changed)
    assert any(message in error for error in gate.validate_python_config(changed))


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


def test_ambient_home_cannot_redirect_governed_tools(tmp_path: Path) -> None:
    with pytest.raises(gate.GateFailure, match="HOME must identify"):
        gate.sanitized_environment({"HOME": str(tmp_path)})


def test_ruff_and_shell_discovery_cover_all_tracked_sources() -> None:
    python = gate.source_files(".py")
    gate.validate_ruff_discovery("\n".join(str(ROOT / path) for path in python))
    posix, zsh = gate.validate_shell_discovery()
    assert sorted(posix + zsh) == gate.source_files(".sh")
    assert zsh == ["spikes/apple-targets/compatibility_probe.sh"]


def test_shellcheck_policy_is_exact(monkeypatch: pytest.MonkeyPatch) -> None:
    original = gate.Path.read_text

    def changed(path: Path, *args, **kwargs) -> str:
        if path.name == ".shellcheckrc":
            return "severity=warning\n"
        return original(path, *args, **kwargs)

    monkeypatch.setattr(gate.Path, "read_text", changed)
    with pytest.raises(gate.GateFailure, match="ShellCheck severity"):
        gate.validate_shell_discovery()


def test_complete_orchestrator_retains_every_governed_phase(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    commands: list[list[str]] = []
    child_environment = {"GATE_ENVIRONMENT": "sanitized"}
    sanitizations: list[bool] = []
    resolutions: list[str] = []
    monkeypatch.setattr(gate, "validate_execution_identity", lambda _packages: None)
    monkeypatch.setattr(gate, "ticketsplease_policy", lambda: ("0.11.0", "1" * 40))
    monkeypatch.setattr(gate, "validate_ticketsplease_receipt", lambda _revision: None)
    monkeypatch.setattr(gate, "digest", lambda _path: "unchanged")
    monkeypatch.setattr(gate, "validate_ruff_discovery", lambda _output: None)
    monkeypatch.setattr(
        gate,
        "sanitized_environment",
        lambda: sanitizations.append(True) or child_environment,
    )

    def resolve(name: str, _candidates, environment: dict[str, str]) -> str:
        assert environment is child_environment
        resolutions.append(name)
        return f"/governed/{name}"

    monkeypatch.setattr(gate, "resolve_tool", resolve)

    def record(command: list[str], *, environment: dict[str, str], capture: bool = False) -> str:
        assert environment is child_environment
        commands.append(command)
        if command[:2] == ["/governed/ticketsplease", "--version"]:
            return "ticketsplease 0.11.0\n"
        if command[:2] == ["/governed/uv", "--version"]:
            return "uv 0.11.28\n"
        del capture
        return ""

    monkeypatch.setattr(gate, "run", record)
    assert gate.main([]) == 0

    python = str(gate.ROOT / ".venv/bin/python")
    ruff = str(gate.ROOT / ".venv/bin/ruff")
    required = (
        ["/governed/uv", "--project", str(gate.ROOT), "--no-config", "lock", "--check"],
        [
            "/governed/uv",
            "--project",
            str(gate.ROOT),
            "--no-config",
            "sync",
            "--locked",
            "--check",
        ],
        [ruff, "format", "--check"],
        [ruff, "check", "--show-files"],
        [ruff, "check"],
        [python, "-m", "pytest", "-c", str(gate.PYPROJECT), *gate.EXPECTED_PYTEST_PATHS],
        [python, "scripts/docs.py", "validate"],
        [python, "scripts/check_ci.py"],
        ["/governed/ticketsplease", "lint"],
        [python, "scripts/check_rust.py"],
    )
    for command in required:
        assert commands.count(command) == 1
    assert sanitizations == [True]
    assert sorted(resolutions) == ["shellcheck", "ticketsplease", "uv", "zsh"]
    posix, zsh = gate.validate_shell_discovery()
    for script in posix:
        shebang = (gate.ROOT / script).read_text().splitlines()[0]
        shell = "bash" if shebang.endswith("bash") else "sh"
        assert (
            commands.count(
                [
                    "/governed/shellcheck",
                    "--severity",
                    "style",
                    "--shell",
                    shell,
                    script,
                ]
            )
            == 1
        )
        shell_path = "/bin/bash" if shell == "bash" else "/bin/sh"
        assert commands.count([shell_path, "-n", script]) == 1
    for script in zsh:
        assert commands.count(["/governed/zsh", "-n", script]) == 1
