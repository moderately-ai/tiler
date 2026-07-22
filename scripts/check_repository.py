#!/usr/bin/env python3
"""Run the complete repository-owned contributor and CI validation gate."""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import os
import pwd
import re
import shlex
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYPROJECT = ROOT / "pyproject.toml"
PYTHON_VERSION = ROOT / ".python-version"
TOOL_VERSIONS = ROOT / "tool-versions.toml"
LOCKS = (ROOT / "Cargo.lock", ROOT / "uv.lock")
EXPECTED_PYTEST_ADDOPTS = ["--noconftest", "--strict-config", "--strict-markers", "-ra"]
EXPECTED_PYTEST_PATHS = [
    "scripts/tests",
    "spikes/embedding",
    "spikes/macro-environment",
    "spikes/numerics/sound_accuracy",
]
EXPECTED_RUFF_INCLUDE = ["scripts/**/*.py", "spikes/**/*.py"]
EXPECTED_PYTHON_PACKAGES = {"markdown-it-py", "mpmath", "pytest", "ruamel-yaml", "ruff"}
ZSH_ENTRYPOINTS = {"spikes/apple-targets/compatibility_probe.sh"}
HOSTILE_PREFIXES = ("UV_", "PYTEST_", "CARGO_TARGET_", "CLIPPY_")
HOSTILE_EXACT = {
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "PYTHONHOME",
    "PYTHONPATH",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "RUSTFLAGS",
    "RUSTFMT",
    "RUSTUP_TOOLCHAIN",
    "SHELLCHECK_OPTS",
    "VIRTUAL_ENV",
}


class GateFailure(RuntimeError):
    """A repository validation invariant was violated."""


def account_home() -> Path:
    """Return the supported Unix account home independently of ambient HOME."""
    return Path(pwd.getpwuid(os.getuid()).pw_dir).resolve()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise GateFailure(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sanitized_environment(source: dict[str, str] | None = None) -> dict[str, str]:
    """Remove ambient controls that can redirect or disable governed tools."""
    environment = dict(os.environ if source is None else source)
    for name in list(environment):
        if name in HOSTILE_EXACT or name.startswith(HOSTILE_PREFIXES):
            del environment[name]
    home = account_home()
    if "HOME" in environment and Path(environment["HOME"]).resolve() != home:
        raise GateFailure(f"HOME must identify the account home {home}")
    environment.update(
        {
            "HOME": str(home),
            "PATH": os.pathsep.join(
                str(path)
                for path in (
                    ROOT / ".venv/bin",
                    home / ".cargo/bin",
                    home / ".local/bin",
                    Path("/opt/homebrew/bin"),
                    Path("/usr/local/bin"),
                    Path("/usr/bin"),
                    Path("/bin"),
                )
                if path.is_dir()
            ),
            "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
            "PYTHONHASHSEED": "0",
        }
    )
    return environment


def resolve_tool(name: str, candidates: tuple[Path, ...], environment: dict[str, str]) -> str:
    """Resolve a governed executable without trusting the inherited PATH order."""
    del environment
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate.resolve())
    raise GateFailure(f"required governed tool is missing from supported locations: {name}")


def run(command: list[str], *, environment: dict[str, str], capture: bool = False) -> str:
    print(f"+ {shlex.join(command)}", flush=True)
    result = subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else ""


def pinned_python_packages(project: dict[str, object]) -> dict[str, str]:
    """Read exact direct development dependency pins from their sole authority."""
    groups = project.get("dependency-groups", {})
    dependencies = groups.get("dev", []) if isinstance(groups, dict) else []
    if not isinstance(dependencies, list):
        raise GateFailure("Python development dependencies must be a list")
    pins: dict[str, str] = {}
    for dependency in dependencies:
        if not isinstance(dependency, str):
            raise GateFailure("Python development dependency pins must be strings")
        match = re.fullmatch(r"([a-z0-9-]+)==([0-9]+\.[0-9]+\.[0-9]+)", dependency)
        if match is None or match.group(1) in pins:
            raise GateFailure(f"invalid exact Python development dependency: {dependency!r}")
        pins[match.group(1)] = match.group(2)
    if set(pins) != EXPECTED_PYTHON_PACKAGES:
        raise GateFailure(
            f"Python development package set changed: expected {sorted(EXPECTED_PYTHON_PACKAGES)}"
        )
    return pins


