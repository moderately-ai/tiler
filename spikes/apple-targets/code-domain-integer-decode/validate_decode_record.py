#!/usr/bin/env python3
"""Validate one retained code-domain integer decode record.

Every population this checks is *derived* from the committed producer
definitions rather than listed here, so a record that dropped a scale, a case, a
witness, or a comparison fails on the population rather than passing quietly with
fewer rows. The two reference grids are recomputed from the producer's own exact
evaluation and compared by digest, so a rewritten `reference.*` row is rejected
by arithmetic rather than trusted.

Exit codes: 0 valid, 2 invalid, 3 usage.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[2]
SHA256 = re.compile(r"[0-9a-f]{64}")
REVISION = re.compile(r"[0-9a-f]{40}")
PATTERN = re.compile(r"[0-9a-f]{8}")

ENVIRONMENT_FIELDS = (
    "date_utc",
    "os_version",
    "os_build",
    "machine",
    "xcode",
    "metal_platform",
    "sdk",
    "sdk_version",
    "sdk_build",
    "requested_target",
    "metal_version",
    "metallib_version",
    "execution",
    "emitted_triple",
    "device",
    "device_registry_id",
    "device_apple9_support",
    "runtime_compiler_images",
    "runtime_compiler_build",
)
"""Every environment row a published measurement is qualified by.

