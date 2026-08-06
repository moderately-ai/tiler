#!/usr/bin/env python3
"""Device-free assertions for the code-domain integer decode probe.

Every test here runs on a host with no Apple toolchain, no GPU, and no `xcrun`.
That is deliberate: the parts of this experiment that can be wrong *silently* are
the exact reference arithmetic, the emitted-operation recognizer, the verdict
classifier, and the record's population — none of which needs a device, and all
of which would otherwise only ever be exercised by the run they are supposed to
be checking. The device dispatch stays hand-run, as `spikes/README.md` describes.

Run them with:

    uv run --with pytest pytest spikes/apple-targets
"""

from __future__ import annotations

import importlib.util
import re
import struct
import subprocess
import sys
from fractions import Fraction
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, HERE / filename)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


probe = _load("_decode_probe_under_test", "decode_probe.py")
validator = _load("_decode_validator_under_test", "validate_decode_record.py")


# ---------------------------------------------------------------------------
# the exact binary32 arithmetic the whole comparison rests on
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        (Fraction(0), 0x00000000),
        (Fraction(1), 0x3F800000),
        (Fraction(-1), 0xBF800000),
        (Fraction(255), 0x437F0000),
        # The smallest positive subnormal, and half of it, which ties to even and
        # therefore rounds to zero rather than to the subnormal.
        (Fraction(1, 2) ** 149, 0x00000001),
        (Fraction(1, 2) ** 150, 0x00000000),
        # Three quarters of the smallest subnormal is above the tie and rounds up.
        (Fraction(3, 4) * Fraction(1, 2) ** 149, 0x00000001),
        # The minimum normal and the largest subnormal.
        (Fraction(1, 2) ** 126, 0x00800000),
        (Fraction((1 << 23) - 1) * Fraction(1, 2) ** 149, 0x007FFFFF),
        # A tie in the normal range: 2**24 + 1 is exactly halfway between two
        # representable values and must round to the even one.
        (Fraction((1 << 24) + 1), 0x4B800000),
        (Fraction((1 << 24) + 3), 0x4B800002),
        # Overflow to infinity rather than to the largest finite value.
        (Fraction(2) ** 128, 0x7F800000),
        (-(Fraction(2) ** 128), 0xFF800000),
    ],
)
def test_the_exact_rounder_reproduces_hand_computed_binary32_patterns(value, expected):
    assert probe.round_to_binary32(value) == expected


def test_the_exact_rounder_carries_the_sign_of_a_zero_it_cannot_derive():
    assert probe.round_to_binary32(Fraction(0)) == 0x00000000
    assert probe.round_to_binary32(Fraction(0), negative_zero=True) == 0x80000000


def test_exact_value_round_trips_every_pattern_the_scale_corpus_uses():
    for scale in probe.SCALES:
        assert probe.round_to_binary32(probe.exact_value(scale.bits)) == scale.bits


def test_exact_value_refuses_a_non_finite_pattern():
    for pattern in (0x7F800000, 0xFF800000, 0x7FC00000):
        with pytest.raises(probe.ProbeFailure):
            probe.exact_value(pattern)


def test_the_rounder_agrees_with_the_platform_conversion_over_the_whole_measured_domain():
    """The exact rounder and a `binary64` round trip agree here, and that is checkable.

    The `binary64` route rounds twice, so it is not the reference. It happens to
    be exact for this experiment's operands — a product of an integer of
    magnitude at most 255 and a `binary32` needs at most 32 significand bits,
    which `binary64` holds — and asserting the agreement over the complete domain
    turns that argument into a checked property. A future scale corpus that broke
    it would fail here rather than quietly disagreeing with a reader's mental
    model.
    """
    for scale in probe.SCALES:
        left = probe.exact_value(scale.bits)
        for difference in range(-probe.CODE_MAX, probe.CODE_MAX + 1):
            exact = probe.decode_exact_difference(difference, scale.bits)
            through_double = struct.unpack("<I", struct.pack("<f", float(left) * difference))[0]
            assert exact == through_double, (scale.name, difference)


# ---------------------------------------------------------------------------
# the finite derivation this experiment exists to test
# ---------------------------------------------------------------------------


