#!/usr/bin/env python3
"""Check the prototype workspace dependency and compatibility boundary."""

from __future__ import annotations

import json
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQUIRED_RUST_TOOLCHAIN = "nightly-2026-07-19"
EXPECTED = {
    "tiler-ir": set(),
    "tiler-reference": {"tiler-ir"},
    "tiler-artifact": {"tiler-ir"},
    "tiler-compiler": {"tiler-artifact", "tiler-ir"},
    "tiler-metal": {"tiler-artifact", "tiler-ir"},
    "tiler-prototype-compile": {
        "tiler-artifact",
        "tiler-compiler",
        "tiler-ir",
        "tiler-metal",
        "tiler-reference",
    },
    "tiler-prototype-run": {"tiler-artifact"},
}
EXPECTED_DEV = {
    "tiler-compiler": {"tiler-reference"},
}


def main() -> int:
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    python_project = tomllib.loads((ROOT / "pyproject.toml").read_text())
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    packages = {package["name"]: package for package in metadata["packages"]}
    errors = []

    if toolchain.get("channel") != REQUIRED_RUST_TOOLCHAIN:
        errors.append(
            "active Rust toolchain: expected pinned "
            f"{REQUIRED_RUST_TOOLCHAIN}, got {toolchain.get('channel')!r}"
        )
    if toolchain.get("profile") != "minimal":
        errors.append("active Rust toolchain must use the minimal rustup profile")
    if set(toolchain.get("components", [])) != {"clippy", "rustfmt"}:
        errors.append("active Rust toolchain must include clippy and rustfmt")

    dev_profile = manifest.get("profile", {}).get("dev", {})
    if dev_profile.get("debug") != "line-tables-only":
        errors.append("profile.dev.debug must remain line-tables-only")
    if dev_profile.get("split-debuginfo") != "unpacked":
        errors.append("profile.dev.split-debuginfo must remain unpacked")
    dependency_profile = dev_profile.get("package", {}).get("*", {})
    if dependency_profile.get("opt-level") != 1:
        errors.append('profile.dev.package."*".opt-level must remain 1')

    project = python_project.get("project", {})
    if project.get("requires-python") != ">=3.11,<3.12":
        errors.append("Python development tooling must remain pinned to Python 3.11")
    development_dependencies = python_project.get("dependency-groups", {}).get("dev", [])
    if development_dependencies != ["pytest==9.0.3", "ruff==0.15.15"]:
        errors.append(
            "Python development dependencies must use the pinned pytest and Ruff versions"
        )
    uv_config = python_project.get("tool", {}).get("uv", {})
    if uv_config.get("package") is not False:
        errors.append("the repository-root uv project must remain non-packaged")
    if uv_config.get("required-version") != ">=0.11.28":
        errors.append("the repository-root uv project requires uv >=0.11.28")

    if set(packages) != set(EXPECTED):
        errors.append(f"workspace packages: expected {sorted(EXPECTED)}, got {sorted(packages)}")

    for name, expected_dependencies in EXPECTED.items():
        package = packages.get(name)
        if package is None:
            continue
        local_dependencies = {
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency["name"] in EXPECTED and dependency["kind"] != "dev"
        }
        local_dev_dependencies = {
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency["name"] in EXPECTED and dependency["kind"] == "dev"
        }
        if local_dependencies != expected_dependencies:
            errors.append(
                f"{name} local dependencies: expected {sorted(expected_dependencies)}, "
                f"got {sorted(local_dependencies)}"
            )
        expected_dev_dependencies = EXPECTED_DEV.get(name, set())
        if local_dev_dependencies != expected_dev_dependencies:
            errors.append(
                f"{name} local dev dependencies: expected "
                f"{sorted(expected_dev_dependencies)}, got {sorted(local_dev_dependencies)}"
            )
        if package["rust_version"] is not None:
            errors.append(
                f"{name} must not claim a stable rust-version under the exact-nightly policy"
            )
        if package["edition"] != "2024":
            errors.append(f"{name} edition: expected 2024, got {package['edition']!r}")
        if package["publish"] != []:
            errors.append(f"{name} must remain publish = false during the prototype")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("prototype workspace boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
