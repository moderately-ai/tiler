#!/usr/bin/env python3
"""Mutation tests for the Apple target experiment evidence gates."""

from __future__ import annotations

import hashlib
import importlib.util
import os
import platform
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
VALIDATOR_PATH = HERE / "validate_compatibility_record.py"
SPEC = importlib.util.spec_from_file_location("compatibility_validator", VALIDATOR_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load compatibility validator")
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


def valid_record() -> dict[str, str]:
    """Build the smallest complete syntactically valid evidence record."""
    digest = "a" * 64
    values = {
        "schema": VALIDATOR.SCHEMA,
        "probe.result_root": ".",
        "probe.repository_base_revision": "b" * 40,
        "probe.source_sha256": digest,
        "probe.harness_sha256": digest,
        "probe.validator_sha256": digest,
        "probe.project_sha256": digest,
        "probe.lock_sha256": digest,
        "probe.input_manifest_file": "input-manifest.tsv",
        "probe.input_manifest_sha256": digest,
        "probe.compiler_flags": VALIDATOR.COMPILER_FLAGS,
        "host.date_utc": "2026-07-21T00:00:00Z",
        "host.developer_dir": "/Applications/Xcode.app/Contents/Developer",
        "host.xcode": "Xcode 26.6 Build version 17F113",
        "host.metal_toolchain_component": (
            "Build Version: 17F109 Toolchain Identifier: com.apple.dt.toolchain.Metal.32023.883"
        ),
        "host.xcrun": "xcrun version 72.",
        "host.os_version": "27.0",
        "host.os_build": "26A5378n",
        "host.machine": "arm64",
        "tool.metal.path": "/toolchain/metal",
        "tool.metal.version": "Apple metal version 32023.883",
        "tool.metal.sha256": digest,
        "tool.metallib.path": "/toolchain/metallib",
        "tool.metallib.version": "AIR-LLD 32023.883",
        "tool.metallib.sha256": digest,
    }
    for sdk in VALIDATOR.SDKS:
        values.update(
            {
                f"sdk.{sdk}.path": f"/SDKs/{sdk}.sdk",
                f"sdk.{sdk}.version": "26.5",
                f"sdk.{sdk}.build": "25F70",
                f"sdk.{sdk}.settings_file": f"/tmp/{sdk}.settings.txt",
                f"sdk.{sdk}.settings_sha256": digest,
            }
        )
    for family, (sdk, target) in VALIDATOR.FAMILIES.items():
        for run in ("a", "b"):
            prefix = f"matrix.{family}.{run}"
            air_digest = "a" * 64 if run == "a" else "b" * 64
            values.update(
                {
                    f"{prefix}.sdk": sdk,
                    f"{prefix}.target": target,
                    f"{prefix}.command.metal": VALIDATOR.metal_command(sdk, target),
                    f"{prefix}.command.metallib": VALIDATOR.metallib_command(sdk),
                    f"{prefix}.air_sha256": air_digest,
                    f"{prefix}.metallib_sha256": digest,
                    f"{prefix}.log_sha256": digest,
                }
            )
        values[f"repro.{family}.air.byte_identical"] = "false"
        values[f"repro.{family}.metallib.byte_identical"] = "true"
    values["probe.status"] = "validated"
    return values


def assert_validator_mutations() -> None:
    baseline = valid_record()
    with tempfile.TemporaryDirectory(prefix="tiler-compat-record-test.") as directory:
        root = Path(directory)
        empty_digest = hashlib.sha256(b"").hexdigest()
        manifest = root / "input-manifest.tsv"
        manifest.write_text(
            "".join(
                f"{path}\t{baseline[field]}\n"
                for path, field in {
                    "spikes/apple-targets/compatibility_probe.sh": "probe.harness_sha256",
                    "spikes/apple-targets/copy.metal": "probe.source_sha256",
                    "spikes/apple-targets/validate_compatibility_record.py": (
                        "probe.validator_sha256"
                    ),
                    "pyproject.toml": "probe.project_sha256",
                    "uv.lock": "probe.lock_sha256",
                }.items()
            ),
            encoding="utf-8",
        )
        baseline["probe.input_manifest_sha256"] = hashlib.sha256(manifest.read_bytes()).hexdigest()
        for sdk in VALIDATOR.SDKS:
            settings = root / "sdk" / f"{sdk}.settings.txt"
            settings.parent.mkdir(exist_ok=True)
            settings.write_text("settings\n", encoding="utf-8")
            baseline[f"sdk.{sdk}.settings_file"] = f"sdk/{sdk}.settings.txt"
            baseline[f"sdk.{sdk}.settings_sha256"] = hashlib.sha256(
                settings.read_bytes()
            ).hexdigest()
        for family in VALIDATOR.FAMILIES:
            for run in ("a", "b"):
                log = root / f"out-{run}" / f"{family}.log"
                log.parent.mkdir(exist_ok=True)
                log.touch()
                baseline[f"matrix.{family}.{run}.log_sha256"] = empty_digest

        record = root / "record.tsv"

        def run(values: dict[str, str]) -> subprocess.CompletedProcess[str]:
            record.write_text(
                "".join(f"{key}\t{value}\n" for key, value in values.items()),
                encoding="utf-8",
            )
            return subprocess.run(
                [sys.executable, str(VALIDATOR_PATH), str(record)],
                check=False,
                capture_output=True,
                text=True,
            )

        if run(baseline).returncode != 0:
            raise AssertionError("validator rejected complete retained evidence fixture")

        retained_log = root / "out-a" / "macos13.log"
        retained_log.write_text("mutated\n", encoding="utf-8")
        if run(baseline).returncode == 0:
            raise AssertionError("validator accepted a mutated retained command log")
        retained_log.write_bytes(b"")

        retained_settings = root / "sdk" / "macosx.settings.txt"
        original_settings = retained_settings.read_bytes()
        retained_settings.write_text("mutated\n", encoding="utf-8")
        if run(baseline).returncode == 0:
            raise AssertionError("validator accepted mutated retained SDK settings")
        retained_settings.write_bytes(original_settings)

        original_manifest = manifest.read_bytes()
        manifest.write_text("mutated\n", encoding="utf-8")
        if run(baseline).returncode == 0:
            raise AssertionError("validator accepted mutated retained input manifest")
        manifest.write_bytes(original_manifest)

        provenance = [
            key for key in baseline if key.startswith(("probe.", "host.", "sdk.", "tool."))
        ]
        for key in provenance:
            mutation = dict(baseline)
            del mutation[key]
            if run(mutation).returncode == 0:
                raise AssertionError(f"validator accepted missing provenance field: {key}")

        malformed = {
            "host.date_utc": "today",
            "host.developer_dir": "relative/path",
            "host.xcode": "unknown",
            "host.metal_toolchain_component": "unknown",
            "host.xcrun": "latest",
            "host.os_version": "latest",
            "host.os_build": "contains spaces",
            "host.machine": "mystery",
            "sdk.macosx.version": "latest",
            "sdk.macosx.path": "/not-an-sdk",
            "sdk.iphoneos.build": "bad build",
            "sdk.iphoneos.settings_file": "settings",
            "sdk.iphonesimulator.settings_sha256": "short",
            "tool.metal.path": "relative/metal",
            "tool.metal.version": "unknown",
            "tool.metallib.path": "/toolchain/wrong",
            "tool.metallib.version": "unknown",
            "tool.metallib.sha256": "short",
        }
        for key, value in malformed.items():
            mutation = dict(baseline)
            mutation[key] = value
            if run(mutation).returncode == 0:
                raise AssertionError(f"validator accepted malformed provenance field: {key}")

        command_mutations = {
            "probe.compiler_flags": "-std=metal3.1 -O2",
            "matrix.macos13.a.command.metal": VALIDATOR.metal_command(
                "iphoneos", "air64-apple-macos13.0"
            ),
            "matrix.macos13.a.command.metallib": VALIDATOR.metallib_command("iphoneos"),
        }
        for key, value in command_mutations.items():
            mutation = dict(baseline)
            mutation[key] = value
            if run(mutation).returncode == 0:
                raise AssertionError(f"validator accepted inconsistent command field: {key}")

        for key, value in {
            "repro.macos13.air.byte_identical": "true",
            "repro.macos13.metallib.byte_identical": "false",
        }.items():
            mutation = dict(baseline)
            mutation[key] = value
            if run(mutation).returncode == 0:
                raise AssertionError(f"validator accepted false reproducibility field: {key}")

        trailing = dict(baseline)
        trailing["unexpected.trailing"] = "value"
        if run(trailing).returncode == 0:
            raise AssertionError("validator accepted data after terminal validation status")


def assert_runtime_injections() -> None:
    if platform.system() != "Darwin":
        print("runtime_injections=skipped reason=non-darwin")
        return
    with tempfile.TemporaryDirectory(prefix="tiler-runtime-probe-test.") as directory:
        executable = Path(directory) / "runtime-probe"
        subprocess.run(
            [
                "xcrun",
                "--sdk",
                "macosx",
                "swiftc",
                str(HERE / "runtime_failure_probe.swift"),
                "-framework",
                "Metal",
                "-o",
                str(executable),
            ],
            check=True,
        )
        injections = subprocess.run(
            [str(executable), "--list-injections"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        if not injections:
            raise AssertionError("runtime probe declared no injectable unexpected outcomes")
        source = (HERE / "runtime_failure_probe.swift").read_text(encoding="utf-8")
        declarations = dict(re.findall(r'case ([A-Za-z]+) = "([a-z-]+)"', source))
        if set(declarations.values()) != set(injections):
            raise AssertionError("listed injections differ from declared unexpected stages")
        for case in declarations:
            if source.count(f"unexpected(.{case}") != 1:
                raise AssertionError(f"unexpected stage is not wired exactly once: {case}")
        for injection in injections:
            environment = dict(os.environ)
            environment["TILER_APPLE_RUNTIME_INJECT"] = injection
            result = subprocess.run(
                [str(executable)],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                raise AssertionError(f"runtime probe accepted injected outcome: {injection}")


def test_compatibility_evidence_mutations() -> None:
    """Every incomplete or corrupted evidence record must fail closed."""
    assert_validator_mutations()


def test_runtime_unexpected_outcome_injections() -> None:
    """Every declared unexpected runtime stage must terminate unsuccessfully."""
    assert_runtime_injections()


def main() -> int:
    assert_validator_mutations()
    assert_runtime_injections()
    print("apple_target_probe_tests=passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
