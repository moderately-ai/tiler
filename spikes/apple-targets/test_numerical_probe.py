#!/usr/bin/env python3
"""Re-establish, in the repository gate, the Apple numerical measurements ADR 0076 rests on.

Two classes of test live here and they fail on different hosts by design.

The **guard** tests are pure functions over synthetic observations. They run
everywhere, including a Linux runner with no Apple toolchain, and they pin the
one rule that separates this harness from one that reads bit patterns: an
observation whose arithmetic cannot be shown to have executed is never evidence
about arithmetic. That rule is the thing most worth protecting from a future
edit, so it must not be reachable only through a GPU. Its two failure modes —
a compilation path with no readable module, and an artifact family with no
attached device — are pinned there too, because a host with no Apple toolchain
at all is exactly where a future edit that defaults one of them would otherwise
go unnoticed.

The **measurement** tests dispatch on a GPU and are conditional. They resolve
the toolchain first and skip when none is present, exactly as
`crates/tiler-metal/src/golden_compilation.rs` does, so the gate stays green on
a host without Xcode. Two mechanisms keep a skip from reading as a pass: the
skip reason is announced on standard error and appears in pytest's `-ra` summary
under the `when_a_toolchain_and_gpu_resolve` name suffix, and setting
`TILER_REQUIRE_METAL_TOOLCHAIN` turns the skip into a failure.

A family whose own execution environment is absent is a third thing again, and
is neither a skip nor a failure. The compile-side tests still run for it; the
device-side ones announce the family and skip that family alone, so the record
never silently loses a family and never gains a device-side claim it did not
measure.
"""

from __future__ import annotations

import hashlib
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
_SPEC = importlib.util.spec_from_file_location("numerical_probe", HERE / "numerical_probe.py")
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError("could not load the Apple numerical probe harness")
PROBE = importlib.util.module_from_spec(_SPEC)
# `dataclasses` resolves a class's own module through `sys.modules`, so the
# harness must be registered before it executes or every frozen dataclass in it
# fails to build.
sys.modules[_SPEC.name] = PROBE
_SPEC.loader.exec_module(PROBE)
_VALIDATOR_SPEC = importlib.util.spec_from_file_location(
    "validate_numerical_record", HERE / "validate_numerical_record.py"
)
if _VALIDATOR_SPEC is None or _VALIDATOR_SPEC.loader is None:
    raise RuntimeError("could not load the Apple numerical record validator")
VALIDATOR = importlib.util.module_from_spec(_VALIDATOR_SPEC)
_VALIDATOR_SPEC.loader.exec_module(VALIDATOR)

RESULTS = HERE / "results"
RECORD = RESULTS / "2026-07-31-numerics-covering-xcode26.6-metal32023.883" / "record.tsv"
EXHAUSTIVE_RECORD = (
    RESULTS / "2026-07-31-numerics-exhaustive-xcode26.6-metal32023.883" / "record.tsv"
)
RECORDS = {PROBE.COVERING: RECORD, PROBE.EXHAUSTIVE_MATRIX: EXHAUSTIVE_RECORD}
"""The retained record for each case matrix, because a run measures exactly one of them.

The gate runs the covering matrix, so `RECORD` is the one it holds itself to. The
exhaustive record is retained evidence for the rows only that sweep produces and
is compared when `TILER_APPLE_NUMERICS_EXHAUSTIVE` selects it; a run of one
matrix is never compared against the other's record, because every case the two
sets do not share would read as decay.
"""

HOST = "macos"
"""The family whose execution environment is the machine running the gate."""

_STATE: dict[str, object] = {}


def probe_run() -> PROBE.Run:
    """Run the probe once per session, or skip with a visible, reproducible reason."""
    if not _STATE:
        directory = Path(tempfile.mkdtemp(prefix="tiler-apple-numerics."))
        try:
            _STATE["run"] = PROBE.probe(directory)
        except PROBE.ProbeUnavailable as unavailable:
            _STATE["skip"] = str(unavailable)
        finally:
            shutil.rmtree(directory, ignore_errors=True)
    if "skip" in _STATE:
        message = f"Apple numerical probe skipped: {_STATE['skip']}"
        if os.environ.get(PROBE.REQUIRE_TOOLCHAIN) is not None:
            raise AssertionError(f"{PROBE.REQUIRE_TOOLCHAIN} is set, but {message}")
        print(message, file=sys.stderr)
        pytest.skip(message)
    run = _STATE["run"]
    assert isinstance(run, PROBE.Run)
    return run


def dispatched_families(run: PROBE.Run) -> tuple[str, ...]:
    """Every family whose own execution environment resolved on this host.

    A family missing from this tuple is announced by the caller and its
    device-side assertions are skipped for that family alone. It is not a
    toolchain skip: the compile-side assertions still hold it to account.
    """
    resolved = []
    for family in PROBE.FAMILIES:
        execution = run.environment[f"family.{family.name}.execution"]
        if not execution.startswith("unavailable:"):
            resolved.append(family.name)
        else:
            print(f"{family.name} has no device side: {execution}", file=sys.stderr)
    return tuple(resolved)


def is_subnormal(bits: int) -> bool:
    """Whether an `f32` pattern is subnormal, for the tests that name `f32` values.

    A test over the whole kernel table must use the kernel's own dtype instead;
    `f16`'s exponent field is four bits narrower and every `f32` normal would read
    as subnormal under this mask.
    """
    return PROBE.F32.is_subnormal(bits)


def synthetic(
    kernel: str,
    operations: int,
    results: dict[int, int] | None,
    *,
    flags: tuple[str, ...] = (),
    family: str = HOST,
) -> PROBE.Observation:
    """Build an offline observation without a toolchain, for the pure guard tests.

    `results=None` builds the compile-side-only shape a family with no attached
    device produces, which is the one an unguarded reading must never classify.
    The operand vector is the named kernel's own, so a synthetic `f16`
    observation is indexed the way a measured one is.
    """
    case = PROBE.Case(family, kernel, PROBE.Configuration("safe", "2", "off"))
    operands = PROBE.BY_NAME[kernel].dtype.operands
    return PROBE.Observation(
        case=case,
        compile_options=("air.compile.denorms_disable",),
        operations=tuple(PROBE.FloatOperation("fadd", flags) for _ in range(operations)),
        results=(
            None
            if results is None
            else tuple(results.get(operand, operand) for operand in operands)
        ),
        applied_options=None,
        archived_options=None,
    )


def synthetic_runtime(
    kernel: str, results: dict[int, int], *, mode: str = "safe", family: str = HOST
) -> PROBE.Observation:
    """Build a runtime observation, whose compile side is unreadable rather than empty."""
    case = PROBE.Case(family, kernel, PROBE.RuntimeConfiguration(mode, "default"))
    operands = PROBE.BY_NAME[kernel].dtype.operands
    return PROBE.Observation(
        case=case,
        compile_options=None,
        operations=None,
        results=tuple(results.get(operand, operand) for operand in operands),
        applied_options=f"math={mode},fpfun=precise,lang=3.1,opt=default",
        archived_options="air.compile.denorms_disable",
    )


# --------------------------------------------------------------------------
# Guard tests. No Apple toolchain required; these must never be skipped.
# --------------------------------------------------------------------------


def test_the_apple9_profile_is_one_exact_indivisible_selection() -> None:
    """No caller can pair the governed offline target with another runtime language."""
    profile = PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26
    assert profile.name == "apple9-f32-unified-msl4-macos26"
    assert profile.schema == "tiler.apple-numerical-behaviour/v7"
    assert profile.msl_version == "metal4.0"
    assert profile.runtime_language == "4.0"
    assert profile.required_gpu_family is PROBE.GpuFamily.APPLE9
    assert profile.dtypes == (PROBE.F32,)
    assert [(family.name, family.sdk, family.target) for family in profile.families] == [
        ("macos", "macosx", "air64-apple-macos26.0")
    ]
    family = profile.family("macos")
    configuration = PROBE.Configuration("safe", "2", "off")
    offline = profile.offline_flags("macos", configuration)
    assert offline[:3] == [
        "-target",
        "air64-apple-macos26.0",
        "-std=metal4.0",
    ]
    runtime = profile.runtime_options(PROBE.RuntimeConfiguration("safe", "default"))
    assert runtime == "math=safe,fpfun=precise,lang=4.0,opt=default"
    assert {
        PROBE.BY_NAME[case.kernel].dtype
        for case in PROBE.cases("macos", profile=profile)
    } == {PROBE.F32}
    assert all(
        case.family == "macos"
        for case in PROBE.runtime_cases("macos", profile=profile)
    )
    materialize = synthetic("materialize", 0, {})
    run = PROBE.Run({"date_utc": "unreported"}, {materialize.case.key: materialize}, {})
    key = f"case.{materialize.case.key}.float_operations"
    assert dict(PROBE.record_rows(run, profile))[key] == "none"
    assert dict(PROBE.record_rows(run))[key] == ""


def test_the_bf16_profile_measures_the_f32_profiles_own_compilation() -> None:
    """The `bf16` row is a neighbouring profile, not a widening of the `f32` one.

    Two things have to hold at once for a `bf16` measurement taken here to be
    transcribable onto the authoritative compile profile. The compilation must be
    the one that profile names — same target, same language standard, same
    device family — which is asserted field by field against the `f32` profile
    rather than against a literal, so a later edit to either one cannot let them
    drift apart silently. And the `f32` profile's own selection must be
    untouched, because four retained records and the target-profile authority
    ledger cite it.
    """
    f32_profile = PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26
    profile = PROBE.APPLE9_F32_BF16_UNIFIED_MSL4_MACOS26
    assert profile.name == "apple9-f32-bf16-unified-msl4-macos26"
    assert profile is not f32_profile
    assert PROBE.PROFILES[profile.name] is profile
    assert f32_profile.dtypes == (PROBE.F32,), (
        "the f32 profile's dtype set moved, which would oblige a re-run whose record no longer "
        "carries the harness digest the retained citations pin"
    )
    assert profile.dtypes == (PROBE.F32, PROBE.BF16)
    for field in ("schema", "msl_version", "runtime_language", "required_gpu_family"):
        assert getattr(profile, field) == getattr(f32_profile, field), field
    assert [(family.name, family.sdk, family.target) for family in profile.families] == [
        (family.name, family.sdk, family.target) for family in f32_profile.families
    ]
    configuration = PROBE.Configuration("safe", "2", "off")
    assert profile.offline_flags("macos", configuration) == f32_profile.offline_flags(
        "macos", configuration
    )
    runtime = PROBE.RuntimeConfiguration("safe", "default")
    assert profile.runtime_options(runtime) == f32_profile.runtime_options(runtime)
    measured = {
        PROBE.BY_NAME[case.kernel].dtype
        for case in PROBE.cases("macos", PROBE.COVERING, profile)
    }
    assert measured == {PROBE.F32, PROBE.BF16}
    kernels = {case.kernel for case in PROBE.cases("macos", PROBE.COVERING, profile)}
    # The kernels the ticket's required evidence is read from: both flush
    # directions, the sign row's kernel, and the arithmetic-free control that
    # separates a flush from a buffer round trip.
    for required in ("multiply_two_bf16", "multiply_half_bf16", "materialize_bf16"):
        assert required in kernels, required
    # Every `f32` case of the neighbouring profile is measured here too, which is
    # what makes this profile's `f32` rows a control on the `bf16` ones rather
    # than a second unrelated population.
    assert {case.key for case in PROBE.cases("macos", PROBE.COVERING, f32_profile)} <= {
        case.key for case in PROBE.cases("macos", PROBE.COVERING, profile)
    }


def test_the_validator_resolves_the_profile_the_record_names() -> None:
    """A record is held to the identity of its own profile, and legacy is refused.

    The validator used to pin one profile name as a constant, which made a second
    measured row unvalidatable without a second copy of every check. Resolving
    the profile from the record is narrower, not looser -- an unknown name and
    the legacy profile are both refused -- so this pins the refusals rather than
    only the acceptance.
    """
    profile = PROBE.APPLE9_F32_BF16_UNIFIED_MSL4_MACOS26
    assert (
        VALIDATOR.resolve_profile({"probe.profile": profile.name}, PROBE) is profile
    )
    with pytest.raises(VALIDATOR.RecordError, match="no producer profile"):
        VALIDATOR.resolve_profile({"probe.profile": "apple9-invented"}, PROBE)
    with pytest.raises(VALIDATOR.RecordError, match="legacy profile"):
        VALIDATOR.resolve_profile({"probe.profile": PROBE.LEGACY_PROFILE.name}, PROBE)
    with pytest.raises(VALIDATOR.RecordError, match="missing required field"):
        VALIDATOR.resolve_profile({}, PROBE)


def test_the_bf16_population_validates_at_its_own_rendered_width() -> None:
    """A narrow-dtype record must pass the same population check the `f32` one does.

    The width is the specific thing at risk: a `bf16` `results` row is four hex
    digits and the validator's shape check was pinned at eight, so this fails on
    a validator that assumed `f32`'s rendering. The negative half holds the
    narrowed pattern to still rejecting a malformed row, so widening the check
    cannot have been done by dropping it.
    """
    profile = PROBE.APPLE9_F32_BF16_UNIFIED_MSL4_MACOS26
    rows = apple9_record_rows(profile)
    VALIDATOR.validate_population(rows, PROBE)
    bf16_results = {
        key: value
        for key, value in rows.items()
        if key.endswith(".results") and "_bf16." in key
    }
    assert bf16_results, "the bf16 profile produced no bf16 results row to check"
    for key, value in bf16_results.items():
        assert all(len(pattern) == 4 for pattern in value.split()), (key, value)
    widened = dict(rows)
    key = next(iter(bf16_results))
    widened[key] = " ".join(f"0000{pattern}" for pattern in rows[key].split())
    with pytest.raises(VALIDATOR.RecordError):
        VALIDATOR.validate_population(widened, PROBE)


def apple9_record_rows(profile=None) -> dict[str, str]:
    """Build a complete producer-defined covering population without a GPU.

    Parameterized by profile so the narrow-dtype row is exercised by the same
    portable check as the `f32` one. That is not symmetry for its own sake: a
    `bf16` results row renders four hex digits where an `f32` row renders eight,
    so a width pinned at `f32`'s would reject every row of the second profile,
    and a host with no Apple toolchain is where that has to be caught.
    """
    profile = PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26 if profile is None else profile
    observations = {}
    for case in (
        *PROBE.cases("macos", PROBE.COVERING, profile),
        *PROBE.runtime_cases("macos", PROBE.COVERING, profile),
    ):
        kernel = PROBE.BY_NAME[case.kernel]
        values = list(kernel.dtype.operands)
        if kernel.witness is not None:
            values[kernel.dtype.operands.index(kernel.witness.operand)] = kernel.witness.executed
        runtime = case.is_runtime
        observations[case.key] = PROBE.Observation(
            case,
            None if runtime else ("air.compile.denorms_disable",),
            None if runtime else (PROBE.FloatOperation("fadd", ()),),
            tuple(values),
            profile.runtime_options(case.configuration) if runtime else None,
            "" if runtime else None,
        )
    run = PROBE.Run({"date_utc": "unreported"}, observations, {})
    return dict(PROBE.record_rows(run, profile))


def test_the_apple9_validator_requires_the_exact_population_and_linkage() -> None:
    rows = apple9_record_rows()
    VALIDATOR.validate_population(rows, PROBE)
    inadmissible_rows = dict(rows)
    kernel = PROBE.BY_NAME["scale_one_bias_zero"]
    assert kernel.witness is not None
    witness_index = kernel.dtype.operands.index(kernel.witness.operand)
    for key in tuple(inadmissible_rows):
        if ".scale_one_bias_zero." not in key or not key.endswith(".results"):
            continue
        patterns = inadmissible_rows[key].split()
        patterns[witness_index] = kernel.dtype.render(kernel.witness.deleted)
        inadmissible_rows[key] = " ".join(patterns)
        witness_key = key.removesuffix(".results") + ".execution_witness"
        inadmissible_rows[witness_key] = (
            f"operand={kernel.dtype.render(kernel.witness.operand)},"
            f"expected={kernel.dtype.render(kernel.witness.executed)},"
            f"observed={kernel.dtype.render(kernel.witness.deleted)},status=not-executed"
        )
    VALIDATOR.validate_population(inadmissible_rows, PROBE)
    disagreeing_rows = dict(rows)
    results_key = next(
        key
        for key in disagreeing_rows
        if ".multiply_two." in key and key.endswith(".results")
    )
    kernel = PROBE.BY_NAME["multiply_two"]
    assert kernel.witness is not None
    patterns = disagreeing_rows[results_key].split()
    witness_index = kernel.dtype.operands.index(kernel.witness.operand)
    unrelated = kernel.dtype.operands[0]
    assert unrelated not in {kernel.witness.executed, kernel.witness.deleted}
    diagnostic = synthetic(
        "multiply_two",
        1,
        {kernel.witness.operand: unrelated},
    )
    diagnostic_rows = dict(
        PROBE.record_rows(
            PROBE.Run(
                {"date_utc": "unreported"},
                {diagnostic.case.key: diagnostic},
                {},
            ),
            PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26,
        )
    )
    assert diagnostic_rows[
        f"case.{diagnostic.case.key}.execution_witness"
    ].endswith("status=disagrees")
    patterns[witness_index] = kernel.dtype.render(unrelated)
    disagreeing_rows[results_key] = " ".join(patterns)
    witness_key = results_key.removesuffix(".results") + ".execution_witness"
    disagreeing_rows[witness_key] = (
        f"operand={kernel.dtype.render(kernel.witness.operand)},"
        f"expected={kernel.dtype.render(kernel.witness.executed)},"
        f"observed={kernel.dtype.render(unrelated)},status=not-executed"
    )
    with pytest.raises(VALIDATOR.RecordError, match="status=disagrees"):
        VALIDATOR.validate_population(disagreeing_rows, PROBE)
    disagreeing_rows[witness_key] = disagreeing_rows[witness_key].replace(
        "status=not-executed", "status=disagrees"
    )
    with pytest.raises(VALIDATOR.RecordError, match="witness disagrees"):
        VALIDATOR.validate_population(disagreeing_rows, PROBE)
    mutations = []
    for suffix in (
        ".results",
        ".execution_witness",
        ".applied_options",
    ):
        changed = dict(rows)
        del changed[next(key for key in changed if key.startswith("case.") and key.endswith(suffix))]
        mutations.append(changed)
    changed = dict(rows)
    case_key = next(iter(PROBE.cases("macos", PROBE.COVERING, PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26))).key
    for key in tuple(changed):
        if key.startswith(f"case.{case_key}."):
            del changed[key]
    mutations.append(changed)
    changed = dict(rows)
    key = next(key for key in changed if key.endswith(".results"))
    changed[key] = " ".join(changed[key].split()[:-1])
    mutations.append(changed)
    changed = dict(rows)
    key = next(key for key in changed if key.endswith(".execution_witness") and changed[key] != "none")
    changed[key] = changed[key].replace("status=executed", "status=not-executed")
    mutations.append(changed)
    changed = dict(rows)
    key = next(key for key in changed if key.endswith(".applied_options"))
    changed[key] = changed[key].replace("lang=4.0", "lang=4.0evil")
    mutations.append(changed)
    changed = dict(rows)
    key = next(key for key in changed if key.startswith("comparison."))
    changed[key] = "agree arbitrary"
    mutations.append(changed)
    changed = dict(rows)
    del changed[next(key for key in changed if key.startswith("comparison."))]
    mutations.append(changed)
    changed = dict(rows)
    key = next(key for key in changed if key.startswith("case.macos."))
    changed[key.replace("case.macos.", "case.ios-device.", 1)] = changed.pop(key)
    mutations.append(changed)
    for mutated in mutations:
        with pytest.raises(VALIDATOR.RecordError):
            VALIDATOR.validate_population(mutated, PROBE)


