#!/usr/bin/env python3
"""Validate one retained named-profile Apple numerical record.

Every profile-dependent expectation is read from the producer's own `Profile`
object, resolved through the record's `probe.profile` row, rather than pinned
here as a constant. That is what lets one validator certify more than one row
without a second copy of these checks drifting from this one — and it is
strictly narrower than a constant would be, because a record is now held to the
identity of the profile *it names* instead of to whichever one this file
happened to be written for.

**Recovering the exact validator a retained record names.** Every record binds
its producers by digest — `probe.harness_sha256`, `probe.host_source_sha256`,
and `probe.validator_sha256` — and `validate_record` checks the *current tree's*
bytes against them, so a record produced before any producer edit is revalidated
from the revision it pins rather than from the working tree:

    git -C <repo> show <probe.repository_base_revision>:spikes/apple-targets/validate_numerical_record.py

`validate_revision_identity` is what makes that recovery sound: it holds the
recorded revision to resolving blobs whose digests are the ones the record
carries, so the pinned revision cannot name a producer set it did not run.
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
REPOSITORY = HERE.parents[1]
MANIFEST_SCHEMA = "tiler.apple-numerical-input-manifest/v1"
SHA256 = re.compile(r"[0-9a-f]{64}")
MACOS_TARGET = re.compile(r"air64-apple-macos([0-9]+)[.]([0-9]+)")


class RecordError(ValueError):
    """The retained record or one of its inputs is incomplete or inconsistent."""


def results_pattern(digits: int) -> re.Pattern[str]:
    """The exact shape a `results` row takes at one dtype's own rendered width.

    Pinned per dtype rather than at `f32`'s eight digits, because a record
    carrying a narrow dtype renders four. Two dtypes now share a width, so this
    checks the shape of a row and never the identity of its format; only the
    kernel named in the case key decides that, and the caller resolves the width
    from that kernel's own dtype.
    """
    return re.compile(rf"[0-9a-f]{{{digits}}}(?: [0-9a-f]{{{digits}}})+")


def resolve_profile(rows: dict[str, str], probe):
    """Resolve the producer profile one record names, refusing an unnamed or legacy row.

    The legacy profile is refused by name rather than by omission: it writes
    `probe.status complete` and carries none of the retained-input rows this
    validator checks, so accepting it here would report a weaker record as a
    validated one.
    """
    name = require(rows, "probe.profile")
    profile = probe.PROFILES.get(name)
    if profile is None:
        raise RecordError(f"probe.profile names no producer profile: {name}")
    if profile is probe.LEGACY_PROFILE:
        raise RecordError("the legacy profile does not produce a validated result directory")
    return profile


def digest(path: Path) -> str:
    """Return the SHA-256 identity of one retained byte string."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def producer():
    """Load the producer definitions the retained population must exactly match."""
    path = HERE / "numerical_probe.py"
    spec = importlib.util.spec_from_file_location("_validated_numerical_probe", path)
    if spec is None or spec.loader is None:
        raise RecordError("could not load numerical producer definitions")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def read_rows(path: Path, *, require_value: bool = True) -> dict[str, str]:
    """Read a unique-key TSV file, optionally requiring nonempty values."""
    rows: dict[str, str] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        key, separator, value = line.partition("\t")
        if not separator or not key or (require_value and not value):
            raise RecordError(f"{path}:{number}: expected a nonempty key/value row")
        if key in rows:
            raise RecordError(f"{path}:{number}: duplicate key {key}")
        rows[key] = value
    return rows


def require(
    rows: dict[str, str], key: str, expected: str | re.Pattern[str] | None = None
) -> str:
    """Return a required field after checking an exact value or full pattern."""
    if key not in rows:
        raise RecordError(f"missing required field: {key}")
    value = rows[key]
    if isinstance(expected, str) and value != expected:
        raise RecordError(f"{key} is {value!r}, expected {expected!r}")
    if isinstance(expected, re.Pattern) and expected.fullmatch(value) is None:
        raise RecordError(f"{key} is malformed: {value!r}")
    return value


