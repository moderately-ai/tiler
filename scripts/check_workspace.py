#!/usr/bin/env python3
"""Check the prototype workspace dependency and compatibility boundary."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    "tiler-ir": set(),
    "tiler-artifact": {"tiler-ir"},
    "tiler-compiler": {"tiler-artifact", "tiler-ir"},
    "tiler-metal": {"tiler-artifact", "tiler-ir"},
    "tiler-prototype-compile": {"tiler-artifact", "tiler-compiler", "tiler-ir", "tiler-metal"},
    "tiler-prototype-run": {"tiler-artifact"},
}


def main() -> int:
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

    if set(packages) != set(EXPECTED):
        errors.append(f"workspace packages: expected {sorted(EXPECTED)}, got {sorted(packages)}")

    for name, expected_dependencies in EXPECTED.items():
        package = packages.get(name)
        if package is None:
            continue
        local_dependencies = {
            dependency["name"] for dependency in package["dependencies"] if dependency["name"] in EXPECTED
        }
        if local_dependencies != expected_dependencies:
            errors.append(
                f"{name} local dependencies: expected {sorted(expected_dependencies)}, "
                f"got {sorted(local_dependencies)}"
            )
        if package["rust_version"] != "1.89":
            errors.append(f"{name} rust-version: expected 1.89, got {package['rust_version']!r}")
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