@pytest.mark.parametrize(
    "profile",
    [PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26, PROBE.APPLE9_F32_BF16_UNIFIED_MSL4_MACOS26],
    ids=lambda profile: profile.name,
)
def test_the_apple9_manifest_requires_every_unique_source(profile) -> None:
    with tempfile.TemporaryDirectory(prefix="tiler-source-inventory.") as directory:
        root = Path(directory)
        sources = root / "sources"
        sources.mkdir()
        kernels = {
            case.kernel
            for case in PROBE.cases("macos", PROBE.COVERING, profile)
        }
        manifest_rows = [
            ("schema", VALIDATOR.MANIFEST_SCHEMA),
            ("profile", profile.name),
            ("msl_version", profile.msl_version),
            ("runtime_language", profile.runtime_language),
        ]
        for path in (HERE / "numerical_probe.py", HERE / "numerical_probe_host.m", PROBE.VALIDATOR):
            relative = path.relative_to(PROBE.REPOSITORY)
            manifest_rows.append((f"input.{relative}", PROBE.digest(path)))
        for name in sorted(kernels):
            source = sources / f"{name}.metal"
            source.write_text(PROBE.BY_NAME[name].source(), encoding="utf-8")
            manifest_rows.append((f"source.sources/{source.name}", PROBE.digest(source)))
        manifest = root / "input-manifest.tsv"
        manifest.write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest_rows),
            encoding="utf-8",
        )
        rows = {
            "probe.profile": profile.name,
            "probe.matrix": PROBE.COVERING,
            "probe.input_manifest_file": manifest.name,
            "probe.input_manifest_sha256": PROBE.digest(manifest),
        }
        VALIDATOR.validate_manifest(root / "record.tsv", rows, PROBE)
        source_index = next(
            index for index, (key, _) in enumerate(manifest_rows) if key.startswith("source.")
        )
        source_key, _ = manifest_rows[source_index]
        source = root / source_key.removeprefix("source.")
        canonical_source = source.read_text(encoding="utf-8")
        source.write_text(f"{canonical_source}// mutation\n", encoding="utf-8")
        manifest_rows[source_index] = (source_key, PROBE.digest(source))
        manifest.write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest_rows),
            encoding="utf-8",
        )
        rows["probe.input_manifest_sha256"] = PROBE.digest(manifest)
        with pytest.raises(VALIDATOR.RecordError, match="canonical producer output"):
            VALIDATOR.validate_manifest(root / "record.tsv", rows, PROBE)
        source.write_text(canonical_source, encoding="utf-8")
        manifest_rows[source_index] = (source_key, PROBE.digest(source))
        manifest_rows.pop()
        manifest.write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest_rows),
            encoding="utf-8",
        )
        rows["probe.input_manifest_sha256"] = PROBE.digest(manifest)
        with pytest.raises(VALIDATOR.RecordError, match="source inventory"):
            VALIDATOR.validate_manifest(root / "record.tsv", rows, PROBE)


def test_profile_boundaries_reject_cross_family_and_unknown_gpu_requirements() -> None:
    profile = PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26
    with pytest.raises(PROBE.ProbeFailure):
        PROBE.cases("ios-device", profile=profile)
    with pytest.raises(PROBE.ProbeFailure):
        PROBE.runtime_cases("ios-device", profile=profile)
    with pytest.raises(TypeError):
        PROBE.Profile(
            "bad",
            profile.schema,
            profile.msl_version,
            profile.runtime_language,
            profile.families,
            profile.dtypes,
            "apple10",
        )


def test_invalid_profile_output_pairings_never_probe_hardware(monkeypatch) -> None:
    called = 0

    def unexpected_probe(*_arguments, **_keywords):
        nonlocal called
        called += 1
        raise AssertionError("probe must not run")

    monkeypatch.setattr(PROBE, "probe", unexpected_probe)
    with pytest.raises(SystemExit):
        PROBE.main(
            [
                "--profile",
                PROBE.APPLE9_F32_UNIFIED_MSL4_MACOS26.name,
                "--record",
                "wrong.tsv",
            ]
        )
    with pytest.raises(SystemExit):
        PROBE.main(["--result-dir", "wrong"])
    assert called == 0


