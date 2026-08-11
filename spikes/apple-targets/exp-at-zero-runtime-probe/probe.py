#!/usr/bin/env python3
"""Measure exact F32 `precise::exp` result bits at both signed zeros on Apple9."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile


HERE = Path(__file__).resolve().parent
APPLE_SPIKES = HERE.parent
REPOSITORY = HERE.parents[2]
KERNEL = HERE / "exp_at_zero.metal"
HOST_SOURCE = APPLE_SPIKES / "numerical_probe_host.m"
HARNESS = Path(__file__).resolve()

SCHEMA = "tiler.apple-exp-zero-runtime/v1"
PROFILE_KEY = "tiler.metal.macos-apple9.msl4-0.f32-bf16.v1"
KERNEL_FUNCTION = "exp_at_zero"
KERNEL_EXPRESSION = "precise::exp(input[tid])"
COMPILE_OPTIONS = "math=safe,fpfun=precise,lang=4.0,opt=default"
INPUTS = ("00000000", "80000000")
EXACT_ONE = "3f800000"
MANIFEST = (
    f"runtime.exp-at-zero\tf32\tsource\texp_at_zero.metal\t{KERNEL_FUNCTION}\t"
    f"{COMPILE_OPTIONS},archive=runtime.metalar\n"
)
INVOCATION = f"f32={','.join(INPUTS)}\n"
ARCHIVE_COMPILER = re.compile(rb"Apple metal version [0-9.]+ \(metalfe-[0-9.]+\)")

PROFILE_ROWS = {
    "profile.key": PROFILE_KEY,
    "profile.dtype": "f32",
    "profile.offline.compiler_version": "32023.883",
    "profile.offline.compiler_build": "metalfe-32023.883",
    "profile.offline.xcode_version": "26.6",
    "profile.offline.xcode_build": "17F113",
    "profile.offline.sdk_version": "26.5",
    "profile.offline.sdk_build": "25F70",
    "profile.execution.os_name": "macOS",
    "profile.execution.os_version": "27.0",
    "profile.execution.os_build": "26A5388g",
    "profile.execution.architecture": "arm64",
    "profile.execution.device": "Apple M4 Max",
}

RETAINED_TOOL_ROWS = {
    "environment.xcode_version": "27.0",
    "environment.xcode_build": "27A5228h",
    "environment.sdk_name": "macosx",
    "environment.sdk_version": "27.0",
    "environment.sdk_build": "26A5388f",
    "environment.clang_version": "Apple clang version 21.0.0 (clang-2100.3.27.1)",
    "environment.metal_version": "Apple metal version 32023.921 (metalfe-32023.921)",
    "environment.runtime_compiler.version": "Apple metal version 32023.921 (metalfe-32023.921)",
    "environment.runtime_compiler.version_source": "serialized-MTLBinaryArchive",
}


class ProbeError(RuntimeError):
    """The measurement or its retained record failed closed."""


def digest(blob: bytes) -> str:
    return hashlib.sha256(blob).hexdigest()


def run(command: list[str], cwd: Path | None = None) -> str:
    completed = subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=False)
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "no diagnostic"
        raise ProbeError(f"{' '.join(command)} failed ({completed.returncode}): {detail}")
    return completed.stdout.strip()


def first_line(text: str, subject: str) -> str:
    lines = text.splitlines()
    if not lines or not lines[0]:
        raise ProbeError(f"{subject} reported no value")
    return lines[0]


def parse_two_line_version(text: str, name: str) -> tuple[str, str]:
    rows = text.splitlines()
    if len(rows) != 2 or not rows[0].startswith(f"{name} ") or not rows[1].startswith("Build version "):
        raise ProbeError(f"unexpected {name} version output: {text!r}")
    return rows[0].removeprefix(f"{name} "), rows[1].removeprefix("Build version ")


def parse_host_output(blob: bytes) -> dict[str, object]:
    try:
        lines = blob.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise ProbeError(f"host output is not UTF-8: {error}") from error
    scalars: dict[str, str] = {}
    results: list[str] = []
    images: list[str] = []
    for line in lines:
        if "=" not in line:
            raise ProbeError(f"host output line has no '=': {line!r}")
        key, value = line.split("=", 1)
        if key == "result":
            results.append(value)
        elif key == "runtime-compiler-image":
            images.append(value)
        elif key in scalars:
            raise ProbeError(f"host output repeats {key}")
        else:
            scalars[key] = value
    expected = {
        "device",
        "registry-id",
        "gpu-family-apple9",
        "case",
        "compilation",
        "dtype",
        "applied",
        "archive",
    }
    missing = sorted(expected - scalars.keys())
    unexpected = sorted(scalars.keys() - expected)
    if missing or unexpected:
        raise ProbeError(f"host output keys mismatch: missing={missing}, unexpected={unexpected}")
    if scalars["case"] != "runtime.exp-at-zero":
        raise ProbeError(f"host case mismatch: {scalars['case']}")
    if scalars["compilation"] != "source" or scalars["dtype"] != "f32":
        raise ProbeError("host did not runtime-compile the F32 source case")
    if scalars["applied"] != COMPILE_OPTIONS:
        raise ProbeError(f"applied options mismatch: {scalars['applied']}")
    if scalars["archive"] != "runtime.metalar":
        raise ProbeError(f"unexpected archive path: {scalars['archive']}")
    if len(results) != len(INPUTS) or any(re.fullmatch(r"[0-9a-f]{8}", row) is None for row in results):
        raise ProbeError(f"host returned {len(results)} malformed/result rows; expected 2")
    if not images:
        raise ProbeError("host reported no runtime compiler image")
    return {"scalars": scalars, "results": tuple(results), "images": tuple(images)}


def parse_record(path: Path) -> dict[str, str]:
    rows: dict[str, str] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        fields = line.split("\t")
        if len(fields) != 2 or not all(fields):
            raise ProbeError(f"record line {number} is not one nonempty key/value pair")
        key, value = fields
        if key in rows:
            raise ProbeError(f"record repeats {key}")
        rows[key] = value
    return rows


def require(rows: dict[str, str], key: str, expected: str) -> None:
    found = rows.get(key)
    if found != expected:
        raise ProbeError(f"{key} mismatch: recorded {found!r}, expected {expected!r}")


def validate_record(
    record_path: Path,
    *,
    rows_override: dict[str, str] | None = None,
    kernel_bytes_override: bytes | None = None,
) -> None:
    rows = dict(rows_override) if rows_override is not None else parse_record(record_path)
    result_dir = record_path.parent
    manifest_path = result_dir / "manifest.tsv"
    invocation_path = result_dir / "invocation.txt"
    host_output_path = result_dir / "host-output.txt"
    runtime_compiler_path = result_dir / "runtime-compiler.txt"
    for path in (manifest_path, invocation_path, host_output_path, runtime_compiler_path):
        if not path.is_file():
            raise ProbeError(f"retained producer output is missing: {path.name}")

    require(rows, "schema", SCHEMA)
    require(rows, "probe.harness_sha256", digest(HARNESS.read_bytes()))
    kernel_bytes = KERNEL.read_bytes() if kernel_bytes_override is None else kernel_bytes_override
    require(rows, "probe.kernel_sha256", digest(kernel_bytes))
    require(rows, "probe.host_source_sha256", digest(HOST_SOURCE.read_bytes()))
    require(rows, "probe.kernel_function", KERNEL_FUNCTION)
    require(rows, "probe.kernel_expression", KERNEL_EXPRESSION)
    require(rows, "probe.compile_options", COMPILE_OPTIONS)
    require(rows, "measurement.route", "MTLDevice.newLibraryWithSource")
    require(rows, "measurement.scope", "runtime-compiler-only-not-production-aot")
    for key, value in PROFILE_ROWS.items():
        require(rows, key, value)

    manifest = manifest_path.read_bytes()
    invocation = invocation_path.read_bytes()
    host_output = host_output_path.read_bytes()
    runtime_compiler = runtime_compiler_path.read_bytes()
    require(rows, "producer.manifest_sha256", digest(manifest))
    require(rows, "producer.invocation_sha256", digest(invocation))
    require(rows, "producer.host_output_sha256", digest(host_output))
    require(rows, "producer.runtime_compiler_sha256", digest(runtime_compiler))
    if manifest != MANIFEST.encode("utf-8"):
        raise ProbeError("retained manifest does not name the exact kernel/function/options")
    if invocation != INVOCATION.encode("ascii"):
        raise ProbeError(f"retained invocation does not carry +0/-0 inputs: {invocation!r}")

    require(rows, "measurement.input.count", str(len(INPUTS)))
    for index, bits in enumerate(INPUTS):
        require(rows, f"measurement.input.{index}", bits)
    observed = parse_host_output(host_output)
    scalars = observed["scalars"]
    results = observed["results"]
    images = observed["images"]
    assert isinstance(scalars, dict) and isinstance(results, tuple) and isinstance(images, tuple)
    require(rows, "measurement.result.count", str(len(results)))
    for index, bits in enumerate(results):
        require(rows, f"measurement.result.{index}", bits)
    exact = "true" if results == (EXACT_ONE, EXACT_ONE) else "false"
    require(rows, "measurement.exact_at_zero", exact)
    require(rows, "environment.device", str(scalars["device"]))
    require(rows, "environment.device_registry_id", str(scalars["registry-id"]))
    require(rows, "environment.gpu_family_apple9", str(scalars["gpu-family-apple9"]))
    require(rows, "environment.runtime_compiler.images", " | ".join(str(image) for image in images))
    compiler_match = ARCHIVE_COMPILER.fullmatch(runtime_compiler.rstrip(b"\n"))
    if compiler_match is None or not runtime_compiler.endswith(b"\n"):
        raise ProbeError("retained runtime compiler scan is not one canonical archive version")
    require(rows, "environment.runtime_compiler.version", compiler_match.group(0).decode("ascii"))
    for key, value in RETAINED_TOOL_ROWS.items():
        require(rows, key, value)

    if rows["environment.gpu_family_apple9"] != "supported":
        raise ProbeError("the measured device is not Apple9")
    if rows["environment.device"] != PROFILE_ROWS["profile.execution.device"]:
        raise ProbeError("the measured device does not match the authoritative execution row")
    for key in ("os_name", "os_version", "os_build", "architecture"):
        require(rows, f"environment.{key}", PROFILE_ROWS[f"profile.execution.{key}"])

    required_keys = {
        "schema",
        "probe.date_utc",
        "probe.repository_base_revision",
        "probe.harness_sha256",
        "probe.kernel_sha256",
        "probe.host_source_sha256",
        "probe.kernel_function",
        "probe.kernel_expression",
        "probe.compile_options",
        "measurement.route",
        "measurement.scope",
        "measurement.input.count",
        "measurement.input.0",
        "measurement.input.1",
        "measurement.result.count",
        "measurement.result.0",
        "measurement.result.1",
        "measurement.exact_at_zero",
        *PROFILE_ROWS.keys(),
        *RETAINED_TOOL_ROWS.keys(),
        "environment.os_name",
        "environment.os_version",
        "environment.os_build",
        "environment.architecture",
        "environment.device",
        "environment.device_registry_id",
        "environment.gpu_family_apple9",
        "environment.xcode_path",
        "environment.xcode_version",
        "environment.xcode_build",
        "environment.sdk_name",
        "environment.sdk_path",
        "environment.sdk_version",
        "environment.sdk_build",
        "environment.clang_path",
        "environment.clang_version",
        "environment.metal_path",
        "environment.metal_version",
        "environment.runtime_compiler.images",
        "environment.runtime_compiler.version",
        "environment.runtime_compiler.version_source",
        "producer.manifest_sha256",
        "producer.invocation_sha256",
        "producer.host_output_sha256",
        "producer.runtime_compiler_sha256",
    }
    missing = sorted(required_keys - rows.keys())
    unexpected = sorted(rows.keys() - required_keys)
    if missing or unexpected:
        raise ProbeError(f"record keys mismatch: missing={missing}, unexpected={unexpected}")


def environment() -> dict[str, str]:
    os_name = run(["sw_vers", "-productName"])
    os_version = run(["sw_vers", "-productVersion"])
    os_build = run(["sw_vers", "-buildVersion"])
    architecture = platform.machine()
    xcode_path = run(["xcode-select", "-p"])
    xcode_version, xcode_build = parse_two_line_version(run(["xcodebuild", "-version"]), "Xcode")
    sdk_name = "macosx"
    sdk_path = run(["xcrun", "--sdk", sdk_name, "--show-sdk-path"])
    sdk_version = run(["xcrun", "--sdk", sdk_name, "--show-sdk-version"])
    sdk_build = run(["xcrun", "--sdk", sdk_name, "--show-sdk-build-version"])
    clang_path = run(["xcrun", "--sdk", sdk_name, "--find", "clang"])
    clang_version = first_line(run(["xcrun", "--sdk", sdk_name, "clang", "--version"]), "clang")
    metal_path = run(["xcrun", "--sdk", sdk_name, "--find", "metal"])
    metal_version = first_line(run(["xcrun", "--sdk", sdk_name, "metal", "--version"]), "metal")
    values = {
        "environment.os_name": os_name,
        "environment.os_version": os_version,
        "environment.os_build": os_build,
        "environment.architecture": architecture,
        "environment.xcode_path": xcode_path,
        "environment.xcode_version": xcode_version,
        "environment.xcode_build": xcode_build,
        "environment.sdk_name": sdk_name,
        "environment.sdk_path": sdk_path,
        "environment.sdk_version": sdk_version,
        "environment.sdk_build": sdk_build,
        "environment.clang_path": clang_path,
        "environment.clang_version": clang_version,
        "environment.metal_path": metal_path,
        "environment.metal_version": metal_version,
    }
    for key in ("os_name", "os_version", "os_build", "architecture"):
        expected = PROFILE_ROWS[f"profile.execution.{key}"]
        if values[f"environment.{key}"] != expected:
            raise ProbeError(
                f"current {key} does not match the authoritative execution row: "
                f"{values[f'environment.{key}']!r} != {expected!r}"
            )
    for key, expected in RETAINED_TOOL_ROWS.items():
        if key.startswith("environment.runtime_compiler."):
            continue
        if values[key] != expected:
            raise ProbeError(
                f"current {key} does not match the retained tool row: "
                f"{values[key]!r} != {expected!r}"
            )
    return values


def build_host(destination: Path) -> None:
    run(
        [
            "xcrun",
            "--sdk",
            "macosx",
            "clang",
            "-fobjc-arc",
            "-O0",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "Metal",
            "-framework",
            "Foundation",
            str(HOST_SOURCE),
            "-o",
            str(destination),
        ]
    )


def write_record(path: Path, rows: dict[str, str]) -> None:
    for key, value in rows.items():
        if not key or not value or any(character in key + value for character in "\t\r\n"):
            raise ProbeError(f"record field is not line-safe: {key!r}={value!r}")
    body = "".join(f"{key}\t{rows[key]}\n" for key in sorted(rows))
    path.write_text(body, encoding="utf-8")


def measure(result_dir: Path) -> None:
    if result_dir.exists():
        raise ProbeError(f"result directory already exists: {result_dir}")
    result_dir.parent.mkdir(parents=True, exist_ok=True)
    env = environment()
    with tempfile.TemporaryDirectory(prefix="tiler-exp-zero-work-") as scratch_name:
        scratch = Path(scratch_name)
        shutil.copyfile(KERNEL, scratch / "exp_at_zero.metal")
        (scratch / "manifest.tsv").write_text(MANIFEST, encoding="utf-8")
        build_host(scratch / "numerical_probe_host")
        completed = subprocess.run(
            [
                str(scratch / "numerical_probe_host"),
                "batch",
                "manifest.tsv",
                INVOCATION.strip(),
            ],
            cwd=scratch,
            capture_output=True,
            check=False,
        )
        if completed.returncode != 0:
            detail = completed.stderr.decode("utf-8", errors="replace").strip()
            raise ProbeError(f"device dispatch failed ({completed.returncode}): {detail}")
        host_output = completed.stdout
        observed = parse_host_output(host_output)
        scalars = observed["scalars"]
        results = observed["results"]
        images = observed["images"]
        assert isinstance(scalars, dict) and isinstance(results, tuple) and isinstance(images, tuple)
        archive = (scratch / "runtime.metalar").read_bytes()
        compiler = ARCHIVE_COMPILER.search(archive)
        if compiler is None:
            raise ProbeError("runtime binary archive carries no attributable compiler version")
        runtime_compiler = compiler.group(0) + b"\n"

        rows = {
            "schema": SCHEMA,
            "probe.date_utc": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
            "probe.repository_base_revision": run(["git", "rev-parse", "HEAD"], cwd=REPOSITORY),
            "probe.harness_sha256": digest(HARNESS.read_bytes()),
            "probe.kernel_sha256": digest(KERNEL.read_bytes()),
            "probe.host_source_sha256": digest(HOST_SOURCE.read_bytes()),
            "probe.kernel_function": KERNEL_FUNCTION,
            "probe.kernel_expression": KERNEL_EXPRESSION,
            "probe.compile_options": COMPILE_OPTIONS,
            "measurement.route": "MTLDevice.newLibraryWithSource",
            "measurement.scope": "runtime-compiler-only-not-production-aot",
            "measurement.input.count": str(len(INPUTS)),
            "measurement.result.count": str(len(results)),
            "measurement.exact_at_zero": "true" if results == (EXACT_ONE, EXACT_ONE) else "false",
            **PROFILE_ROWS,
            **env,
            "environment.device": str(scalars["device"]),
            "environment.device_registry_id": str(scalars["registry-id"]),
            "environment.gpu_family_apple9": str(scalars["gpu-family-apple9"]),
            "environment.runtime_compiler.images": " | ".join(str(image) for image in images),
            "environment.runtime_compiler.version": compiler.group(0).decode("ascii"),
            "environment.runtime_compiler.version_source": "serialized-MTLBinaryArchive",
            "producer.manifest_sha256": digest(MANIFEST.encode("utf-8")),
            "producer.invocation_sha256": digest(INVOCATION.encode("ascii")),
            "producer.host_output_sha256": digest(host_output),
            "producer.runtime_compiler_sha256": digest(runtime_compiler),
        }
        for index, bits in enumerate(INPUTS):
            rows[f"measurement.input.{index}"] = bits
        for index, bits in enumerate(results):
            rows[f"measurement.result.{index}"] = bits

        stage = Path(tempfile.mkdtemp(prefix=f".{result_dir.name}.", dir=result_dir.parent))
        try:
            (stage / "manifest.tsv").write_text(MANIFEST, encoding="utf-8")
            (stage / "invocation.txt").write_text(INVOCATION, encoding="ascii")
            (stage / "host-output.txt").write_bytes(host_output)
            (stage / "runtime-compiler.txt").write_bytes(runtime_compiler)
            write_record(stage / "record.tsv", rows)
            validate_record(stage / "record.tsv")
            os.replace(stage, result_dir)
        except BaseException:
            shutil.rmtree(stage, ignore_errors=True)
            raise


def demonstrate_failures(record_path: Path) -> None:
    rows = parse_record(record_path)
    perturbations: list[tuple[str, dict[str, str] | None, bytes | None]] = []
    perturbations.append(("kernel", None, KERNEL.read_bytes() + b"// subject perturbation\n"))
    input_rows = dict(rows)
    input_rows["measurement.input.1"] = "00000000"
    perturbations.append(("input", input_rows, None))
    result_rows = dict(rows)
    result_rows["measurement.result.0"] = "3f800001"
    perturbations.append(("result", result_rows, None))
    version_source_rows = dict(rows)
    version_source_rows["environment.runtime_compiler.version_source"] = "offline-metal"
    perturbations.append(("version-source", version_source_rows, None))
    xcode_version_rows = dict(rows)
    xcode_version_rows["environment.xcode_version"] = "0.0"
    perturbations.append(("xcode-version", xcode_version_rows, None))
    for name, mutated_rows, mutated_kernel in perturbations:
        try:
            validate_record(
                record_path,
                rows_override=mutated_rows,
                kernel_bytes_override=mutated_kernel,
            )
        except ProbeError as error:
            print(f"{name} perturbation rejected: {error}")
        else:
            raise ProbeError(f"{name} perturbation unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--result-dir", type=Path)
    action.add_argument("--validate", type=Path, metavar="RECORD")
    action.add_argument("--demonstrate-failures", type=Path, metavar="RECORD")
    arguments = parser.parse_args()
    try:
        if arguments.result_dir is not None:
            measure(arguments.result_dir.resolve())
            print(arguments.result_dir.resolve() / "record.tsv")
        elif arguments.validate is not None:
            validate_record(arguments.validate.resolve())
            print(f"validated: {arguments.validate.resolve()}")
        else:
            demonstrate_failures(arguments.demonstrate_failures.resolve())
    except (OSError, ProbeError) as error:
        print(f"exp-at-zero probe: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