def test_a_normal_scale_makes_the_two_reference_models_identical():
    """The exhaustive-finite claim, recomputed rather than cited.

    "if the scale is a normal F32, the decode is bit-identical under
    `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` and under a
    subnormal-preserving F32" is the derivation the profile record states over 256
    codes and 256 zero points. Here it is evaluated over all 65,536 cells for every
    normal scale in the corpus.
    """
    for scale in probe.SCALES:
        if not scale.normal:
            continue
        entry = probe.reference(scale)
        assert entry.exact == entry.flushed, scale.name
        assert entry.differing_cells == ()


def test_a_subnormal_scale_makes_the_models_differ_in_every_cell_off_the_diagonal():
    """And the count is exact, which is what makes the subnormal case a measurement.

    Where the code equals its zero point the widened difference is zero and both
    models produce `+0.0`, so the models can only differ in the other 65,280
    cells — and they differ in all of them, because a subnormal scale is flushed
    before the multiply under the model and is not under the exact evaluation.
    """
    off_diagonal = probe.GRID_CELLS - (probe.CODE_MAX + 1)
    for scale in probe.SCALES:
        if scale.normal:
            continue
        entry = probe.reference(scale)
        assert len(entry.differing_cells) == off_diagonal, scale.name


def test_the_minimum_normal_scale_produces_no_subnormal_result_at_all():
    """The tightest point of the derivation, at the exact boundary.

    `|v * scale| >= scale` for every nonzero widened difference, so at the `f32`
    minimum normal every nonzero product is normal and the flush has nothing to
    act on. The smallest nonzero magnitude produced is exactly `2**-126`.
    """
    scale = probe.SCALE_BY_NAME["min_normal"]
    assert scale.bits == 0x00800000
    entry = probe.reference(scale)
    assert not any(probe.is_subnormal(bits) for bits in entry.exact)
    magnitudes = {bits & ~probe.SIGN_BIT for bits in entry.exact}
    assert min(magnitudes - {0}) == 0x00800000


def test_the_mid_subnormal_scale_separates_input_flushing_from_result_flushing():
    """Why the corpus carries `2**-127` and not only `2**-149`.

    At `2**-127` every widened difference of magnitude at least two has an
    exactly *normal* product. A device that flushed only subnormal results would
    return those products unchanged; a device that flushes subnormal inputs
    returns a signed zero. The two hypotheses therefore make different
    predictions on this scale and the same prediction on `2**-149`, which is what
    makes this the discriminating member of the corpus.
    """
    scale = probe.SCALE_BY_NAME["mid_subnormal"]
    assert not scale.normal
    normal_products = [
        difference
        for difference in range(-probe.CODE_MAX, probe.CODE_MAX + 1)
        if abs(difference) >= 2
        and not probe.is_subnormal(probe.decode_exact_difference(difference, scale.bits))
        and probe.decode_exact_difference(difference, scale.bits) & ~probe.SIGN_BIT != 0
    ]
    assert len(normal_products) == 2 * (probe.CODE_MAX - 1)
    for difference in normal_products:
        flushed = probe.decode_flushed_difference(difference, scale.bits)
        assert flushed & ~probe.SIGN_BIT == 0, difference


def test_code_equal_to_zero_point_produces_positive_zero_under_both_models():
    """The registered exceptional contract, over the whole diagonal and every scale."""
    for scale in probe.SCALES:
        entry = probe.reference(scale)
        for code in range(probe.CODE_MIN, probe.CODE_MAX + 1):
            cell = code * (probe.CODE_MAX + 1) + code
            assert entry.exact[cell] == 0x00000000, (scale.name, code)
            assert entry.flushed[cell] == 0x00000000, (scale.name, code)


def test_no_reference_value_can_be_the_seeded_sentinel():
    """The host distinguishes an unwritten cell from a written zero by this pattern.

    The corpus is finite, so the claim is checked rather than argued — and it has
    to be, because the decode writes a genuine zero for the whole diagonal.
    """
    for entry in probe.references().values():
        assert probe.SENTINEL not in entry.exact
        assert probe.SENTINEL not in entry.flushed