def test_the_producer_revision_must_resolve_the_recorded_blobs(monkeypatch) -> None:
    rows = {
        "probe.repository_base_revision": "0" * 40,
        "probe.harness_sha256": "1" * 64,
        "probe.host_source_sha256": "2" * 64,
        "probe.validator_sha256": "3" * 64,
    }
    with pytest.raises(VALIDATOR.RecordError, match="all-zero"):
        VALIDATOR.validate_revision_identity(rows)
    tree = subprocess.run(
        ["git", "-C", str(PROBE.REPOSITORY), "rev-parse", "HEAD^{tree}"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    rows["probe.repository_base_revision"] = tree
    with pytest.raises(VALIDATOR.RecordError, match="not a commit"):
        VALIDATOR.validate_revision_identity(rows)
    blobs = {
        "spikes/apple-targets/numerical_probe.py": b"probe",
        "spikes/apple-targets/numerical_probe_host.m": b"host",
        "spikes/apple-targets/validate_numerical_record.py": b"validator",
    }
    rows["probe.repository_base_revision"] = "a" * 40
    rows["probe.harness_sha256"] = hashlib.sha256(blobs["spikes/apple-targets/numerical_probe.py"]).hexdigest()
    rows["probe.host_source_sha256"] = hashlib.sha256(
        blobs["spikes/apple-targets/numerical_probe_host.m"]
    ).hexdigest()
    rows["probe.validator_sha256"] = hashlib.sha256(
        blobs["spikes/apple-targets/validate_numerical_record.py"]
    ).hexdigest()
    monkeypatch.setattr(
        VALIDATOR,
        "revision_blob",
        lambda _revision, relative: blobs[relative],
    )
    monkeypatch.setattr(VALIDATOR, "revision_object_type", lambda _revision: "commit")
    VALIDATOR.validate_revision_identity(rows)
    rows["probe.harness_sha256"] = "f" * 64
    with pytest.raises(VALIDATOR.RecordError, match="revision blob mismatch"):
        VALIDATOR.validate_revision_identity(rows)


def test_every_kernel_states_a_witness_that_could_prove_its_arithmetic_ran() -> None:
    """A witness must be able to separate execution from deletion, or be absent.

    Without this, a kernel could ship a witness whose executed and deleted
    results coincide, and every observation from it would read as admissible
    while proving nothing. The witness's `executed` value is derived from the
    kernel under *both* flush hypotheses rather than trusted: a witness whose
    result depends on flushing would make the guard depend on the behaviour under
    test, and a witness whose intermediate is subnormal does that without any of
    its three stated values being subnormal.
    """
    for kernel in PROBE.KERNELS:
        witness = kernel.witness
        if witness is None:
            continue
        assert witness.executed != witness.deleted, kernel.name
        for value in (witness.operand, witness.executed, witness.deleted):
            assert not kernel.dtype.is_subnormal(value), (
                f"{kernel.name}: a witness value may not be subnormal, or the witness would "
                f"depend on the behaviour under test"
            )
            assert value <= kernel.dtype.mask, (
                f"{kernel.name}: {value:x} does not fit {kernel.dtype.name}, so it is a pattern "
                f"of another dtype"
            )
        assert witness.deleted == witness.operand, (
            f"{kernel.name}: every kernel here stores its loaded value when its operations are "
            f"removed, so a `deleted` that is not the operand is a mis-stated witness"
        )
        for flushes in (False, True):
            assert PROBE.evaluate(kernel, witness.operand, flushes=flushes) == witness.executed, (
                f"{kernel.name}: the witness must give the same result whether or not subnormals "
                f"are flushed, and {'flushing' if flushes else 'exact'} arithmetic disagrees"
            )


def test_a_subnormal_probe_separates_flushing_from_preserving() -> None:
    """Each probe's two candidate results must be distinguishable and derived, not asserted.

    Both are computed from the kernel the probe is read with, under exact
    arithmetic and under the sign-preserving flush this hardware was measured to
    perform. That is stronger than checking that `flushing` is a zero, which was
    true only while every kernel was a bare multiply: an additive kernel whose
    bias dominates flushes its operand to zero and still returns a normal value.
    """
    seen = 0
    for kernel in PROBE.KERNELS:
        dtype = kernel.dtype
        for probe in kernel.subnormal_probes:
            seen += 1
            spelled = f"{kernel.name}/{dtype.render(probe.operand)}"
            assert probe.preserving != probe.flushing, spelled
            assert dtype.is_subnormal(probe.operand) or dtype.is_subnormal(probe.preserving), (
                "a subnormal probe must have a subnormal operand or a subnormal exact result"
            )
            assert PROBE.evaluate(kernel, probe.operand, flushes=False) == probe.preserving, (
                f"{spelled}: declared preserving result {dtype.render(probe.preserving)} is not "
                f"the exact one"
            )
            assert PROBE.evaluate(kernel, probe.operand, flushes=True) == probe.flushing, (
                f"{spelled}: declared flushing result {dtype.render(probe.flushing)} is not what "
                f"substituting a signed zero gives"
            )
            assert probe.operand in dtype.operands, spelled
    assert seen >= 4, "the probe table lost its kernels"


def test_an_order_probe_separates_two_evaluation_orders() -> None:
    """The reassociation probe's two candidates must both be derivable and distinct.

    `ordered` is the left-to-right value the source spells, so it is derived the
    same way every other candidate is. `reassociated` is what summing the two
    small terms first gives, and it is stated rather than derived because the
    harness models one evaluation order and not every legal one — which is
    exactly why the kernel needs a witness on a different operand, since here
    `ordered` and the operand coincide.
    """
    probe = PROBE.REASSOCIATION
    kernel = PROBE.BY_NAME["reassociation_chain"]
    assert probe.ordered != probe.reassociated
    assert probe.operand in kernel.dtype.operands
    assert PROBE.evaluate(kernel, probe.operand, flushes=False) == probe.ordered
    assert PROBE.evaluate(kernel, probe.operand, flushes=True) == probe.ordered, (
        "the order probe must not depend on flushing, or it would measure two things at once"
    )
    assert probe.ordered == probe.operand, (
        "this chain returns its operand when it is not reassociated, which is the reason its "
        "witness has to live on another operand"
    )
    witness = kernel.witness
    assert witness is not None and witness.operand != probe.operand


def test_a_permutation_probe_separates_two_contributor_orders() -> None:
    """The permutation probe's two candidates come from two kernels, and are derived.

    `ordered` is what the canonical chain evaluates to and `permuted` is what its
    source-reordered twin evaluates to, both derived under exact arithmetic and
    under the sign-preserving flush rather than stated. The twin is held to being
    a genuine permutation — the same contributors, the same operator, a different
    order — because a "twin" that changed a constant would move the value for a
    reason that is not contributor order.
    """
    probe = PROBE.PERMUTATION
    ordered = PROBE.BY_NAME["permutation_chain"]
    permuted = PROBE.BY_NAME["permutation_chain_reordered"]
    assert probe.ordered != probe.permuted
    assert probe.operand in ordered.dtype.operands
    assert ordered.dtype is permuted.dtype
    assert sorted(step.constant for step in ordered.steps) == sorted(
        step.constant for step in permuted.steps
    ), "the twin must carry the same contributors, or it measures more than their order"
    assert [step.operator for step in ordered.steps] == ["+"] * 3
    assert [step.operator for step in permuted.steps] == ["+"] * 3
    assert [step.constant for step in ordered.steps] != [
        step.constant for step in permuted.steps
    ], "a twin in the same order would be the same kernel twice"
    for flushes in (False, True):
        assert PROBE.evaluate(ordered, probe.operand, flushes=flushes) == probe.ordered
        assert PROBE.evaluate(permuted, probe.operand, flushes=flushes) == probe.permuted, (
            "the permutation probe must not depend on flushing, or it would measure two "
            "things at once"
        )
    assert probe.ordered not in ordered.dtype.operands
    assert probe.permuted not in ordered.dtype.operands, (
        "a candidate equal to some operand would collide with the value a deleted chain "
        "returns on that lane"
    )


def test_the_permutation_probe_is_unreachable_by_reassociating_the_canonical_order() -> None:
    """The isolation this probe rests on, enumerated rather than argued.

    Reassociation moves the parentheses over a fixed leaf order; permutation
    moves the leaves. This probe isolates the second only if its `permuted`
    candidate cannot be produced by *any* parenthesization of the canonical leaf
    order — otherwise a device returning it would be evidence of the neighbouring
    licence and this would be a second reading of finding 17.

    The canonical order has four leaves, so there are exactly five full binary
    trees over it and the check is finite and complete. It runs for every operand
    in the vector rather than only the probe's, because a lane that admitted the
    permuted value would make the returned result vector ambiguous even where the
    probe does not read it.
    """
    ordered = PROBE.BY_NAME["permutation_chain"]
    dtype = ordered.dtype
    constants = [step.constant for step in ordered.steps]

    def trees(leaves: list[float]) -> list[float]:
        if len(leaves) == 1:
            return list(leaves)
        results: list[float] = []
        for split in range(1, len(leaves)):
            for left in trees(leaves[:split]):
                for right in trees(leaves[split:]):
                    results.append(dtype.as_float(dtype.as_bits(left + right)))
        return results

    for operand in dtype.operands:
        shapes = trees([dtype.as_float(value) for value in [operand, *constants]])
        assert len(shapes) == 5, "four leaves have exactly five parenthesizations"
        reachable = {dtype.as_bits(value) for value in shapes}
        assert PROBE.PERMUTATION.ordered in reachable, (
            "the canonical left fold is one of the five, so the ordered candidate must be "
            "reachable or the enumeration is wrong"
        )
        assert PROBE.PERMUTATION.permuted not in reachable, (
            f"operand {dtype.render(operand)}: reassociating the canonical order reaches "
            f"{dtype.render(PROBE.PERMUTATION.permuted)}, so the probe does not isolate "
            f"contributor permutation"
        )


def test_a_permutation_verdict_never_reports_a_permuted_result_as_reassociated() -> None:
    """The permutation classifier names its own licence, and refuses a third order.

    Perturbation in both directions: the canonical chain's own value is read as
    `left-to-right`, its twin's as `permuted`, and a result equal to neither
    candidate lands in `unexpected-result` instead of being forced into one. A
    classifier that could only ever return the two candidates would report the
    row it was given rather than the row that was measured.
    """
    probe = PROBE.PERMUTATION
    ordered_kernel = PROBE.BY_NAME["permutation_chain"]
    permuted_kernel = PROBE.BY_NAME["permutation_chain_reordered"]
    assert ordered_kernel.witness is not None and permuted_kernel.witness is not None
    ordered_results = dict.fromkeys(PROBE.F32.operands, probe.ordered)
    ordered_results[ordered_kernel.witness.operand] = ordered_kernel.witness.executed
    assert (
        PROBE.permutation_verdict(synthetic("permutation_chain", 3, ordered_results), probe)
        is PROBE.Verdict.LEFT_TO_RIGHT
    )
    permuted_results = dict.fromkeys(PROBE.F32.operands, probe.permuted)
    permuted_results[permuted_kernel.witness.operand] = permuted_kernel.witness.executed
    verdict = PROBE.permutation_verdict(
        synthetic("permutation_chain_reordered", 3, permuted_results), probe
    )
    assert verdict is PROBE.Verdict.PERMUTED and verdict.is_evidence
    third = dict.fromkeys(PROBE.F32.operands, 0x7F800000)
    third[ordered_kernel.witness.operand] = ordered_kernel.witness.executed
    assert (
        PROBE.permutation_verdict(synthetic("permutation_chain", 3, third), probe)
        is PROBE.Verdict.UNEXPECTED_RESULT
    ), "a third order must land in unexpected-result rather than being forced into a candidate"
    assert not PROBE.Verdict.UNEXPECTED_RESULT.is_evidence


def test_an_observation_with_no_emitted_arithmetic_is_never_evidence() -> None:
    observation = synthetic("multiply_two", 0, {PROBE.INPUT_FLUSH.operand: 0x00400000})
    assert (
        PROBE.subnormal_verdict(observation, PROBE.INPUT_FLUSH)
        is PROBE.Verdict.NO_EMITTED_ARITHMETIC
    )
    assert not PROBE.subnormal_verdict(observation, PROBE.INPUT_FLUSH).is_evidence


def test_an_observation_whose_witness_shows_deletion_is_never_evidence() -> None:
    """The emitted operation count alone must not be able to admit an observation.

    This is the layer that catches a stage below the emitted IR removing the
    arithmetic; the measurement test below shows that stage is real on this
    toolchain row.
    """
    probe = PROBE.IDENTITY_VALUED_FLUSH
    witness = PROBE.BY_NAME["scale_one_bias_zero"].witness
    assert witness is not None
    observation = synthetic(
        "scale_one_bias_zero",
        2,
        {witness.operand: witness.deleted, probe.operand: probe.preserving},
    )
    assert PROBE.subnormal_verdict(observation, probe) is PROBE.Verdict.ARITHMETIC_NOT_EXECUTED
    assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED, (
        "the unguarded reading must still be 'preserved', or this test is not exercising the trap"
    )


def test_an_observation_from_a_witnessless_kernel_is_never_evidence() -> None:
    """A kernel that is an identity on every operand can prove nothing about arithmetic."""
    assert PROBE.BY_NAME["multiply_one"].witness is None
    probe = PROBE.IDENTITY_VALUED_FLUSH
    observation = synthetic("multiply_one", 1, {probe.operand: probe.preserving})
    assert PROBE.subnormal_verdict(observation, probe) is PROBE.Verdict.NO_EXECUTION_WITNESS


def test_a_witnessed_surviving_operation_is_admitted_as_evidence() -> None:
    """The guard must admit a real observation, or it would only ever refuse."""
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    observation = synthetic(
        "multiply_two",
        1,
        {witness.operand: witness.executed, PROBE.INPUT_FLUSH.operand: PROBE.INPUT_FLUSH.flushing},
    )
    verdict = PROBE.subnormal_verdict(observation, PROBE.INPUT_FLUSH)
    assert verdict is PROBE.Verdict.FLUSHED_TO_ZERO
    assert verdict.is_evidence


def test_every_generated_kernel_declares_the_entry_point_and_the_exact_constants() -> None:
    """Emission must stay in the shape the record describes and use no decimal literals."""
    for kernel in PROBE.KERNELS:
        dtype = kernel.dtype
        source = kernel.source()
        assert f"kernel void {PROBE.ENTRY_POINT}(" in source, kernel.name
        assert f"ulong v1 = {len(dtype.operands)}ul;" in source, kernel.name
        assert f"device const {dtype.metal_type} *b0" in source, kernel.name
        assert f"device {dtype.metal_type} *b1" in source, kernel.name
        for step in kernel.steps:
            assert dtype.literal(step.constant) in source, kernel.name
        assert source.count("= fma(") == (1 if kernel.fused else 0), kernel.name
        for operator in (" * ", " + ", " / "):
            spelled = sum(1 for step in kernel.steps if step.operator == operator.strip())
            assert source.count(operator) == (0 if kernel.fused else spelled), (
                f"{kernel.name}: the emitted statements must be exactly the declared steps"
            )
        assert ".0f" not in source and "e+" not in source, (
            f"{kernel.name}: a decimal float literal would put host rendering in the path"
        )
        for other in PROBE.DTYPES:
            if other is dtype:
                continue
            assert f" {other.metal_type} " not in source, (
                f"{kernel.name}: a second scalar type in one translation unit would make the "
                f"measured arithmetic ambiguous"
            )


def test_every_kernel_names_its_dtype_exactly_when_it_is_not_the_default() -> None:
    """A case key has to say which dtype it measured, without renaming the ones that existed.

    `f32` kernels keep their bare names, so every case key the retained record and
    the research memo cite keeps its exact meaning; every other dtype's kernels
    carry the suffix, so a reader of a key knows the width its `results` row is
    rendered at without consulting the harness.
    """
    for kernel in PROBE.KERNELS:
        suffixed = kernel.name.endswith(f"_{kernel.dtype.name}")
        assert suffixed == (kernel.dtype is not PROBE.DEFAULT_DTYPE), kernel.name
        for other in PROBE.DTYPES:
            if other is kernel.dtype:
                continue
            assert not kernel.name.endswith(f"_{other.name}"), kernel.name
    assert PROBE.F16_KERNELS, "the second dtype lost every kernel"
    assert set(PROBE.F16_KERNELS) < set(PROBE.BY_NAME)
    assert PROBE.BF16_KERNELS, "the third dtype lost every kernel"
    assert set(PROBE.BF16_KERNELS) < set(PROBE.BY_NAME)
    assert not set(PROBE.F16_KERNELS) & set(PROBE.BF16_KERNELS)
    # The two narrow dtypes must ask the same questions, or a difference between
    # them would be a difference in coverage rather than in the hardware.
    #
    # One question is not askable at `bfloat`, and the exclusion is named here
    # rather than tolerated as a gap: MSL provides no `bfloat` overload of `fma`,
    # so `fused_pair_bf16` cannot be compiled at all -- `metal` rejects
    # `bfloat v6 = fma(v3, v4, v5)` because the call promotes to `float`. That is
    # a fact about the language rather than about coverage, and spelling it
    # `bfloat(fma(...))` would measure a fusion at `f32` precision narrowed
    # afterwards, which is a different operation. Naming the exclusion keeps this
    # assertion able to say no: a second divergence, or this one in the other
    # direction, still fails.
    askable_at_bf16_except = {"fused_pair"}
    f16_questions = [name.removesuffix("_f16") for name in PROBE.F16_KERNELS]
    bf16_questions = [name.removesuffix("_bf16") for name in PROBE.BF16_KERNELS]
    assert (
        set(f16_questions) - set(bf16_questions) == askable_at_bf16_except
    ), "the narrow dtypes' kernel sets diverged beyond the question MSL cannot express at bfloat"
    assert not set(bf16_questions) - set(
        f16_questions
    ), "bf16 asks a question f16 does not, which no exclusion covers"
    assert [
        name for name in f16_questions if name not in askable_at_bf16_except
    ] == bf16_questions, "the narrow dtypes' shared questions fell out of order"
    assert set(PROBE.NARROW_KERNELS) == {PROBE.F16.name, PROBE.BF16.name}


def test_every_dtype_renders_at_its_own_width_and_declares_a_consistent_format() -> None:
    """The widths, masks, and boundaries each dtype states must agree with `struct`'s.

    Nothing here needs a GPU and everything here is what a `case.*.results` row's
    meaning rests on: a dtype whose declared exponent mask disagreed with the
    format it packs would classify a normal as subnormal and derive both of a
    probe's candidates wrongly, in a way no device measurement would contradict.
    """
    assert PROBE.DEFAULT_DTYPE in PROBE.DTYPES
    assert len({dtype.name for dtype in PROBE.DTYPES}) == len(PROBE.DTYPES)
    for dtype in PROBE.DTYPES:
        assert dtype.digits * 4 == dtype.bits
        assert len(dtype.render(dtype.mask)) == dtype.digits
        assert dtype.exponent_mask | dtype.mantissa_mask | dtype.sign_mask == dtype.mask
        assert not dtype.exponent_mask & dtype.mantissa_mask
        assert dtype.as_bits(0.0) == 0
        assert dtype.as_bits(-0.0) == dtype.sign_mask
        assert not dtype.is_subnormal(0) and not dtype.is_subnormal(dtype.sign_mask)
        assert dtype.flush(dtype.sign_mask | 1) == dtype.sign_mask, (
            f"{dtype.name}: the flush must preserve the sign, which is finding 3"
        )
        assert len(dtype.operands) == len(PROBE.DEFAULT_DTYPE.operands), (
            f"{dtype.name}: every dtype answers the same questions, so the vectors are aligned"
        )
        for operand in dtype.operands:
            assert operand <= dtype.mask, f"{dtype.name}: {operand:x}"
            assert dtype.as_bits(dtype.as_float(operand)) == operand, (
                f"{dtype.name}: {dtype.render(operand)} does not survive its own round trip"
            )
        subnormals = [operand for operand in dtype.operands if dtype.is_subnormal(operand)]
        assert len(subnormals) == 4, f"{dtype.name}: {[dtype.render(v) for v in subnormals]}"


def test_no_kernel_can_produce_the_sentinel_its_dtype_seeds_the_output_with() -> None:
    """An unwritten element must stay distinguishable from a written one.

    The dispatch host seeds the output buffer with the dtype's sentinel and
    reports an element that still carries it as never written. That refusal is
    only sound while no kernel can return the pattern, which is a property of the
    kernel table and the operand vectors rather than of the host, so it is checked
    here — where a new kernel or a new operand is added.
    """
    for kernel in PROBE.KERNELS:
        dtype = kernel.dtype
        assert dtype.sentinel <= dtype.mask, dtype.name
        for operand in dtype.operands:
            for flushes in (False, True):
                assert PROBE.evaluate(kernel, operand, flushes=flushes) != dtype.sentinel, (
                    f"{kernel.name} can return {dtype.render(dtype.sentinel)}, which the host "
                    f"reads as an element no kernel wrote"
                )
        if kernel.witness is not None:
            assert kernel.witness.executed != dtype.sentinel, kernel.name


def test_the_operation_count_sees_every_spelling_this_front_end_emits() -> None:
    """A surviving operation reported as zero is indistinguishable from a deleted one.

    That is the reading finding 7 rests on, and it is exactly what went wrong once:
    `FUSED_INTRINSIC` named only the LLVM spellings while this front end emits
    `@air.fma.f32`, so a kernel whose whole body was one fused multiply-add
    counted zero operations. Widening the dtype puts the same question again in a
    new spelling, so the parse is pinned here over a module fragment rather than
    trusted — including the `call` to the generated canonicalization helper, which
    appears at `-O0`, is not arithmetic, and must not be counted.

    **Every line below is copied from a module this toolchain actually emitted**,
    which is the only thing that makes the pin evidence rather than a restatement
    of the pattern. That distinction has already cost once in the other
    direction: the helper `call` was pinned here in an *unmangled* spelling that
    the front end never produces. It named no fused intrinsic either way, so no
    count was ever wrong, but a reader checking the recognizer against a real
    module would have found the pinned line absent from it. The helper is a
    file-local C++ function and is mangled with its parameter type — `f`, `Dh`,
    and `DF16b` for the three dtypes — so the three spellings differ in more than
    the substring naming the dtype.

    A `bfloat` operand list is the case that matters most here, because this
    front end has **no** `air.fma.bf16`: a source-level `fma` on `bfloat` is
    `fpext` to `float`, `air.fma.f32`, and `fptrunc` back. The conversions are
    deliberately not counted — they are not arithmetic that can flush — while the
    `f32` fused call inside them is, which is exactly the reading that says such
    a kernel measures `f32` and not `bfloat`.
    """
    fragment = "\n".join(
        (
            "  %31 = call half @_ZL31tiler_canonicalize_nan_f16_7e00Dh(half noundef %30) #1",
            "  %31 = call bfloat @_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b"
            "(bfloat noundef %30) #1",
            "  %16 = fmul half %8, 0xH4000",
            "  %17 = fdiv half %16, 0xH4200",
            "  %18 = tail call half @air.fma.f16(half %17, half 0xH3E00, half 0xH3C00) #2",
            "  %19 = fadd reassoc nsz arcp afn half %18, 0xH3C00",
            "  %9 = fmul reassoc nsz arcp contract afn bfloat %8, 0xR402B",
            "  %9 = fdiv bfloat %8, 0xR4040",
            "  %9 = fadd bfloat %8, 0xR0080",
            "  %7 = fpext bfloat %6 to float",
            "  %8 = tail call float @air.fma.f32(float %7, float 1.5, float 1.0) #2",
            "  %9 = fptrunc float %8 to bfloat",
            "  %21 = fcmp oeq half %19, 0xH0000",
            "  %22 = fmul float %20, 2.000000e+00",
        )
    )
    found = PROBE.float_operations(fragment)
    assert [str(operation) for operation in found] == [
        "fmul",
        "fdiv",
        "air.fma.f16",
        "fadd+reassoc+nsz+arcp+afn",
        "fmul+reassoc+nsz+arcp+contract+afn",
        "fdiv",
        "fadd",
        "air.fma.f32",
        "fmul",
    ], found


def test_an_unreadable_module_and_a_module_with_no_arithmetic_are_never_confused() -> None:
    """`None` and `()` must stay different, because only one of them is a measurement.

    `()` says the harness read the module and found no arithmetic. `None` says
    the compilation path gave it no module to read, which is the runtime path's
    situation. Collapsing them would either fabricate a compile-side fact for
    every runtime case or classify every one of them `no-emitted-arithmetic`,
    and this is the assertion that stops a future edit doing either.
    """
    probe = PROBE.INPUT_FLUSH
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    admissible = {witness.operand: witness.executed, probe.operand: probe.flushing}

    measured_empty = synthetic("multiply_two", 0, admissible)
    assert measured_empty.operations == ()
    assert measured_empty.operation_count == 0
    assert PROBE.subnormal_verdict(measured_empty, probe) is PROBE.Verdict.NO_EMITTED_ARITHMETIC

    unreadable = synthetic_runtime("multiply_two", admissible)
    assert unreadable.operations is None
    assert unreadable.operation_count is None
    assert PROBE.subnormal_verdict(unreadable, probe) is PROBE.Verdict.FLUSHED_TO_ZERO


def test_an_observation_that_was_never_dispatched_is_never_evidence() -> None:
    """The mirror of the rule above, for the layer a family with no device loses.

    A compile-side-only observation has layer 1 and no layer 2, and layer 1 is
    necessary and never sufficient — the `-O0` measurement below is the proof
    that it passes on an observation whose arithmetic did not run. So it may
    never yield `preserved` or `flushed-to-zero` under any operand pattern, and
    it gets its own verdict rather than being classified by the layer it has.
    `results` is `None` and never `()` for the same reason `operations` is:
    `()` would assert a dispatch that returned nothing.
    """
    probe = PROBE.INPUT_FLUSH
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    undispatched = synthetic("multiply_two", 1, None, family="ios-device")
    assert undispatched.results is None
    assert undispatched.guard_layers == (PROBE.EMITTED_ARITHMETIC,)
    for candidate in (
        PROBE.INPUT_FLUSH,
        PROBE.NEGATIVE_INPUT_FLUSH,
        PROBE.RESULT_FLUSH,
        PROBE.IDENTITY_VALUED_FLUSH,
    ):
        verdict = PROBE.subnormal_verdict(undispatched, candidate)
        assert verdict is PROBE.Verdict.NO_DEVICE_OBSERVATION, candidate
        assert not verdict.is_evidence
    with pytest.raises(PROBE.ProbeFailure):
        undispatched.result_for(probe.operand)
    with pytest.raises(PROBE.ProbeFailure):
        PROBE.naive_verdict(undispatched, probe)


def test_the_record_omits_the_results_row_for_a_case_that_was_never_dispatched() -> None:
    """The record must not carry an empty row where no dispatch happened.

    A reader of `case.<key>.results` is entitled to treat it as a measurement, so
    a case from a family with no attached device has to have no such row rather
    than an empty one. This is the same contract `float_operations` carries on
    the runtime path, in the other direction.
    """
    dispatched = synthetic("multiply_two", 1, {PROBE.INPUT_FLUSH.operand: 0x00000000})
    undispatched = synthetic("multiply_two", 1, None, family="ios-device")
    run = PROBE.Run(
        environment={"date_utc": "unreported"},
        observations={
            dispatched.case.key: dispatched,
            undispatched.case.key: undispatched,
        },
        hazards={},
    )
    rows = dict(PROBE.record_rows(run))
    assert f"case.{dispatched.case.key}.results" in rows
    assert f"case.{undispatched.case.key}.results" not in rows
    assert f"case.{undispatched.case.key}.float_operations" in rows
    assert rows["probe.guard_layers.offline_without_device"] == PROBE.EMITTED_ARITHMETIC


def test_every_family_metal_platform_declares_a_distinct_target_and_sdk() -> None:
    """The three families must be three different compilations, not one relabelled.

    A family that shared another's `--sdk` and `-target` would produce identical
    rows for a reason that has nothing to do with Apple's toolchain, and the
    record's headline result — that the families agree — would be a tautology.
    """
    assert len(PROBE.FAMILIES) == 3
    for attribute in ("name", "metal_platform", "target"):
        values = [getattr(family, attribute) for family in PROBE.FAMILIES]
        assert len(set(values)) == len(values), attribute
    assert {family.sdk for family in PROBE.FAMILIES} == {
        "macosx",
        "iphoneos",
        "iphonesimulator",
    }
    assert [family.name for family in PROBE.FAMILIES] == list(PROBE.FAMILY_BY_NAME)


def test_every_family_is_offered_exactly_the_same_case_set() -> None:
    """A per-family difference must be in what the toolchain did, never in what was asked."""
    shapes = {
        family.name: sorted(
            (case.kernel, case.configuration.key) for case in PROBE.cases(family.name)
        )
        for family in PROBE.FAMILIES
    }
    reference = shapes[HOST]
    for name, shape in shapes.items():
        assert shape == reference, name
    for family in PROBE.FAMILIES:
        for case in PROBE.cases(family.name):
            assert case.key.startswith(f"{family.name}."), case.key


def test_the_runtime_path_declares_only_the_guard_layer_it_can_supply() -> None:
    """A runtime observation must advertise one layer, and the offline one both."""
    assert synthetic_runtime("multiply_two", {}).guard_layers == (PROBE.EXECUTION_WITNESS,)
    assert synthetic("multiply_two", 1, {}).guard_layers == (
        PROBE.EMITTED_ARITHMETIC,
        PROBE.EXECUTION_WITNESS,
    )


def test_the_surviving_guard_layer_still_refuses_a_deleted_operation() -> None:
    """Losing layer 1 must not cost the guard its ability to refuse.

    Layer 2 is the layer that caught the trap at `-O0` where layer 1 passed, so
    it has to keep refusing on a path that has only layer 2. The unguarded
    reading of the same observation must still be `preserved`, or this test is
    not exercising the trap at all.
    """
    probe = PROBE.IDENTITY_VALUED_FLUSH
    witness = PROBE.BY_NAME["scale_one_bias_zero"].witness
    assert witness is not None
    observation = synthetic_runtime(
        "scale_one_bias_zero",
        {witness.operand: witness.deleted, probe.operand: probe.preserving},
        mode="relaxed",
    )
    assert PROBE.subnormal_verdict(observation, probe) is PROBE.Verdict.ARITHMETIC_NOT_EXECUTED
    assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED


def test_every_runtime_case_has_an_offline_case_to_be_compared_against() -> None:
    """A runtime case with no counterpart would be measured and never compared."""
    for family in PROBE.FAMILIES:
        for selection in (PROBE.COVERING, PROBE.EXHAUSTIVE_MATRIX):
            offline = {
                (
                    case.kernel,
                    case.configuration.math_mode,
                    case.configuration.fp32_functions,
                )
                for case in PROBE.cases(family.name, selection)
                if isinstance(case.configuration, PROBE.Configuration)
                and case.configuration.optimization == PROBE.RUNTIME_PAIRED_OPTIMIZATION
            }
            for case in PROBE.runtime_cases(family.name, selection):
                assert isinstance(case.configuration, PROBE.RuntimeConfiguration)
                coordinates = (
                    case.kernel,
                    case.configuration.math_mode,
                    case.configuration.fp32_functions,
                )
                assert coordinates in offline, case.key


def test_the_covering_matrix_reaches_every_axis_the_exhaustive_one_sweeps() -> None:
    """The set the gate runs must not quietly stop covering an axis.

    The exhaustive sweep is behind an environment switch, so the covering set is
    what almost every run measures. It is allowed to be smaller; it is not
    allowed to lose a kernel, a math mode, an optimization level, a contraction
    setting, or an fp32-functions value, because a value nothing measures on an
    ordinary run is a value the retained record stops protecting.
    """
    covering = PROBE.cases(HOST, PROBE.COVERING)
    exhaustive = PROBE.cases(HOST, PROBE.EXHAUSTIVE_MATRIX)
    assert len(covering) < len(exhaustive), "the switch must select a genuinely larger sweep"
    assert {case.key for case in covering} <= {case.key for case in exhaustive}, (
        "the covering set must be a subset, or the record it pins is not a subset of the other"
    )
    for attribute, expected in (
        ("kernel", {kernel.name for kernel in PROBE.KERNELS}),
        ("math_mode", set(PROBE.MATH_MODES)),
        ("optimization", set(PROBE.OPTIMIZATIONS)),
        ("fp_contract", set(PROBE.FP_CONTRACTS)),
        ("fp32_functions", set(PROBE.FP32_FUNCTION_MODES)),
    ):
        found = {
            case.kernel if attribute == "kernel" else getattr(case.configuration, attribute)
            for case in covering
        }
        assert found == expected, f"the covering set reaches {found} of {expected} for {attribute}"


def test_a_record_from_one_matrix_is_never_compared_against_a_run_of_the_other() -> None:
    """The two case sets differ, so comparing across them would report decay that is not there."""
    assert PROBE.matrix_mismatch({"probe.matrix": PROBE.matrix()}) == ""
    other = PROBE.EXHAUSTIVE_MATRIX if PROBE.matrix() == PROBE.COVERING else PROBE.COVERING
    assert PROBE.matrix_mismatch({"probe.matrix": other})
    assert PROBE.matrix_mismatch({}), "a record with no matrix row states nothing and is refused"
    for selection, path in RECORDS.items():
        assert PROBE.read_record(path)["probe.matrix"] == selection, path


def test_a_runtime_case_is_never_compared_against_another_family() -> None:
    """A cross-family difference must not be able to read as a cross-path divergence.

    The two comparisons answer different questions, and only one of them is
    about the two compilers. Pairing a runtime case against another family's
    offline row would report a target difference as a compiler disagreement.
    """
    probe = PROBE.INPUT_FLUSH
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    flushing = {witness.operand: witness.executed, probe.operand: probe.flushing}
    preserving = {witness.operand: witness.executed, probe.operand: probe.preserving}
    here = synthetic("multiply_two", 1, flushing, family="macos")
    elsewhere = synthetic("multiply_two", 1, preserving, family="ios-simulator")
    runtime_here = synthetic_runtime("multiply_two", flushing, family="macos")
    run = PROBE.Run(
        environment={"date_utc": "unreported"},
        observations={
            here.case.key: here,
            elsewhere.case.key: elsewhere,
            runtime_here.case.key: runtime_here,
        },
        hazards={},
    )
    comparisons = PROBE.path_comparisons(run)
    assert len(comparisons) == 1
    assert comparisons[0].candidates == (here.case.key,)
    assert comparisons[0].agreement is PROBE.Agreement.AGREE


def test_a_runtime_result_matching_no_offline_candidate_is_the_divergence_outcome() -> None:
    """Only `differ` may mean the two compilers disagree.

    `agree-on-some` arises where the offline candidates differ from each other,
    which happens only on an axis `MTLCompileOptions` cannot express; treating it
    as a disagreement would report a missing flag as a compiler divergence.
    """
    candidates = ("f.k.safe.O2.contract-off", "f.k.safe.O2.contract-fast")
    results = (0x3F800000,) * len(PROBE.F32.operands)
    key = "f.k.runtime.safe.opt-default"
    everything = PROBE.PathComparison(key, candidates, candidates, results)
    some = PROBE.PathComparison(key, candidates, candidates[:1], results)
    nothing = PROBE.PathComparison(key, candidates, (), results)
    assert everything.agreement is PROBE.Agreement.AGREE
    assert some.agreement is PROBE.Agreement.AGREE_ON_SOME
    assert nothing.agreement is PROBE.Agreement.DIFFER
    assert not everything.agreement.is_divergence
    assert not some.agreement.is_divergence
    assert nothing.agreement.is_divergence
    assert "3f800000" in nothing.render(), "a divergence must record what the runtime path returned"


def test_the_record_omits_the_float_operations_row_for_a_runtime_case() -> None:
    """The record must not carry an empty row where no module was read.

    A reader of `case.<key>.float_operations` is entitled to treat it as a
    measurement, so a runtime case has to have no such row rather than an empty
    one. The comparison row has to be present for the same reason: it is what
    makes a divergence fail the gate instead of being quietly rewritten.
    """
    probe = PROBE.INPUT_FLUSH
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    results = {witness.operand: witness.executed, probe.operand: probe.flushing}
    offline = synthetic("multiply_two", 1, results)
    runtime = synthetic_runtime("multiply_two", results)
    run = PROBE.Run(
        environment={"date_utc": "unreported", "family.macos.device": "synthetic"},
        observations={offline.case.key: offline, runtime.case.key: runtime},
        hazards={},
    )
    rows = dict(PROBE.record_rows(run))
    assert f"case.{offline.case.key}.float_operations" in rows
    assert f"case.{runtime.case.key}.float_operations" not in rows
    assert f"case.{runtime.case.key}.compile_options" not in rows
    assert rows[f"case.{runtime.case.key}.applied_options"] == runtime.applied_options
    assert rows[f"comparison.{runtime.case.key}"].startswith("agree ")
    assert rows["probe.guard_layers.runtime"] == PROBE.EXECUTION_WITNESS


def test_a_record_row_may_not_carry_a_tab_or_a_newline() -> None:
    """The record's one-line-per-row format is enforced, not assumed.

    Several rows carry captured tool diagnostics, and one of them on the measured
    row is an abort message from inside the iOS Simulator. A diagnostic with a
    newline in it would split into two rows that `read_record` then rejects, so
    the format is checked where the rows are built.
    """
    observation = synthetic("multiply_two", 1, {})
    run = PROBE.Run(
        environment={"date_utc": "unreported"},
        observations={observation.case.key: observation},
        hazards={"cross_family_load.example": "refused: line one\nline two"},
    )
    with pytest.raises(PROBE.ProbeFailure):
        PROBE.record_rows(run)


def test_the_record_comparison_detects_a_changed_cross_path_verdict() -> None:
    """A rewritten `comparison.` or `hazard.` row must fail the comparison, not pass silently."""
    probe = PROBE.INPUT_FLUSH
    witness = PROBE.BY_NAME["multiply_two"].witness
    assert witness is not None
    results = {witness.operand: witness.executed, probe.operand: probe.flushing}
    offline = synthetic("multiply_two", 1, results)
    runtime = synthetic_runtime("multiply_two", results)
    run = PROBE.Run(
        environment={"date_utc": "unreported"},
        observations={offline.case.key: offline, runtime.case.key: runtime},
        hazards={"cross_family_load.example": "loaded and ran; results 00000000"},
    )
    stored = dict(PROBE.record_rows(run))
    assert not PROBE.compare_record(run, stored)
    changed = dict(stored)
    changed[f"comparison.{runtime.case.key}"] = "differ candidates=none"
    assert PROBE.compare_record(run, changed)
    changed = dict(stored)
    changed["hazard.cross_family_load.example"] = "refused: the module was rejected"
    assert PROBE.compare_record(run, changed), (
        "a change in what the cross-family load does must be a finding, not a silent rewrite"
    )


REGISTRY_ID_FAMILIES = ("macos", "ios-device", "ios-simulator")
"""Every family whose record may carry a `device_registry_id` row."""

PAIRED_REGISTRY_IDS: dict[str, str] = {
    "2026-07-24-numerics-families-xcode26.6-metal32023.883": "4294968621",
    "2026-07-25-numerics-covering-xcode26.6-metal32023.883": "4294968621",
    "2026-07-25-numerics-exhaustive-xcode26.6-metal32023.883": "4294968621",
    "2026-07-27-numerics-covering-xcode26.6-metal32023.883": "4294968452",
    "2026-07-27-numerics-exhaustive-xcode26.6-metal32023.883": "4294968452",
    "2026-07-31-numerics-covering-xcode26.6-metal32023.883": "4294968452",
    "2026-07-31-numerics-exhaustive-xcode26.6-metal32023.883": "4294968452",
}
"""Every retained record that dispatched both macOS and the iOS Simulator, with the ID it measured.

The values are transcribed from the records rather than derived, so a record
rewritten to agree with this table fails the completeness assertion below rather
than passing quietly. They deliberately disagree across measurements: the same
named Apple M4 Max reported `4294968621` on 2026-07-24 and 2026-07-25 and
`4294968452` on 2026-07-27, 2026-07-30, and 2026-07-31. That disagreement is the
measured fact, not decay, which is why it is asserted positively.
"""

MACOS_ONLY_REGISTRY_IDS: dict[str, str] = {
    "2026-07-30-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883": "4294968452",
    "2026-07-30-numerics-exhaustive-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883": "4294968452",
    "2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883": "4294968452",
    "2026-07-31-numerics-exhaustive-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883": "4294968452",
}
"""The named-profile records, which select the macOS family alone and so have no pair to check.

They are enumerated rather than skipped, and the absence of their simulator row
is asserted, because "excluded from the pair check" has to be a checked property
of the record instead of a claim in a comment that a later profile could quietly
falsify.
"""


def registry_id_rows(directory: str) -> dict[str, str]:
    """Read one enumerated record's registry-ID rows, failing when the record is absent.

    A missing record must fail rather than contribute nothing: an enumeration
    that silently loses a member reports the same green result as one that
    checked every member, which is the failure mode this whole check exists to
    avoid.
    """
    path = RESULTS / directory / "record.tsv"
    assert path.is_file(), f"an enumerated retained record is missing: {path}"
    rows = PROBE.read_record(path)
    return {
        family: rows[f"environment.family.{family}.device_registry_id"]
        for family in REGISTRY_ID_FAMILIES
        if f"environment.family.{family}.device_registry_id" in rows
    }


def registry_id_violations(observed: dict[str, dict[str, str]]) -> list[str]:
    """Every way an observed population departs from the enumerated one, named exactly.

    Taking the parsed rows as an argument is what lets the test perturb one
    within-measurement value and watch this refuse it, without touching a
    retained record on disk.
    """
    violations = []
    for directory, expected in sorted(PAIRED_REGISTRY_IDS.items()):
        rows = observed[directory]
        for family in ("macos", "ios-simulator"):
            if rows.get(family) != expected:
                violations.append(f"{directory}: {family}={rows.get(family)!r}, expected {expected}")
        if not rows.get("ios-device", "").startswith("unavailable:"):
            violations.append(f"{directory}: ios-device is not recorded as unavailable")
    for directory, expected in sorted(MACOS_ONLY_REGISTRY_IDS.items()):
        rows = observed[directory]
        if rows.get("macos") != expected:
            violations.append(f"{directory}: macos={rows.get('macos')!r}, expected {expected}")
        if "ios-simulator" in rows:
            violations.append(f"{directory}: has a simulator row, so it is not macOS-only")
    return violations


def test_the_registry_id_agrees_within_a_measurement_and_is_free_between_them() -> None:
    """The two families that dispatch report one registry ID per run, and runs may differ.

    `registryID` is documented by `MTLDevice.h` as "the IORegistry ID for the
    Metal device", "global to all tasks", and usable "to identify the GPU across
    task boundaries". That is a correlation handle inside one active
    environment, and it is the only property the retained records support: the
    same named Apple M4 Max reports two different IDs across them. So the
    invariant worth pinning is equality between the macOS host and the iOS
    Simulator *within* one measurement — finding 13's evidence that the
    simulator dispatches on the host GPU — and explicitly not stability of the
    value between measurements.

    The population is enumerated and counted rather than discovered, because a
    check that iterates whatever it happens to find reports the same success
    when it finds nothing. The enumeration is then held to covering every
    retained record that carries a registry-ID row, so a new record cannot join
    the results directory without joining this check.
    """
    enumerated = {**PAIRED_REGISTRY_IDS, **MACOS_ONLY_REGISTRY_IDS}
    assert len(enumerated) == len(PAIRED_REGISTRY_IDS) + len(MACOS_ONLY_REGISTRY_IDS), (
        "a directory is enumerated as both paired and macOS-only"
    )
    carrying = sorted(
        directory.name
        for directory in RESULTS.iterdir()
        if directory.is_dir()
        and (directory / "record.tsv").is_file()
        and any(
            key.endswith(".device_registry_id")
            for key in PROBE.read_record(directory / "record.tsv")
        )
    )
    assert carrying == sorted(enumerated), (
        f"the enumerated registry-ID population ({len(enumerated)} records) is not the retained "
        f"one ({len(carrying)} records). Add the new record to `PAIRED_REGISTRY_IDS` or "
        f"`MACOS_ONLY_REGISTRY_IDS` with the value it measured. Enumerated: {sorted(enumerated)}. "
        f"Retained: {carrying}"
    )
    observed = {directory: registry_id_rows(directory) for directory in enumerated}
    assert not registry_id_violations(observed), registry_id_violations(observed)
    assert len(set(PAIRED_REGISTRY_IDS.values()) | set(MACOS_ONLY_REGISTRY_IDS.values())) > 1, (
        "the retained records no longer disagree across measurements. Either a raw measurement was "
        "rewritten to make them agree, which is forbidden, or the historical rows were dropped -- "
        "and the evidence that registry ID is not a durable hardware identity went with them"
    )
    perturbed = {directory: dict(rows) for directory, rows in observed.items()}
    paired = min(PAIRED_REGISTRY_IDS)
    borrowed = next(
        value for value in PAIRED_REGISTRY_IDS.values() if value != PAIRED_REGISTRY_IDS[paired]
    )
    perturbed[paired]["ios-simulator"] = borrowed
    assert registry_id_violations(perturbed), (
        "a macOS/simulator disagreement inside one measurement was accepted, so this check cannot "
        "detect the one thing it pins"
    )
    macos_only = min(MACOS_ONLY_REGISTRY_IDS)
    perturbed = {directory: dict(rows) for directory, rows in observed.items()}
    perturbed[macos_only]["ios-simulator"] = MACOS_ONLY_REGISTRY_IDS[macos_only]
    assert registry_id_violations(perturbed), (
        "a macOS-only record that gained a simulator row was accepted, so its exclusion from the "
        "pair check is an assumption rather than a checked property"
    )


def test_an_absent_xcrun_is_classified_as_an_absent_toolchain() -> None:
    """Resolution must refuse with the skip classification, not a bare exception."""
    original = os.environ.get("PATH", "")
    with tempfile.TemporaryDirectory(prefix="tiler-empty-path.") as empty:
        os.environ["PATH"] = empty
        try:
            with pytest.raises(PROBE.ProbeUnavailable) as caught:
                PROBE.resolve()
        finally:
            os.environ["PATH"] = original
    assert caught.value.reason is PROBE.Reason.TOOLCHAIN


FAKE_DTYPES = {"k": PROBE.F32, "a": PROBE.F32, "b": PROBE.F32}
"""The dtype of every case key the fake dispatch hosts below report."""


def fake_host(directory: str, script: str) -> tuple[Path, PROBE.Attachment, Path]:
    """Write an executable stand-in for the dispatch host and a one-entry manifest."""
    host = Path(directory) / "host"
    host.write_text(script, encoding="utf-8")
    host.chmod(0o755)
    manifest = Path(directory) / "manifest.tsv"
    manifest.write_text(
        "k\tf32\tlibrary\t/absent.metallib\ttiler_probe\n",
        encoding="utf-8",
    )
    attachment = PROBE.Attachment(PROBE.FAMILY_BY_NAME[HOST], True, "", (), "macosx", (), ())
    return host, attachment, manifest


def test_a_dispatch_reporting_a_pattern_too_wide_for_its_dtype_is_a_defect() -> None:
    """A result that does not fit the element that produced it must never reach the record.

    The host prints an `f16` result at four digits, so an eight-digit pattern
    under an `f16` case key means the manifest, the host, or the buffer width
    disagreed about what was dispatched. Left unchecked it would be recorded as a
    plausible measurement, because nothing downstream re-derives the width.
    """
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        results = "\n".join("echo result=3f800000" for _ in PROBE.F16.operands)
        host, attachment, manifest = fake_host(
            directory, f"#!/bin/sh\necho device=fake\necho case=k\n{results}\n"
        )
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", {"k": PROBE.F16})
        reported = PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)
    assert reported.entries["k"].results == (0x3F800000,) * len(PROBE.F32.operands), (
        "the same output must be accepted as f32, or this test is rejecting it for another reason"
    )


def test_a_dispatch_reporting_a_case_the_manifest_never_asked_for_is_a_defect() -> None:
    """An unrecognized case key has no dtype, so its results have no width to be read at."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        results = "\n".join("echo result=00000000" for _ in PROBE.F32.operands)
        host, attachment, manifest = fake_host(
            directory, f"#!/bin/sh\necho device=fake\necho case=unasked\n{results}\n"
        )
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)


def test_every_dtype_is_given_its_own_operand_group_on_every_invocation() -> None:
    """An entry must never be able to resolve a vector the harness did not pass.

    Passing only the dtypes a manifest happens to use would make the host's
    refusal of a missing group unreachable, which is the check that stops an
    `f16` entry being dispatched over `f32` operands.
    """
    arguments = PROBE.operand_arguments()
    assert len(arguments) == len(PROBE.DTYPES)
    for dtype, argument in zip(PROBE.DTYPES, arguments, strict=True):
        name, _, patterns = argument.partition("=")
        assert name == dtype.name
        assert patterns.split(",") == [dtype.render(value) for value in dtype.operands]
        for pattern in patterns.split(","):
            assert len(pattern) == dtype.digits, pattern


def test_a_host_reporting_no_metal_device_is_classified_as_an_absent_device() -> None:
    """The one skip axis the offline driver's classification has no name for.

    `golden_compilation` compiles and links and never dispatches, so it cannot
    distinguish a host with a Metal compiler and no usable GPU. This probe can,
    and that outcome must be a skip rather than a failure.
    """
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host, attachment, manifest = fake_host(directory, "#!/bin/sh\necho no device >&2\nexit 3\n")
        with pytest.raises(PROBE.ProbeUnavailable) as caught:
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)
    assert caught.value.reason is PROBE.Reason.DEVICE


def test_a_host_that_fails_for_any_other_reason_is_a_defect_not_a_skip() -> None:
    """A dispatch that reaches the GPU and fails must never be mistaken for a skip."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host, attachment, manifest = fake_host(
            directory, "#!/bin/sh\necho pipeline exploded >&2\nexit 4\n"
        )
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)


