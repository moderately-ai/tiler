"""Mutation tests for the fail-closed Rust workspace gate."""

from __future__ import annotations

import copy
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPOSITORY_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

import check_rust  # noqa: E402
import check_workspace  # noqa: E402


@pytest.fixture(scope="module")
def metadata() -> dict[str, object]:
    """Load the unmodified locked metadata once through the canonical pin."""
    toolchain = check_workspace.configured_toolchain(REPOSITORY_ROOT)
    return check_workspace.cargo_metadata(REPOSITORY_ROOT, toolchain)


def contract_copy(tmp_path: Path, metadata: dict[str, object]) -> tuple[Path, dict[str, object]]:
    """Copy only governed manifests and rewrite metadata paths to that copy."""
    root = tmp_path / "repo"
    root.mkdir()
    for name in ("Cargo.toml", "rustfmt.toml", "rust-toolchain.toml"):
        shutil.copy2(REPOSITORY_ROOT / name, root / name)
    for relative in check_workspace.EXPECTED_MEMBERS:
        destination = root / relative
        destination.mkdir(parents=True)
        shutil.copy2(REPOSITORY_ROOT / relative / "Cargo.toml", destination / "Cargo.toml")
    encoded = json.dumps(metadata).replace(str(REPOSITORY_ROOT), str(root))
    return root, json.loads(encoded)


def replace(path: Path, old: str, new: str) -> None:
    """Apply one exact textual manifest mutation."""
    source = path.read_text()
    assert old in source
    path.write_text(source.replace(old, new, 1))


def errors_for(root: Path, metadata: dict[str, object]) -> str:
    """Render contract errors for concise assertions."""
    return "\n".join(check_workspace.validate_manifest_contract(root, metadata))


def test_current_workspace_contract_is_complete(metadata: dict[str, object]) -> None:
    """Keep the checked-in manifests and resolved target graph canonical."""
    assert check_workspace.validate_manifest_contract(REPOSITORY_ROOT, metadata) == []


