#!/usr/bin/env python3
"""Validate the exact supported GitHub Actions execution contract."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from ruamel.yaml import YAML
from ruamel.yaml.error import YAMLError

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/rust.yml"

READ_VERSIONS = """uv run --locked python - <<'PY' >> "$GITHUB_OUTPUT"
import tomllib
from pathlib import Path

rust = tomllib.loads(Path("rust-toolchain.toml").read_text())["toolchain"]
tools = tomllib.loads(Path("tool-versions.toml").read_text())
print(f"rust_channel={rust['channel']}")
print(f"rust_profile={rust['profile']}")
print(f"rust_components={','.join(rust['components'])}")
print(f"ticketsplease={tools['ticketsplease']}")
print(f"ticketsplease_rev={tools['ticketsplease_rev']}")
PY
"""
INSTALL_RUST = (
    "rustup toolchain install '${{ steps.tool-versions.outputs.rust_channel }}' "
    "--profile '${{ steps.tool-versions.outputs.rust_profile }}' --component "
    "'${{ steps.tool-versions.outputs.rust_components }}'"
)
INSTALL_TICKETSPLEASE = (
    "rustup run '${{ steps.tool-versions.outputs.rust_channel }}' cargo install "
    "--git https://github.com/moderately-ai/ticketsplease --rev "
    "'${{ steps.tool-versions.outputs.ticketsplease_rev }}' --locked --root \"$HOME/.local\" "
    "ticketsplease-cli"
)

EXPECTED: dict[str, Any] = {
    "name": "Repository validation",
    "on": {"pull_request": None, "push": {"branches": ["main"]}},
    "permissions": {"contents": "read"},
    "jobs": {
        "check": {
            "name": "Complete gate (${{ matrix.profile }})",
            "timeout-minutes": 60,
            "strategy": {
                "fail-fast": False,
                "matrix": {
                    "include": [
                        {"profile": "macOS arm64", "runner": "macos-15"},
                        {"profile": "Ubuntu x64", "runner": "ubuntu-24.04"},
                    ]
                },
            },
            "runs-on": "${{ matrix.runner }}",
            "steps": [
                {"uses": "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"},
                {
                    "uses": "astral-sh/setup-uv@08807647e7069bb48b6ef5acd8ec9567f424441b",
                    "with": {
                        "version-file": "pyproject.toml",
                        "enable-cache": True,
                    },
                },
                {
                    "name": "Normalize the governed uv location",
                    "run": 'mkdir -p "$HOME/.local/bin"\n'
                    'install -m 0755 "$(command -v uv)" "$HOME/.local/bin/uv"\n',
                },
                {
                    "name": "Read governed tool versions",
                    "id": "tool-versions",
                    "shell": "bash",
                    "run": READ_VERSIONS,
                },
                {
                    "name": "Install the governed Rust toolchain",
                    "run": INSTALL_RUST,
                },
                {
                    "name": "Install ticketsplease",
                    "run": INSTALL_TICKETSPLEASE,
                },
                {
                    "name": "Record the governed ticketsplease revision",
                    "run": 'mkdir -p "$HOME/.local/share/tiler"\n'
                    "printf '%s\\n' '${{ steps.tool-versions.outputs.ticketsplease_rev }}' \\\n"
                    '  > "$HOME/.local/share/tiler/ticketsplease-revision"\n',
                },
                {
                    "name": "Install Ubuntu system dependencies",
                    "if": "runner.os == 'Linux'",
                    "run": "sudo apt-get update\n"
                    "sudo apt-get install --no-install-recommends -y shellcheck time zsh\n",
                },
                {
                    "name": "Install macOS system dependencies",
                    "if": "runner.os == 'macOS'",
                    "run": "brew install shellcheck",
                },
                {
                    "name": "Run the complete repository gate",
                    "run": "uv run --locked python scripts/check_repository.py",
                },
            ],
        }
    },
}


class CiFailure(RuntimeError):
    """The checked-in CI contract is malformed or weaker than expected."""


def compare(actual: Any, expected: Any, path: str = "workflow") -> list[str]:
    """Recursively compare YAML values with path-specific failures."""
    if type(actual) is not type(expected):
        return [f"{path}: expected {type(expected).__name__}, got {type(actual).__name__}"]
    if isinstance(expected, dict):
        errors = []
        actual_keys, expected_keys = set(actual), set(expected)
        if actual_keys != expected_keys:
            errors.append(
                f"{path}: expected keys {sorted(expected_keys)}, got {sorted(actual_keys)}"
            )
        for key in sorted(actual_keys & expected_keys):
            errors.extend(compare(actual[key], expected[key], f"{path}.{key}"))
        return errors
    if isinstance(expected, list):
        if len(actual) != len(expected):
            return [f"{path}: expected {len(expected)} items, got {len(actual)}"]
        errors = []
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected, strict=True)):
            errors.extend(compare(actual_item, expected_item, f"{path}[{index}]"))
        return errors
    return [] if actual == expected else [f"{path}: expected {expected!r}, got {actual!r}"]


def validate_source(source: str) -> list[str]:
    """Parse and validate one workflow source value."""
    parser = YAML(typ="safe", pure=True)
    parser.version = (1, 2)
    parser.allow_duplicate_keys = False
    try:
        actual = parser.load(source)
    except YAMLError as error:
        return [f"workflow: malformed YAML: {error}"]
    return compare(actual, EXPECTED)


def validate_repository(root: Path = ROOT) -> list[str]:
    """Require exactly the one governed workflow and validate it."""
    workflow_root = root / ".github/workflows"
    observed = sorted(path.name for path in workflow_root.glob("*.yml"))
    observed += sorted(path.name for path in workflow_root.glob("*.yaml"))
    if observed != ["rust.yml"]:
        return [f"workflow set: expected ['rust.yml'], got {observed}"]
    workflow = workflow_root / "rust.yml"
    if workflow.is_symlink() or not workflow.is_file():
        return ["workflow set: rust.yml must be a regular non-symlink file"]
    return validate_source(workflow.read_text(encoding="utf-8"))


def main() -> int:
    """Validate CI and return a process status."""
    errors = validate_repository()
    if errors:
        print("CI validation failed:\n" + "\n".join(errors), file=sys.stderr)
        return 1
    print("CI validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