def test_a_truncated_dispatch_is_a_defect() -> None:
    """A host that returns fewer results than operands must not be silently accepted."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host, attachment, manifest = fake_host(
            directory, "#!/bin/sh\necho device=fake\necho case=k\necho result=00000000\n"
        )
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)


def test_a_dispatch_that_reports_no_case_is_a_defect() -> None:
    """A batch that printed a device and nothing else must not read as an empty success."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host, attachment, manifest = fake_host(directory, "#!/bin/sh\necho device=fake\n")
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)


def test_a_batch_reports_every_case_separately() -> None:
    """Amortizing the process launch must not amortize the results.

    One launch dispatches a whole manifest, so the parser has to keep each
    entry's results attached to its own case key. A parser that ran two entries
    together would report one case's bit patterns under another's name.
    """
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        results_a = "\n".join(f"echo result={value:08x}" for value in PROBE.F32.operands)
        results_b = "\n".join("echo result=00000000" for _ in PROBE.F32.operands)
        host, attachment, manifest = fake_host(
            directory,
            "#!/bin/sh\necho device=fake\necho registry-id=7\n"
            f"echo case=a\n{results_a}\n"
            f"echo case=b\necho applied=math=safe\n{results_b}\n"
            "echo runtime-compiler-image=/x/GPUCompiler.framework/y\n",
        )
        reported = PROBE.dispatch_batch(host, attachment, manifest, "the fake host", FAKE_DTYPES)
    assert reported.device == "fake"
    assert reported.registry_id == "7"
    assert reported.entries["a"].results == PROBE.F32.operands
    assert reported.entries["b"].results == (0,) * len(PROBE.F32.operands)
    assert reported.entries["a"].applied_options is None
    assert reported.entries["b"].applied_options == "math=safe"
    assert reported.compiler_images == ("/x/GPUCompiler.framework/y",)


