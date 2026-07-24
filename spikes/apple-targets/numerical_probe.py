#!/usr/bin/env python3
"""Reproduce the Apple GPU floating-point behaviour ADR 0076 depends on.

The record ADR 0076 builds on was measured by a hand-built Objective-C host that
was never checked in, so nothing re-established it. This module is that harness,
owned by the repository: it generates probe kernels in the emitter's output
shape, compiles them offline through `xcrun metal` and `xcrun metallib`, reads
the emitted LLVM IR, dispatches the linked library on the local GPU through
`numerical_probe_host.m`, and classifies what came back.

Scope note. Every value this module produces is qualified by one host, one GPU,
and the two compiler builds that host resolves. `environment()` captures that row
and `write_record` stores it beside the observations, because none of these
observations is a portable guarantee about Metal.

# Two compilation paths, compared case by case

Tiler's Metal story has two compilation stages: `xcrun metal` at build time and
runtime pipeline creation through a command stream. An artifact's declared
numerical realization has to be true of whichever one actually runs, so the same
generated source bytes go through both here — offline through `xcrun`, and in
process through `newLibraryWithSource:options:` with an explicit
`MTLCompileOptions` — and `path_comparisons` pairs the two case by case rather
than in aggregate.

These are not the same compiler. On the measured row the offline driver reports
`metalfe-32023.883` and the library the runtime path compiles embeds
`metalfe-32023.921`; `environment()` records the latter so the pair is never
mistaken for one compiler invoked twice.

`MTLCompileOptions` also exposes a different surface from the offline flag set.
`OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART` names every offline selection with
no property to set, and the harness records the gap rather than substituting the
nearest thing: `RUNTIME_PAIRED_OPTIMIZATION` explains which offline row a runtime
case is paired against, and a runtime result that matches only some of its
offline candidates is reported as a *measurement of the missing axis* rather
than as a disagreement between the paths.

# The reason a returned bit pattern is not, by itself, evidence

A relaxed math mode can make a kernel *look* like it honours a strict contract
by deleting the arithmetic that would have violated it. `x * 1.0` is such a
kernel: it is an identity on every operand, so a subnormal operand returns
unchanged whether the multiply flushed it or was never executed. Concluding
"subnormals are preserved" from it infers the wrong fact, and does so precisely
under the modes least worth trusting.

Counting floating-point operations in the emitted LLVM IR is necessary and
**not sufficient**, which this harness measured rather than assumed. At `-O0`
under `relaxed` the `scale 1.0, bias +0.0` kernel still carries two
floating-point operations in the front end's IR and the GPU nonetheless returns
every operand unchanged, so a later stage — the AIR-to-ISA compilation the
driver performs at pipeline creation — removed them after the IR this harness
can read. So the guard has two layers, and `subnormal_verdict` applies both:

1. the emitted module must contain at least one floating-point operation; and
2. the same kernel, in the same configuration, must return an **execution
   witness**: a designated non-subnormal operand whose result differs from the
   operand exactly when the arithmetic ran.

A kernel with no possible witness — one that is an identity on every operand —
can never support a preservation claim from this harness at all, and
`Kernel.witness` is `None` for exactly those kernels. Whether such a kernel's
operations were deleted or special-cased in hardware is not distinguished here,
and does not need to be: neither supports a claim about what arithmetic does.

## What replaces layer 1 on the runtime path, which has no readable IR

`newLibraryWithSource:options:` returns an opaque `MTLLibrary`. There is no
emitted module to read, so layer 1 is **unavailable** on that path and the
harness says so rather than substituting something for it:

- `Observation.operations` is `None`, never `()`, for a runtime observation.
  `()` would assert a measured absence of arithmetic; `None` records that the
  question was never asked. `subnormal_verdict` skips layer 1 only for `None`,
  and `record_rows` omits the `float_operations` row entirely instead of writing
  an empty one, so no reader of the record can mistake the two.
- Layer 2 carries the admissibility decision alone, which is sound because layer
  2 is device-side and *sufficient* where layer 1 is compile-side and merely
  *necessary*: an observation layer 1 would reject emitted no arithmetic, so
  nothing ran, so the kernel returns its operands, so layer 2 rejects it as
  `arithmetic-not-executed`. The converse fails, and this harness measured it
  failing — at `-O0` layer 1 passed with two emitted operations and layer 2 was
  the layer that caught the deletion. The layer being lost is the one that
  demonstrably did not catch the hard case.
- A guard that never refuses anything is not a guard, so the runtime path must
  keep demonstrating that layer 2 still discriminates *on that path*: the trap
  kernel is admitted under `safe` and refused under `relaxed` and `fast` in the
  same run. That live discrimination is what stands in for layer 1's assurance.

`scan_archive` recovers what little does survive the runtime path, and is
deliberately not part of the guard. A serialized `MTLBinaryArchive` embeds the
runtime compiler's version string and the module's `air.compile.*` option names,
but the container has no published layout and its string table is stored
concatenated without separators, so the harness can only test it for the
presence of a byte sequence. Presence is decidable; the option *set* is not, and
neither is attachment to the module's `air.compile_options` node, which the
offline path resolves properly. It is corroboration, not evidence.
"""

from __future__ import annotations

import argparse
import enum
import hashlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
HOST_SOURCE = HERE / "numerical_probe_host.m"

SCHEMA = "tiler.apple-numerical-behaviour/v2"
"""Record format identity. Bump this whenever a key's meaning changes.

v2 adds the runtime-compilation path. `case.*.float_operations` and
`case.*.compile_options` became conditional rather than universal — a case
compiled at runtime has neither, because nothing readable survives that path —
and `comparison.*`, `environment.runtime_compiler`, `case.*.applied_options`,
and `case.*.archived_options` are new.
"""

REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"
"""Turns an absent toolchain, SDK, or GPU from a skip into a failure.

This is deliberately the same variable `crates/tiler-metal/src/golden_compilation.rs`
reads, so one ambient input makes every conditional Apple check in the
repository strict. It can only make this harness stricter; nothing here lets an
environment variable weaken a check.
"""

TARGET = "air64-apple-macos13.0"
MSL_VERSION = "metal3.1"
FP32_FUNCTIONS = "precise"
ENTRY_POINT = "tiler_probe"