def validate_manifest(record: Path, rows: dict[str, str], probe) -> None:
    """Validate the retained producer inputs and canonical generated sources."""
    profile = resolve_profile(rows, probe)
    manifest_name = require(rows, "probe.input_manifest_file", "input-manifest.tsv")
    manifest = record.parent / manifest_name
    if digest(manifest) != require(rows, "probe.input_manifest_sha256", SHA256):
        raise RecordError("retained input manifest digest mismatch")
    inputs = read_rows(manifest)
    require(inputs, "schema", MANIFEST_SCHEMA)
    require(inputs, "profile", profile.name)
    require(inputs, "msl_version", profile.msl_version)
    require(inputs, "runtime_language", profile.runtime_language)
    expected_inputs = {
        "spikes/apple-targets/numerical_probe.py",
        "spikes/apple-targets/numerical_probe_host.m",
        "spikes/apple-targets/validate_numerical_record.py",
    }
    found_inputs = {
        key.removeprefix("input.")
        for key in inputs
        if key.startswith("input.")
    }
    if found_inputs != expected_inputs:
        raise RecordError("manifest producer inputs are incomplete or contain extras")
    for relative in expected_inputs:
        if digest(REPOSITORY / relative) != require(inputs, f"input.{relative}", SHA256):
            raise RecordError(f"producer input digest mismatch: {relative}")
    source_rows = {
        key.removeprefix("source."): value
        for key, value in inputs.items()
        if key.startswith("source.")
    }
    expected_sources = {
        f"sources/{name}.metal"
        for family in profile.families
        for name in {case.kernel for case in probe.cases(family.name, rows["probe.matrix"], profile)}
    }
    if set(source_rows) != expected_sources:
        raise RecordError("manifest source inventory does not match the profile case population")
    for relative, expected in source_rows.items():
        source = (record.parent / relative).resolve()
        if source.parent != (record.parent / "sources").resolve():
            raise RecordError(f"source escapes the retained sources directory: {relative}")
        if source.suffix != ".metal" or digest(source) != expected:
            raise RecordError(f"retained source digest mismatch: {relative}")
        kernel_name = source.stem
        if source.read_text(encoding="utf-8") != probe.BY_NAME[kernel_name].source():
            raise RecordError(f"retained source is not the canonical producer output: {relative}")


def emitted_triple_pattern(family) -> re.Pattern[str]:
    """The emitted triple one family's requested target must have produced.

    Derived from the family's own `-target` rather than pinned, because the two
    are different values and only the requested one is a producer input: the
    front end raises the deployment floor and rewrites the platform spelling, so
    `air64-apple-macos26.0` is emitted as `air64_v28-apple-macosx26.0.0`. What is
    checked is the OS version surviving that rewrite, which is the part a wrong
    target would move.

    The leading component is deliberately unconstrained: `air64_v28` names the
    AIR version this toolchain build emits, which is a property of the compiler
    rather than of the requested target, and pinning it would fail a record on
    the next toolchain for a reason that is not a defect.
    """
    matched = MACOS_TARGET.fullmatch(family.target)
    if matched is None:
        raise RecordError(f"no emitted-triple rule for requested target: {family.target}")
    major, minor = matched.group(1), matched.group(2)
    return re.compile(rf".*macosx{re.escape(major)}[.]{re.escape(minor)}[.]0")