# --------------------------------------------------------------------------
# Measurement tests. Conditional on a resolved toolchain and GPU.
# --------------------------------------------------------------------------


def test_every_family_compiles_its_own_target_when_a_toolchain_and_gpu_resolve() -> None:
    """Each family must have produced a distinct module, or nothing below separates them.

    The requested target is not the emitted one — `-std=metal3.1` raises the
    deployment floor — so the emitted triple is what proves three different
    compilations happened rather than one repeated.
    """
    run = probe_run()
    triples = {
        family.name: run.environment[f"family.{family.name}.emitted_triple"]
        for family in PROBE.FAMILIES
    }
    assert len(set(triples.values())) == len(triples), triples
    for family in PROBE.FAMILIES:
        assert triples[family.name].startswith("air64"), triples[family.name]
        assert run.environment[f"family.{family.name}.requested_target"] == family.target
        for case in PROBE.cases(family.name):
            observation = run.observations[case.key]
            assert observation.compile_options is not None, case.key
            assert observation.operations is not None, case.key


def test_the_safe_math_mode_still_disables_denormals_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 1, for every family. `air.compile.denorms_disable` under every math mode.

    Under `safe` it appears beside `air.compile.fast_math_disable` and no
    emitted operation carries a fast-math flag, so the strictest selection the
    driver offers declares fast math disabled and denormals disabled together.
    This is the compile-side half, so it holds a family with no attached device
    to exactly the same account as one with a GPU.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            for contract in PROBE.FP_CONTRACTS:
                observation = run.of(family.name, "scale_two_bias_one", mode, contract=contract)
                assert "air.compile.denorms_disable" in observation.compile_options, (
                    f"{family.name}/{mode}/{contract} did not declare denorms_disable"
                )
        for contract in PROBE.FP_CONTRACTS:
            safe = run.of(family.name, "scale_two_bias_one", "safe", contract=contract)
            assert "air.compile.fast_math_disable" in safe.compile_options, family.name
            assert "air.compile.fast_math_enable" not in safe.compile_options, family.name
            expected = () if contract != "fast" else ("contract",)
            for operation in safe.operations:
                assert operation.flags == expected, (
                    f"{family.name}/safe/{contract} attached {operation.flags} to "
                    f"a {operation.opcode}"
                )
            fast = run.of(family.name, "scale_two_bias_one", "fast", contract=contract)
            assert "air.compile.fast_math_enable" in fast.compile_options, family.name
            for operation in fast.operations:
                assert "nnan" in operation.flags or operation.flags == ("fast",), family.name


def test_the_families_agree_on_every_compile_side_row_when_a_toolchain_and_gpu_resolve() -> None:
    """The headline compile-side result, stated as one assertion over all three families.

    If the subnormal flush were a per-family property this is where it would show
    up first: `air.compile.denorms_disable` emitted for one family and not
    another, or a different fast-math licence set, or a different surviving
    operation count. A failure here is a finding about Apple's toolchain, not a
    harness defect, and it would mean `MetalSubnormalArithmetic` has to vary by
    family rather than being one declared constant.
    """
    run = probe_run()
    reference = PROBE.FAMILIES[0].name
    for family in PROBE.FAMILIES[1:]:
        for case in PROBE.cases(reference):
            here = run.observations[case.key]
            there = run.of(
                family.name,
                case.kernel,
                case.configuration.math_mode,
                case.configuration.optimization,
                case.configuration.fp_contract,
                case.configuration.fp32_functions,
            )
            assert there.compile_options == here.compile_options, (
                f"{family.name} declared {there.compile_options} where {reference} declared "
                f"{here.compile_options} for {case.key}"
            )
            assert there.operations == here.operations, (
                f"{family.name} emitted {there.operations} where {reference} emitted "
                f"{here.operations} for {case.key}"
            )


def test_a_family_with_no_device_yields_no_device_side_row_when_a_toolchain_resolves() -> None:
    """The refusal that makes the compile-side-only rows safe to keep.

    Every case of a family with no attached device must carry no results and no
    admissible verdict, in the live run and in the rendered record alike. The
    hazard row is what makes this non-obvious: on this host the macOS GPU loads
    and runs that family's metallib, so the refusal is structural rather than a
    consequence of the substitute being impossible.
    """
    run = probe_run()
    absent = [
        family
        for family in PROBE.FAMILIES
        if run.environment[f"family.{family.name}.execution"].startswith("unavailable:")
    ]
    if not absent:
        pytest.skip("every declared family resolved a device on this host")
    rows = dict(PROBE.record_rows(run))
    for family in absent:
        detail = run.environment[f"family.{family.name}.execution"]
        assert detail.removeprefix("unavailable:").strip(), family.name
        for case in PROBE.cases(family.name):
            observation = run.observations[case.key]
            assert observation.results is None, case.key
            assert observation.guard_layers == (PROBE.EMITTED_ARITHMETIC,), case.key
            assert f"case.{case.key}.results" not in rows, case.key
            for probe in (PROBE.INPUT_FLUSH, PROBE.RESULT_FLUSH, PROBE.IDENTITY_VALUED_FLUSH):
                verdict = PROBE.subnormal_verdict(observation, probe)
                assert verdict is PROBE.Verdict.NO_DEVICE_OBSERVATION, case.key
        assert not PROBE.runtime_cases(family.name) or all(
            PROBE.Case(family.name, case.kernel, case.configuration).key not in run.observations
            for case in PROBE.runtime_cases(family.name)
        ), f"{family.name} has runtime observations without an execution environment"


def test_the_host_gpu_runs_a_foreign_family_module_when_a_toolchain_and_gpu_resolve() -> None:
    """The convenient substitute for a missing device works, which is why it is refused.

    A reader is entitled to ask why the device gap cannot be closed by loading
    the iOS module on this Mac. The answer is not that the load fails; it is that
    the GPU and driver executing it are the Mac's. Recording the outcome under
    `hazard.` keeps that reasoning checkable, and would notice if Apple ever
    started refusing the load.
    """
    run = probe_run()
    if not run.hazards:
        pytest.skip("every declared family resolved a device, so there is no substitute to refuse")
    for name, outcome in sorted(run.hazards.items()):
        print(f"hazard.{name}: {outcome}", file=sys.stderr)
        assert outcome.startswith(("loaded and ran; results ", "refused: ")), outcome
        assert f"hazard.{name}" in dict(PROBE.record_rows(run))
        assert not any(name.startswith("case.") for name in run.hazards), (
            "a hazard must never be recorded as a case"
        )


def test_the_module_flag_is_not_a_summary_of_the_licences_when_a_toolchain_and_gpu_resolve() -> (
    None
):
    """A relaxed module still declares `fast_math_disable` while relaxing every operation.

    ADR 0076 item 4 depends on this: an artifact-side reader that inferred the
    delivered realization from the module flag would read the opposite of the
    licences actually applied. It holds for every family.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for contract in PROBE.FP_CONTRACTS:
            relaxed = run.of(family.name, "scale_two_bias_one", "relaxed", contract=contract)
            assert "air.compile.fast_math_disable" in relaxed.compile_options, family.name
            assert relaxed.operations, "the relaxed module must retain operations to carry flags"
            for operation in relaxed.operations:
                assert {"reassoc", "nsz", "arcp", "afn"} <= set(operation.flags), operation.flags
                assert ("contract" in operation.flags) == (contract == "fast"), (
                    f"{family.name}/relaxed/{contract} attached {operation.flags}"
                )


def test_input_and_result_flushing_are_separable_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 2. Both dimensions flush, and each is isolated by its own kernel.

    `multiply_two` doubles a subnormal whose exact result is *normal*, so a
    returned zero can only come from flushing the operand. `multiply_half`
    halves the smallest normal, so a returned zero can only come from flushing
    the result. Asserted for every family whose own GPU answered.
    """
    run = probe_run()
    assert not is_subnormal(PROBE.INPUT_FLUSH.preserving), "the input probe must isolate the input"
    assert not is_subnormal(PROBE.RESULT_FLUSH.operand), "the result probe must isolate the result"
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in ("0", "2"):
                doubled = run.of(family, "multiply_two", mode, optimization)
                halved = run.of(family, "multiply_half", mode, optimization)
                assert PROBE.subnormal_verdict(doubled, PROBE.INPUT_FLUSH) is PROBE.Verdict(
                    "flushed-to-zero"
                ), f"{family}/{mode}/O{optimization} input flush"
                assert doubled.result_for(PROBE.INPUT_FLUSH.operand) == 0x00000000
                assert PROBE.subnormal_verdict(halved, PROBE.RESULT_FLUSH) is PROBE.Verdict(
                    "flushed-to-zero"
                ), f"{family}/{mode}/O{optimization} result flush"
                assert halved.result_for(PROBE.RESULT_FLUSH.operand) == 0x00000000


def test_the_flush_preserves_the_sign_of_zero_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 3. A negative subnormal flushes to negative zero, not positive zero.

    ADR 0076 item 1 makes this load-bearing: a flush behaviour that does not
    state which zero it produces is under-specified against measured hardware.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in ("0", "2"):
                observation = run.of(family, "multiply_two", mode, optimization)
                verdict = PROBE.subnormal_verdict(observation, PROBE.NEGATIVE_INPUT_FLUSH)
                assert verdict is PROBE.Verdict.FLUSHED_TO_ZERO, f"{family}/{mode}/O{optimization}"
                result = observation.result_for(PROBE.NEGATIVE_INPUT_FLUSH.operand)
                assert result == 0x80000000, f"{family}/{mode}/O{optimization} gave {result:08x}"


def test_materialization_is_unaffected_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 4. A load and a store return every bit pattern unchanged in every mode.

    The limit is a property of arithmetic, not of materialization, which is what
    lets the Metal emitter record the gap per arithmetic statement rather than
    per kernel.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "materialize", mode).operation_count == 0, family.name
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            assert run.of(family, "materialize", mode).results == PROBE.F32.operands, (
                f"{family}/{mode}"
            )


def test_the_math_mode_changes_a_conforming_result_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 5. `MultiplyThenAdd { scale 1.0, bias +0.0 }` on negative zero diverges.

    IEEE-754 round-to-nearest requires `+0.0`, which only `safe` returns.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for optimization in ("0", "2"):
            safe = run.of(family, "scale_one_bias_zero", "safe", optimization)
            assert safe.result_for(0x80000000) == 0x00000000, f"{family}/O{optimization}"
            for mode in ("relaxed", "fast"):
                observation = run.of(family, "scale_one_bias_zero", mode, optimization)
                assert observation.result_for(0x80000000) == 0x80000000, (
                    f"{family}/{mode}/O{optimization}"
                )


def test_contraction_changes_a_conforming_result_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 6. A multiply and an add as two statements fuse only under `=fast`.

    The per-statement emission rule is therefore a measured defence against
    `-ffp-contract=on` and measurably not a defence against `=fast`.
    """
    run = probe_run()
    operand = 0x3EB97EF9
    for family in dispatched_families(run):
        for contract in ("off", "on"):
            observation = run.of(family, "contraction_pair", "safe", contract=contract)
            assert observation.result_for(operand) == 0x3FC58F9E, f"{family}/{contract}"
        fused = run.of(family, "contraction_pair", "safe", contract="fast")
        assert fused.result_for(operand) == 0x3FC58F9D, family


def test_the_canonicalization_is_not_a_contraction_barrier_when_a_toolchain_and_gpu_resolve() -> (
    None
):
    """No fusion is observed through the canonicalization, and that is not a defence.

    The control matters because the absence of fusion here is a scheduling
    outcome, not a guarantee: the same source without the canonicalization does
    fuse under the same flags, so `-ffp-contract=off` remains the only thing
    closing the case.
    """
    run = probe_run()
    operand = 0x3EB97EF9
    for family in dispatched_families(run):
        for contract in PROBE.FP_CONTRACTS:
            observation = run.of(
                family, "contraction_pair_canonicalized", "safe", contract=contract
            )
            assert observation.result_for(operand) == 0x3FC58F9E, f"{family}/{contract}"
        control = run.of(family, "contraction_pair", "safe", contract="fast")
        assert control.result_for(operand) == 0x3FC58F9D, (
            "the control must fuse, or this test proves nothing about sensitivity"
        )


def test_a_relaxed_mode_deletes_the_arithmetic_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 7. The trap, measured: relaxation removes the operation that would flush.

    `x * 1.0` folds to a copy, so the identity kernel retains nothing to flush.
    The `scale 1.0, bias +0.0` kernel retains exactly one operation under `safe`
    — the `+0.0` add, unremovable without `nsz` — and none under `relaxed`. The
    surviving add is what flushes, so the identical licence that breaks signed
    zero also deletes the operation that would have flushed. Compile-side, so
    every family answers.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "multiply_one", mode).operation_count == 0, (
                f"{family.name}/{mode}: x * 1.0 must fold"
            )
        safe = run.of(family.name, "scale_one_bias_zero", "safe")
        assert safe.operation_count == 1, family.name
        assert safe.operations[0].opcode == "fadd", family.name
        for mode in ("relaxed", "fast"):
            assert run.of(family.name, "scale_one_bias_zero", mode).operation_count == 0, (
                f"{family.name}/{mode}"
            )


def test_a_deleted_operation_never_reads_as_preservation_when_a_toolchain_and_gpu_resolve() -> None:
    """The trap is live on this row, and the guard refuses it in every configuration.

    This is the assertion that distinguishes this harness from one that
    reproduces the numbers: under `relaxed` and `fast` the bit patterns say
    "preserved" and the guard says the arithmetic cannot be shown to have run.
    """
    run = probe_run()
    probe = PROBE.IDENTITY_VALUED_FLUSH
    for family in dispatched_families(run):
        for mode in ("relaxed", "fast"):
            for optimization in ("0", "2"):
                observation = run.of(family, "scale_one_bias_zero", mode, optimization)
                assert observation.result_for(probe.operand) == probe.preserving
                assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED
                guarded = PROBE.subnormal_verdict(observation, probe)
                assert not guarded.is_evidence, (
                    f"{family}/{mode}/O{optimization} was admitted as {guarded}"
                )
        safe = run.of(family, "scale_one_bias_zero", "safe")
        assert PROBE.subnormal_verdict(safe, probe) is PROBE.Verdict.FLUSHED_TO_ZERO, (
            "the same kernel under safe must be admitted, or the guard simply refuses everything"
        )
        admitted = PROBE.subnormal_verdict(
            run.of(family, "multiply_two", "fast"), PROBE.INPUT_FLUSH
        )
        assert admitted is PROBE.Verdict.FLUSHED_TO_ZERO, (
            "the guard must still admit a witnessed observation under a relaxed mode"
        )


def test_the_emitted_operation_count_alone_is_insufficient_when_a_toolchain_and_gpu_resolve() -> (
    None
):
    """A stage below the emitted IR also deletes operations, so counting is not enough.

    At `-O0` under `relaxed` the front end still emits both operations and the
    GPU returns every operand unchanged, including negative zero. Only the
    execution witness catches that, which is why the guard has two layers rather
    than one — and why a family that can supply only the first layer can support
    no verdict at all.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in ("relaxed", "fast"):
            observation = run.of(family.name, "scale_one_bias_zero", mode, "0")
            assert observation.operation_count == 2, (
                f"{family.name}/{mode}: the front end must still emit both"
            )
    for family in dispatched_families(run):
        for mode in ("relaxed", "fast"):
            observation = run.of(family, "scale_one_bias_zero", mode, "0")
            witness = observation.kernel.witness
            assert witness is not None
            assert observation.result_for(witness.operand) == witness.deleted, f"{family}/{mode}"
            assert (
                PROBE.subnormal_verdict(observation, PROBE.IDENTITY_VALUED_FLUSH)
                is PROBE.Verdict.ARITHMETIC_NOT_EXECUTED
            ), f"{family}/{mode}"


