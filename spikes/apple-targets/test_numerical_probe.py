#!/usr/bin/env python3
"""Re-establish, in the repository gate, the Apple numerical measurements ADR 0076 rests on.

Two classes of test live here and they fail on different hosts by design.

The **guard** tests are pure functions over synthetic observations. They run
everywhere, including a Linux runner with no Apple toolchain, and they pin the
one rule that separates this harness from one that reads bit patterns: an
observation whose arithmetic cannot be shown to have executed is never evidence
about arithmetic. That rule is the thing most worth protecting from a future
edit, so it must not be reachable only through a GPU.

The **measurement** tests dispatch on the local GPU and are conditional. They
resolve the toolchain first and skip when none is present, exactly as
`crates/tiler-metal/src/golden_compilation.rs` does, so the gate stays green on
a host without Xcode. Two mechanisms keep a skip from reading as a pass: the
skip reason is announced on standard error and appears in pytest's `-ra` summary
under the `when_a_toolchain_and_gpu_resolve` name suffix, and setting
`TILER_REQUIRE_METAL_TOOLCHAIN` turns the skip into a failure.
"""

from __future__ import annotations

import importlib.util
import os
import shutil
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

RECORD = HERE / "results" / "2026-07-24-numerics-xcode26.6-metal32023.883" / "record.tsv"

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


def is_subnormal(bits: int) -> bool:
    return bits & 0x7F800000 == 0 and bits & 0x007FFFFF != 0


def synthetic(
    kernel: str, operations: int, results: dict[int, int], *, flags: tuple[str, ...] = ()
) -> PROBE.Observation:
    """Build an observation without a toolchain, for the pure guard tests."""
    case = PROBE.Case(kernel, PROBE.Configuration("safe", "2", "off"))
    return PROBE.Observation(
        case=case,
        compile_options=("air.compile.denorms_disable",),
        operations=tuple(PROBE.FloatOperation("fadd", flags) for _ in range(operations)),
        results=tuple(results.get(operand, operand) for operand in PROBE.OPERANDS),
    )


# --------------------------------------------------------------------------
# Guard tests. No Apple toolchain required; these must never be skipped.
# --------------------------------------------------------------------------


def test_every_kernel_states_a_witness_that_could_prove_its_arithmetic_ran() -> None:
    """A witness must be able to separate execution from deletion, or be absent.

    Without this, a kernel could ship a witness whose executed and deleted
    results coincide, and every observation from it would read as admissible
    while proving nothing.
    """
    for kernel in PROBE.KERNELS:
        witness = kernel.witness
        if witness is None:
            continue
        assert witness.executed != witness.deleted, kernel.name
        for value in (witness.operand, witness.executed, witness.deleted):
            assert not is_subnormal(value), (
                f"{kernel.name}: a witness value may not be subnormal, or the witness would "
                f"depend on the behaviour under test"
            )