@pytest.mark.parametrize(
    ("relative", "old", "new", "category"),
    (
        ("Cargo.toml", 'resolver = "3"', 'resolver = "2"', "workspace.resolver"),
        (
            "Cargo.toml",
            'missing_docs = "warn"',
            'missing_docs = "allow"',
            "workspace.lints.rust",
        ),
        (
            "Cargo.toml",
            'unsafe_code = "forbid"',
            'unsafe_code = "allow"',
            "workspace.lints.rust",
        ),
        (
            "rustfmt.toml",
            'edition = "2024"',
            "disable_all_formatting = true",
            "rustfmt.config",
        ),
        (
            "crates/tiler-artifact/Cargo.toml",
            "[lints]\nworkspace = true",
            "[lints]\nworkspace = false",
            "package.tiler-artifact.lints",
        ),
        (
            "crates/tiler-artifact/Cargo.toml",
            "version.workspace = true",
            'version = "0.0.0"',
            "package.tiler-artifact.manifest",
        ),
    ),
)
def test_manifest_policy_mutations_fail_at_the_contract_boundary(
    tmp_path: Path,
    metadata: dict[str, object],
    relative: str,
    old: str,
    new: str,
    category: str,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    replace(root / relative, old, new)
    assert category in errors_for(root, copied_metadata)


@pytest.mark.parametrize(
    ("field", "value"),
    (
        ("optional", True),
        ("target", "cfg(unix)"),
        ("rename", "renamed-ir"),
        ("kind", "dev"),
        ("uses_default_features", False),
        ("features", ["bypass"]),
    ),
)
def test_dependency_semantics_cannot_hide_behind_the_same_name_set(
    tmp_path: Path,
    metadata: dict[str, object],
    field: str,
    value: object,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    artifact["dependencies"][0][field] = value
    assert "package.tiler-artifact.dependencies" in errors_for(root, copied_metadata)


def test_unlisted_external_dependency_is_rejected(
    tmp_path: Path, metadata: dict[str, object]
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    extra = copy.deepcopy(artifact["dependencies"][0])
    extra.update(name="num-traits", source=check_workspace.CRATES_IO, req="^0.2.19", path=None)
    artifact["dependencies"].append(extra)
    assert "package.tiler-artifact.dependencies" in errors_for(root, copied_metadata)


@pytest.mark.parametrize(("field", "value"), (("test", False), ("doctest", False), ("doc", False)))
def test_library_target_enablement_is_required(
    tmp_path: Path,
    metadata: dict[str, object],
    field: str,
    value: object,
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    artifact = next(package for package in packages if package["name"] == "tiler-artifact")
    artifact["targets"][0][field] = value
    assert "package.tiler-artifact.targets" in errors_for(root, copied_metadata)


def test_expected_integration_test_and_proof_binary_role_are_required(
    tmp_path: Path, metadata: dict[str, object]
) -> None:
    root, copied_metadata = contract_copy(tmp_path, metadata)
    packages = copied_metadata["packages"]
    ir = next(package for package in packages if package["name"] == "tiler-ir")
    ir["targets"] = [target for target in ir["targets"] if target["name"] != "typed_handles"]
    runner = next(package for package in packages if package["name"] == "tiler-prototype-run")
    runner["targets"][0]["test"] = False
    rendered = errors_for(root, copied_metadata)
    assert "package.tiler-ir.targets" in rendered
    assert "package.tiler-prototype-run.targets" in rendered


@pytest.mark.parametrize(
    "name",
    (
        "RUSTUP_TOOLCHAIN",
        "RUSTUP_HOME",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTDOC",
        "RUSTDOCFLAGS",
        "RUSTFMT",
        "CLIPPY_DRIVER",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_RUSTDOC",
        "CARGO_BUILD_RUSTDOCFLAGS",
        "CARGO_PROFILE_RELEASE_LTO",
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
    ),
)
def test_hostile_environment_controls_are_rejected(name: str) -> None:
    assert check_rust.hostile_environment({name: "bypass"}) == [name]
    with pytest.raises(check_rust.GateFailure, match="environment.hostile"):
        check_rust.sanitized_environment({name: "bypass"})


def test_cargo_home_and_target_directory_are_sanitized(tmp_path: Path) -> None:
    environment = check_rust.sanitized_environment(
        {"PATH": os.environ["PATH"], "CARGO_HOME": str(tmp_path), "CARGO_TARGET_DIR": "elsewhere"}
    )
    assert environment["CARGO_HOME"] == str(Path.home() / ".cargo")
    assert environment["CARGO_TARGET_DIR"] == str(check_rust.ROOT / "target")
    assert str(tmp_path) not in environment["PATH"]
    assert environment["PATH"].endswith("/usr/bin:/bin")


def test_ambient_home_cannot_redirect_governed_tools(tmp_path: Path) -> None:
    with pytest.raises(check_rust.GateFailure, match="HOME must identify"):
        check_rust.sanitized_environment({"HOME": str(tmp_path)})


def test_hostile_cargo_config_is_rejected(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    config = tmp_path / "config.toml"
    config.write_text('[target.aarch64-apple-darwin]\nrunner = "/usr/bin/true"\n')
    monkeypatch.setattr(check_rust, "cargo_config_paths", lambda _environment: [config])
    with pytest.raises(check_rust.GateFailure, match="cargo-config.hostile"):
        check_rust.validate_cargo_configs({})


def test_nested_workspace_cargo_config_is_discovered(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = tmp_path / "repo"
    shape_root = root / "spikes/shapes/nightly-dependent-static-shapes"
    config = shape_root / ".cargo/config.toml"
    config.parent.mkdir(parents=True)
    config.write_text('[build]\nrustdoc = "/usr/bin/true"\n')
    monkeypatch.setattr(check_rust, "ROOT", root)
    monkeypatch.setattr(check_rust, "SHAPE_ROOT", shape_root)

    assert config in check_rust.cargo_config_paths({})
    with pytest.raises(check_rust.GateFailure, match="cargo-config.hostile"):
        check_rust.validate_cargo_configs({})


def test_supported_host_profile_is_explicit() -> None:
    base = {
        'target_pointer_width="64"',
        'target_endian="little"',
        'target_has_atomic="64"',
    }
    check_rust.validate_host(
        base
        | {
            'target_os="macos"',
            'target_arch="aarch64"',
            'target_env=""',
        }
    )
    check_rust.validate_host(
        base
        | {
            'target_os="linux"',
            'target_arch="x86_64"',
            'target_env="gnu"',
        }
    )
    with pytest.raises(check_rust.GateFailure, match="unproved host pair"):
        check_rust.validate_host(
            base
            | {
                'target_os="linux"',
                'target_arch="aarch64"',
                'target_env="gnu"',
            }
        )
    with pytest.raises(check_rust.GateFailure, match="unproved host pair"):
        check_rust.validate_host(
            base
            | {
                'target_os="macos"',
                'target_arch="x86_64"',
                'target_env=""',
            }
        )
    with pytest.raises(check_rust.GateFailure, match="64-bit little-endian"):
        check_rust.validate_host(
            {
                'target_os="linux"',
                'target_arch="x86_64"',
                'target_env="gnu"',
                'target_pointer_width="32"',
                'target_endian="little"',
                'target_has_atomic="64"',
            }
        )


def test_lockfile_mutation_is_a_typed_failure(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    lock = tmp_path / "Cargo.lock"
    lock.write_text("before")
    monkeypatch.setattr(check_rust, "LOCKFILES", (lock,))
    snapshot = check_rust.snapshot_lockfiles()
    lock.write_text("after")
    with pytest.raises(check_rust.GateFailure, match="lockfile.mutated"):
        check_rust.verify_lockfiles(snapshot)


def test_command_plan_uses_one_pin_locked_operations_and_release_numerics(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    commands: list[list[str]] = []
    command_environments: list[dict[str, str]] = []
    phases: list[str] = []
    monkeypatch.setattr(
        check_rust, "validate_cargo_configs", lambda _environment: phases.append("configs")
    )
    monkeypatch.setattr(
        check_rust,
        "sanitized_environment",
        lambda environment: phases.append("sanitize") or environment,
    )
    monkeypatch.setattr(check_rust, "governed_rustup", lambda: "/rustup")
    monkeypatch.setattr(check_rust, "snapshot_lockfiles", lambda: phases.append("snapshot") or {})
    monkeypatch.setattr(
        check_rust, "verify_lockfiles", lambda _snapshot: phases.append("verify-locks")
    )
    monkeypatch.setattr(check_rust, "verify_toolchain", lambda *_args: phases.append("toolchain"))
    monkeypatch.setattr(check_rust, "validate_workspace", lambda *_args: phases.append("workspace"))

    def record(
        command: list[str],
        *,
        environment: dict[str, str],
        cwd: Path = check_rust.ROOT,
        capture: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        del cwd, capture
        commands.append(command)
        command_environments.append(environment.copy())
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(check_rust, "run", record)
    check_rust.run_gate({"PATH": os.environ["PATH"]})

    toolchain = check_workspace.configured_toolchain(REPOSITORY_ROOT)
    prefix = ["/rustup", "run", toolchain, "cargo"]
    assert phases == [
        "configs",
        "sanitize",
        "snapshot",
        "toolchain",
        "workspace",
        "verify-locks",
    ]
    assert commands == [
        prefix + ["fmt", "--all", "--check"],
        prefix + ["check", "--workspace", "--all-targets", "--locked"],
        prefix + ["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
        prefix + ["test", "--workspace", "--locked"],
        prefix
        + [
            "test",
            "--release",
            "--locked",
            "-p",
            "tiler-reference",
            "-p",
            "tiler-compiler",
        ],
        prefix + ["doc", "--workspace", "--no-deps", "--locked"],
        ["/bin/bash", str(check_rust.SHAPE_ROOT / "check.sh"), "/rustup", toolchain],
    ]
    assert "RUSTDOCFLAGS" not in command_environments[0]
    assert command_environments[5]["RUSTDOCFLAGS"] == "-D warnings"
    assert command_environments[6]["CARGO_TARGET_DIR"] == str(check_rust.SHAPE_ROOT / "target")
    assert not (check_rust.SHAPE_ROOT / "rust-toolchain.toml").exists()


def test_lock_verification_runs_after_a_failed_phase(monkeypatch: pytest.MonkeyPatch) -> None:
    phases: list[str] = []
    monkeypatch.setattr(check_rust, "validate_cargo_configs", lambda _environment: None)
    monkeypatch.setattr(check_rust, "sanitized_environment", lambda environment: environment)
    monkeypatch.setattr(check_rust, "governed_rustup", lambda: "/rustup")
    monkeypatch.setattr(check_rust, "snapshot_lockfiles", lambda: {})
    monkeypatch.setattr(check_rust, "verify_toolchain", lambda *_args: None)
    monkeypatch.setattr(check_rust, "validate_workspace", lambda *_args: None)
    monkeypatch.setattr(
        check_rust, "verify_lockfiles", lambda _snapshot: phases.append("verify-locks")
    )
    monkeypatch.setattr(
        check_rust,
        "run",
        lambda *_args, **_kwargs: (_ for _ in ()).throw(check_rust.GateFailure("boom")),
    )

    with pytest.raises(check_rust.GateFailure, match="boom"):
        check_rust.run_gate({})
    assert phases == ["verify-locks"]