def test_the_additive_path_flushes_its_input_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 20. An add whose subnormal operand comes straight from the buffer flushes it.

    Every other adding kernel here adds *after* a multiply, so an additive-path
    input flush was asserted by ADR 0076 and re-established by nothing. This
    kernel adds `2**-126` to the operand and nothing else, so the operand reaching
    the add is the one the buffer supplied.

    The three outcomes are distinct and only one of them is this flush.
    `00800000` is the operand flushed to a signed zero before the add, leaving
    the bias standing alone. `00400000` is the operand preserved, giving a
    subnormal sum. `00000000` would be the operand preserved and the subnormal
    *result* flushed instead, which is a different mechanism and lands on
    `unexpected-result` rather than being read as agreement.
    """
    run = probe_run()
    probe = PROBE.ADDITIVE_INPUT_FLUSH
    assert probe.flushing not in {0x00000000, 0x80000000}, (
        "this probe exists because a flush does not have to show up as a returned zero"
    )
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            for optimization in ("0", "2"):
                operations = run.of(
                    family.name, "add_smallest_normal", mode, optimization
                ).operations
                assert operations is not None
                assert [operation.opcode for operation in operations] == ["fadd"], (
                    f"{family.name}/{mode}/O{optimization}: adding a nonzero constant is an "
                    f"identity on no operand, so the add must survive: {operations}"
                )
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in ("0", "2"):
                observation = run.of(family, "add_smallest_normal", mode, optimization)
                verdict = PROBE.subnormal_verdict(observation, probe)
                assert verdict is PROBE.Verdict.FLUSHED_TO_ZERO, (
                    f"{family}/{mode}/O{optimization} additive input flush: {verdict}"
                )
                assert verdict.is_evidence
                assert observation.result_for(probe.operand) == 0x00800000
            for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                assert (
                    PROBE.subnormal_verdict(
                        run.runtime(family, "add_smallest_normal", mode, optimization), probe
                    )
                    is PROBE.Verdict.FLUSHED_TO_ZERO
                ), f"{family}/{mode}/{optimization} runtime additive input flush"


def test_a_power_of_two_division_is_not_a_division_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 15's compile-side half, and the reason the flush is measured on other divisors.

    `x / 2.0f` and `x / 0.5f` are emitted as a single `fmul` under
    `-fmetal-math-mode=safe` with `-ffp-contract=off`, which is the strictest
    selection the offline driver offers and the one where a rewrite is least
    expected. A probe that isolated the flush on those divisors would therefore
    be measuring the multiplier a second time under a division's name.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for kernel in ("divide_by_half", "divide_by_two"):
            operations = run.of(family.name, kernel, "safe").operations
            assert operations is not None
            assert [operation.opcode for operation in operations] == ["fmul"], (
                f"{family.name}/{kernel} emitted {operations}, so the rewrite this finding "
                f"records did not happen and the divisor choice below needs revisiting"
            )


def test_division_flushes_both_dimensions_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 15. A surviving `fdiv` flushes its subnormal input and its subnormal result.

    `divide_by_three_eighths` divides a subnormal by `0.375`, whose exact result
    is *normal*, so a returned zero can only come from flushing the operand.
    `divide_by_three` divides the smallest normal by `3.0`, whose exact result is
    subnormal, so a returned zero can only come from flushing the result. Neither
    divisor is a power of two, so neither is rewritten into the multiply the test
    above measures.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        operations = run.of(family.name, "divide_by_three_eighths", "safe").operations
        assert operations is not None
        assert [operation.opcode for operation in operations] == ["fdiv"], (
            f"{family.name}: the divisor must survive as a division under safe, or this "
            f"measures something else: {operations}"
        )
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            divided = run.of(family, "divide_by_three_eighths", mode)
            halved = run.of(family, "divide_by_three", mode)
            assert (
                PROBE.subnormal_verdict(divided, PROBE.DIVIDED_INPUT_FLUSH)
                is PROBE.Verdict.FLUSHED_TO_ZERO
            ), f"{family}/{mode} division input flush"
            assert (
                PROBE.subnormal_verdict(divided, PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH)
                is PROBE.Verdict.FLUSHED_TO_ZERO
            ), f"{family}/{mode} division signed zero"
            assert divided.result_for(PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH.operand) == 0x80000000
            assert (
                PROBE.subnormal_verdict(halved, PROBE.DIVIDED_RESULT_FLUSH)
                is PROBE.Verdict.FLUSHED_TO_ZERO
            ), f"{family}/{mode} division result flush"


def test_a_source_level_fma_fuses_whatever_contraction_says_when_a_toolchain_gpu_resolve() -> None:
    """Finding 16. `-ffp-contract=off` is not a defence against an `fma` written in the source.

    The fused kernel carries the identical constants as `contraction_pair`, so
    the two differ in exactly one thing. The pair returns the separately rounded
    result at `off` and `on`; the `fma` returns the fused one at every setting
    including `off`. The per-statement emission rule finding 6 records is
    therefore a rule about what the *emitter* may write, not something the
    contraction flag can enforce on its behalf.
    """
    run = probe_run()
    operand = 0x3EB97EF9
    for family in PROBE.FAMILIES:
        for contract in PROBE.FP_CONTRACTS:
            operations = run.of(family.name, "fused_pair", "safe", contract=contract).operations
            assert operations is not None
            assert [operation.opcode for operation in operations] == ["air.fma.f32"], (
                f"{family.name}/{contract}: the fused call must be visible to the operation "
                f"count, or a surviving operation reads as a deleted one: {operations}"
            )
    for family in dispatched_families(run):
        separate = run.of(family, "contraction_pair", "safe", contract="off").result_for(operand)
        fused = run.of(family, "contraction_pair", "safe", contract="fast").result_for(operand)
        assert separate != fused, "the offline control must fuse, or this test proves nothing"
        for contract in PROBE.FP_CONTRACTS:
            observation = run.of(family, "fused_pair", "safe", contract=contract)
            assert observation.result_for(operand) == fused, f"{family}/{contract}"


def test_the_relaxed_modes_reassociate_a_chain_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 17. Reassociation is observable here, and the earlier negative result is bounded.

    `(x + 2**-24) + 2**-24` returns `x` for `x = 1.0` when the parentheses are
    honoured, because each add is a tie that rounds to even, and `1.0 + 2**-23`
    when the two small terms are summed first. Under `safe` the module keeps two
    `fadd`s and the device returns the ordered value; under `relaxed` and `fast`
    the module keeps **one** and the device returns the reassociated one. The
    admissibility guard carries this exactly as it carries a subnormal claim: the
    ordered result is the operand, so only the witness on another operand
    separates an unreassociated chain from a deleted one.
    """
    run = probe_run()
    probe = PROBE.REASSOCIATION
    for family in PROBE.FAMILIES:
        ordered = run.of(family.name, "reassociation_chain", "safe").operations
        assert ordered is not None
        assert [operation.opcode for operation in ordered] == ["fadd", "fadd"], family.name
        for mode in ("relaxed", "fast"):
            relaxed = run.of(family.name, "reassociation_chain", mode).operations
            assert relaxed is not None
            assert len(relaxed) == 1 and "reassoc" in relaxed[0].flags, (
                f"{family.name}/{mode}: the two adds must fold into one carrying reassoc"
            )
    for family in dispatched_families(run):
        assert (
            PROBE.order_verdict(run.of(family, "reassociation_chain", "safe"), probe)
            is PROBE.Verdict.LEFT_TO_RIGHT
        ), family
        for mode in ("relaxed", "fast"):
            verdict = PROBE.order_verdict(run.of(family, "reassociation_chain", mode), probe)
            assert verdict is PROBE.Verdict.REASSOCIATED, f"{family}/{mode}: {verdict}"
            assert verdict.is_evidence
        for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
            assert (
                PROBE.order_verdict(
                    run.runtime(family, "reassociation_chain", "safe", optimization), probe
                )
                is PROBE.Verdict.LEFT_TO_RIGHT
            ), f"{family}/{optimization}"
            for mode in ("relaxed", "fast"):
                assert (
                    PROBE.order_verdict(
                        run.runtime(family, "reassociation_chain", mode, optimization), probe
                    )
                    is PROBE.Verdict.REASSOCIATED
                ), f"{family}/{mode}/{optimization}"


def test_the_safe_mode_preserves_contributor_order_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 31. Under `safe` the chain is folded over the contributors as written.

    Two kernels carrying the same three contributors in two orders. Under `safe`
    both keep all three `fadd`s and the device returns each one's own left-deep
    value — `00000000` for the canonical order and `40000000` for the source
    permuted twin. The twin is the perturbation: it establishes that the result
    lane moves when the contributor order moves, so the canonical kernel's value
    is a preserved order rather than a shape nothing could disturb.

    Under `relaxed` and `fast` the canonical chain keeps **no** arithmetic at all
    — the licence folds the cancelling pair away and then removes the surviving
    identity add — so those observations are inadmissible by the guard's first
    layer rather than being read as an order. That is asserted here rather than
    skipped, because "the relaxed modes did not permute" and "the relaxed modes
    deleted the question" are different facts and only one of them is true.
    """
    run = probe_run()
    probe = PROBE.PERMUTATION
    for family in PROBE.FAMILIES:
        for kernel in ("permutation_chain", "permutation_chain_reordered"):
            emitted = run.of(family.name, kernel, "safe").operations
            assert emitted is not None
            assert [operation.opcode for operation in emitted] == ["fadd"] * 3, (
                f"{family.name}/{kernel}: `safe` must keep every contributor's own add"
            )
            assert not any(operation.flags for operation in emitted), (
                f"{family.name}/{kernel}: no `safe` add may carry a relaxation flag"
            )
        for mode in ("relaxed", "fast"):
            relaxed = run.of(family.name, "permutation_chain", mode).operations
            assert relaxed is not None and not relaxed, (
                f"{family.name}/{mode}: the canonical chain's arithmetic must be gone entirely"
            )
    for family in dispatched_families(run):
        assert (
            PROBE.permutation_verdict(run.of(family, "permutation_chain", "safe"), probe)
            is PROBE.Verdict.LEFT_TO_RIGHT
        ), family
        moved = PROBE.permutation_verdict(
            run.of(family, "permutation_chain_reordered", "safe"), probe
        )
        assert moved is PROBE.Verdict.PERMUTED and moved.is_evidence, f"{family}: {moved}"
        for mode in ("relaxed", "fast"):
            assert (
                PROBE.permutation_verdict(run.of(family, "permutation_chain", mode), probe)
                is PROBE.Verdict.NO_EMITTED_ARITHMETIC
            ), f"{family}/{mode}"
        for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
            assert (
                PROBE.permutation_verdict(
                    run.runtime(family, "permutation_chain", "safe", optimization), probe
                )
                is PROBE.Verdict.LEFT_TO_RIGHT
            ), f"{family}/{optimization}"
            assert (
                PROBE.permutation_verdict(
                    run.runtime(family, "permutation_chain_reordered", "safe", optimization),
                    probe,
                )
                is PROBE.Verdict.PERMUTED
            ), f"{family}/{optimization}"


def test_the_fp32_function_mode_moves_no_measured_value_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 18. `-fmetal-math-fp32-functions=fast` changes nothing measured here.

    This closes the boundary the record carried: `prototype-metal-numerical-realization`
    reported that the signed-zero divergence also reproduces under `=fast`, and
    nothing re-established it while the flag was pinned to `precise`. It does,
    identically, on both compilation paths — and so do both flush dimensions.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            for kernel in ("multiply_two", "multiply_half", "scale_one_bias_zero"):
                precise = run.of(family.name, kernel, mode)
                relaxed_functions = run.of(family.name, kernel, mode, fp32_functions="fast")
                assert relaxed_functions.operations == precise.operations, (
                    f"{family.name}/{kernel}/{mode}: the fp32-functions selection changed the "
                    f"emitted arithmetic, which finding 18 says it does not"
                )
                assert relaxed_functions.compile_options == precise.compile_options, family.name
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for kernel in ("multiply_two", "multiply_half", "scale_one_bias_zero"):
                assert (
                    run.of(family, kernel, mode, fp32_functions="fast").results
                    == run.of(family, kernel, mode).results
                ), f"{family}/{kernel}/{mode}"
                for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                    assert (
                        run.runtime(family, kernel, mode, optimization, "fast").results
                        == run.runtime(family, kernel, mode, optimization).results
                    ), f"{family}/{kernel}/{mode}/{optimization} runtime"
        safe = run.of(family, "scale_one_bias_zero", "safe", fp32_functions="fast")
        assert safe.result_for(0x80000000) == 0x00000000, family
        for mode in ("relaxed", "fast"):
            observation = run.of(family, "scale_one_bias_zero", mode, fp32_functions="fast")
            assert observation.result_for(0x80000000) == 0x80000000, f"{family}/{mode}"


def test_the_further_optimization_levels_behave_like_o2_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 19. `-O1`, `-O3`, and `-Os` behave like `-O2`, and `-O0` remains the outlier.

    The covering set carries `scale_one_bias_zero` under `safe`, which is the
    kernel and mode where the level is known to move the surviving operation
    count; the exhaustive sweep carries all three levels across all three modes
    and the four kernels whose counts or results a level could change. That
    `-O0` is the one level at which the front end keeps arithmetic a later stage
    then removes is therefore a statement about `-O0` and not about "low
    optimization levels".
    """
    run = probe_run()
    levels = [level for level in PROBE.OPTIMIZATIONS if level not in {"0", "2"}]
    assert levels, "the widened optimization axis disappeared"
    for family in PROBE.FAMILIES:
        reference = run.of(family.name, "scale_one_bias_zero", "safe", "2")
        for level in levels:
            observation = run.of(family.name, "scale_one_bias_zero", "safe", level)
            assert observation.operations == reference.operations, f"{family.name}/O{level}"
            assert observation.compile_options == reference.compile_options, (
                f"{family.name}/{level}"
            )
        assert run.of(family.name, "scale_one_bias_zero", "safe", "0").operation_count == 2, (
            f"{family.name}: -O0 must remain the level at which both operations survive, or "
            f"finding 19 is stating a contrast that no longer exists"
        )
    for family in dispatched_families(run):
        reference = run.of(family, "scale_one_bias_zero", "safe", "2").results
        for level in levels:
            assert run.of(family, "scale_one_bias_zero", "safe", level).results == reference, (
                f"{family}/O{level}"
            )


# --------------------------------------------------------------------------
# The second dtype. Every kernel below is an `f32` kernel above with its
# constants respelled at `f16`'s boundaries and nothing else changed.
# --------------------------------------------------------------------------

F16_FLUSH_PROBES = (
    ("multiply_two_f16", "multiply_two", PROBE.INPUT_FLUSH_F16, PROBE.INPUT_FLUSH),
    (
        "multiply_two_f16",
        "multiply_two",
        PROBE.NEGATIVE_INPUT_FLUSH_F16,
        PROBE.NEGATIVE_INPUT_FLUSH,
    ),
    ("multiply_half_f16", "multiply_half", PROBE.RESULT_FLUSH_F16, PROBE.RESULT_FLUSH),
    (
        "add_smallest_normal_f16",
        "add_smallest_normal",
        PROBE.ADDITIVE_INPUT_FLUSH_F16,
        PROBE.ADDITIVE_INPUT_FLUSH,
    ),
    (
        "divide_by_three_eighths_f16",
        "divide_by_three_eighths",
        PROBE.DIVIDED_INPUT_FLUSH_F16,
        PROBE.DIVIDED_INPUT_FLUSH,
    ),
    (
        "divide_by_three_eighths_f16",
        "divide_by_three_eighths",
        PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH_F16,
        PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH,
    ),
    (
        "divide_by_three_f16",
        "divide_by_three",
        PROBE.DIVIDED_RESULT_FLUSH_F16,
        PROBE.DIVIDED_RESULT_FLUSH,
    ),
)
"""Each `f16` probe beside the `f32` probe that asks the identical question.

Pairing them is what makes the comparison a statement about the dtype. Each row
isolates one flush dimension — input, result, sign, additive path, division — in
both formats, over kernels that differ only in the width of the constants.
"""