OPERANDS: tuple[int, ...] = (
    0x00000001,  # smallest positive subnormal
    0x00400000,  # mid subnormal; doubling it is the smallest normal
    0x007FFFFF,  # largest subnormal
    0x00800000,  # smallest positive normal; halving it is subnormal
    0x80400000,  # negative mid subnormal, for the sign of the flushed zero
    0x80000000,  # negative zero, which is not subnormal
    0x3EB97EF9,  # an ordinary normal whose scale-then-bias result reveals fusion
    0x3F800000,  # 1.0, the execution witness for the scaling kernels
)
"""The one operand vector every dispatch uses, so one launch answers every case."""

MATH_MODES = ("safe", "relaxed", "fast")
FP_CONTRACTS = ("off", "on", "fast")

RUNTIME_LANGUAGE = "3.1"
"""`MTLLanguageVersion3_1`, the exact counterpart of the offline `-std=metal3.1`."""

RUNTIME_OPTIMIZATIONS = ("default", "size")
"""`MTLLibraryOptimizationLevel`, which is the whole optimization surface here.

Neither value is `-O0`. The offline `-O0` cases therefore have no runtime
counterpart at all, which is why the `-O0` refinement of finding 7 stays an
offline-only measurement.
"""

OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART = (
    "-target: MTLCompileOptions has no target property; the runtime compiler "
    "targets the device and OS it is running on",
    "-ffp-contract: MTLCompileOptions has no contraction property; the source-level "
    "`#pragma METAL fp contract(...)` is accepted by this front end but changing the "
    "source would break the byte-identical pairing the comparison depends on",
    "-O0: MTLLibraryOptimizationLevel offers Default and Size only",
)
"""Every offline selection with no `MTLCompileOptions` property, and what is there instead.

Enumerated by reading the complete `@interface MTLCompileOptions` in
`Metal.framework/Headers/MTLLibrary.h` of macOS SDK 26.5, not by searching it.
`mathMode`, `mathFloatingPointFunctions`, and `languageVersion` are exact
counterparts of `-fmetal-math-mode`, `-fmetal-math-fp32-functions`, and `-std`;
`preprocessorMacros` has no offline selection in use here to correspond to.
"""

RUNTIME_PAIRED_OPTIMIZATION = "2"
"""The offline optimization level a runtime case is compared against.

`MTLLibraryOptimizationLevelDefault` is documented as "optimize for program
performance", so `-O2` is the offline row whose selection the runtime path can
express. The contraction axis is not narrowed the same way: a runtime case is
compared against *every* offline contraction setting recorded for its kernel,
mode, and this level, so a kernel on which contraction is unobservable yields a
plain agreement and a kernel on which it is observable reports which offline
setting the runtime default behaves like, instead of a spurious disagreement
against an arbitrarily chosen one.
"""

ARCHIVE_COMPILER = re.compile(rb"Apple metal version [0-9.]+ \(metalfe-[0-9.]+\)")
"""The runtime compiler's own version string, delimited by a literal prefix and `)`.

Unlike the option names below, this one is unambiguously bounded in the
container, so scanning for it yields the exact string and not a prefix of it.
"""

ARCHIVE_OPTION_PROBES = (
    "air.compile.denorms_disable",
    "air.compile.denorms_enable",
    "air.compile.fast_math_disable",
    "air.compile.fast_math_enable",
    "air.compile.framebuffer_fetch_enable",
)
"""The `air.compile.*` names a serialized binary archive is tested for, one by one.

A containment test is the strongest thing available: the container stores its
strings concatenated with no separator, so `air.compile.denorms_disable` is
immediately followed by the next name and no pattern can recover the *set*. Each
name here is therefore probed individually and the result reports presence only.
"""

FLOAT_FLAGS = ("nnan", "ninf", "nsz", "arcp", "contract", "afn", "reassoc", "fast")
_FLAG_GROUP = "|".join(FLOAT_FLAGS)
FLOAT_OPERATION = re.compile(
    rf"^\s+%\S+ = (?:tail\s+)?(fadd|fsub|fmul|fdiv|frem|fneg|call)"
    rf"((?:\s+(?:{_FLAG_GROUP}))*)\s"
)
"""Matches an LLVM floating-point instruction and its fast-math flag list.

`fcmp` is deliberately absent: a comparison is not arithmetic and cannot flush a
subnormal, so counting it would let a NaN test stand in for a surviving multiply.
"""

FUSED_INTRINSIC = re.compile(r"@(llvm\.(?:fma|fmuladd)\.\S+?)\(")
COMPILE_OPTIONS = re.compile(r"^!air\.compile_options = !\{(.*)\}$", re.MULTILINE)
METADATA_STRING = re.compile(r'^!(\d+) = !\{!"([^"]+)"\}$', re.MULTILINE)

CANONICALIZATION = """\
// Replaces an arithmetic NaN with the canonical pattern 0x7fc00000, spelled as
// an integer test exactly as the Metal emitter spells it.
static inline float tiler_canonicalize_nan_f32_7fc00000(float value) {
    uint pattern = as_type<uint>(value);
    bool nan = (pattern & 0x7f800000u) == 0x7f800000u
        && (pattern & 0x007fffffu) != 0x00000000u;
    return nan ? as_type<float>(0x7fc00000u) : value;
}
"""


class Reason(enum.Enum):
    """Why the probe could not run, in the classification the gate skips on.

    `TOOLCHAIN` and `SDK` mirror `DriverError::ToolchainUnavailable` and
    `::SdkUnavailable`. `DEVICE` is the one axis that classification has no name
    for, because the offline driver never dispatches; a host with a Metal
    compiler and no usable GPU is a real configuration and is a skip, not a
    defect.
    """

    TOOLCHAIN = "toolchain-unavailable"
    SDK = "sdk-unavailable"
    DEVICE = "device-unavailable"


class ProbeUnavailable(RuntimeError):
    """No qualified Apple toolchain, SDK, or GPU resolved."""

    def __init__(self, reason: Reason, detail: str) -> None:
        super().__init__(f"{reason.value}: {detail}")
        self.reason = reason
        self.detail = detail


class ProbeFailure(RuntimeError):
    """The toolchain and device resolved and something else went wrong.

    Never a skip. Every construction site is a case where the probe reached the
    tools, so the failure is a defect in the harness, the kernels, or the host.
    """