def test_the_workload_scale_corpus_spans_the_measured_range_and_the_boundary():
    """The ticket's stated inputs, held to rather than assumed."""
    values = {scale.name: float(probe.exact_value(scale.bits)) for scale in probe.SCALES}
    assert values["workload_min"] == pytest.approx(1.358e-5, rel=1e-6)
    assert values["profile_min"] == pytest.approx(2.352e-5, rel=1e-6)
    assert values["workload_max"] == pytest.approx(1.536e-1, rel=1e-6)
    assert probe.SCALE_BY_NAME["min_normal"].bits == 0x00800000
    assert any(not scale.normal for scale in probe.SCALES)
    assert probe.SCALE_BY_NAME["unit"].bits == 0x3F800000


# ---------------------------------------------------------------------------
# the emitted-operation recognizer, pinned against modules this toolchain emitted
# ---------------------------------------------------------------------------

EMITTED_O2 = """\
  %6 = icmp ult i32 %4, 65536
  br i1 %6, label %7, label %20
  %8 = zext i32 %4 to i64
  %9 = getelementptr inbounds i8, i8 addrspace(1)* %0, i64 %8
  %10 = load i8, i8 addrspace(1)* %9, align 1, !tbaa !24, !alias.scope !27, !noalias !30
  %13 = zext i8 %10 to i32
  %14 = zext i8 %12 to i32
  %15 = sub nsw i32 %13, %14
  %16 = tail call float @air.convert.f.f32.s.i32(i32 %15) #2
  %17 = load float, float addrspace(1)* %2, align 4, !tbaa !36
  %18 = fmul float %16, %17
  store float %18, float addrspace(1)* %19, align 4, !tbaa !36
declare float @air.convert.f.f32.s.i32(i32) local_unnamed_addr #1
"""
"""Copied verbatim from the `-O2` module `xcrun metal 32023.883` emitted for this kernel."""

EMITTED_O0 = """\
  %23 = zext i32 %22 to i64
  %26 = icmp ult i64 %24, %25
  %27 = zext i1 %26 to i8
  %29 = trunc i8 %28 to i1
  %40 = zext i8 %39 to i32
  %42 = zext i8 %41 to i32
  %45 = sub nsw i32 %43, %44
  %47 = call float @air.convert.f.f32.s.i32(i32 %46) #1
  %53 = fmul float %51, %52
"""
"""Copied verbatim from the `-O0` module the same command emitted."""


def test_the_recognizer_reads_the_int_to_float_conversion_this_front_end_actually_emits():
    """The conversion is a call, not `sitofp`, and reading it as absent would be silent.

    `air.convert.f.f32.s.i32` is what this front end lowers `float(int)` to. A
    recognizer naming only the LLVM conversion opcodes would report the
    conversion stage missing from every module — indistinguishable from a stage a
    compiler deleted, which is the reading a reader would act on.
    """
    assert probe.operations(EMITTED_O2) == (
        "zext:i32-to-i64",
        "zext:i8-to-i32",
        "zext:i8-to-i32",
        "sub+nsw:i32",
        "call:air.convert.f.f32.s.i32",
        "fmul:float",
    )


def test_the_recognizer_reads_the_unoptimized_module_including_its_boolean_plumbing():
    assert probe.operations(EMITTED_O0) == (
        "zext:i32-to-i64",
        "zext:i1-to-i8",
        "trunc:i8-to-i1",
        "zext:i8-to-i32",
        "zext:i8-to-i32",
        "sub+nsw:i32",
        "call:air.convert.f.f32.s.i32",
        "fmul:float",
    )


def test_the_recognizer_can_report_a_missing_stage():
    """A guard that never refuses anything is not a guard."""
    without_subtraction = "\n".join(
        line for line in EMITTED_O2.splitlines() if "sub nsw" not in line
    )
    assert "sub+nsw:i32" not in probe.operations(without_subtraction)
    assert probe.operations("") == ()


def test_the_recognizer_carries_fast_math_flags_and_does_not_count_a_comparison():
    assert probe.operations("  %2 = fmul reassoc nsz float %0, %1\n") == ("fmul+reassoc+nsz:float",)
    assert probe.operations("  %2 = fcmp oeq float %0, %1\n") == ()
    assert probe.operations("  %2 = icmp ult i32 %0, 65536\n") == ()