def test_the_second_dtype_declares_the_same_denormals_disable_when_a_toolchain_gpu_resolve() -> (
    None
):
    """The module-level declaration does not vary by dtype, which is why it cannot settle one.

    `air.compile.denorms_disable` is emitted for the `f16` kernels exactly as for
    the `f32` ones, in every math mode and for every family. That is the argument
    for expecting the flush to be dtype-independent; the measurement below is the
    reason the argument does not hold. This is the compile side, so a family with
    no attached device answers it too.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for kernel in PROBE.F16_KERNELS:
            for mode in PROBE.MATH_MODES:
                observation = run.of(family.name, kernel, mode)
                assert observation.compile_options is not None, f"{family.name}/{kernel}/{mode}"
                assert "air.compile.denorms_disable" in observation.compile_options, (
                    f"{family.name}/{kernel}/{mode} declared {observation.compile_options}"
                )
        for mode in PROBE.MATH_MODES:
            wide = run.of(family.name, "multiply_two", mode)
            narrow = run.of(family.name, "multiply_two_f16", mode)
            assert narrow.compile_options == wide.compile_options, (
                f"{family.name}/{mode}: the two dtypes declared different module options, so a "
                f"per-dtype difference in the result would have a compile-side explanation"
            )
            assert [operation.opcode for operation in narrow.operations] == [
                operation.opcode for operation in wide.operations
            ], f"{family.name}/{mode}: {narrow.operations} against {wide.operations}"


def test_the_second_dtype_preserves_what_the_first_flushes_when_a_toolchain_and_gpu_resolve() -> (
    None
):
    """The headline. The subnormal flush is not dtype-independent on this row.

    Every probe below isolates one flush dimension in both dtypes over kernels
    that differ only in the width of their constants, under the identical
    two-layer guard. The `f32` observation is admitted as `flushed-to-zero` and
    the `f16` one as `preserved`, in every math mode, on both dispatchable
    families — from modules that declare `air.compile.denorms_disable`
    identically, which the test above pins.

    Both verdicts are evidence, which is the point: this is not a kernel whose
    arithmetic was deleted returning its operand. Each `f16` kernel carries an
    execution witness that reports `executed`, and the guard would have refused
    the observation otherwise.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for narrow_name, wide_name, narrow_probe, wide_probe in F16_FLUSH_PROBES:
                narrow = run.of(family, narrow_name, mode)
                wide = run.of(family, wide_name, mode)
                narrow_verdict = PROBE.subnormal_verdict(narrow, narrow_probe)
                wide_verdict = PROBE.subnormal_verdict(wide, wide_probe)
                assert wide_verdict is PROBE.Verdict.FLUSHED_TO_ZERO, (
                    f"{family}/{mode}/{wide_name}: {wide_verdict}"
                )
                assert narrow_verdict is PROBE.Verdict.PRESERVED, (
                    f"{family}/{mode}/{narrow_name}: {narrow_verdict}. If this became "
                    f"flushed-to-zero the dtype dependence has gone away and the record's "
                    f"finding 21 must be restated, not the test relaxed"
                )
                assert narrow_verdict.is_evidence and wide_verdict.is_evidence
                assert narrow.result_for(narrow_probe.operand) == narrow_probe.preserving
                assert wide.result_for(wide_probe.operand) == wide_probe.flushing


def test_the_second_dtype_flush_hypothesis_is_refuted_by_sign_when_a_toolchain_gpu_resolve() -> (
    None
):
    """A preserved negative subnormal is the reading a flushed one cannot produce.

    Finding 3 measures the `f32` flush producing `80000000` for `80400000`. The
    `f16` twin returns `8400`, the exactly doubled subnormal, so the returned
    pattern is not a signed zero at all and no sign convention reconciles the two.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            wide = run.of(family, "multiply_two", mode)
            narrow = run.of(family, "multiply_two_f16", mode)
            assert wide.result_for(0x80400000) == 0x80000000, f"{family}/{mode}"
            assert narrow.result_for(0x8200) == 0x8400, f"{family}/{mode}"
            assert narrow.result_for(0x8000) == 0x8000, (
                f"{family}/{mode}: negative zero is not subnormal and must survive"
            )


def test_the_second_dtype_materializes_unchanged_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 4's counterpart, and the control the preservation result needs.

    A load and a store of `half` returning every operand unchanged is what rules
    out the buffer round trip as the explanation for a preserved subnormal: the
    `f16` path in and out of device memory is exact, so a doubled subnormal came
    from the multiply and not from the transfer.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "materialize_f16", mode).operation_count == 0, family.name
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            assert run.of(family, "materialize_f16", mode).results == PROBE.F16.operands, (
                f"{family}/{mode}"
            )


def test_the_second_dtype_keeps_the_trap_and_the_guard_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 7 reproduces in `f16`, so the guard is doing the same work at the new width.

    The identity multiply folds, the `scale 1.0, bias +0.0` kernel keeps exactly
    one `fadd` under `safe` and none under the relaxed modes, and its unguarded
    reading under those modes is `preserved` while the guard refuses it. That
    matters more here than in `f32`: the dtype's admissible observations *are*
    `preserved`, so a guard that stopped discriminating would make the headline
    result indistinguishable from the trap.
    """
    run = probe_run()
    probe = PROBE.IDENTITY_VALUED_FLUSH_F16
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "multiply_one_f16", mode).operation_count == 0, (
                f"{family.name}/{mode}: x * 1.0h must fold"
            )
        safe = run.of(family.name, "scale_one_bias_zero_f16", "safe")
        assert safe.operation_count == 1 and safe.operations[0].opcode == "fadd", family.name
        for mode in ("relaxed", "fast"):
            assert run.of(family.name, "scale_one_bias_zero_f16", mode).operation_count == 0, (
                f"{family.name}/{mode}"
            )
        unoptimized = run.of(family.name, "scale_one_bias_zero_f16", "relaxed", "0")
        assert unoptimized.operation_count == 2, (
            f"{family.name}: -O0 must remain the level at which the front end keeps both"
        )
    for family in dispatched_families(run):
        for mode in ("relaxed", "fast"):
            for optimization in ("0", "2"):
                observation = run.of(family, "scale_one_bias_zero_f16", mode, optimization)
                assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED
                guarded = PROBE.subnormal_verdict(observation, probe)
                assert not guarded.is_evidence, (
                    f"{family}/{mode}/O{optimization} was admitted as {guarded}"
                )
        safe = run.of(family, "scale_one_bias_zero_f16", "safe")
        assert PROBE.subnormal_verdict(safe, probe) is PROBE.Verdict.PRESERVED, (
            "the same kernel under safe must be admitted, or the guard simply refuses everything"
        )
        assert safe.result_for(0x8000) == 0x0000, (
            f"{family}: the signed-zero divergence of finding 5 must reproduce in f16, or the "
            f"surviving fadd is not the operation finding 7 says it is"
        )
        # The identity multiply is refused by layer 1 before its missing witness
        # is reached, because the fold leaves a measured-empty operation list.
        # `no-execution-witness` is what the same kernel yields on the runtime
        # path, where there is no module to read; both are inadmissible and the
        # verdicts name which layer did the refusing.
        witnessless = run.of(family, "multiply_one_f16", "safe")
        assert witnessless.kernel.witness is None, family
        assert witnessless.results is not None and witnessless.operations == (), family
        assert PROBE.subnormal_verdict(witnessless, probe) is (
            PROBE.Verdict.NO_EMITTED_ARITHMETIC
        ), family
        assert PROBE.naive_verdict(witnessless, probe) is PROBE.Verdict.PRESERVED, (
            "the unguarded reading must still be 'preserved', or this kernel is not the control "
            "the preservation result needs"
        )


# --------------------------------------------------------------------------
# The third dtype, which is the one that discriminates. `bfloat16` carries
# `f32`'s exponent field, so every one of its subnormals is an `f32` subnormal
# too — unlike `f16`, whose subnormals are all `f32` normals.
# --------------------------------------------------------------------------

BF16_FLUSH_PROBES = (
    ("multiply_two_bf16", "multiply_two", PROBE.INPUT_FLUSH_BF16, PROBE.INPUT_FLUSH),
    (
        "multiply_two_bf16",
        "multiply_two",
        PROBE.NEGATIVE_INPUT_FLUSH_BF16,
        PROBE.NEGATIVE_INPUT_FLUSH,
    ),
    ("multiply_half_bf16", "multiply_half", PROBE.RESULT_FLUSH_BF16, PROBE.RESULT_FLUSH),
    (
        "add_smallest_normal_bf16",
        "add_smallest_normal",
        PROBE.ADDITIVE_INPUT_FLUSH_BF16,
        PROBE.ADDITIVE_INPUT_FLUSH,
    ),
    (
        "divide_by_three_eighths_bf16",
        "divide_by_three_eighths",
        PROBE.DIVIDED_INPUT_FLUSH_BF16,
        PROBE.DIVIDED_INPUT_FLUSH,
    ),
    (
        "divide_by_three_eighths_bf16",
        "divide_by_three_eighths",
        PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH_BF16,
        PROBE.DIVIDED_NEGATIVE_INPUT_FLUSH,
    ),
    (
        "divide_by_three_bf16",
        "divide_by_three",
        PROBE.DIVIDED_RESULT_FLUSH_BF16,
        PROBE.DIVIDED_RESULT_FLUSH,
    ),
)
"""Each `bf16` probe beside the `f32` probe asking the identical question.

The rows are the `f16` table's rows at a third width, deliberately: the three
dtypes are then read with one set of questions and any difference between them
is a difference in the answer.
"""


def bfloat_families(run: PROBE.Run) -> tuple[str, ...]:
    """Every family that both has a device and accepted a `bfloat` pipeline on it.

    Two different absences are filtered here and the record distinguishes them:
    a family with no attached device was never asked, while a family whose device
    refused pipeline creation for `bfloat` answered and refused. Neither yields a
    device-side row, so neither can be asserted on; the reason each is missing is
    printed rather than swallowed.
    """
    resolved = []
    for family in dispatched_families(run):
        support = run.environment[f"family.{family}.device_bfloat_support"]
        if support == "supported":
            resolved.append(family)
        else:
            print(f"{family} refused a bfloat pipeline: {support}", file=sys.stderr)
    return tuple(resolved)


def test_the_third_dtype_declares_the_same_denormals_disable_when_a_toolchain_gpu_resolve() -> None:
    """`bfloat` modules carry the identical declaration, and it settles nothing here either.

    This is the compile side, so every family answers it — including the one
    whose device then refuses to run the module. That a module which compiles,
    links, and declares `air.compile.denorms_disable` can still fail pipeline
    creation is itself the sharpest form of finding 22's point.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for kernel in PROBE.BF16_KERNELS:
            for mode in PROBE.MATH_MODES:
                observation = run.of(family.name, kernel, mode)
                assert observation.compile_options is not None, f"{family.name}/{kernel}/{mode}"
                assert "air.compile.denorms_disable" in observation.compile_options, (
                    f"{family.name}/{kernel}/{mode} declared {observation.compile_options}"
                )
        for mode in PROBE.MATH_MODES:
            wide = run.of(family.name, "multiply_two", mode)
            narrow = run.of(family.name, "multiply_two_bf16", mode)
            assert narrow.compile_options == wide.compile_options, (
                f"{family.name}/{mode}: the two dtypes declared different module options"
            )
            assert [operation.opcode for operation in narrow.operations] == [
                operation.opcode for operation in wide.operations
            ], f"{family.name}/{mode}: {narrow.operations} against {wide.operations}"


def test_the_third_dtype_flushes_like_the_first_when_a_toolchain_and_gpu_resolve() -> None:
    """The discriminating result: `bfloat16` flushes what `f16` preserves.

    Read against the two dtypes already measured this is not a third data point;
    it is the one that separates the two explanations finding 21 left open.

    - "narrow formats are evaluated at a wider internal precision, rounding
      once" predicts `f16` preserved — its subnormals are `f32` **normals** — and
      predicts `bf16` **flushed**, because `bf16` carries `f32`'s exponent field
      and every `bf16` subnormal is an `f32` subnormal meeting the `f32` flush.
    - "this hardware honours subnormals natively in narrow formats" predicts both
      narrow dtypes preserve.

    The second prediction is what this test refuses. Every `bf16` probe is
    admitted as `flushed-to-zero` under the identical two-layer guard that admits
    the `f16` twin as `preserved`, so the surviving explanation is the first —
    and a reviewer wanting to overturn that has to overturn a witnessed
    measurement rather than an argument.
    """
    run = probe_run()
    for family in bfloat_families(run):
        for mode in PROBE.MATH_MODES:
            for narrow_name, wide_name, narrow_probe, wide_probe in BF16_FLUSH_PROBES:
                narrow = run.of(family, narrow_name, mode)
                wide = run.of(family, wide_name, mode)
                narrow_verdict = PROBE.subnormal_verdict(narrow, narrow_probe)
                wide_verdict = PROBE.subnormal_verdict(wide, wide_probe)
                assert wide_verdict is PROBE.Verdict.FLUSHED_TO_ZERO, (
                    f"{family}/{mode}/{wide_name}: {wide_verdict}"
                )
                assert narrow_verdict is PROBE.Verdict.FLUSHED_TO_ZERO, (
                    f"{family}/{mode}/{narrow_name}: {narrow_verdict}. If this became "
                    f"preserved, the wider-internal-precision explanation of finding 21 is "
                    f"refuted and finding 24 must be restated, not the test relaxed"
                )
                assert narrow_verdict.is_evidence and wide_verdict.is_evidence
                assert narrow.result_for(narrow_probe.operand) == narrow_probe.flushing
                assert wide.result_for(wide_probe.operand) == wide_probe.flushing


def test_the_three_dtypes_do_not_agree_when_a_toolchain_and_gpu_resolve() -> None:
    """The one-line statement of the dtype axis, over the identical isolation.

    Doubling the mid subnormal returns a signed zero in `f32`, the exactly
    doubled subnormal in `f16`, and a signed zero again in `bf16`. Three formats,
    one question, two answers — which is why a target fact carrying no dtype
    cannot state any of them.
    """
    run = probe_run()
    for family in bfloat_families(run):
        for mode in PROBE.MATH_MODES:
            assert run.of(family, "multiply_two", mode).result_for(0x00400000) == 0x00000000
            assert run.of(family, "multiply_two_f16", mode).result_for(0x0200) == 0x0400
            assert run.of(family, "multiply_two_bf16", mode).result_for(0x0040) == 0x0000
            # The sign of the flushed zero, which finding 3 measures for `f32`.
            assert run.of(family, "multiply_two_bf16", mode).result_for(0x8040) == 0x8000
            assert run.of(family, "multiply_two_bf16", mode).result_for(0x8000) == 0x8000, (
                f"{family}/{mode}: negative zero is not subnormal and must survive"
            )


def test_the_third_dtype_materializes_unchanged_when_a_toolchain_and_gpu_resolve() -> None:
    """The control the flush result needs as much as the preservation result did.

    A load and a store of `bfloat` returning every operand unchanged — subnormals
    included — is what rules out the buffer round trip as the explanation for a
    flushed one. Without it, a `bfloat` transfer that quietly normalized its
    values would produce the same zeros the arithmetic does.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "materialize_bf16", mode).operation_count == 0, family.name
    for family in bfloat_families(run):
        for mode in PROBE.MATH_MODES:
            assert run.of(family, "materialize_bf16", mode).results == PROBE.BF16.operands, (
                f"{family}/{mode}: a bfloat subnormal did not survive the buffer round trip, so "
                f"the flush measured above is not attributable to the arithmetic"
            )


def test_the_third_dtype_keeps_the_trap_and_the_guard_when_a_toolchain_and_gpu_resolve() -> None:
    """The trap must still be refused at the third width, and admitted under `safe`.

    In `bf16` the admissible verdict is `flushed-to-zero`, so the trap's
    unguarded reading and the real result are different words again, as in `f32`
    and unlike `f16`. The guard is held to both directions here anyway: a guard
    that refused everything would make the flush result unreachable rather than
    wrong, and that failure is invisible from the headline test alone.
    """
    run = probe_run()
    probe = PROBE.IDENTITY_VALUED_FLUSH_BF16
    for family in PROBE.FAMILIES:
        for mode in PROBE.MATH_MODES:
            assert run.of(family.name, "multiply_one_bf16", mode).operation_count == 0, (
                f"{family.name}/{mode}: x * 1.0 in bfloat must fold"
            )
        safe = run.of(family.name, "scale_one_bias_zero_bf16", "safe")
        assert safe.operation_count == 1 and safe.operations[0].opcode == "fadd", family.name
        for mode in ("relaxed", "fast"):
            assert run.of(family.name, "scale_one_bias_zero_bf16", mode).operation_count == 0, (
                f"{family.name}/{mode}"
            )
        unoptimized = run.of(family.name, "scale_one_bias_zero_bf16", "relaxed", "0")
        assert unoptimized.operation_count == 2, (
            f"{family.name}: -O0 must remain the level at which the front end keeps both"
        )
    for family in bfloat_families(run):
        for mode in ("relaxed", "fast"):
            for optimization in ("0", "2"):
                observation = run.of(family, "scale_one_bias_zero_bf16", mode, optimization)
                guarded = PROBE.subnormal_verdict(observation, probe)
                assert not guarded.is_evidence, (
                    f"{family}/{mode}/O{optimization} was admitted as {guarded}"
                )
        safe = run.of(family, "scale_one_bias_zero_bf16", "safe")
        assert PROBE.subnormal_verdict(safe, probe) is PROBE.Verdict.FLUSHED_TO_ZERO, (
            "the same kernel under safe must be admitted, or the guard simply refuses everything"
        )
        assert safe.result_for(0x8000) == 0x0000, (
            f"{family}: the signed-zero divergence of finding 5 must reproduce in bfloat"
        )


