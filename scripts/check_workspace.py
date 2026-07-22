#!/usr/bin/env python3
"""Validate Tiler's exact Rust workspace and resolved Cargo boundary."""

from __future__ import annotations

import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

EXPECTED_MEMBERS = (
    "crates/tiler-artifact",
    "crates/tiler-compiler",
    "crates/tiler-ir",
    "crates/tiler-metal",
    "crates/tiler-reference",
    "prototypes/serial-sum-compile",
    "prototypes/serial-sum-run",
)
EXPECTED_EXCLUDES = (
    "spikes/extensions/operation-api",
    "spikes/extensions/proc-macro-visibility",
    "spikes/indexing/index-access-model",
    "spikes/macro-environment/fixture",
)
EXPECTED_WORKSPACE_PACKAGE = {
    "version": "0.0.0",
    "edition": "2024",
    "license": "MIT OR Apache-2.0",
    "repository": "https://github.com/moderately-ai/tiler",
    "publish": False,
}
EXPECTED_WORKSPACE_DEPENDENCIES: dict[str, object] = {
    "num-bigint": "0.4.6",
    "num-integer": "0.1.46",
    "num-traits": "0.2.19",
    "tiler-artifact": {"path": "crates/tiler-artifact"},
    "tiler-compiler": {"path": "crates/tiler-compiler"},
    "tiler-ir": {"path": "crates/tiler-ir"},
    "tiler-metal": {"path": "crates/tiler-metal"},
    "tiler-reference": {"path": "crates/tiler-reference"},
    "trybuild": "1.0.114",
}
EXPECTED_RUST_LINTS = {"missing_docs": "warn", "unsafe_code": "forbid"}
EXPECTED_CLIPPY_LINTS = {
    "all": {"level": "warn", "priority": -1},
    "pedantic": {"level": "warn", "priority": -1},
}
EXPECTED_RUSTFMT = {"edition": "2024", "max_width": 100}

PACKAGE_DESCRIPTIONS = {
    "tiler-artifact": "Target-neutral artifact and execution contracts for Tiler",
    "tiler-compiler": "Target-independent optimization and scheduling for Tiler",
    "tiler-ir": "Target-independent tensor compiler representations for Tiler",
    "tiler-metal": "Pure structured-kernel-to-Metal-source lowering for Tiler",
    "tiler-reference": "Target-independent executable reference semantics for Tiler",
    "tiler-prototype-compile": "Non-published producer for Tiler's serial-Sum value proof",
    "tiler-prototype-run": "Non-published runner for Tiler's serial-Sum value proof",
}
PACKAGE_DIRS = {
    "tiler-artifact": "crates/tiler-artifact",
    "tiler-compiler": "crates/tiler-compiler",
    "tiler-ir": "crates/tiler-ir",
    "tiler-metal": "crates/tiler-metal",
    "tiler-reference": "crates/tiler-reference",
    "tiler-prototype-compile": "prototypes/serial-sum-compile",
    "tiler-prototype-run": "prototypes/serial-sum-run",
}


def dependency(
    name: str,
    *,
    kind: str | None = None,
    path: str | None = None,
    requirement: str = "*",
    source: str | None = None,
) -> dict[str, object]:
    """Build one exact normalized Cargo metadata dependency contract."""
    return {
        "name": name,
        "source": source,
        "req": requirement,
        "kind": kind,
        "rename": None,
        "optional": False,
        "uses_default_features": True,
        "features": [],
        "target": None,
        "registry": None,
        "path": path,
    }


CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
EXPECTED_DEPENDENCIES = {
    "tiler-artifact": [dependency("tiler-ir", path="crates/tiler-ir")],
    "tiler-compiler": [
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-reference", kind="dev", path="crates/tiler-reference"),
    ],
    "tiler-ir": [
        dependency("num-bigint", requirement="^0.4.6", source=CRATES_IO),
        dependency("num-integer", requirement="^0.1.46", source=CRATES_IO),
        dependency("num-traits", requirement="^0.2.19", source=CRATES_IO),
        dependency("trybuild", kind="dev", requirement="^1.0.114", source=CRATES_IO),
    ],
    "tiler-metal": [
        dependency("tiler-artifact", path="crates/tiler-artifact"),
        dependency("tiler-ir", path="crates/tiler-ir"),
    ],
    "tiler-reference": [dependency("tiler-ir", path="crates/tiler-ir")],
    "tiler-prototype-compile": [
        dependency("tiler-artifact", path="crates/tiler-artifact"),
        dependency("tiler-compiler", path="crates/tiler-compiler"),
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-metal", path="crates/tiler-metal"),
        dependency("tiler-reference", path="crates/tiler-reference"),
    ],
    "tiler-prototype-run": [dependency("tiler-artifact", path="crates/tiler-artifact")],
}

EXPECTED_TESTS = {
    "tiler-ir": {
        "index_region": "crates/tiler-ir/tests/index_region.rs",
        "index_region_ui": "crates/tiler-ir/tests/index_region_ui.rs",
        "shape_evidence": "crates/tiler-ir/tests/shape_evidence.rs",
        "shape_evidence_ui": "crates/tiler-ir/tests/shape_evidence_ui.rs",
        "typed_handles": "crates/tiler-ir/tests/typed_handles.rs",
    },
    "tiler-reference": {"serial_sum_slice": "crates/tiler-reference/tests/serial_sum_slice.rs"},
}


def expected_member_manifest(name: str) -> dict[str, object]:
    """Return the complete authored manifest contract for one package."""
    manifest: dict[str, object] = {
        "package": {
            "name": name,
            "description": PACKAGE_DESCRIPTIONS[name],
            "version": {"workspace": True},
            "edition": {"workspace": True},
            "license": {"workspace": True},
            "repository": {"workspace": True},
            "publish": {"workspace": True},
        }
    }
    if name.startswith("tiler-prototype-"):
        manifest["bin"] = [
            {
                "name": name,
                "path": "src/main.rs",
                "test": True,
                "doc": True,
            }
        ]
    else:
        manifest["lib"] = {"test": True, "doctest": True, "doc": True}
    normal = {
        item["name"]: {"workspace": True}
        for item in EXPECTED_DEPENDENCIES[name]
        if item["kind"] is None
    }
    development = {
        item["name"]: {"workspace": True}
        for item in EXPECTED_DEPENDENCIES[name]
        if item["kind"] == "dev"
    }
    if normal:
        manifest["dependencies"] = normal
    if development:
        manifest["dev-dependencies"] = development
    manifest["lints"] = {"workspace": True}
    return manifest


def load_toml(path: Path) -> dict[str, Any]:
    """Load one UTF-8 TOML document."""
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"cannot parse {path}: {error}") from error


def relative(root: Path, value: str | None) -> str | None:
    """Normalize a metadata path relative to the workspace."""
    if value is None:
        return None
    try:
        return Path(value).resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return value


def normalize_dependency(root: Path, raw: dict[str, object]) -> dict[str, object]:
    """Select every output-affecting dependency field from Cargo metadata."""
    return {
        "name": raw.get("name"),
        "source": raw.get("source"),
        "req": raw.get("req"),
        "kind": raw.get("kind"),
        "rename": raw.get("rename"),
        "optional": raw.get("optional"),
        "uses_default_features": raw.get("uses_default_features"),
        "features": raw.get("features"),
        "target": raw.get("target"),
        "registry": raw.get("registry"),
        "path": relative(root, raw.get("path") if isinstance(raw.get("path"), str) else None),
    }


def normalize_target(root: Path, raw: dict[str, object]) -> dict[str, object]:
    """Select the governed target-role fields from Cargo metadata."""
    return {
        "name": raw.get("name"),
        "kind": raw.get("kind"),
        "crate_types": raw.get("crate_types"),
        "src_path": relative(
            root, raw.get("src_path") if isinstance(raw.get("src_path"), str) else None
        ),
        "edition": raw.get("edition"),
        "doc": raw.get("doc"),
        "doctest": raw.get("doctest"),
        "test": raw.get("test"),
    }