def test_the_compile_option_reader_resolves_the_attached_node_only():
    module = (
        "!air.compile_options = !{!17, !18}\n"
        '!17 = !{!"air.compile.denorms_disable"}\n'
        '!18 = !{!"air.compile.fast_math_disable"}\n'
        '!19 = !{!"air.compile.fast_math_enable"}\n'
    )
    assert probe.compile_options(module) == (
        "air.compile.denorms_disable",
        "air.compile.fast_math_disable",
    )


# ---------------------------------------------------------------------------
# the kernel, the grid, and the populations
# ---------------------------------------------------------------------------


def test_the_kernel_spells_every_decode_stage_as_its_own_statement():
    """The registered evaluation, one statement per operation, as the emitter writes it."""
    source = probe.kernel_source()
    for statement in (
        "uchar v3 = b0[v0];",
        "uchar v4 = b1[v0];",
        "int v5 = int(v3);",
        "int v6 = int(v4);",
        "int v7 = v5 - v6;",
        "float v8 = float(v7);",
        "float v9 = b2[0];",
        "float v10 = v8 * v9;",
    ):
        assert statement in source, statement


def test_no_decode_operand_is_a_compile_time_constant():
    """What replaces the numerical probe's execution witness, and why it is stronger.

    Every operand of the arithmetic under test arrives in a buffer, so no stage of
    either compiler can fold it. The numerical probe needs a two-layer guard
    precisely because its operands are immediates; here the same protection is a
    property of the kernel, and this test is what keeps it one.
    """
    source = probe.kernel_source()
    assert "as_type" not in source
    body = source.split("if (v2) {", 1)[1]
    bindings = {}
    for line in body.splitlines():
        found = re.fullmatch(r"(?:\w+) (v\d+) = (.+);", line.strip())
        if found is not None:
            bindings[found.group(1)] = found.group(2)
    assert "b3[v0] = v10;" in body

    loaded = set()

    def walk(name: str) -> None:
        expression = bindings[name]
        if re.fullmatch(r"b\d\[\w+\]", expression):
            loaded.add(expression)
            return
        operands = re.findall(r"\bv\d+\b", expression)
        assert operands, expression
        if any(operator in expression for operator in ("-", "*")):
            # An arithmetic expression may name bound values and nothing else; a
            # literal operand here is exactly what a compiler could fold against.
            assert re.fullmatch(r"v\d+ [-*] v\d+", expression), expression
        for operand in operands:
            walk(operand)

    walk("v10")
    assert loaded == {"b0[v0]", "b1[v0]", "b2[0]"}


def test_the_grid_covers_the_complete_code_and_zero_point_domain_exactly_once():
    codes = probe.codes()
    zeros = probe.zero_points()
    assert len(codes) == len(zeros) == probe.GRID_CELLS
    assert len(set(zip(codes, zeros, strict=True))) == probe.GRID_CELLS
    assert set(codes) == set(range(probe.CODE_MIN, probe.CODE_MAX + 1))
    assert set(zeros) == set(range(probe.CODE_MIN, probe.CODE_MAX + 1))


def test_every_witness_names_a_distinct_cell_inside_the_grid():
    cells = [witness.cell for witness in probe.WITNESSES]
    assert len(set(cells)) == len(cells)
    for witness in probe.WITNESSES:
        assert 0 <= witness.cell < probe.GRID_CELLS
        assert probe.codes()[witness.cell] == witness.code
        assert probe.zero_points()[witness.cell] == witness.zero_point


def test_the_case_population_is_both_paths_at_every_level_and_every_scale():
    keys = [case.key for case in probe.cases()]
    assert len(keys) == len(set(keys))
    assert len(keys) == (
        len(probe.OFFLINE_OPTIMIZATIONS) + len(probe.RUNTIME_OPTIMIZATIONS)
    ) * len(probe.SCALES)
    assert sum(1 for case in probe.cases() if case.path is probe.Compilation.OFFLINE) == len(
        probe.OFFLINE_OPTIMIZATIONS
    ) * len(probe.SCALES)