def test_a_subnormal_probe_separates_flushing_from_preserving() -> None:
    """Each probe's two candidate results must be distinguishable and correctly typed."""
    for probe in (
        PROBE.INPUT_FLUSH,
        PROBE.NEGATIVE_INPUT_FLUSH,
        PROBE.RESULT_FLUSH,
        PROBE.IDENTITY_VALUED_FLUSH,
    ):
        assert probe.preserving != probe.flushing
        assert probe.flushing in {0x00000000, 0x80000000}
        assert is_subnormal(probe.operand) or is_subnormal(probe.preserving), (
            "a subnormal probe must have a subnormal operand or a subnormal exact result"
        )


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
        source = kernel.source()
        assert f"kernel void {PROBE.ENTRY_POINT}(" in source, kernel.name
        assert f"ulong v1 = {len(PROBE.OPERANDS)}ul;" in source, kernel.name
        for constant in (kernel.scale_bits, kernel.bias_bits):
            if constant is not None:
                assert f"as_type<float>(0x{constant:08x}u)" in source, kernel.name
        assert ".0f" not in source and "e+" not in source, (
            f"{kernel.name}: a decimal float literal would put host rendering in the path"
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


def test_a_host_reporting_no_metal_device_is_classified_as_an_absent_device() -> None:
    """The one skip axis the offline driver's classification has no name for.

    `golden_compilation` compiles and links and never dispatches, so it cannot
    distinguish a host with a Metal compiler and no usable GPU. This probe can,
    and that outcome must be a skip rather than a failure.
    """
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host = Path(directory) / "host"
        host.write_text("#!/bin/sh\necho no device >&2\nexit 3\n", encoding="utf-8")
        host.chmod(0o755)
        with pytest.raises(PROBE.ProbeUnavailable) as caught:
            PROBE.dispatch(host, Path(directory) / "absent.metallib")
    assert caught.value.reason is PROBE.Reason.DEVICE


def test_a_host_that_fails_for_any_other_reason_is_a_defect_not_a_skip() -> None:
    """A dispatch that reaches the GPU and fails must never be mistaken for a skip."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host = Path(directory) / "host"
        host.write_text("#!/bin/sh\necho pipeline exploded >&2\nexit 4\n", encoding="utf-8")
        host.chmod(0o755)
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch(host, Path(directory) / "absent.metallib")


def test_a_truncated_dispatch_is_a_defect() -> None:
    """A host that returns fewer results than operands must not be silently accepted."""
    with tempfile.TemporaryDirectory(prefix="tiler-probe-host.") as directory:
        host = Path(directory) / "host"
        host.write_text("#!/bin/sh\necho device=fake\necho result=00000000\n", encoding="utf-8")
        host.chmod(0o755)
        with pytest.raises(PROBE.ProbeFailure):
            PROBE.dispatch(host, Path(directory) / "absent.metallib")


# --------------------------------------------------------------------------
# Measurement tests. Conditional on a resolved toolchain and GPU.
# --------------------------------------------------------------------------


def test_the_safe_math_mode_still_disables_denormals_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 1. `air.compile.denorms_disable` is emitted under every math mode.

    Under `safe` it appears beside `air.compile.fast_math_disable` and no
    emitted operation carries a fast-math flag, so the strictest selection the
    driver offers declares fast math disabled and denormals disabled together.
    """
    run = probe_run()
    for mode in PROBE.MATH_MODES:
        for contract in PROBE.FP_CONTRACTS:
            observation = run.of("scale_two_bias_one", mode, contract=contract)
            assert "air.compile.denorms_disable" in observation.compile_options, (
                f"{mode}/{contract} did not declare denorms_disable"
            )
    for contract in PROBE.FP_CONTRACTS:
        safe = run.of("scale_two_bias_one", "safe", contract=contract)
        assert "air.compile.fast_math_disable" in safe.compile_options
        assert "air.compile.fast_math_enable" not in safe.compile_options
        expected = () if contract != "fast" else ("contract",)
        for operation in safe.operations:
            assert operation.flags == expected, (
                f"safe/{contract} attached {operation.flags} to a {operation.opcode}"
            )
        fast = run.of("scale_two_bias_one", "fast", contract=contract)
        assert "air.compile.fast_math_enable" in fast.compile_options
        for operation in fast.operations:
            assert "nnan" in operation.flags or operation.flags == ("fast",)


def test_the_module_flag_is_not_a_summary_of_the_licences_when_a_toolchain_and_gpu_resolve() -> (
    None
):
    """A relaxed module still declares `fast_math_disable` while relaxing every operation.

    ADR 0076 item 4 depends on this: an artifact-side reader that inferred the
    delivered realization from the module flag would read the opposite of the
    licences actually applied.
    """
    run = probe_run()
    for contract in PROBE.FP_CONTRACTS:
        relaxed = run.of("scale_two_bias_one", "relaxed", contract=contract)
        assert "air.compile.fast_math_disable" in relaxed.compile_options
        assert relaxed.operations, "the relaxed module must retain operations to carry flags"
        for operation in relaxed.operations:
            assert {"reassoc", "nsz", "arcp", "afn"} <= set(operation.flags), operation.flags
            assert ("contract" in operation.flags) == (contract == "fast"), (
                f"relaxed/{contract} attached {operation.flags}"
            )


def test_input_and_result_flushing_are_separable_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 2. Both dimensions flush, and each is isolated by its own kernel.

    `multiply_two` doubles a subnormal whose exact result is *normal*, so a
    returned zero can only come from flushing the operand. `multiply_half`
    halves the smallest normal, so a returned zero can only come from flushing
    the result.
    """
    run = probe_run()
    assert not is_subnormal(PROBE.INPUT_FLUSH.preserving), "the input probe must isolate the input"
    assert not is_subnormal(PROBE.RESULT_FLUSH.operand), "the result probe must isolate the result"
    for mode in PROBE.MATH_MODES:
        for optimization in ("0", "2"):
            doubled = run.of("multiply_two", mode, optimization)
            halved = run.of("multiply_half", mode, optimization)
            assert PROBE.subnormal_verdict(doubled, PROBE.INPUT_FLUSH) is PROBE.Verdict(
                "flushed-to-zero"
            ), f"{mode}/O{optimization} input flush"
            assert doubled.result_for(PROBE.INPUT_FLUSH.operand) == 0x00000000
            assert PROBE.subnormal_verdict(halved, PROBE.RESULT_FLUSH) is PROBE.Verdict(
                "flushed-to-zero"
            ), f"{mode}/O{optimization} result flush"
            assert halved.result_for(PROBE.RESULT_FLUSH.operand) == 0x00000000


def test_the_flush_preserves_the_sign_of_zero_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 3. A negative subnormal flushes to negative zero, not positive zero.

    ADR 0076 item 1 makes this load-bearing: a flush behaviour that does not
    state which zero it produces is under-specified against measured hardware.
    """
    run = probe_run()
    for mode in PROBE.MATH_MODES:
        for optimization in ("0", "2"):
            observation = run.of("multiply_two", mode, optimization)
            verdict = PROBE.subnormal_verdict(observation, PROBE.NEGATIVE_INPUT_FLUSH)
            assert verdict is PROBE.Verdict.FLUSHED_TO_ZERO, f"{mode}/O{optimization}"
            result = observation.result_for(PROBE.NEGATIVE_INPUT_FLUSH.operand)
            assert result == 0x80000000, f"{mode}/O{optimization} returned {result:08x}"


def test_materialization_is_unaffected_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 4. A load and a store return every bit pattern unchanged in every mode.

    The limit is a property of arithmetic, not of materialization, which is what
    lets the Metal emitter record the gap per arithmetic statement rather than
    per kernel.
    """
    run = probe_run()
    for mode in PROBE.MATH_MODES:
        observation = run.of("materialize", mode)
        assert observation.operation_count == 0, mode
        assert observation.results == PROBE.OPERANDS, mode


def test_the_math_mode_changes_a_conforming_result_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 5. `MultiplyThenAdd { scale 1.0, bias +0.0 }` on negative zero diverges.

    IEEE-754 round-to-nearest requires `+0.0`, which only `safe` returns.
    """
    run = probe_run()
    for optimization in ("0", "2"):
        safe = run.of("scale_one_bias_zero", "safe", optimization)
        assert safe.result_for(0x80000000) == 0x00000000, optimization
        for mode in ("relaxed", "fast"):
            observation = run.of("scale_one_bias_zero", mode, optimization)
            assert observation.result_for(0x80000000) == 0x80000000, f"{mode}/O{optimization}"


def test_contraction_changes_a_conforming_result_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 6. A multiply and an add as two statements fuse only under `=fast`.

    The per-statement emission rule is therefore a measured defence against
    `-ffp-contract=on` and measurably not a defence against `=fast`.
    """
    run = probe_run()
    operand = 0x3EB97EF9
    for contract in ("off", "on"):
        observation = run.of("contraction_pair", "safe", contract=contract)
        assert observation.result_for(operand) == 0x3FC58F9E, contract
    fused = run.of("contraction_pair", "safe", contract="fast")
    assert fused.result_for(operand) == 0x3FC58F9D


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
    for contract in PROBE.FP_CONTRACTS:
        observation = run.of("contraction_pair_canonicalized", "safe", contract=contract)
        assert observation.result_for(operand) == 0x3FC58F9E, contract
    assert run.of("contraction_pair", "safe", contract="fast").result_for(operand) == 0x3FC58F9D, (
        "the control must fuse, or this test proves nothing about sensitivity"
    )


def test_a_relaxed_mode_deletes_the_arithmetic_when_a_toolchain_and_gpu_resolve() -> None:
    """Finding 7. The trap, measured: relaxation removes the operation that would flush.

    `x * 1.0` folds to a copy, so the identity kernel retains nothing to flush.
    The `scale 1.0, bias +0.0` kernel retains exactly one operation under `safe`
    — the `+0.0` add, unremovable without `nsz` — and none under `relaxed`. The
    surviving add is what flushes, so the identical licence that breaks signed
    zero also deletes the operation that would have flushed.
    """
    run = probe_run()
    for mode in PROBE.MATH_MODES:
        assert run.of("multiply_one", mode).operation_count == 0, f"{mode}: x * 1.0 must fold"
    safe = run.of("scale_one_bias_zero", "safe")
    assert safe.operation_count == 1
    assert safe.operations[0].opcode == "fadd"
    for mode in ("relaxed", "fast"):
        assert run.of("scale_one_bias_zero", mode).operation_count == 0, mode


def test_a_deleted_operation_never_reads_as_preservation_when_a_toolchain_and_gpu_resolve() -> None:
    """The trap is live on this row, and the guard refuses it in every configuration.

    This is the assertion that distinguishes this harness from one that
    reproduces the numbers: under `relaxed` and `fast` the bit patterns say
    "preserved" and the guard says the arithmetic cannot be shown to have run.
    """
    run = probe_run()
    probe = PROBE.IDENTITY_VALUED_FLUSH
    for mode in ("relaxed", "fast"):
        for optimization in ("0", "2"):
            observation = run.of("scale_one_bias_zero", mode, optimization)
            assert observation.result_for(probe.operand) == probe.preserving
            assert PROBE.naive_verdict(observation, probe) is PROBE.Verdict.PRESERVED
            guarded = PROBE.subnormal_verdict(observation, probe)
            assert not guarded.is_evidence, f"{mode}/O{optimization} was admitted as {guarded}"
    safe = run.of("scale_one_bias_zero", "safe")
    assert PROBE.subnormal_verdict(safe, probe) is PROBE.Verdict.FLUSHED_TO_ZERO, (
        "the same kernel under safe must be admitted, or the guard simply refuses everything"
    )
    admitted = PROBE.subnormal_verdict(run.of("multiply_two", "fast"), PROBE.INPUT_FLUSH)
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
    than one.
    """
    run = probe_run()
    for mode in ("relaxed", "fast"):
        observation = run.of("scale_one_bias_zero", mode, "0")
        assert observation.operation_count == 2, f"{mode}: the front end must still emit both"
        witness = observation.kernel.witness
        assert witness is not None
        assert observation.result_for(witness.operand) == witness.deleted, mode
        assert (
            PROBE.subnormal_verdict(observation, PROBE.IDENTITY_VALUED_FLUSH)
            is PROBE.Verdict.ARITHMETIC_NOT_EXECUTED
        ), mode


def test_the_retained_record_still_holds_when_a_toolchain_and_gpu_resolve() -> None:
    """Every case the checked-in record pins must reproduce on the same environment row.

    This is the anti-decay mechanism. A hand-run measurement in this repository
    stopped being true within the hour once unrelated work changed the compiled
    source, and nothing noticed. When the live environment row differs from the
    record's the comparison is announced and skipped, because a different
    toolchain build legitimately produces different values and silently
    accepting them would defeat the point.
    """
    run = probe_run()
    stored = PROBE.read_record(RECORD)
    differing = {
        key: (stored.get(f"environment.{key}"), run.environment[key])
        for key in PROBE.QUALIFYING
        if stored.get(f"environment.{key}") != run.environment[key]
    }
    if differing:
        message = f"retained record comparison skipped, environment row differs: {differing}"
        print(message, file=sys.stderr)
        pytest.skip(message)
    differences = PROBE.compare_record(run, stored)
    assert not differences, (
        "the retained record no longer describes this toolchain row. If the change is intended, "
        "regenerate it with `uv run --locked python spikes/apple-targets/numerical_probe.py "
        f"--record {RECORD}` and say in the research record what moved. Differences: {differences}"
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