def expected_targets(name: str) -> list[dict[str, object]]:
    """Return the exact target set for one governed package."""
    package_dir = PACKAGE_DIRS[name]
    if name.startswith("tiler-prototype-"):
        targets = [
            {
                "name": name,
                "kind": ["bin"],
                "crate_types": ["bin"],
                "src_path": f"{package_dir}/src/main.rs",
                "edition": "2024",
                "doc": True,
                "doctest": False,
                "test": True,
            }
        ]
    else:
        targets = [
            {
                "name": name.replace("-", "_"),
                "kind": ["lib"],
                "crate_types": ["lib"],
                "src_path": f"{package_dir}/src/lib.rs",
                "edition": "2024",
                "doc": True,
                "doctest": True,
                "test": True,
            }
        ]
    for test_name, test_path in EXPECTED_TESTS.get(name, {}).items():
        targets.append(
            {
                "name": test_name,
                "kind": ["test"],
                "crate_types": ["bin"],
                "src_path": test_path,
                "edition": "2024",
                "doc": False,
                "doctest": False,
                "test": True,
            }
        )
    return sorted(targets, key=lambda target: (str(target["kind"]), str(target["name"])))


def validate_manifest_contract(root: Path, metadata: dict[str, object]) -> list[str]:
    """Return typed violations of the exact workspace contract."""
    errors: list[str] = []
    manifest = load_toml(root / "Cargo.toml")
    rustfmt = load_toml(root / "rustfmt.toml")
    workspace = manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        return ["workspace.root: [workspace] is missing"]

    checks = (
        (workspace.get("members"), list(EXPECTED_MEMBERS), "workspace.members"),
        (workspace.get("exclude"), list(EXPECTED_EXCLUDES), "workspace.exclude"),
        (workspace.get("resolver"), "3", "workspace.resolver"),
        (workspace.get("package"), EXPECTED_WORKSPACE_PACKAGE, "workspace.package"),
        (
            workspace.get("dependencies"),
            EXPECTED_WORKSPACE_DEPENDENCIES,
            "workspace.dependencies",
        ),
        (
            workspace.get("lints", {}).get("rust")
            if isinstance(workspace.get("lints"), dict)
            else None,
            EXPECTED_RUST_LINTS,
            "workspace.lints.rust",
        ),
        (
            workspace.get("lints", {}).get("clippy")
            if isinstance(workspace.get("lints"), dict)
            else None,
            EXPECTED_CLIPPY_LINTS,
            "workspace.lints.clippy",
        ),
        (rustfmt, EXPECTED_RUSTFMT, "rustfmt.config"),
    )
    for actual, expected, label in checks:
        if actual != expected:
            errors.append(f"{label}: expected {expected!r}, got {actual!r}")

    if set(manifest) != {"workspace", "profile"}:
        errors.append(
            f"workspace.root-tables: expected ['profile', 'workspace'], got {sorted(manifest)}"
        )
    if set(workspace) != {
        "members",
        "exclude",
        "resolver",
        "package",
        "dependencies",
        "lints",
    }:
        errors.append(f"workspace.tables: unexpected keys {sorted(workspace)}")

    expected_profiles = {
        "dev": {
            "debug": "line-tables-only",
            "split-debuginfo": "unpacked",
            "package": {"*": {"opt-level": 1}},
        }
    }
    if manifest.get("profile") != expected_profiles:
        errors.append(f"workspace.profiles: unexpected contract {manifest.get('profile')!r}")

    packages_raw = metadata.get("packages")
    if not isinstance(packages_raw, list):
        return [*errors, "workspace.metadata: packages are missing"]
    packages = {
        package.get("name"): package for package in packages_raw if isinstance(package, dict)
    }
    if set(packages) != set(PACKAGE_DIRS):
        errors.append(
            f"workspace.packages: expected {sorted(PACKAGE_DIRS)}, got {sorted(packages)}"
        )

    for name, package_dir in PACKAGE_DIRS.items():
        package = packages.get(name)
        if not isinstance(package, dict):
            continue
        package_manifest = load_toml(root / package_dir / "Cargo.toml")
        expected_authored_manifest = expected_member_manifest(name)
        if package_manifest != expected_authored_manifest:
            errors.append(
                f"package.{name}.manifest: expected {expected_authored_manifest!r}, got "
                f"{package_manifest!r}"
            )
        if package_manifest.get("lints") != {"workspace": True}:
            errors.append(f"package.{name}.lints: must inherit the workspace policy exactly")

        package_fields = {
            "version": package.get("version"),
            "edition": package.get("edition"),
            "license": package.get("license"),
            "repository": package.get("repository"),
            "publish": package.get("publish"),
            "rust_version": package.get("rust_version"),
            "features": package.get("features"),
            "links": package.get("links"),
            "default_run": package.get("default_run"),
        }
        expected_fields = {
            "version": "0.0.0",
            "edition": "2024",
            "license": "MIT OR Apache-2.0",
            "repository": "https://github.com/moderately-ai/tiler",
            "publish": [],
            "rust_version": None,
            "features": {},
            "links": None,
            "default_run": None,
        }
        if package_fields != expected_fields:
            errors.append(
                f"package.{name}.resolved-fields: expected {expected_fields!r}, got "
                f"{package_fields!r}"
            )

        raw_dependencies = package.get("dependencies")
        actual_dependencies = (
            sorted(
                (normalize_dependency(root, item) for item in raw_dependencies),
                key=lambda item: (str(item["name"]), str(item["kind"])),
            )
            if isinstance(raw_dependencies, list)
            else None
        )
        expected_dependencies = sorted(
            EXPECTED_DEPENDENCIES[name],
            key=lambda item: (str(item["name"]), str(item["kind"])),
        )
        if actual_dependencies != expected_dependencies:
            errors.append(
                f"package.{name}.dependencies: expected {expected_dependencies!r}, got "
                f"{actual_dependencies!r}"
            )

        raw_targets = package.get("targets")
        actual_targets = (
            sorted(
                (normalize_target(root, item) for item in raw_targets),
                key=lambda item: (str(item["kind"]), str(item["name"])),
            )
            if isinstance(raw_targets, list)
            else None
        )
        target_contract = expected_targets(name)
        if actual_targets != target_contract:
            errors.append(
                f"package.{name}.targets: expected {target_contract!r}, got {actual_targets!r}"
            )
    return errors