# ---------------------------------------------------------------------------
# the verdict classifier, held to being able to say each of its four words
# ---------------------------------------------------------------------------


def _observation(case, returned):
    return probe.Observation(
        case=case, returned=returned, applied=None, options=None, emitted=None
    )


def test_the_verdict_reports_agreement_only_where_the_models_agree():
    scale = probe.SCALE_BY_NAME["workload_max"]
    entry = probe.reference(scale)
    case = probe.Case(probe.Compilation.OFFLINE, "O2", scale.name)
    assert probe.verdict(_observation(case, entry.exact), entry) is probe.Verdict.BOTH_MODELS_AGREE


def test_the_verdict_separates_the_two_models_where_they_disagree():
    scale = probe.SCALE_BY_NAME["min_subnormal"]
    entry = probe.reference(scale)
    case = probe.Case(probe.Compilation.OFFLINE, "O2", scale.name)
    assert (
        probe.verdict(_observation(case, entry.flushed), entry)
        is probe.Verdict.FLUSH_WHERE_MODELS_DIFFER
    )
    assert (
        probe.verdict(_observation(case, entry.exact), entry)
        is probe.Verdict.EXACT_WHERE_MODELS_DIFFER
    )


def test_one_wrong_cell_out_of_the_whole_grid_is_a_divergence_named_by_its_inputs():
    """The stop condition's other branch, demonstrated rather than assumed reachable."""
    scale = probe.SCALE_BY_NAME["workload_max"]
    entry = probe.reference(scale)
    case = probe.Case(probe.Compilation.OFFLINE, "O2", scale.name)
    perturbed = list(entry.exact)
    perturbed[1] ^= 1
    observation = _observation(case, tuple(perturbed))
    assert probe.verdict(observation, entry) is probe.Verdict.DIVERGENT
    named = probe.divergences(observation, entry)
    assert len(named) == 1
    assert named[0].startswith("code=1,zero_point=0,")


def test_an_observation_of_the_wrong_size_is_refused():
    scale = probe.SCALE_BY_NAME["unit"]
    case = probe.Case(probe.Compilation.OFFLINE, "O2", scale.name)
    with pytest.raises(probe.ProbeFailure):
        _observation(case, (0,) * (probe.GRID_CELLS - 1))


def test_the_derivation_predicts_agreement_for_normal_scales_and_flushing_for_subnormal_ones():
    for scale in probe.SCALES:
        predicted = probe.reference(scale).predicted
        if scale.normal:
            assert predicted is probe.Verdict.BOTH_MODELS_AGREE, scale.name
        else:
            assert predicted is probe.Verdict.FLUSH_WHERE_MODELS_DIFFER, scale.name


# ---------------------------------------------------------------------------
# the record and its validator, over a synthetic run
# ---------------------------------------------------------------------------


def _git_available() -> bool:
    resolved = subprocess.run(
        ["git", "-C", str(probe.REPOSITORY), "rev-parse", "HEAD"],
        check=False,
        capture_output=True,
        text=True,
    )
    return resolved.returncode == 0