None may be absent and none may be empty: a record naming no device, no SDK
build, or no runtime compiler is a measurement whose applicability cannot be
stated, which is worse than no record.
"""


class RecordError(ValueError):
    """The retained record or one of its inputs is incomplete or inconsistent."""


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def producer():
    """Load the producer definitions the retained populations must exactly match."""
    path = HERE / "decode_probe.py"
    spec = importlib.util.spec_from_file_location("_validated_decode_probe", path)
    if spec is None or spec.loader is None:
        raise RecordError("could not load the decode producer definitions")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def read_rows(path: Path) -> dict[str, str]:
    """Read a unique-key TSV file, requiring a nonempty key and value on every line."""
    rows: dict[str, str] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        key, separator, value = line.partition("\t")
        if not separator or not key or not value:
            raise RecordError(f"{path}:{number}: expected a nonempty key/value row")
        if key in rows:
            raise RecordError(f"{path}:{number}: duplicate key {key}")
        rows[key] = value
    return rows


def require(
    rows: dict[str, str], key: str, expected: str | re.Pattern[str] | None = None
) -> str:
    if key not in rows:
        raise RecordError(f"missing required field: {key}")
    value = rows[key]
    if isinstance(expected, str) and value != expected:
        raise RecordError(f"{key} is {value!r}, expected {expected!r}")
    if isinstance(expected, re.Pattern) and expected.fullmatch(value) is None:
        raise RecordError(f"{key} is malformed: {value!r}")
    return value


def validate_provenance(rows: dict[str, str]) -> None:
    """Check the recorded revision resolves and the three executable producers match."""
    revision = require(rows, "probe.repository_base_revision", REVISION)
    resolved = subprocess.run(
        ["git", "-C", str(REPOSITORY), "cat-file", "-t", revision],
        check=False,
        capture_output=True,
        text=True,
    )
    if resolved.returncode != 0 or resolved.stdout.strip() != "commit":
        raise RecordError(f"recorded revision {revision} does not resolve to a commit")
    for key, path in (
        ("probe.harness_sha256", HERE / "decode_probe.py"),
        ("probe.host_source_sha256", HERE / "decode_probe_host.m"),
        ("probe.validator_sha256", Path(__file__).resolve()),
    ):
        recorded = require(rows, key, SHA256)
        if recorded != digest(path):
            raise RecordError(f"{key} does not match the checked-in {path.name}")


def validate_manifest(record: Path, rows: dict[str, str], probe) -> None:
    """Validate the retained producer inputs and the canonical generated source."""
    manifest_name = require(rows, "probe.input_manifest_file", "input-manifest.tsv")
    manifest = record.parent / manifest_name
    if not manifest.is_file():
        raise RecordError(f"retained input manifest is missing: {manifest}")
    if digest(manifest) != require(rows, "probe.input_manifest_sha256", SHA256):
        raise RecordError("retained input manifest digest mismatch")
    inputs = read_rows(manifest)
    require(inputs, "schema", probe.MANIFEST_SCHEMA)
    require(inputs, "profile", probe.PROFILE)
    require(inputs, "msl_version", probe.MSL_VERSION)
    require(inputs, "runtime_language", probe.RUNTIME_LANGUAGE)

    expected_inputs = {
        f"input.{path.relative_to(REPOSITORY)}": digest(path)
        for path in (
            HERE / "decode_probe.py",
            HERE / "decode_probe_host.m",
            Path(__file__).resolve(),
        )
    }
    kernel = record.parent / "sources" / "decode_strict_affine_u8.metal"
    if not kernel.is_file():
        raise RecordError(f"retained kernel source is missing: {kernel}")
    if kernel.read_text(encoding="utf-8") != probe.kernel_source():
        raise RecordError("retained kernel source is not the producer's canonical bytes")
    expected_sources = {f"source.sources/{kernel.name}": digest(kernel)}

    named = {key: value for key, value in inputs.items() if key.startswith(("input.", "source."))}
    if named != {**expected_inputs, **expected_sources}:
        raise RecordError(
            "retained input manifest does not match the producer inputs and sources"
        )
    retained = sorted(entry.name for entry in (record.parent / "sources").iterdir())
    if retained != [kernel.name]:
        raise RecordError(f"sources/ holds {retained}, expected exactly [{kernel.name!r}]")


def validate_scales(rows: dict[str, str], probe) -> None:
    """Every scale is named once, classified, and carries its exact value."""
    named = require(rows, "probe.scales").split()
    expected = [scale.name for scale in probe.SCALES]
    if named != expected:
        raise RecordError(f"probe.scales is {named}, expected {expected}")
    subnormal = 0
    for scale in probe.SCALES:
        bits = require(rows, f"probe.scale.{scale.name}.bits", PATTERN)
        if int(bits, 16) != scale.bits:
            raise RecordError(f"probe.scale.{scale.name}.bits is {bits}, not the producer's")
        require(rows, f"probe.scale.{scale.name}.exact", scale.hexadecimal())
        require(rows, f"probe.scale.{scale.name}.class", scale.classification)
        require(rows, f"probe.scale.{scale.name}.role", scale.role)
        if not scale.normal:
            subnormal += 1
    # The ticket's inputs require at least one deliberately subnormal scale and
    # the normal/subnormal boundary itself. A corpus that lost either would still
    # produce a complete-looking record whose subnormal question was never asked.
    if subnormal < 1:
        raise RecordError("the scale corpus carries no subnormal scale")
    if not any(scale.bits == 0x00800000 for scale in probe.SCALES):
        raise RecordError("the scale corpus omits the f32 minimum normal")


def validate_references(rows: dict[str, str], probe, referenced: dict) -> None:
    """Recompute both reference grids and hold the retained rows to them."""
    for scale in probe.SCALES:
        entry = referenced[scale.name]
        prefix = f"reference.{scale.name}"
        if require(rows, f"{prefix}.exact_sha256", SHA256) != probe.grid_digest(entry.exact):
            raise RecordError(f"{prefix}.exact_sha256 does not match the recomputed grid")
        if require(rows, f"{prefix}.flush_sha256", SHA256) != probe.grid_digest(entry.flushed):
            raise RecordError(f"{prefix}.flush_sha256 does not match the recomputed grid")
        require(rows, f"{prefix}.models_differ", str(len(entry.differing_cells)))
        require(
            rows,
            f"{prefix}.exact_subnormal_results",
            str(sum(1 for bits in entry.exact if probe.is_subnormal(bits))),
        )
        require(rows, f"{prefix}.derivation_predicts", entry.predicted.value)
        # A normal scale makes the two models identical and a subnormal one makes
        # them differ in every cell whose code differs from its zero point. Both
        # are finite consequences of the corpus, so a record claiming otherwise is
        # rejected here rather than read as a surprising measurement.
        expected_differ = 0 if scale.normal else probe.GRID_CELLS - (probe.CODE_MAX + 1)
        if len(entry.differing_cells) != expected_differ:
            raise RecordError(
                f"{prefix} models differ in {len(entry.differing_cells)} cells, "
                f"expected {expected_differ}"
            )


def validate_cases(rows: dict[str, str], probe, referenced: dict) -> None:
    """Hold the case population, its rows, and its internal consistency."""
    expected_keys = [case.key for case in probe.cases()]
    # Keyed on one row every case must have, rather than on parsing a case key
    # back out of a field name: a `witness.<name>` field carries two segments and
    # a population derived by splitting would silently admit it as a case.
    present = {key for key in expected_keys if f"case.{key}.cells" in rows}
    if present != set(expected_keys):
        raise RecordError(
            f"the record carries {len(present)} of {len(expected_keys)} cases"
        )
    unexpected = sorted(
        key
        for key in rows
        if key.startswith("case.")
        and not any(key.startswith(f"case.{expected}.") for expected in expected_keys)
    )
    if unexpected:
        raise RecordError(f"unexpected case rows: {unexpected}")

    for case in probe.cases():
        entry = referenced[case.scale]
        prefix = f"case.{case.key}"
        require(rows, f"{prefix}.cells", str(probe.GRID_CELLS))
        require(rows, f"{prefix}.returned_sha256", SHA256)
        distinct = int(require(rows, f"{prefix}.distinct_returned"))
        if not 1 <= distinct <= probe.GRID_CELLS:
            raise RecordError(f"{prefix}.distinct_returned is out of range: {distinct}")
        exact_matches = int(require(rows, f"{prefix}.exact_matches"))
        flush_matches = int(require(rows, f"{prefix}.flush_matches"))
        for name, value in (("exact_matches", exact_matches), ("flush_matches", flush_matches)):
            if not 0 <= value <= probe.GRID_CELLS:
                raise RecordError(f"{prefix}.{name} is out of range: {value}")
        recorded = require(rows, f"{prefix}.verdict")
        derived = _verdict_of(probe, exact_matches, flush_matches)
        if recorded != derived:
            raise RecordError(
                f"{prefix}.verdict is {recorded!r} but its match counts derive {derived!r}"
            )
        agrees = require(rows, f"{prefix}.agrees_with_derivation")
        expected_agreement = "yes" if recorded == entry.predicted.value else "no"
        if agrees != expected_agreement:
            raise RecordError(
                f"{prefix}.agrees_with_derivation is {agrees!r}, expected {expected_agreement!r}"
            )
        diagonal = require(rows, f"{prefix}.code_equals_zero_point_positive_zero")
        if not re.fullmatch(rf"\d+/{probe.CODE_MAX + 1}", diagonal):
            raise RecordError(f"{prefix}.code_equals_zero_point_positive_zero is {diagonal!r}")
        for witness in probe.WITNESSES:
            value = require(rows, f"{prefix}.witness.{witness.name}")
            found = re.fullmatch(
                r"returned=([0-9a-f]{8}),exact=([0-9a-f]{8}),flush=([0-9a-f]{8})", value
            )
            if found is None:
                raise RecordError(f"{prefix}.witness.{witness.name} is malformed: {value!r}")
            if int(found.group(2), 16) != entry.exact[witness.cell]:
                raise RecordError(
                    f"{prefix}.witness.{witness.name} states an exact value the reference "
                    "does not"
                )
            if int(found.group(3), 16) != entry.flushed[witness.cell]:
                raise RecordError(
                    f"{prefix}.witness.{witness.name} states a flush value the reference "
                    "does not"
                )
        if case.path is probe.Compilation.OFFLINE:
            require(rows, f"{prefix}.compile_options")
            require(rows, f"{prefix}.emitted_operations")
            if f"{prefix}.applied" in rows:
                raise RecordError(f"{prefix} is an offline case and carries an applied row")
        else:
            require(
                rows,
                f"{prefix}.applied",
                f"math={probe.MATH_MODE},fpfun={probe.FP32_FUNCTIONS},"
                f"lang={probe.RUNTIME_LANGUAGE},opt={case.level}",
            )
            for absent in ("compile_options", "emitted_operations"):
                if f"{prefix}.{absent}" in rows:
                    raise RecordError(
                        f"{prefix} is a runtime case; the runtime path returns an opaque "
                        f"MTLLibrary and can carry no {absent} row"
                    )
        named = [key for key in rows if key.startswith(f"divergence.{case.key}.")]
        if recorded == probe.Verdict.DIVERGENT.value and not named:
            raise RecordError(f"{prefix} is divergent and names no diverging cell")
        if recorded != probe.Verdict.DIVERGENT.value and named:
            raise RecordError(f"{prefix} is not divergent and names {len(named)} diverging cells")


def _verdict_of(probe, exact_matches: int, flush_matches: int) -> str:
    if exact_matches == probe.GRID_CELLS and flush_matches == probe.GRID_CELLS:
        return probe.Verdict.BOTH_MODELS_AGREE.value
    if exact_matches == probe.GRID_CELLS:
        return probe.Verdict.EXACT_WHERE_MODELS_DIFFER.value
    if flush_matches == probe.GRID_CELLS:
        return probe.Verdict.FLUSH_WHERE_MODELS_DIFFER.value
    return probe.Verdict.DIVERGENT.value


def validate_comparisons(rows: dict[str, str], probe) -> None:
    """Every runtime case is compared against its paired offline row, and none is missing."""
    expected = {
        f"comparison.{level}.{scale.name}"
        for level in probe.RUNTIME_OPTIMIZATIONS
        for scale in probe.SCALES
    }
    present = {key for key in rows if key.startswith("comparison.")}
    if present != expected:
        raise RecordError(
            f"the record carries {len(present)} comparisons, expected {len(expected)}"
        )
    for key in sorted(expected):
        value = rows[key]
        if value != "agree" and not re.fullmatch(r"differ:\d+-cells", value):
            raise RecordError(f"{key} is malformed: {value!r}")


def validate(record: Path) -> None:
    probe = producer()
    rows = read_rows(record)
    require(rows, "schema", probe.SCHEMA)
    require(rows, "probe.profile", probe.PROFILE)
    require(rows, "probe.family", "macos")
    require(rows, "probe.required_gpu_family", probe.REQUIRED_GPU_FAMILY)
    require(rows, "probe.entry_point", probe.ENTRY_POINT)
    require(rows, "probe.code_type", "u8")
    require(rows, "probe.code_domain", f"{probe.CODE_MIN}..{probe.CODE_MAX}")
    require(rows, "probe.zero_point_domain", f"{probe.CODE_MIN}..{probe.CODE_MAX}")
    require(rows, "probe.grid_cells", str(probe.GRID_CELLS))
    require(rows, "probe.grid_order", probe.GRID_ORDER)
    require(rows, "probe.sentinel", f"{probe.SENTINEL:08x}")
    require(rows, "probe.sentinel_reachable", "no")
    require(rows, "probe.runtime_target_contract", "execution-environment-no-target-property")
    require(rows, "probe.population.cases", str(len(probe.cases())))
    require(
        rows,
        "probe.population.dispatched_cells",
        str(len(probe.cases()) * probe.GRID_CELLS),
    )
    require(
        rows,
        "probe.population.comparisons",
        str(len(probe.RUNTIME_OPTIMIZATIONS) * len(probe.SCALES)),
    )
    for field in ENVIRONMENT_FIELDS:
        require(rows, f"environment.{field}")
    if require(rows, "environment.device_apple9_support") != "supported":
        raise RecordError("the record was not taken on a device reporting Apple9 support")

    validate_provenance(rows)
    validate_scales(rows, probe)
    referenced = probe.references()
    # The sentinel claim is a positive property of the finite corpus, so it is
    # rechecked here rather than inherited from the producer that wrote the row.
    for name, entry in referenced.items():
        if probe.SENTINEL in entry.exact or probe.SENTINEL in entry.flushed:
            raise RecordError(f"the seeded sentinel is a reachable value for scale {name}")
    validate_references(rows, probe, referenced)
    validate_cases(rows, probe, referenced)
    validate_comparisons(rows, probe)
    validate_manifest(record, rows, probe)
    require(rows, "probe.status", "validated")


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("record", type=Path)
    parsed = parser.parse_args(arguments)
    if not parsed.record.is_file():
        print(f"no such record: {parsed.record}", file=sys.stderr)
        return 3
    try:
        validate(parsed.record)
    except RecordError as invalid:
        print(f"invalid record: {invalid}", file=sys.stderr)
        return 2
    print(f"validated {parsed.record}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