def cargo_metadata(root: Path, toolchain: str) -> dict[str, object]:
    """Read locked metadata through the exact selected toolchain."""
    result = subprocess.run(
        [
            "rustup",
            "run",
            toolchain,
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    parsed = json.loads(result.stdout)
    if not isinstance(parsed, dict):
        raise ValueError("cargo metadata did not return an object")
    return parsed


def configured_toolchain(root: Path) -> str:
    """Return the repository's sole exact dated Rust toolchain pin."""
    toolchain = load_toml(root / "rust-toolchain.toml").get("toolchain")
    if not isinstance(toolchain, dict):
        raise ValueError("rust-toolchain.toml: [toolchain] is missing")
    if toolchain.get("profile") != "minimal":
        raise ValueError("rust-toolchain.toml: profile must be 'minimal'")
    if toolchain.get("components") != ["clippy", "rustfmt"]:
        raise ValueError("rust-toolchain.toml: components must be ['clippy', 'rustfmt']")
    channel = toolchain.get("channel")
    if not isinstance(channel, str) or not channel.startswith("nightly-"):
        raise ValueError("rust-toolchain.toml: channel must be an exact dated nightly")
    date = channel.removeprefix("nightly-")
    try:
        year, month, day = (int(part) for part in date.split("-"))
    except ValueError as error:
        raise ValueError("rust-toolchain.toml: malformed dated nightly") from error
    if not (2020 <= year <= 9999 and 1 <= month <= 12 and 1 <= day <= 31):
        raise ValueError("rust-toolchain.toml: malformed dated nightly")
    return channel


def main() -> int:
    try:
        toolchain = configured_toolchain(ROOT)
        metadata = cargo_metadata(ROOT, toolchain)
        errors = validate_manifest_contract(ROOT, metadata)
    except (OSError, ValueError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"workspace.validation: {error}", file=sys.stderr)
        return 1
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Rust workspace boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