class Verdict(enum.Enum):
    """What one subnormal observation is admissible evidence of.

    Only `FLUSHED_TO_ZERO` and `PRESERVED` are claims about arithmetic. The rest
    record precisely why the observation cannot support either claim, which is
    the difference between this harness and one that reads bit patterns alone.
    """

    FLUSHED_TO_ZERO = "flushed-to-zero"
    PRESERVED = "preserved"
    NO_EMITTED_ARITHMETIC = "no-emitted-arithmetic"
    ARITHMETIC_NOT_EXECUTED = "arithmetic-not-executed"
    NO_EXECUTION_WITNESS = "no-execution-witness"
    WITNESS_DISAGREES = "witness-disagrees"
    UNEXPECTED_RESULT = "unexpected-result"

    @property
    def is_evidence(self) -> bool:
        """Whether this verdict may be cited as a fact about arithmetic."""
        return self in {Verdict.FLUSHED_TO_ZERO, Verdict.PRESERVED}


@dataclass(frozen=True)
class Witness:
    """Proof that a kernel's arithmetic actually ran in one configuration.

    `operand` must not be subnormal and must not produce a subnormal, so the
    witness is independent of the behaviour under test. `executed` is the result
    when every emitted operation ran; `deleted` is the result when they were all
    removed, which for these kernels is the operand itself.
    """

    operand: int
    executed: int
    deleted: int


@dataclass(frozen=True)
class SubnormalProbe:
    """One operand whose two possible results separate flushing from preserving."""

    operand: int
    preserving: int
    flushing: int


@dataclass(frozen=True)
class Kernel:
    """One probe kernel in the Metal emitter's output shape.

    `scale_bits` and `bias_bits` are exact `f32` bit patterns emitted through
    `as_type<float>`, never decimal literals, so no rendering step stands between
    the stated constant and the compiled one. `witness` is `None` exactly when
    the kernel is an identity on every operand and therefore cannot prove its own
    arithmetic ran.
    """

    name: str
    purpose: str
    scale_bits: int | None
    bias_bits: int | None
    canonicalized: bool
    witness: Witness | None

    def source(self) -> str:
        """Render the complete translation unit for this kernel."""
        lines = ["#include <metal_stdlib>", "using namespace metal;", ""]
        if self.canonicalized and (self.scale_bits is not None or self.bias_bits is not None):
            lines += [CANONICALIZATION]
        lines += [
            f"kernel void {ENTRY_POINT}(",
            "        device const float *b0 [[buffer(0)]],",
            "        device float *b1 [[buffer(1)]],",
            "        uint tiler_global_invocation_index [[thread_position_in_grid]]) {",
            "    ulong v0 = ulong(tiler_global_invocation_index);",
            f"    ulong v1 = {len(OPERANDS)}ul;",
            "    bool v2 = v0 < v1;",
            "    if (v2) {",
            "        float v3 = b0[v0];",
        ]
        register, current = 4, "v3"
        for constant, operator in ((self.scale_bits, "*"), (self.bias_bits, "+")):
            if constant is None:
                continue
            lines.append(f"        float v{register} = as_type<float>(0x{constant:08x}u);")
            lines.append(f"        float v{register + 1} = {current} {operator} v{register};")
            current = f"v{register + 1}"
            register += 2
            if self.canonicalized:
                helper = "tiler_canonicalize_nan_f32_7fc00000"
                lines.append(f"        float v{register} = {helper}({current});")
                current = f"v{register}"
                register += 1
        lines += [f"        b1[v0] = {current};", "    }", "}", ""]
        return "\n".join(lines)


NEGATIVE_ZERO = 0x80000000
POSITIVE_ZERO = 0x00000000

KERNELS: tuple[Kernel, ...] = (
    Kernel(
        name="materialize",
        purpose="a load and a store with no arithmetic at all",
        scale_bits=None,
        bias_bits=None,
        canonicalized=False,
        witness=None,
    ),
    Kernel(
        name="multiply_two",
        purpose="isolates input flushing: a subnormal operand whose exact result is normal",
        scale_bits=0x40000000,
        bias_bits=None,
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40000000, deleted=0x3F800000),
    ),
    Kernel(
        name="multiply_half",
        purpose="isolates result flushing: a normal operand whose exact result is subnormal",
        scale_bits=0x3F000000,
        bias_bits=None,
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x3F000000, deleted=0x3F800000),
    ),
    Kernel(
        name="multiply_one",
        purpose="the identity multiply: no witness exists, so it can prove nothing",
        scale_bits=0x3F800000,
        bias_bits=None,
        canonicalized=True,
        witness=None,
    ),
    Kernel(
        name="scale_one_bias_zero",
        purpose="the emitter's MultiplyThenAdd shape whose relaxed form deletes its arithmetic",
        scale_bits=0x3F800000,
        bias_bits=POSITIVE_ZERO,
        canonicalized=True,
        witness=Witness(operand=NEGATIVE_ZERO, executed=POSITIVE_ZERO, deleted=NEGATIVE_ZERO),
    ),
    Kernel(
        name="scale_two_bias_one",
        purpose="the shape the checked-in pointwise golden emits",
        scale_bits=0x40000000,
        bias_bits=0x3F800000,
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40400000, deleted=0x3F800000),
    ),
    Kernel(
        name="contraction_pair",
        purpose="a multiply and an add as two statements, with no canonicalization between them",
        scale_bits=0x3FC00000,
        bias_bits=0x3F800000,
        canonicalized=False,
        witness=Witness(operand=0x3F800000, executed=0x40200000, deleted=0x3F800000),
    ),
    Kernel(
        name="contraction_pair_canonicalized",
        purpose="the same pair with the emitter's canonicalization interposed",
        scale_bits=0x3FC00000,
        bias_bits=0x3F800000,
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40200000, deleted=0x3F800000),
    ),
)
BY_NAME = {kernel.name: kernel for kernel in KERNELS}

INPUT_FLUSH = SubnormalProbe(operand=0x00400000, preserving=0x00800000, flushing=POSITIVE_ZERO)
"""Doubling this subnormal has an exactly representable *normal* result.

A returned zero can therefore only come from flushing the operand, never from
rounding the result, which is what separates input flushing from result flushing.
"""