def revision_blob(revision: str, relative: str) -> bytes:
    """Read one producer blob at the exact committed revision, refusing ambiguity."""
    result = subprocess.run(
        ["git", "-C", str(REPOSITORY), "show", f"{revision}:{relative}"],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise RecordError(f"producer revision does not resolve {relative}")
    return result.stdout


def revision_object_type(revision: str) -> str:
    """Resolve the exact Git object type named by a recorded revision."""
    result = subprocess.run(
        ["git", "-C", str(REPOSITORY), "cat-file", "-t", revision],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RecordError("producer revision does not resolve to a Git object")
    return result.stdout.strip()


def validate_revision_identity(rows: dict[str, str]) -> None:
    """Bind every executable producer input to one real committed revision."""
    revision = require(rows, "probe.repository_base_revision", re.compile(r"[0-9a-f]{40}"))
    if revision == "0" * 40:
        raise RecordError("producer revision is the all-zero sentinel")
    object_type = revision_object_type(revision)
    if object_type != "commit":
        raise RecordError(f"producer revision resolves to {object_type!r}, not a commit")
    committed_inputs = {
        "spikes/apple-targets/numerical_probe.py": rows["probe.harness_sha256"],
        "spikes/apple-targets/numerical_probe_host.m": rows["probe.host_source_sha256"],
        "spikes/apple-targets/validate_numerical_record.py": rows["probe.validator_sha256"],
    }
    for relative, expected in committed_inputs.items():
        if hashlib.sha256(revision_blob(revision, relative)).hexdigest() != expected:
            raise RecordError(f"producer revision blob mismatch: {relative}")


def validate_population(rows: dict[str, str], probe) -> None:
    """Require the exact producer-defined cases, rows, witnesses, and comparisons."""
    profile = resolve_profile(rows, probe)
    selection = rows["probe.matrix"]
    offline = tuple(
        case
        for family in profile.families
        for case in probe.cases(family.name, selection, profile)
    )
    runtime = tuple(
        case
        for family in profile.families
        for case in probe.runtime_cases(family.name, selection, profile)
    )
    expected_cases = {case.key: case for case in (*offline, *runtime)}
    fields: dict[str, set[str]] = {
        case.key: (
            {"compile_options", "float_operations", "results", "execution_witness"}
            if not case.is_runtime
            else {"applied_options", "archived_options", "results", "execution_witness"}
        )
        for case in (*offline, *runtime)
    }
    observed_fields: dict[str, set[str]] = {}
    for key in rows:
        if not key.startswith("case."):
            continue
        body = key.removeprefix("case.")
        case_key, separator, field = body.rpartition(".")
        if not separator:
            raise RecordError(f"malformed case row: {key}")
        observed_fields.setdefault(case_key, set()).add(field)
    if set(observed_fields) != set(expected_cases):
        raise RecordError("case population does not match the selected profile matrix")
    for case_key, expected_fields in fields.items():
        if observed_fields[case_key] != expected_fields:
            raise RecordError(f"{case_key} has an incomplete or extra row family")
        case = expected_cases[case_key]
        kernel = probe.BY_NAME[case.kernel]
        patterns = require(
            rows, f"case.{case_key}.results", results_pattern(kernel.dtype.digits)
        ).split()
        if len(patterns) != len(kernel.dtype.operands):
            raise RecordError(f"{case_key} returned the wrong operand population")
        values = tuple(int(pattern, 16) for pattern in patterns)
        witness = kernel.witness
        if witness is None:
            expected_witness = "none"
        else:
            index = kernel.dtype.operands.index(witness.operand)
            observed = values[index]
            status = probe.witness_status(witness, observed)
            expected_witness = (
                f"operand={kernel.dtype.render(witness.operand)},"
                f"expected={kernel.dtype.render(witness.executed)},"
                f"observed={kernel.dtype.render(observed)},status={status.value}"
            )
        require(rows, f"case.{case_key}.execution_witness", expected_witness)
        if witness is not None and status is probe.WitnessStatus.DISAGREES:
            raise RecordError(f"{case_key} execution witness disagrees with both controls")
        if case.is_runtime:
            require(
                rows,
                f"case.{case_key}.applied_options",
                profile.runtime_options(case.configuration),
            )
    result_by_case = {
        case_key: tuple(
            int(value, 16)
            for value in rows[f"case.{case_key}.results"].split()
        )
        for case_key in expected_cases
    }
    expected_comparisons: dict[str, str] = {}
    for case in runtime:
        configuration = case.configuration
        candidates = tuple(sorted(
            candidate.key
            for candidate in offline
            if candidate.kernel == case.kernel
            and candidate.family == case.family
            and candidate.configuration.math_mode == configuration.math_mode
            and candidate.configuration.fp32_functions == configuration.fp32_functions
            and candidate.configuration.optimization == probe.RUNTIME_PAIRED_OPTIMIZATION
        ))
        matched = tuple(
            candidate for candidate in candidates
            if result_by_case[candidate] == result_by_case[case.key]
        )
        if not matched:
            raise RecordError(f"{case.key} disagrees with every paired offline case")
        expected_comparisons[f"comparison.{case.key}"] = probe.PathComparison(
            case.key,
            candidates,
            matched,
            result_by_case[case.key],
            probe.BY_NAME[case.kernel].dtype,
        ).render()
    observed_comparisons = {
        key: value for key, value in rows.items() if key.startswith("comparison.")
    }
    if observed_comparisons != expected_comparisons:
        raise RecordError("comparison population or linkage does not match producer definitions")


def validate_record(record: Path) -> None:
    """Validate profile identity, provenance, dispatch rows, and path agreement."""
    lines = record.read_text(encoding="utf-8").splitlines()
    if not lines or lines[-1] != "probe.status\tvalidated":
        raise RecordError("record does not end with probe.status=validated")
    rows = read_rows(record, require_value=False)
    probe = producer()
    profile = resolve_profile(rows, probe)
    require(rows, "schema", profile.schema)
    require(rows, "probe.families", " ".join(family.name for family in profile.families))
    require(rows, "probe.dtypes", " ".join(dtype.name for dtype in profile.dtypes))
    require(rows, "probe.fixed_flags", f"-std={profile.msl_version}")
    require(rows, "probe.runtime_fixed_options", f"lang={profile.runtime_language}")
    require(
        rows,
        "probe.required_gpu_family",
        profile.required_gpu_family.value if profile.required_gpu_family else "none",
    )
    require(rows, "probe.runtime_target_contract", "execution-environment-no-target-property")
    require(rows, "probe.validator_sha256", SHA256)
    if digest(Path(__file__).resolve()) != rows["probe.validator_sha256"]:
        raise RecordError("validator digest mismatch")
    require(rows, "probe.repository_base_revision", re.compile(r"[0-9a-f]{40}"))
    require(rows, "probe.harness_sha256", SHA256)
    require(rows, "probe.host_source_sha256", SHA256)
    if digest(HERE / "numerical_probe.py") != rows["probe.harness_sha256"]:
        raise RecordError("numerical harness digest mismatch")
    if digest(HERE / "numerical_probe_host.m") != rows["probe.host_source_sha256"]:
        raise RecordError("dispatch host digest mismatch")
    validate_revision_identity(rows)
    require(rows, "probe.matrix", re.compile(r"covering|exhaustive"))
    require(rows, "environment.machine", "arm64")
    for family in profile.families:
        prefix = f"environment.family.{family.name}"
        require(rows, f"{prefix}.requested_target", family.target)
        if profile.required_gpu_family is probe.GpuFamily.APPLE9:
            require(rows, f"{prefix}.device_apple9_support", "supported")
        require(rows, f"{prefix}.device")
        require(rows, f"{prefix}.device_registry_id")
        require(rows, f"{prefix}.metal_version")
        require(rows, f"{prefix}.metallib_version")
        require(rows, f"{prefix}.runtime_compiler_build")
        require(rows, f"{prefix}.emitted_triple", emitted_triple_pattern(family))
        # A profile that measures `bf16` asked its device whether it would run a
        # `bfloat` kernel at all, so the answer has to be a measured one. The
        # producer writes `unmeasured-by-profile` for a profile that never asked,
        # and accepting that word here would let a record claim the dispatchability
        # half of a `bf16` row it did not measure.
        support = require(rows, f"{prefix}.device_bfloat_support")
        if probe.BF16 in profile.dtypes and support == "unmeasured-by-profile":
            raise RecordError(
                f"{prefix}.device_bfloat_support is unmeasured, but the profile carries bf16"
            )
    validate_population(rows, probe)
    validate_manifest(record, rows, probe)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("record", type=Path)
    arguments = parser.parse_args()
    try:
        validate_record(arguments.record)
    except (OSError, UnicodeError, RecordError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