def validate_python_config(project: dict[str, object]) -> list[str]:
    errors: list[str] = []
    package = project.get("project", {})
    if not isinstance(package, dict) or package.get("requires-python") != ">=3.11,<3.12":
        errors.append("Python runtime policy must remain exactly 3.11")
    try:
        pinned_python_packages(project)
    except GateFailure as error:
        errors.append(str(error))
    tool = project.get("tool", {})
    if not isinstance(tool, dict):
        return ["pyproject [tool] must be a table"]
    uv = tool.get("uv", {})
    if (
        not isinstance(uv, dict)
        or set(uv) != {"package", "required-version"}
        or uv.get("package") is not False
        or not isinstance(uv.get("required-version"), str)
        or re.fullmatch(r"==\d+\.\d+\.\d+", uv["required-version"]) is None
    ):
        errors.append("uv project and exact tool-version authority changed")
    pytest = tool.get("pytest", {})
    pytest_options = pytest.get("ini_options", {}) if isinstance(pytest, dict) else {}
    expected_pytest = {"addopts": EXPECTED_PYTEST_ADDOPTS, "testpaths": EXPECTED_PYTEST_PATHS}
    if pytest_options != expected_pytest or set(pytest) != {"ini_options"}:
        errors.append("pytest configuration must equal the canonical collection contract")
    ruff = tool.get("ruff", {})
    if not isinstance(ruff, dict):
        errors.append("Ruff configuration must be a table")
    else:
        expected_ruff = {
            "target-version": "py311",
            "line-length": 100,
            "include": EXPECTED_RUFF_INCLUDE,
            "lint": {"select": ["B", "E", "F", "I", "SIM", "UP"]},
        }
        if ruff != expected_ruff:
            errors.append("Ruff configuration must equal the canonical lint contract")
    return errors


def validate_python_version_authority(source: str) -> None:
    """Keep uv's interpreter selector aligned with the exact supported minor."""
    require(source == "3.11\n", ".python-version must remain the exact supported 3.11 selector")


def validate_execution_identity(packages: dict[str, str]) -> None:
    expected_prefix = (ROOT / ".venv").resolve()
    require(Path(sys.prefix).resolve() == expected_prefix, "gate must run in this checkout's .venv")
    require(sys.version_info[:2] == (3, 11), "gate requires the repository's Python 3.11")
    for package, expected in packages.items():
        actual = importlib.metadata.version(package)
        require(actual == expected, f"{package} must be {expected}, got {actual}")


def ticketsplease_policy() -> tuple[str, str]:
    policy = tomllib.loads(TOOL_VERSIONS.read_text(encoding="utf-8"))
    require(
        policy.keys() == {"schema_version", "ticketsplease", "ticketsplease_rev"}
        and policy["schema_version"] == 1,
        "tool-versions.toml has an unsupported schema",
    )
    expected = policy["ticketsplease"]
    require(
        isinstance(expected, str) and re.fullmatch(r"\d+\.\d+\.\d+", expected) is not None,
        "ticketsplease version authority is malformed",
    )
    require(
        isinstance(policy["ticketsplease_rev"], str)
        and re.fullmatch(r"[0-9a-f]{40}", policy["ticketsplease_rev"]) is not None,
        "ticketsplease revision authority is malformed",
    )
    return expected, policy["ticketsplease_rev"]


def validate_ticketsplease_receipt(revision: str, path: Path | None = None) -> None:
    """Require bootstrap provenance for the installed contributor binary."""
    receipt = path or account_home() / ".local/share/tiler/ticketsplease-revision"
    try:
        observed = receipt.read_text(encoding="utf-8")
    except OSError as error:
        raise GateFailure(f"ticketsplease revision receipt is missing: {receipt}") from error
    require(observed == f"{revision}\n", "ticketsplease revision receipt does not match policy")


def source_files(suffix: str) -> list[str]:
    """Discover checked-in and pending repository sources outside generated trees."""
    found = []
    for base in (ROOT / "scripts", ROOT / "spikes"):
        for path in base.rglob(f"*{suffix}"):
            relative = path.relative_to(ROOT)
            if path.is_file() and not {"target", "__pycache__"} & set(relative.parts):
                found.append(relative.as_posix())
    if suffix == ".sh" and (ROOT / "deps.sh").is_file():
        found.append("deps.sh")
    return sorted(found)


def validate_ruff_discovery(output: str) -> None:
    expected = source_files(".py")
    observed = sorted(
        Path(line).resolve().relative_to(ROOT).as_posix() for line in output.splitlines() if line
    )
    require(observed == expected, f"Ruff discovery mismatch: expected {expected}, got {observed}")