def test_a_device_that_refused_a_dtype_is_not_a_device_that_was_absent() -> None:
    """The two reasons a `results` row can be missing must stay different classes.

    On the measured row the iOS Simulator compiles and links every `bfloat`
    module and then fails pipeline creation, while the `IOsDevice` family is
    never asked at all. Collapsing those into one word would let a future run
    where the simulator started working look identical to today's, and would let
    a real dispatch defect hide behind an absence.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        support = run.environment[f"family.{family.name}.device_bfloat_support"]
        execution = run.environment[f"family.{family.name}.execution"]
        assert support, family.name
        if execution.startswith("unavailable:"):
            assert support.startswith("unavailable:"), (
                f"{family.name}: a family with no device cannot have answered a capability probe"
            )
        for kernel in PROBE.BF16_KERNELS:
            observation = run.of(family.name, kernel, "safe")
            # The compile side runs for every family regardless, so the module
            # facts are present even where the device refused to run them.
            assert observation.compile_options is not None, f"{family.name}/{kernel}"
            if support == "supported":
                assert observation.results is not None and not observation.refusal, (
                    f"{family.name}/{kernel}"
                )
                continue
            assert observation.results is None, f"{family.name}/{kernel}"
            verdict = PROBE.subnormal_verdict(observation, PROBE.IDENTITY_VALUED_FLUSH_BF16)
            assert not verdict.is_evidence, f"{family.name}/{kernel}"
            if execution.startswith("unavailable:"):
                assert not observation.refusal, f"{family.name}/{kernel}: never asked"
                assert verdict is PROBE.Verdict.NO_DEVICE_OBSERVATION, f"{family.name}/{kernel}"
            else:
                assert observation.refusal, f"{family.name}/{kernel}: asked and refused"
                assert verdict is PROBE.Verdict.DEVICE_REFUSED_DTYPE, f"{family.name}/{kernel}"


# --------------------------------------------------------------------------
# Runtime-compilation measurements. The same kernels through
# `newLibraryWithSource:options:` instead of a linked metallib.
# --------------------------------------------------------------------------


def test_the_two_compilation_paths_agree_case_by_case_when_a_toolchain_and_gpu_resolve() -> None:
    """The headline. No case returns different bits through the two compilers.

    A divergence here would mean an artifact's declared numerical realization
    cannot be inferred from the offline build alone, because the compiler that
    actually runs the kernel is a different one. It is reported case by case
    rather than in aggregate so the failure names which family, kernel, and mode.
    """
    run = probe_run()
    comparisons = PROBE.path_comparisons(run)
    assert comparisons, "the probe produced no cross-path comparison at all"
    diverging = [
        comparison.render() for comparison in comparisons if comparison.agreement.is_divergence
    ]
    assert not diverging, (
        "the offline and runtime compilation paths returned different results. This is a "
        "load-bearing divergence, not a harness defect: report it before changing anything. "
        f"Diverging cases: {diverging}"
    )


def test_each_family_identifies_both_of_its_compilers_when_a_toolchain_and_gpu_resolve() -> None:
    """A per-family claim is worth nothing without naming both compilers behind it.

    The offline driver and the runtime compiler are separately versioned, and on
    the measured row the runtime one belongs to the *execution environment*, so
    it differs between the macOS family and the simulator family. A family whose
    runtime compiler went unidentified would let one family's provenance be read
    onto another's numbers, which is exactly the confusion these rows exist to
    prevent.
    """
    run = probe_run()
    for family in PROBE.FAMILIES:
        offline = run.environment[f"family.{family.name}.metal_version"]
        assert "metalfe-" in offline, f"{family.name}: {offline}"
    identified = set()
    for family in dispatched_families(run):
        prefix = f"family.{family}"
        build = run.environment[f"{prefix}.runtime_compiler_build"]
        images = run.environment[f"{prefix}.runtime_compiler_images"]
        assert images != "unreported", (
            f"{family} loaded no image matching {PROBE.COMPILER_IMAGE_MARKERS}, so its runtime "
            f"compiler is unidentified"
        )
        assert "metalfe-" in build, f"{family}: {build}"
        identified.add(build)
        print(
            f"{family}: offline={run.environment[f'{prefix}.metal_version']!r} "
            f"runtime={run.environment[f'{prefix}.runtime_compiler']!r} build={build!r} "
            f"images={images!r}",
            file=sys.stderr,
        )
    assert identified, "no dispatched family identified a runtime compiler"


def test_runtime_input_and_result_flushing_when_a_toolchain_and_gpu_resolve() -> None:
    """Findings 2 and 3, re-established through `newLibraryWithSource:options:`.

    The execution witness carries the admissibility decision alone here, because
    the runtime path has no readable module. It is the layer that caught the
    trap at `-O0` where counting emitted operations did not, so what is lost is
    the weaker layer; see the harness module documentation.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                doubled = run.runtime(family, "multiply_two", mode, optimization)
                halved = run.runtime(family, "multiply_half", mode, optimization)
                assert doubled.operations is None, "a runtime case must not claim a readable module"
                assert (
                    PROBE.subnormal_verdict(doubled, PROBE.INPUT_FLUSH)
                    is PROBE.Verdict.FLUSHED_TO_ZERO
                ), f"{family}/{mode}/{optimization} input flush"
                assert (
                    PROBE.subnormal_verdict(halved, PROBE.RESULT_FLUSH)
                    is PROBE.Verdict.FLUSHED_TO_ZERO
                ), f"{family}/{mode}/{optimization} result flush"
                assert (
                    PROBE.subnormal_verdict(doubled, PROBE.NEGATIVE_INPUT_FLUSH)
                    is PROBE.Verdict.FLUSHED_TO_ZERO
                ), f"{family}/{mode}/{optimization} signed zero"
                assert doubled.result_for(PROBE.NEGATIVE_INPUT_FLUSH.operand) == 0x80000000


def test_the_second_dtype_preserves_through_runtime_compilation_when_a_toolchain_gpu_resolve() -> (
    None
):
    """The dtype dependence is a property of the two compilers, not of one of them.

    Each family's runtime compiler is a different build from its offline driver
    (finding 12), so reproducing the preservation there is what stops the result
    being read as an artefact of `xcrun metal`. The execution witness carries the
    admissibility decision alone on this path, because there is no readable
    module.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                for narrow_name, wide_name, narrow_probe, wide_probe in F16_FLUSH_PROBES:
                    narrow = run.runtime(family, narrow_name, mode, optimization)
                    wide = run.runtime(family, wide_name, mode, optimization)
                    assert narrow.operations is None
                    assert PROBE.subnormal_verdict(narrow, narrow_probe) is (
                        PROBE.Verdict.PRESERVED
                    ), f"{family}/{mode}/{optimization}/{narrow_name}"
                    assert PROBE.subnormal_verdict(wide, wide_probe) is (
                        PROBE.Verdict.FLUSHED_TO_ZERO
                    ), f"{family}/{mode}/{optimization}/{wide_name}"
                assert (
                    run.runtime(family, "materialize_f16", mode, optimization).results
                    == PROBE.F16.operands
                ), f"{family}/{mode}/{optimization}"


def test_runtime_materialization_is_unaffected_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 4 through the runtime path, where no emitted-operation count backs it up."""
    run = probe_run()
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                observation = run.runtime(family, "materialize", mode, optimization)
                assert observation.results == PROBE.F32.operands, f"{family}/{mode}/{optimization}"


def test_the_runtime_math_mode_changes_a_result_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 5 through `MTLCompileOptions.mathMode` rather than `-fmetal-math-mode`.

    IEEE-754 round-to-nearest requires `+0.0` for `(-0.0) * 1.0 + (+0.0)`, and
    only `MTLMathModeSafe` returns it.
    """
    run = probe_run()
    for family in dispatched_families(run):
        for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
            safe = run.runtime(family, "scale_one_bias_zero", "safe", optimization)
            assert safe.result_for(0x80000000) == 0x00000000, f"{family}/{optimization}"
            for mode in ("relaxed", "fast"):
                observation = run.runtime(family, "scale_one_bias_zero", mode, optimization)
                assert observation.result_for(0x80000000) == 0x80000000, (
                    f"{family}/{mode}/{optimization}"
                )


def test_the_runtime_guard_still_discriminates_when_a_toolchain_and_gpu_resolve() -> None:
    """The live demonstration that stands in for the layer the runtime path lacks.

    A guard that never refuses anything is not a guard, and on this path only one
    layer is left to do the refusing. So every run must show that layer both
    refusing the trap kernel under the relaxed modes and admitting it under
    `safe`, in the same process, on results the unguarded reading calls
    `preserved` — and it must do so in every family that has a device.
    """
    run = probe_run()
    probe = PROBE.IDENTITY_VALUED_FLUSH
    for family in dispatched_families(run):
        for mode in ("relaxed", "fast"):
            for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
                observation = run.runtime(family, "scale_one_bias_zero", mode, optimization)
                assert observation.result_for(probe.operand) == probe.preserving
                assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED
                guarded = PROBE.subnormal_verdict(observation, probe)
                assert not guarded.is_evidence, (
                    f"{family}/{mode}/{optimization} was admitted as {guarded}"
                )
        admitted = PROBE.subnormal_verdict(
            run.runtime(family, "scale_one_bias_zero", "safe"), probe
        )
        assert admitted is PROBE.Verdict.FLUSHED_TO_ZERO, (
            "the same kernel under safe must be admitted, or the guard simply refuses everything"
        )
        witnessed = PROBE.subnormal_verdict(
            run.runtime(family, "multiply_two", "fast"), PROBE.INPUT_FLUSH
        )
        assert witnessed is PROBE.Verdict.FLUSHED_TO_ZERO, (
            "the guard must still admit a witnessed observation under a relaxed mode"
        )
        assert run.runtime(family, "multiply_one", "safe").kernel.witness is None, (
            "the witnessless kernel must stay witnessless, or the trap has no control"
        )


def test_the_runtime_path_does_not_contract_the_pair_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 6 has no `MTLCompileOptions` counterpart, so it is measured instead.

    `MTLCompileOptions` exposes no `-ffp-contract`, so rather than substituting a
    setting the comparison reports which offline contraction rows the runtime
    default behaves like. It behaves like `off` and `on` and not like `fast`,
    which is the separately rounded result. Recorded rather than assumed: a
    runtime path that fused would silently break the per-statement emission
    rule's only measured defence.
    """
    run = probe_run()
    operand = 0x3EB97EF9
    for family in dispatched_families(run):
        separate = run.of(family, "contraction_pair", "safe", contract="off").result_for(operand)
        fused = run.of(family, "contraction_pair", "safe", contract="fast").result_for(operand)
        assert separate != fused, "the offline control must fuse, or this test proves nothing"
        for optimization in PROBE.RUNTIME_OPTIMIZATIONS:
            observation = run.runtime(family, "contraction_pair", "safe", optimization)
            assert observation.result_for(operand) == separate, f"{family}/{optimization}"
            assert observation.result_for(operand) != fused, f"{family}/{optimization}"


def test_the_runtime_module_options_match_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 1's module-flag half, as far as the runtime path allows it to be checked.

    This is corroboration, not evidence: a serialized binary archive can only be
    tested for the presence of a byte sequence, where the offline path resolves
    the module's `air.compile_options` node properly. The per-operation fast-math
    flag list, which is the other half of finding 1, has no runtime counterpart
    at all and is not checked here. In the iOS Simulator the archive cannot be
    written at all, and the announced reason is what stands in its place.
    """
    run = probe_run()
    checked = 0
    for family in dispatched_families(run):
        for mode in PROBE.MATH_MODES:
            observation = run.runtime(family, "scale_two_bias_one", mode)
            archived = observation.archived_options
            assert archived is not None
            if archived.startswith("unavailable:"):
                print(f"archive scan unavailable for {family}/{mode}: {archived}", file=sys.stderr)
                continue
            offline = run.of(family, "scale_two_bias_one", mode).compile_options
            assert offline is not None
            assert set(archived.split()) == set(offline), (
                f"{family}/{mode}: runtime archive named {archived!r}, offline module declared "
                f"{offline!r}"
            )
            assert "air.compile.denorms_disable" in archived, f"{family}/{mode}"
            checked += 1
    if checked == 0:
        pytest.skip("no execution environment on this host could serialize a binary archive")


def test_the_host_fails_closed_on_a_bad_option_when_a_toolchain_and_gpu_resolve() -> None:
    """Every runtime row's meaning rests on this, so it is checked rather than assumed.

    A host that ignored an unrecognized selection would leave the property at its
    API default — `mathFloatingPointFunctions` defaults to `Fast`, not the
    `precise` the offline row pins — and the record would then name a
    configuration the library was not built with. A malformed manifest is
    rejected for the same reason: an entry that lost a field would dispatch
    something nobody asked for.
    """
    try:
        toolchain = PROBE.resolve()
    except PROBE.ProbeUnavailable as unavailable:
        message = f"Apple numerical probe skipped: {unavailable}"
        if os.environ.get(PROBE.REQUIRE_TOOLCHAIN) is not None:
            raise AssertionError(
                f"{PROBE.REQUIRE_TOOLCHAIN} is set, but {message}"
            ) from unavailable
        print(message, file=sys.stderr)
        pytest.skip(message)
    with tempfile.TemporaryDirectory(prefix="tiler-probe-options.") as directory:
        host = Path(directory) / "numerical_probe_host"
        toolchain.build_host(host, "macosx")
        source = Path(directory) / "probe.metal"
        source.write_text(PROBE.BY_NAME["multiply_two"].source(), encoding="utf-8")
        wide = Path(directory) / "probe_f16.metal"
        wide.write_text(PROBE.BY_NAME["multiply_two_f16"].source(), encoding="utf-8")
        accepted = "math=safe,fpfun=precise,lang=3.1,opt=default"
        entry = f"k\tf32\tsource\t{source}\t{PROBE.ENTRY_POINT}"
        rejected = (
            f"{entry}\tmath=bogus,fpfun=precise,lang=3.1,opt=default",
            f"{entry}\tmathMode=safe,fpfun=precise,lang=3.1,opt=default",
            f"{entry}\tmath=safe,fpfun=precise,lang=3.1",
            f"{entry}\tmath=safe,math=fast,fpfun=precise,lang=3.1,opt=default",
            entry,
            f"k\tf32\tlibrary\t{source}\t{PROBE.ENTRY_POINT}\t{accepted}",
            f"{entry}\t{accepted}\n{entry}\t{accepted}",
            # The dtype field is rejected on the same terms as an option: an
            # unknown one, and an absent one that shifts every later field.
            f"k\tf64\tsource\t{source}\t{PROBE.ENTRY_POINT}\t{accepted}",
            f"k\tsource\t{source}\t{PROBE.ENTRY_POINT}\t{accepted}",
            "",
        )
        manifest = Path(directory) / "manifest.tsv"
        operands = PROBE.operand_arguments()
        for body in rejected:
            manifest.write_text(f"{body}\n" if body else "", encoding="utf-8")
            result = subprocess.run(
                [str(host), "batch", str(manifest), *operands],
                check=False,
                capture_output=True,
                text=True,
            )
            assert result.returncode == 2, f"{body!r} was not rejected: {result.returncode}"
        # An entry whose dtype has no operand group must be refused before the
        # device is touched, or it would be dispatched over another dtype's
        # vector and its results recorded as if they answered the same question.
        manifest.write_text(
            f"k\tf16\tsource\t{wide}\t{PROBE.ENTRY_POINT}\t{accepted}\n", encoding="utf-8"
        )
        for arguments in (
            [str(host), "batch", str(manifest), "f32=3f800000"],
            [str(host), "batch", str(manifest), "f64=3c00"],
            [str(host), "batch", str(manifest), "f16=3c00", "f16=4000"],
            [str(host), "batch", str(manifest), "f16=zzzz"],
            [str(host), "batch", str(manifest), "f16=13c00"],
        ):
            result = subprocess.run(arguments, check=False, capture_output=True, text=True)
            assert result.returncode == 2, f"{arguments[3:]!r} was accepted: {result.returncode}"
        manifest.write_text(f"{entry}\t{accepted}\n", encoding="utf-8")
        result = subprocess.run(
            [str(host), "batch", str(manifest), *operands],
            check=False,
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0, (
            f"the accepted control was refused, so the rejections above prove nothing: "
            f"{result.stderr.strip()}"
        )
        assert f"applied={accepted}" in result.stdout, result.stdout
        assert "dtype=f32" in result.stdout, result.stdout


def test_the_retained_record_still_holds_when_a_toolchain_and_gpu_resolve() -> None:
    """Every case the checked-in record pins must reproduce on the same environment row.

    This is the anti-decay mechanism. A hand-run measurement in this repository
    stopped being true within the hour once unrelated work changed the compiled
    source, and nothing noticed. When the live environment row differs from the
    record's the comparison is announced and skipped, because a different
    toolchain build legitimately produces different values and silently
    accepting them would defeat the point. Every per-family field is part of that
    row, so a host with a different simulator runtime, or none at all, announces
    the difference instead of comparing across it.
    """
    run = probe_run()
    stored = PROBE.read_record(RECORDS[PROBE.matrix()])
    differing = {
        key: (stored.get(f"environment.{key}"), run.environment[key])
        for key in PROBE.qualifying_keys(run.environment)
        if stored.get(f"environment.{key}") != run.environment[key]
    }
    if differing:
        message = f"retained record comparison skipped, environment row differs: {differing}"
        print(message, file=sys.stderr)
        pytest.skip(message)
    assert not PROBE.matrix_mismatch(stored), "the wrong retained record was selected for this run"
    differences = PROBE.compare_record(run, stored)
    assert not differences, (
        "the retained record no longer describes this toolchain row. If the change is intended, "
        "regenerate it with `uv run python spikes/apple-targets/numerical_probe.py "
        f"--record {RECORDS[PROBE.matrix()]}` and say in the research record what moved. "
        f"Differences: {differences}"
    )
    mutated = dict(stored)
    corrupted = next(key for key in sorted(mutated) if key.endswith(".results"))
    mutated[corrupted] = "deadbeef"
    assert PROBE.compare_record(run, mutated), (
        "the record comparison accepted a corrupted row, so it cannot detect decay"
    )


def main() -> int:
    """Run every test outside pytest, reporting a skip rather than failing."""
    for name, test in sorted(globals().items()):
        if name.startswith("test_") and callable(test):
            try:
                test()
            except BaseException as error:
                if type(error).__name__ == "Skipped":
                    print(f"{name}=skipped")
                    continue
                raise
            print(f"{name}=passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