NEGATIVE_INPUT_FLUSH = SubnormalProbe(
    operand=0x80400000, preserving=0x80800000, flushing=NEGATIVE_ZERO
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

RESULT_FLUSH = SubnormalProbe(operand=0x00800000, preserving=0x00400000, flushing=POSITIVE_ZERO)
"""Halving the smallest normal has an exactly representable *subnormal* result."""

IDENTITY_VALUED_FLUSH = SubnormalProbe(
    operand=0x00400000, preserving=0x00400000, flushing=POSITIVE_ZERO
)
"""The probe for a kernel whose exact result is the operand itself.

`scale 1.0, bias +0.0` computes an identity, so its preserving result and its
deleted result are the same bit pattern. Nothing about the returned value can
distinguish arithmetic that preserved a subnormal from arithmetic that never
ran, which is exactly why an observation using this probe is admissible only
through the execution witness.
"""


@dataclass(frozen=True)
class Configuration:
    """One offline compilation selection."""

    math_mode: str
    optimization: str
    fp_contract: str

    @property
    def key(self) -> str:
        return f"{self.math_mode}.O{self.optimization}.contract-{self.fp_contract}"

    def flags(self) -> list[str]:
        return [
            "-target",
            TARGET,
            f"-std={MSL_VERSION}",
            f"-O{self.optimization}",
            f"-fmetal-math-mode={self.math_mode}",
            f"-fmetal-math-fp32-functions={FP32_FUNCTIONS}",
            f"-ffp-contract={self.fp_contract}",
        ]


@dataclass(frozen=True)
class RuntimeConfiguration:
    """One in-process `MTLCompileOptions` selection.

    The two properties that have no offline counterpart in the harness's fixed
    flags — `languageVersion` and `mathFloatingPointFunctions` — are pinned to
    the counterparts of `MSL_VERSION` and `FP32_FUNCTIONS` rather than left at
    their API defaults, because `mathFloatingPointFunctions` defaults to `Fast`
    and an unpinned runtime case would not be comparable to any offline row.
    """

    math_mode: str
    optimization: str

    @property
    def key(self) -> str:
        return f"runtime.{self.math_mode}.opt-{self.optimization}"

    def options(self, archive: Path | None = None) -> str:
        selections = [
            f"math={self.math_mode}",
            f"fpfun={FP32_FUNCTIONS}",
            f"lang={RUNTIME_LANGUAGE}",
            f"opt={self.optimization}",
        ]
        if archive is not None:
            selections.append(f"archive={archive}")
        return ",".join(selections)


@dataclass(frozen=True)
class Case:
    kernel: str
    configuration: Configuration | RuntimeConfiguration

    @property
    def key(self) -> str:
        return f"{self.kernel}.{self.configuration.key}"

    @property
    def is_runtime(self) -> bool:
        return isinstance(self.configuration, RuntimeConfiguration)


def cases() -> tuple[Case, ...]:
    """Every kernel and configuration pair the recorded findings need.

    The set is assembled per finding and then deduplicated, so a case shared by
    two findings is compiled and dispatched once and a finding cannot quietly
    lose its configuration when another one changes.
    """
    selected: list[Case] = []

    def add(kernel: str, mode: str, optimization: str, contract: str) -> None:
        selected.append(Case(kernel, Configuration(mode, optimization, contract)))

    # The emitted module's own denormal and fast-math declarations, and the
    # fast-math flags each mode attaches, across every contraction selection.
    for mode in MATH_MODES:
        for contract in FP_CONTRACTS:
            add("scale_two_bias_one", mode, "2", contract)
    # Input flushing and result flushing, separately, at both optimization
    # levels and in every math mode. `relaxed` is included even though the
    # originating record only claimed `safe` and `fast`, because these two
    # kernels carry execution witnesses and so can close the gap rather than
    # record it as a boundary.
    for mode in MATH_MODES:
        for optimization in ("0", "2"):
            add("multiply_two", mode, optimization, "off")
            add("multiply_half", mode, optimization, "off")
    # Materialization, which the record claims is untouched.
    for mode in MATH_MODES:
        add("materialize", mode, "2", "off")
    # The signed-zero divergence and the arithmetic-deletion trap.
    for mode in MATH_MODES:
        for optimization in ("0", "2"):
            add("scale_one_bias_zero", mode, optimization, "off")
            add("multiply_one", mode, optimization, "off")
    # Contraction, and the control showing the canonicalization is not a barrier.
    for contract in FP_CONTRACTS:
        add("contraction_pair", "safe", "2", contract)
        add("contraction_pair_canonicalized", "safe", "2", contract)

    unique: dict[str, Case] = {}
    for case in selected:
        unique.setdefault(case.key, case)
    return tuple(unique.values())


def runtime_cases() -> tuple[Case, ...]:
    """Every runtime-compilation case, derived from the offline set rather than listed.

    Deriving it is what keeps the two paths comparable. A runtime case exists for
    each kernel and math mode the offline probe already covers, so no runtime
    case can be added that has nothing to be compared against and no offline case
    can be dropped while its runtime partner survives. Both optimization levels
    the runtime surface offers are swept, so an optimization-dependent runtime
    divergence has somewhere to show up.
    """
    pairs: dict[tuple[str, str], None] = {}
    for case in cases():
        assert isinstance(case.configuration, Configuration)
        pairs.setdefault((case.kernel, case.configuration.math_mode), None)
    return tuple(
        Case(kernel, RuntimeConfiguration(mode, optimization))
        for kernel, mode in pairs
        for optimization in RUNTIME_OPTIMIZATIONS
    )


@dataclass(frozen=True)
class FloatOperation:
    opcode: str
    flags: tuple[str, ...]

    def __str__(self) -> str:
        return self.opcode if not self.flags else f"{self.opcode}+{'+'.join(self.flags)}"


EMITTED_ARITHMETIC = "emitted-arithmetic"
EXECUTION_WITNESS = "execution-witness"


@dataclass(frozen=True)
class Observation:
    """One case's compile-side and device-side facts.

    `compile_options` and `operations` are `None` exactly when the compilation
    path gave the harness no readable module — never `()`. An empty tuple is a
    measured absence of arithmetic; `None` records that the question could not be
    asked, and only the second of those is true of the runtime path. Everything
    that consumes them must distinguish the two, which is why neither field has a
    default: a construction site has to state which it means.

    `archived_options` and `applied_options` are the runtime path's own
    compile-side facts, and both are `None` on the offline path. See
    `scan_archive` for why `archived_options` is corroboration and not evidence.
    """

    case: Case
    compile_options: tuple[str, ...] | None
    operations: tuple[FloatOperation, ...] | None
    results: tuple[int, ...]
    applied_options: str | None
    archived_options: str | None

    @property
    def kernel(self) -> Kernel:
        return BY_NAME[self.case.kernel]

    @property
    def operation_count(self) -> int | None:
        """How many floating-point operations the module emitted, or `None` if unreadable."""
        return None if self.operations is None else len(self.operations)

    @property
    def guard_layers(self) -> tuple[str, ...]:
        """Which layers of the admissibility guard this observation's path can supply."""
        if self.operations is None:
            return (EXECUTION_WITNESS,)
        return (EMITTED_ARITHMETIC, EXECUTION_WITNESS)

    def result_for(self, operand: int) -> int:
        return self.results[OPERANDS.index(operand)]

    def flags_for(self, opcode: str) -> tuple[tuple[str, ...], ...]:
        if self.operations is None:
            raise ProbeFailure(f"{self.case.key} has no readable module to take flags from")
        return tuple(op.flags for op in self.operations if op.opcode == opcode)


def subnormal_verdict(observation: Observation, probe: SubnormalProbe) -> Verdict:
    """Classify one subnormal observation, refusing to over-read a deleted operation.

    The two guard layers run before the returned pattern is even consulted; see
    the module documentation for why the emitted operation count alone is not
    enough on this toolchain row, and for why layer 1 is skipped rather than
    assumed when the path could not supply it.
    """
    if observation.operations is not None and not observation.operations:
        return Verdict.NO_EMITTED_ARITHMETIC
    witness = observation.kernel.witness
    if witness is None:
        return Verdict.NO_EXECUTION_WITNESS
    witnessed = observation.result_for(witness.operand)
    if witnessed == witness.deleted:
        return Verdict.ARITHMETIC_NOT_EXECUTED
    if witnessed != witness.executed:
        return Verdict.WITNESS_DISAGREES
    result = observation.result_for(probe.operand)
    if result == probe.flushing:
        return Verdict.FLUSHED_TO_ZERO
    if result == probe.preserving:
        return Verdict.PRESERVED
    return Verdict.UNEXPECTED_RESULT


def naive_verdict(observation: Observation, probe: SubnormalProbe) -> Verdict:
    """Classify the same observation from the returned bit pattern alone.

    This is the reading a probe without the guard would produce. It exists so a
    test can assert that the two disagree on the trap kernel; it must never be
    used to state a fact.
    """
    result = observation.result_for(probe.operand)
    if result == probe.flushing:
        return Verdict.FLUSHED_TO_ZERO
    if result == probe.preserving:
        return Verdict.PRESERVED
    return Verdict.UNEXPECTED_RESULT


def _run(command: list[str]) -> subprocess.CompletedProcess[str]:
    """Run one command, reporting an absent executable as a failed run, not an exception.

    `record_rows` and `environment` fall back to `unreported` for a tool that does
    not answer, and that fallback is only reachable if a missing executable
    arrives here as a return code. A host with no `git` is the case that proves
    it: the portable guard tests render a record on one, and every caller here
    already inspects `returncode`.
    """
    try:
        return subprocess.run(command, check=False, capture_output=True, text=True)
    except OSError as unavailable:
        return subprocess.CompletedProcess(
            command, returncode=127, stdout="", stderr=str(unavailable)
        )


def _first_line(text: str) -> str:
    return text.strip().splitlines()[0].strip() if text.strip() else ""


@dataclass(frozen=True)
class Toolchain:
    """The resolved offline compiler, linker, SDK, and host compiler."""

    sdk_path: str
    sdk_version: str
    sdk_build: str
    metal_path: str
    metal_version: str
    metallib_version: str
    clang_path: str

    def compile_ir(self, source: Path, destination: Path, configuration: Configuration) -> None:
        self._metal(["-S", "-emit-llvm"], source, destination, configuration)

    def compile_air(self, source: Path, destination: Path, configuration: Configuration) -> None:
        self._metal(["-c"], source, destination, configuration)

    def _metal(
        self, mode: list[str], source: Path, destination: Path, configuration: Configuration
    ) -> None:
        command = [
            "xcrun",
            "--sdk",
            "macosx",
            "metal",
            *configuration.flags(),
            *mode,
            str(source),
            "-o",
            str(destination),
        ]
        result = _run(command)
        if result.returncode != 0:
            raise ProbeFailure(f"metal failed for {source.name}: {result.stderr.strip()}")

    def link(self, air: Path, destination: Path) -> None:
        result = _run(["xcrun", "--sdk", "macosx", "metallib", str(air), "-o", str(destination)])
        if result.returncode != 0:
            raise ProbeFailure(f"metallib failed for {air.name}: {result.stderr.strip()}")

    def build_host(self, destination: Path) -> None:
        result = _run(
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
        if result.returncode != 0:
            raise ProbeFailure(f"the dispatch host did not build: {result.stderr.strip()}")


def resolve() -> Toolchain:
    """Resolve the offline toolchain, SDK, and host compiler, or refuse to run.

    Every refusal here is a `ProbeUnavailable`, which callers turn into a skip.
    A tool that resolves and then fails raises `ProbeFailure` instead, so a
    broken toolchain cannot be mistaken for an absent one.
    """
    if platform.system() != "Darwin":
        raise ProbeUnavailable(Reason.TOOLCHAIN, f"host is {platform.system()}, not Darwin")
    if shutil.which("xcrun") is None:
        raise ProbeUnavailable(Reason.TOOLCHAIN, "xcrun is not on PATH")
    sdk_path = _run(["xcrun", "--sdk", "macosx", "--show-sdk-path"])
    if sdk_path.returncode != 0 or not Path(_first_line(sdk_path.stdout)).is_dir():
        raise ProbeUnavailable(Reason.SDK, f"macosx SDK did not resolve: {sdk_path.stderr.strip()}")
    sdk_version = _run(["xcrun", "--sdk", "macosx", "--show-sdk-version"])
    sdk_build = _run(["xcrun", "--sdk", "macosx", "--show-sdk-build-version"])
    if sdk_version.returncode != 0 or sdk_build.returncode != 0:
        raise ProbeUnavailable(Reason.SDK, "the macosx SDK reported no version or build")
    found = {}
    for tool in ("metal", "metallib", "clang"):
        located = _run(["xcrun", "--sdk", "macosx", "--find", tool])
        path = _first_line(located.stdout)
        if located.returncode != 0 or not path:
            raise ProbeUnavailable(Reason.TOOLCHAIN, f"{tool} was not found by xcrun")
        found[tool] = path
    versions = {}
    for tool in ("metal", "metallib"):
        reported = _run(["xcrun", "--sdk", "macosx", tool, "--version"])
        if reported.returncode != 0:
            raise ProbeUnavailable(Reason.TOOLCHAIN, f"{tool} did not report a version")
        versions[tool] = _first_line(reported.stdout)
        if not versions[tool]:
            raise ProbeUnavailable(Reason.TOOLCHAIN, f"{tool} reported an empty version")
    return Toolchain(
        sdk_path=_first_line(sdk_path.stdout),
        sdk_version=_first_line(sdk_version.stdout),
        sdk_build=_first_line(sdk_build.stdout),
        metal_path=found["metal"],
        metal_version=versions["metal"],
        metallib_version=versions["metallib"],
        clang_path=found["clang"],
    )


def compile_options(ir: str) -> tuple[str, ...]:
    """Return the `air.compile_options` strings the emitted module declares.

    The named metadata node is resolved rather than substring-matched, so an
    `air.compile.*` string that the module defines but does not attach to
    `air.compile_options` cannot be reported as a declared option.
    """
    node = COMPILE_OPTIONS.search(ir)
    if node is None:
        return ()
    strings = dict(METADATA_STRING.findall(ir))
    referenced = re.findall(r"!(\d+)", node.group(1))
    return tuple(strings[identifier] for identifier in referenced if identifier in strings)


def float_operations(ir: str) -> tuple[FloatOperation, ...]:
    """Return every floating-point arithmetic instruction the module contains."""
    found: list[FloatOperation] = []
    for line in ir.splitlines():
        match = FLOAT_OPERATION.match(line)
        if match is None:
            continue
        opcode, raw = match.group(1), match.group(2)
        if opcode == "call":
            intrinsic = FUSED_INTRINSIC.search(line)
            if intrinsic is None:
                continue
            opcode = intrinsic.group(1)
        found.append(FloatOperation(opcode, tuple(raw.split())))
    return tuple(found)


@dataclass(frozen=True)
class Dispatch:
    """What one run of the dispatch host reported."""

    device: str
    results: tuple[int, ...]
    applied_options: str
    archive: str


def _dispatch(host: Path, arguments: list[str], subject: str) -> Dispatch:
    """Run the dispatch host once and parse its `key=value` lines.

    Both compilation modes come through here, so the device-side procedure is
    literally the same code for the offline and runtime paths and a difference
    between them cannot be an artefact of dispatching them differently.
    """
    result = _run([str(host), *arguments, *(f"{value:08x}" for value in OPERANDS)])
    if result.returncode == 3:
        raise ProbeUnavailable(Reason.DEVICE, result.stderr.strip() or "no default Metal device")
    if result.returncode != 0:
        raise ProbeFailure(f"dispatch of {subject} failed: {result.stderr.strip()}")
    device, applied, archive, values = "", "", "", []
    for line in result.stdout.splitlines():
        key, _, value = line.partition("=")
        if key == "device":
            device = value
        elif key == "applied":
            applied = value
        elif key == "archive":
            archive = value
        elif key == "archive-unavailable":
            archive = f"unavailable:{value}"
        elif key == "result":
            values.append(int(value, 16))
    if len(values) != len(OPERANDS):
        raise ProbeFailure(f"dispatch returned {len(values)} results, expected {len(OPERANDS)}")
    return Dispatch(device, tuple(values), applied, archive)


def dispatch(host: Path, library: Path) -> tuple[str, tuple[int, ...]]:
    """Run one offline-linked library on the local GPU and return the device and results."""
    reported = _dispatch(host, ["library", str(library), ENTRY_POINT], library.name)
    return reported.device, reported.results


def dispatch_source(
    host: Path, source: Path, configuration: RuntimeConfiguration, archive: Path
) -> Dispatch:
    """Compile one source file in the host process and dispatch what came out.

    The source is the byte-identical file the offline path compiles, so the only
    difference between the two observations is which compiler produced the
    library.
    """
    return _dispatch(
        host,
        ["source", str(source), ENTRY_POINT, configuration.options(archive)],
        f"{source.name} at {configuration.key}",
    )


@dataclass(frozen=True)
class Archive:
    """What a scan of a serialized binary archive found, and nothing more."""

    compiler: str
    present: tuple[str, ...]


def scan_archive(path: Path) -> Archive:
    """Test a serialized `MTLBinaryArchive` for the byte sequences it may contain.

    This is a containment test over a container with no published layout, and it
    is the only compile-side artefact the runtime path leaves behind. It reports
    which of `ARCHIVE_OPTION_PROBES` are present and never that the ones absent
    from the list are absent from the module, because the strings are stored
    concatenated and the set is not recoverable. Nothing in the admissibility
    guard consults it; see the module documentation.
    """
    blob = path.read_bytes()
    found = ARCHIVE_COMPILER.search(blob)
    return Archive(
        compiler=found.group(0).decode("ascii") if found else "unreported",
        present=tuple(name for name in ARCHIVE_OPTION_PROBES if name.encode("ascii") in blob),
    )


@dataclass(frozen=True)
class Run:
    """Everything one complete probe execution observed."""

    environment: dict[str, str]
    observations: dict[str, Observation]

    def of(self, kernel: str, mode: str, optimization: str = "2", contract: str = "off"):
        """Return one offline observation by its case coordinates, failing loudly if absent."""
        return self._at(Case(kernel, Configuration(mode, optimization, contract)).key)

    def runtime(self, kernel: str, mode: str, optimization: str = "default"):
        """Return one runtime-compilation observation by its case coordinates."""
        return self._at(Case(kernel, RuntimeConfiguration(mode, optimization)).key)

    def _at(self, key: str) -> Observation:
        if key not in self.observations:
            raise KeyError(f"the probe did not run case {key}")
        return self.observations[key]


class Agreement(enum.Enum):
    """How one runtime case's results relate to its offline candidates.

    `AGREE_ON_SOME` is deliberately not a disagreement. It arises only where the
    offline candidates differ from each other, which means the axis separating
    them is one `MTLCompileOptions` cannot express; the runtime path then behaves
    like one of them and the comparison reports which, rather than pretending the
    two paths were asked the same question.
    """

    AGREE = "agree"
    AGREE_ON_SOME = "agree-on-some"
    DIFFER = "differ"

    @property
    def is_divergence(self) -> bool:
        """Whether this is the outcome that means the two compilers disagree."""
        return self is Agreement.DIFFER


@dataclass(frozen=True)
class PathComparison:
    """One runtime case set against every offline case it can be compared with."""

    runtime_case: str
    candidates: tuple[str, ...]
    matched: tuple[str, ...]
    runtime_results: tuple[int, ...]

    @property
    def agreement(self) -> Agreement:
        if not self.matched:
            return Agreement.DIFFER
        if len(self.matched) == len(self.candidates):
            return Agreement.AGREE
        return Agreement.AGREE_ON_SOME

    def render(self) -> str:
        """One record row's worth of the comparison, complete enough to act on."""
        summary = f"{self.agreement.value} candidates={','.join(self.candidates)}"
        if self.agreement is Agreement.DIFFER:
            return f"{summary} runtime={' '.join(f'{v:08x}' for v in self.runtime_results)}"
        return f"{summary} matched={','.join(self.matched)}"


def path_comparisons(run: Run) -> tuple[PathComparison, ...]:
    """Pair every runtime case with the offline cases it can legitimately be compared to.

    The candidate set is every offline case for the same kernel and math mode at
    `RUNTIME_PAIRED_OPTIMIZATION`, across whatever contraction settings the
    offline probe recorded. Deriving the set instead of naming one row is what
    keeps a kernel that becomes contraction-sensitive from reading as a
    divergence between the two compilers when it is nothing of the kind.
    """
    compared: list[PathComparison] = []
    for key in sorted(run.observations):
        observation = run.observations[key]
        configuration = observation.case.configuration
        if not isinstance(configuration, RuntimeConfiguration):
            continue
        candidates = {
            other: run.observations[other].results
            for other in sorted(run.observations)
            for offline in [run.observations[other].case.configuration]
            if isinstance(offline, Configuration)
            and run.observations[other].case.kernel == observation.case.kernel
            and offline.math_mode == configuration.math_mode
            and offline.optimization == RUNTIME_PAIRED_OPTIMIZATION
        }
        if not candidates:
            raise ProbeFailure(f"{key} has no offline case to be compared against")
        compared.append(
            PathComparison(
                runtime_case=key,
                candidates=tuple(candidates),
                matched=tuple(
                    name for name, results in candidates.items() if results == observation.results
                ),
                runtime_results=observation.results,
            )
        )
    return tuple(compared)


def environment(toolchain: Toolchain, device: str, runtime_compiler: str) -> dict[str, str]:
    """Capture the exact host row every measurement below is qualified by.

    `metal_version` and `runtime_compiler` are two different compilers and are
    recorded separately for that reason. On the measured row they are different
    builds, so collapsing them would make a cross-path agreement look like a
    tautology and would hide the toolchain whose numerics a runtime-compiled
    kernel actually delivers.
    """
    xcode = _run(["xcodebuild", "-version"])
    return {
        "date_utc": datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "os_version": _first_line(_run(["sw_vers", "-productVersion"]).stdout),
        "os_build": _first_line(_run(["sw_vers", "-buildVersion"]).stdout),
        "machine": _first_line(_run(["uname", "-m"]).stdout),
        "xcode": " ".join(xcode.stdout.split()) if xcode.returncode == 0 else "unreported",
        "sdk_version": toolchain.sdk_version,
        "sdk_build": toolchain.sdk_build,
        "metal_version": toolchain.metal_version,
        "metallib_version": toolchain.metallib_version,
        "runtime_compiler": runtime_compiler,
        "device": device,
    }


QUALIFYING = (
    "os_version",
    "os_build",
    "machine",
    "xcode",
    "sdk_version",
    "sdk_build",
    "metal_version",
    "metallib_version",
    "runtime_compiler",
    "device",
)
"""The environment fields that make two runs comparable.

`date_utc` is excluded because it changes every run and qualifies nothing.
"""


def probe(work_directory: Path) -> Run:
    """Compile, link, dispatch, and classify every case on both compilation paths.

    Raises `ProbeUnavailable` when no toolchain, SDK, or GPU resolves, and
    `ProbeFailure` for anything that goes wrong after they do.
    """
    toolchain = resolve()
    work_directory.mkdir(parents=True, exist_ok=True)
    host = work_directory / "numerical_probe_host"
    toolchain.build_host(host)

    device = ""
    runtime_compiler = ""
    observations: dict[str, Observation] = {}

    def observe_device(observed: str) -> None:
        nonlocal device
        if device and observed != device:
            raise ProbeFailure(f"the GPU changed mid-run: {device} then {observed}")
        device = observed

    for case in cases():
        kernel = BY_NAME[case.kernel]
        assert isinstance(case.configuration, Configuration)
        stem = case.key.replace(".", "_")
        source = work_directory / f"{stem}.metal"
        source.write_text(kernel.source(), encoding="utf-8")
        ir_path = work_directory / f"{stem}.ll"
        air_path = work_directory / f"{stem}.air"
        library = work_directory / f"{stem}.metallib"
        toolchain.compile_ir(source, ir_path, case.configuration)
        toolchain.compile_air(source, air_path, case.configuration)
        toolchain.link(air_path, library)
        ir = ir_path.read_text(encoding="utf-8")
        observed_device, results = dispatch(host, library)
        observe_device(observed_device)
        observations[case.key] = Observation(
            case=case,
            compile_options=compile_options(ir),
            operations=float_operations(ir),
            results=results,
            applied_options=None,
            archived_options=None,
        )

    for case in runtime_cases():
        kernel = BY_NAME[case.kernel]
        assert isinstance(case.configuration, RuntimeConfiguration)
        stem = case.key.replace(".", "_")
        # The runtime path compiles the same bytes the offline path compiled, so
        # the file is written once per case rather than shared: a case that
        # generated different source would otherwise be invisible here.
        source = work_directory / f"{stem}.metal"
        source.write_text(kernel.source(), encoding="utf-8")
        archive_path = work_directory / f"{stem}.archive.metallib"
        reported = dispatch_source(host, source, case.configuration, archive_path)
        observe_device(reported.device)
        if reported.archive.startswith("unavailable:") or not reported.archive:
            archived = reported.archive or "unavailable:the host reported no archive"
        else:
            archive = scan_archive(Path(reported.archive))
            archived = " ".join(archive.present)
            if runtime_compiler and archive.compiler != runtime_compiler:
                raise ProbeFailure(
                    f"the runtime compiler changed mid-run: "
                    f"{runtime_compiler} then {archive.compiler}"
                )
            runtime_compiler = archive.compiler
        observations[case.key] = Observation(
            case=case,
            compile_options=None,
            operations=None,
            results=reported.results,
            applied_options=reported.applied_options,
            archived_options=archived,
        )

    return Run(environment(toolchain, device, runtime_compiler or "unreported"), observations)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record_rows(run: Run) -> list[tuple[str, str]]:
    """Render one run as the ordered key/value rows of the checked-in record."""
    revision = _run(["git", "-C", str(REPOSITORY), "rev-parse", "HEAD"])
    rows: list[tuple[str, str]] = [
        ("schema", SCHEMA),
        ("probe.repository_base_revision", _first_line(revision.stdout) or "unreported"),
        ("probe.harness_sha256", digest(Path(__file__).resolve())),
        ("probe.host_source_sha256", digest(HOST_SOURCE)),
        ("probe.target", TARGET),
        (
            "probe.fixed_flags",
            f"-std={MSL_VERSION} -fmetal-math-fp32-functions={FP32_FUNCTIONS}",
        ),
        ("probe.entry_point", ENTRY_POINT),
        ("probe.operands", " ".join(f"{value:08x}" for value in OPERANDS)),
        (
            "probe.runtime_fixed_options",
            f"fpfun={FP32_FUNCTIONS} lang={RUNTIME_LANGUAGE}",
        ),
        ("probe.runtime_paired_optimization", f"-O{RUNTIME_PAIRED_OPTIMIZATION}"),
        ("probe.guard_layers.offline", f"{EMITTED_ARITHMETIC} {EXECUTION_WITNESS}"),
        ("probe.guard_layers.runtime", EXECUTION_WITNESS),
    ]
    rows += [
        (f"probe.offline_flag_without_runtime_counterpart.{index}", gap)
        for index, gap in enumerate(OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART)
    ]
    rows += [(f"environment.{key}", value) for key, value in run.environment.items()]
    for key in sorted(run.observations):
        observation = run.observations[key]
        # A runtime case gets no `float_operations` row at all. Writing an empty
        # one would read as a module measured to contain no arithmetic, which is
        # the single reading this harness must never let a record support.
        if observation.compile_options is not None:
            rows.append((f"case.{key}.compile_options", " ".join(observation.compile_options)))
        if observation.operations is not None:
            rows.append(
                (f"case.{key}.float_operations", " ".join(str(op) for op in observation.operations))
            )
        if observation.applied_options is not None:
            rows.append((f"case.{key}.applied_options", observation.applied_options))
        if observation.archived_options is not None:
            rows.append((f"case.{key}.archived_options", observation.archived_options))
        rows.append(
            (f"case.{key}.results", " ".join(f"{value:08x}" for value in observation.results))
        )
    for comparison in path_comparisons(run):
        rows.append((f"comparison.{comparison.runtime_case}", comparison.render()))
    rows.append(("probe.status", "complete"))
    return rows


def write_record(run: Run, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    body = "".join(f"{key}\t{value}\n" for key, value in record_rows(run))
    destination.write_text(body, encoding="utf-8")


COMPARED_PREFIXES = ("case.", "comparison.")
"""The record rows a live run must reproduce exactly on the same environment row.

`comparison.` is included so a divergence between the two compilation paths, or
a change in which offline contraction setting the runtime path behaves like,
fails the gate rather than merely being rewritten into the record.
"""


def compare_record(run: Run, stored: dict[str, str]) -> list[str]:
    """Return every way a retained record disagrees with a live run's case rows.

    The environment row is deliberately not compared here. A different toolchain
    build legitimately produces different values, so deciding whether the two
    runs are comparable at all belongs to the caller; only once they are is a
    difference in a case row a finding.
    """
    live = dict(record_rows(run))
    stored_cases = {
        key: value for key, value in stored.items() if key.startswith(COMPARED_PREFIXES)
    }
    live_cases = {key: value for key, value in live.items() if key.startswith(COMPARED_PREFIXES)}
    differences: list[str] = []
    for key in sorted(set(stored_cases) | set(live_cases)):
        if key not in stored_cases:
            differences.append(f"{key}: observed but absent from the retained record")
        elif key not in live_cases:
            differences.append(f"{key}: retained but no longer produced by the probe")
        elif stored_cases[key] != live_cases[key]:
            differences.append(
                f"{key}: retained {stored_cases[key]!r}, observed {live_cases[key]!r}"
            )
    return differences


def read_record(path: Path) -> dict[str, str]:
    rows: dict[str, str] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        key, tab, value = line.partition("\t")
        if not tab or key in rows:
            raise ProbeFailure(f"{path}:{number}: malformed or duplicated record row")
        rows[key] = value
    return rows


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--record", type=Path, help="write the measured record to this path")
    parser.add_argument(
        "--work-dir",
        type=Path,
        help="keep the generated sources, IR, AIR, and libraries here instead of a temporary tree",
    )
    parsed = parser.parse_args(arguments)
    try:
        if parsed.work_dir is not None:
            run = probe(parsed.work_dir.resolve())
        else:
            with tempfile.TemporaryDirectory(prefix="tiler-apple-numerics.") as directory:
                run = probe(Path(directory))
    except ProbeUnavailable as unavailable:
        print(f"numerical_probe: skipped, {unavailable}", file=sys.stderr)
        return 0
    for key in QUALIFYING:
        print(f"{key}={run.environment[key]}")
    for key in sorted(run.observations):
        observation = run.observations[key]
        results = " ".join(f"{value:08x}" for value in observation.results)
        count = observation.operation_count
        print(f"{key}\tfp-ops={'unreadable' if count is None else count}\t{results}")
    for comparison in path_comparisons(run):
        print(f"comparison.{comparison.runtime_case}\t{comparison.render()}")
    if parsed.record is not None:
        write_record(run, parsed.record)
        print(f"record={parsed.record}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