def validate_shell_discovery() -> tuple[list[str], list[str]]:
    require(
        (ROOT / ".shellcheckrc").read_text(encoding="utf-8") == "severity=style\n",
        "ShellCheck severity policy changed",
    )
    scripts = source_files(".sh")
    zsh = sorted(set(scripts) & ZSH_ENTRYPOINTS)
    require(set(zsh) == ZSH_ENTRYPOINTS, "the exact governed zsh entrypoint is missing")
    posix = sorted(set(scripts) - ZSH_ENTRYPOINTS)
    for script in posix:
        shebang = (ROOT / script).read_text(encoding="utf-8").splitlines()[0]
        require(
            shebang in {"#!/bin/sh", "#!/usr/bin/env bash"},
            f"unsupported shell entrypoint dialect: {script}: {shebang}",
        )
    return posix, zsh


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.parse_args(arguments)
    project = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    validate_python_version_authority(PYTHON_VERSION.read_text(encoding="utf-8"))
    errors = validate_python_config(project)
    if errors:
        raise GateFailure("\n".join(errors))
    packages = pinned_python_packages(project)
    validate_execution_identity(packages)
    environment = sanitized_environment()
    uv = resolve_tool(
        "uv",
        (
            account_home() / ".local/bin/uv",
            Path("/opt/homebrew/bin/uv"),
            Path("/usr/local/bin/uv"),
        ),
        environment,
    )
    ticketsplease = resolve_tool(
        "ticketsplease", (account_home() / ".local/bin/ticketsplease",), environment
    )
    shellcheck = resolve_tool(
        "shellcheck",
        (
            Path("/opt/homebrew/bin/shellcheck"),
            Path("/usr/local/bin/shellcheck"),
            Path("/usr/bin/shellcheck"),
        ),
        environment,
    )
    installed_ticketsplease = run(
        [ticketsplease, "--version"], environment=environment, capture=True
    ).split()
    expected_ticketsplease, expected_ticketsplease_revision = ticketsplease_policy()
    require(
        len(installed_ticketsplease) == 2 and installed_ticketsplease[1] == expected_ticketsplease,
        f"ticketsplease must be {expected_ticketsplease}",
    )
    validate_ticketsplease_receipt(expected_ticketsplease_revision)
    before = {path: digest(path) for path in LOCKS}

    uv_requirement = project["tool"]["uv"]["required-version"]
    installed_uv = run([uv, "--version"], environment=environment, capture=True).split()
    require(
        len(installed_uv) >= 2
        and installed_uv[0] == "uv"
        and f"=={installed_uv[1]}" == uv_requirement,
        f"uv must satisfy {uv_requirement}",
    )

    run(
        [uv, "--project", str(ROOT), "--no-config", "lock", "--check"],
        environment=environment,
    )
    run(
        [uv, "--project", str(ROOT), "--no-config", "sync", "--locked", "--check"],
        environment=environment,
    )
    ruff = str(ROOT / ".venv/bin/ruff")
    python = str(ROOT / ".venv/bin/python")
    run([ruff, "format", "--check"], environment=environment)
    discovered = run([ruff, "check", "--show-files"], environment=environment, capture=True)
    validate_ruff_discovery(discovered)
    run([ruff, "check"], environment=environment)
    run(
        [python, "-m", "pytest", "-c", str(PYPROJECT), *EXPECTED_PYTEST_PATHS],
        environment=environment,
    )
    run([python, "scripts/docs.py", "validate"], environment=environment)
    run([python, "scripts/check_ci.py"], environment=environment)

    posix_shell, zsh = validate_shell_discovery()
    for script in posix_shell:
        shebang = (ROOT / script).read_text(encoding="utf-8").splitlines()[0]
        shell = "bash" if shebang.endswith("bash") else "sh"
        run(
            [
                shellcheck,
                "--severity",
                "style",
                "--shell",
                shell,
                script,
            ],
            environment=environment,
        )
        shell_path = "/bin/bash" if shell == "bash" else "/bin/sh"
        run([shell_path, "-n", script], environment=environment)
    for script in zsh:
        zsh_path = resolve_tool(
            "zsh", (Path("/bin/zsh"), Path("/usr/bin/zsh"), Path("/usr/local/bin/zsh")), environment
        )
        run([zsh_path, "-n", script], environment=environment)
    run([ticketsplease, "lint"], environment=environment)
    run([python, "scripts/check_rust.py"], environment=environment)

    after = {path: digest(path) for path in LOCKS}
    require(before == after, "a repository lockfile changed during validation")
    print("complete repository validation passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (GateFailure, OSError, subprocess.CalledProcessError) as error:
        print(f"repository validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
