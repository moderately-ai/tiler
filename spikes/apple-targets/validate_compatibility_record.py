#!/usr/bin/env python3
"""Validate completeness and syntax of an Apple compatibility probe record."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

SCHEMA = "tiler.apple-target-compatibility/v1"
SDKS = ("macosx", "iphoneos", "iphonesimulator")
FAMILIES = {
    "macos13": ("macosx", "air64-apple-macos13.0"),
    "macos14": ("macosx", "air64-apple-macos14.0"),
    "ios16": ("iphoneos", "air64-apple-ios16.0"),
    "ios17": ("iphoneos", "air64-apple-ios17.0"),
    "iossim16": ("iphonesimulator", "air64-apple-ios16.0-simulator"),
    "iossim17": ("iphonesimulator", "air64-apple-ios17.0-simulator"),
}
SHA256 = re.compile(r"[0-9a-f]{64}")
VERSION = re.compile(r"[0-9]+(?:[.][0-9]+)+")


class RecordError(ValueError):
    """The probe record is incomplete, ambiguous, or malformed."""


def read_record(path: Path) -> dict[str, str]:
    """Read a unique-key TSV record without accepting malformed lines."""
    values: dict[str, str] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        key, separator, value = line.partition("\t")
        if not separator or not key or not value:
            raise RecordError(f"line {number} is not a nonempty key/value pair")
        if key in values:
            raise RecordError(f"duplicate key: {key}")
        values[key] = value
    return values


def require(values: dict[str, str], key: str, pattern: re.Pattern[str] | None = None) -> str:
    """Return one required field after optional full-match validation."""
    try:
        value = values[key]
    except KeyError as error:
        raise RecordError(f"missing required field: {key}") from error
    if pattern is not None and pattern.fullmatch(value) is None:
        raise RecordError(f"malformed field {key}: {value!r}")
    return value


def validate(values: dict[str, str]) -> None:
    """Validate producer provenance and the complete compile matrix."""
    if require(values, "schema") != SCHEMA:
        raise RecordError("unsupported record schema")
    require(values, "probe.result_root")
    require(values, "probe.source_sha256", SHA256)
    require(values, "probe.compiler_flags")
    require(values, "host.date_utc", re.compile(r"\d{4}-\d{2}-\d{2}T.+Z"))
    require(values, "host.developer_dir", re.compile(r"/.+"))
    require(values, "host.xcode", re.compile(r"Xcode .+ Build version .+"))
    require(
        values,
        "host.metal_toolchain_component",
        re.compile(r".*Build Version: [A-Za-z0-9]+.*Toolchain Identifier: .+"),
    )
    require(values, "host.xcrun", re.compile(r"xcrun version \d+[.]"))
    require(values, "host.os_version", VERSION)
    require(values, "host.os_build", re.compile(r"[A-Za-z0-9]+"))
    require(values, "host.machine", re.compile(r"arm64|x86_64"))

    for sdk in SDKS:
        require(values, f"sdk.{sdk}.path", re.compile(r"/.+[.]sdk"))
        require(values, f"sdk.{sdk}.version", VERSION)
        require(values, f"sdk.{sdk}.build", re.compile(r"[A-Za-z0-9]+"))
        require(values, f"sdk.{sdk}.settings_file", re.compile(r".+[.]settings[.]txt"))
        require(values, f"sdk.{sdk}.settings_sha256", SHA256)

    require(values, "tool.metal.path", re.compile(r"/.*/metal"))
    require(values, "tool.metal.version", re.compile(r"Apple metal version \d+(?:[.]\d+)+.*"))
    require(values, "tool.metal.sha256", SHA256)
    require(values, "tool.metallib.path", re.compile(r"/.*/metallib"))
    require(values, "tool.metallib.version", re.compile(r"AIR-LLD \d+(?:[.]\d+)+.*"))
    require(values, "tool.metallib.sha256", SHA256)

    for family, (sdk, target) in FAMILIES.items():
        for run in ("a", "b"):
            prefix = f"matrix.{family}.{run}"
            if require(values, f"{prefix}.sdk") != sdk:
                raise RecordError(f"wrong SDK for {prefix}")
            if require(values, f"{prefix}.target") != target:
                raise RecordError(f"wrong target for {prefix}")
            require(values, f"{prefix}.command.metal")
            require(values, f"{prefix}.command.metallib")
            require(values, f"{prefix}.air_sha256", SHA256)
            require(values, f"{prefix}.metallib_sha256", SHA256)
            require(values, f"{prefix}.log_sha256", SHA256)
        for artifact in ("air", "metallib"):
            require(
                values,
                f"repro.{family}.{artifact}.byte_identical",
                re.compile(r"true|false"),
            )


def file_digest(path: Path) -> str:
    """Hash one retained evidence file."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_retained_files(values: dict[str, str], record: Path) -> None:
    """Verify the retained SDK extracts and command logs against the record."""
    result_root = Path(require(values, "probe.result_root"))
    if not result_root.is_absolute():
        result_root = (record.parent / result_root).resolve()
    if result_root != record.parent.resolve():
        raise RecordError("probe.result_root does not identify the record directory")

    for sdk in SDKS:
        settings = Path(require(values, f"sdk.{sdk}.settings_file"))
        if not settings.is_absolute():
            settings = (result_root / settings).resolve()
        expected = require(values, f"sdk.{sdk}.settings_sha256", SHA256)
        if file_digest(settings) != expected:
            raise RecordError(f"retained SDK settings digest mismatch: {sdk}")

    for family in FAMILIES:
        for run in ("a", "b"):
            log = result_root / f"out-{run}" / f"{family}.log"
            expected = require(values, f"matrix.{family}.{run}.log_sha256", SHA256)
            if file_digest(log) != expected:
                raise RecordError(f"retained command log digest mismatch: {family}/{run}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("record", type=Path)
    args = parser.parse_args()
    try:
        values = read_record(args.record)
        validate(values)
        validate_retained_files(values, args.record)
    except (OSError, UnicodeError, RecordError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