@pytest.fixture(scope="module")
def synthetic_run():
    """A complete run whose every case delivered exactly what the derivation predicts.

    No device is involved: each case's returned grid is the reference model its
    scale predicts. That is deliberately the *passing* shape, so every rejection
    below is a rejection of one named mutation of a record that was otherwise
    valid, rather than of a record that was broken in several ways at once.
    """
    referenced = probe.references()
    observations = {}
    for case in probe.cases():
        entry = referenced[case.scale]
        returned = entry.exact if entry.scale.normal else entry.flushed
        observations[case.key] = probe.Observation(
            case=case,
            returned=returned,
            applied=(
                None
                if case.path is probe.Compilation.OFFLINE
                else f"math={probe.MATH_MODE},fpfun={probe.FP32_FUNCTIONS},"
                f"lang={probe.RUNTIME_LANGUAGE},opt={case.level}"
            ),
            options=("air.compile.denorms_disable",)
            if case.path is probe.Compilation.OFFLINE
            else None,
            emitted=("sub+nsw:i32", "call:air.convert.f.f32.s.i32", "fmul:float")
            if case.path is probe.Compilation.OFFLINE
            else None,
        )
    environment = {
        "date_utc": "2026-07-31T00:00:00Z",
        "os_version": "27.0",
        "os_build": "26A5388g",
        "machine": "arm64",
        "xcode": "Xcode 26.6 Build version 17F113",
        "metal_platform": "MetalPlatform::MacOs",
        "sdk": "macosx",
        "sdk_version": "26.5",
        "sdk_build": "25F70",
        "requested_target": probe.TARGET,
        "metal_version": "Apple metal version 32023.883 (metalfe-32023.883)",
        "metallib_version": "AIR-LLD 32023.883 (metalfe-32023.883)",
        "execution": "macos-host-gpu",
        "emitted_triple": "air64_v28-apple-macosx26.0.0",
        "device": "Apple M4 Max",
        "device_registry_id": "4294968452",
        "device_apple9_support": "supported",
        "runtime_compiler_images": "/System/Library/PrivateFrameworks/GPUCompiler.framework/x",
        "runtime_compiler_build": "metalfe-32023.921",
    }
    return probe.Run(observations, referenced, environment, "air64_v28-apple-macosx26.0.0")


@pytest.fixture(scope="module")
def retained(tmp_path_factory, synthetic_run):
    if not _git_available():
        pytest.skip("the retained-record checks need git to resolve the recorded revision")
    destination = tmp_path_factory.mktemp("retained") / "result"
    probe.write_result(synthetic_run, destination)
    return destination


def _rewrite(record: Path, key: str, value: str | None) -> None:
    """Replace or delete one row of a retained record in place."""
    lines = []
    for line in record.read_text(encoding="utf-8").splitlines():
        name, _, _ = line.partition("\t")
        if name != key:
            lines.append(line)
        elif value is not None:
            lines.append(f"{key}\t{value}")
    record.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")


def test_the_synthetic_record_validates_and_agrees_with_the_derivation_everywhere(retained):
    assert validator.main([str(retained / "record.tsv")]) == 0
    rows = validator.read_rows(retained / "record.tsv")
    for case in probe.cases():
        assert rows[f"case.{case.key}.agrees_with_derivation"] == "yes"


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("schema", "tiler.apple-code-domain-integer-decode/v0"),
        ("probe.profile", "apple8-f32-msl3-macos13"),
        ("probe.grid_cells", "256"),
        ("probe.sentinel", "00000000"),
        ("probe.population.cases", "1"),
        ("probe.population.dispatched_cells", "1"),
        ("environment.device_apple9_support", "unsupported"),
        ("environment.device", None),
        ("probe.scale.min_normal.bits", "00400000"),
        ("reference.unit.exact_sha256", "0" * 64),
        ("reference.min_subnormal.models_differ", "0"),
        ("reference.unit.derivation_predicts", "matches-flush-model-where-models-differ"),
        ("case.offline.O2.unit.exact_matches", "65535"),
        ("case.offline.O2.unit.verdict", "divergent"),
        ("case.offline.O2.unit.agrees_with_derivation", "no"),
        ("case.offline.O2.unit.witness.difference_maximum", "returned=00000000,"
         "exact=00000000,flush=00000000"),
        ("case.offline.O2.unit.cells", None),
        ("case.runtime.default.unit.applied", "math=fast,fpfun=fast,lang=3.1,opt=default"),
        ("comparison.default.unit", "agreed"),
        ("comparison.size.min_normal", None),
        ("probe.harness_sha256", "0" * 64),
        ("probe.validator_sha256", "0" * 64),
        ("probe.repository_base_revision", "0" * 40),
        ("probe.status", "unvalidated"),
    ],
)
def test_the_validator_rejects_one_mutated_row_at_a_time(tmp_path, retained, key, value):
    """Each mutation is one edit to an otherwise valid record, and each must be caught.

    A record's rows are only evidence if a wrong one is refused, so every row kind
    the record carries is perturbed here and watched to fail. The population rows
    matter most: a check that stopped running would report the same "no
    disagreement" a check that ran and found none does.
    """
    copy = tmp_path / key.replace(".", "_")
    copy.mkdir()
    for entry in retained.iterdir():
        if entry.is_dir():
            (copy / entry.name).mkdir()
            for child in entry.iterdir():
                (copy / entry.name / child.name).write_bytes(child.read_bytes())
        else:
            (copy / entry.name).write_bytes(entry.read_bytes())
    _rewrite(copy / "record.tsv", key, value)
    assert validator.main([str(copy / "record.tsv")]) == 2


def test_the_validator_rejects_a_record_whose_retained_source_is_not_the_producers(
    tmp_path, retained
):
    copy = tmp_path / "source"
    copy.mkdir()
    (copy / "sources").mkdir()
    for entry in retained.iterdir():
        if entry.is_dir():
            for child in entry.iterdir():
                (copy / entry.name / child.name).write_bytes(child.read_bytes())
        else:
            (copy / entry.name).write_bytes(entry.read_bytes())
    kernel = copy / "sources" / "decode_strict_affine_u8.metal"
    kernel.write_text(kernel.read_text(encoding="utf-8") + "// edited\n", encoding="utf-8")
    assert validator.main([str(copy / "record.tsv")]) == 2


def test_the_validator_rejects_an_extra_retained_source(tmp_path, retained):
    copy = tmp_path / "extra"
    copy.mkdir()
    (copy / "sources").mkdir()
    for entry in retained.iterdir():
        if entry.is_dir():
            for child in entry.iterdir():
                (copy / entry.name / child.name).write_bytes(child.read_bytes())
        else:
            (copy / entry.name).write_bytes(entry.read_bytes())
    (copy / "sources" / "unlisted.metal").write_text("", encoding="utf-8")
    assert validator.main([str(copy / "record.tsv")]) == 2


def test_the_validator_rejects_a_divergent_verdict_that_names_no_cell(tmp_path, synthetic_run):
    """A divergence must arrive with its exact inputs, which is the ticket's stop condition."""
    if not _git_available():
        pytest.skip("the retained-record checks need git to resolve the recorded revision")
    scale = probe.SCALE_BY_NAME["workload_max"]
    case = probe.Case(probe.Compilation.OFFLINE, "O2", scale.name)
    entry = synthetic_run.referenced[scale.name]
    perturbed = list(entry.exact)
    perturbed[7] ^= 1
    observations = dict(synthetic_run.observations)
    observations[case.key] = probe.Observation(
        case=case,
        returned=tuple(perturbed),
        applied=None,
        options=("air.compile.denorms_disable",),
        emitted=("sub+nsw:i32",),
    )
    diverging = probe.Run(
        observations, synthetic_run.referenced, synthetic_run.environment, "triple"
    )
    destination = tmp_path / "diverging"
    probe.write_result(diverging, destination)
    rows = validator.read_rows(destination / "record.tsv")
    assert rows[f"case.{case.key}.verdict"] == probe.Verdict.DIVERGENT.value
    assert rows[f"case.{case.key}.agrees_with_derivation"] == "no"
    assert rows[f"divergence.{case.key}.0"].startswith("code=7,zero_point=0,")
    _rewrite(destination / "record.tsv", f"divergence.{case.key}.0", None)
    assert validator.main([str(destination / "record.tsv")]) == 2


def test_the_producer_refuses_to_overwrite_an_existing_result_directory(tmp_path, synthetic_run):
    if not _git_available():
        pytest.skip("the retained-record checks need git to resolve the recorded revision")
    destination = tmp_path / "twice"
    probe.write_result(synthetic_run, destination)
    with pytest.raises(probe.ProbeFailure):
        probe.write_result(synthetic_run, destination)


def test_a_record_row_carrying_a_tab_is_refused_rather_than_split(synthetic_run):
    """A diagnostic with a tab in it would silently become two rows."""
    environment = dict(synthetic_run.environment)
    environment["device"] = "Apple\tM4 Max"
    tampered = probe.Run(
        synthetic_run.observations, synthetic_run.referenced, environment, "triple"
    )
    with pytest.raises(probe.ProbeFailure):
        probe.record_rows(tampered)
