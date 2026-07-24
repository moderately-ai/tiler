#!/usr/bin/env python3
"""Reproduce the Apple GPU floating-point behaviour ADR 0076 depends on, per artifact family.

The record ADR 0076 builds on was measured by a hand-built Objective-C host that
was never checked in, so nothing re-established it. This module is that harness,
owned by the repository: it generates probe kernels in the emitter's output
shape, compiles them offline through `xcrun metal` and `xcrun metallib`, reads
the emitted LLVM IR, dispatches the linked library through
`numerical_probe_host.m`, and classifies what came back.

Scope note. Every value this module produces is qualified by one host, one GPU,
and the compiler builds that host resolves. `environment()` captures that row and
`write_record` stores it beside the observations, because none of these
observations is a portable guarantee about Metal.

# Three artifact families, and the two halves that do not have the same reach

`tiler_metal::target::MetalPlatform` declares `MacOs`, `IOsDevice`, and
`IOsSimulator`, and `MetalTargetFacts::new` requires a caller to state
`subnormal_arithmetic` for whichever one it is emitting for. `FAMILIES` names all
three, each with the exact `--sdk` and `-target` that produce its artifact, and
the two halves of the probe reach differently far:

- **The compile side needs no device.** Every case is compiled for every family
  and the emitted module is read for all of them, so a per-family difference in
  `air.compile.denorms_disable`, in the fast-math licence spellings, or in the
  surviving operation count is a first-class result rather than an assumption
  that the macOS row generalizes.
- **The device side is bounded by attached hardware.** A family is dispatched
  only in *its own* execution environment: macOS on the host GPU, iOS Simulator
  through `simctl spawn` on a booted runtime, and `IOsDevice` nowhere, because
  this host has no iPhone or iPad attached.

The convenient substitute is available and is refused. On the measured row
`MTLCreateSystemDefaultDevice` on macOS loads an `air64-apple-ios16.0` metallib
and runs it without complaint — `hazard.cross_family_load.*` records that
outcome — but the GPU and driver executing it are the Mac's, so the result is a
fact about macOS running a foreign module and not a fact about an iOS device.
`Execution.NONE` is what stops the harness taking it.

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

## The two ways a layer can be missing, and why neither may be defaulted

Both layers can be absent, on opposite paths, and the data model keeps the
absence distinguishable from a measurement in both directions.

- **Layer 1 is absent on the runtime path.** `newLibraryWithSource:options:`
  returns an opaque `MTLLibrary`; there is no emitted module to read.
  `Observation.operations` is `None`, never `()`, because `()` asserts a
  *measured* absence of arithmetic while `None` records a question that was never
  asked. `record_rows` omits the `float_operations` row entirely rather than
  writing an empty one.
- **Layer 2 is absent for a family with no attached device.** Nothing was
  dispatched, so there is no returned bit pattern at all.
  `Observation.results` is `None`, never `()`, for exactly the same reason, and
  `record_rows` omits the `results` row. `subnormal_verdict` returns
  `Verdict.NO_DEVICE_OBSERVATION` before consulting anything else, and
  `result_for` raises rather than inventing a value, so no code path can read a
  bit pattern that was never measured.

Neither field has a default: a construction site has to state which it means.

Losing layer 1 costs a compile-side cross-check, because layer 2 is *sufficient*
where layer 1 is merely *necessary* — an observation layer 1 would reject emitted
no arithmetic, so nothing ran, so the kernel returns its operands, so layer 2
rejects it as `arithmetic-not-executed`. The converse fails, and this harness
measured it failing at `-O0`. Losing layer 2 is therefore the expensive
direction: a compile-side-only observation can never be admissible evidence
about arithmetic, which is why it gets its own verdict instead of being silently
classified by the layer that remains.

A guard that never refuses anything is not a guard, so where only layer 2 is
left the harness must keep demonstrating that layer 2 still discriminates *on
that path*: the trap kernel is admitted under `safe` and refused under `relaxed`
and `fast` in the same run.

# Two compilation paths, compared case by case within a family

Tiler's Metal story has two compilation stages: `xcrun metal` at build time and
runtime pipeline creation through a command stream. An artifact's declared
numerical realization has to be true of whichever one actually runs, so the same
generated source bytes go through both here — offline through `xcrun`, and in
process through `newLibraryWithSource:options:` with an explicit
`MTLCompileOptions` — and `path_comparisons` pairs the two case by case, within
one family, rather than in aggregate or across families.

These are not the same compiler, and on this host they are not even the same
compiler per family. The offline driver is one binary shared by every SDK; the
runtime compiler is whatever the *execution environment* loads, which is the
host's `GPUCompiler.framework` for macOS and the simulator runtime's own copy
for `IOsSimulator`. `report_compiler_images` in the dispatch host reports the
image dyld actually loaded, and `compiler_build` recovers its build string, so a
family's runtime compiler is identified rather than inherited from another
family's row.

`MTLCompileOptions` also exposes a different surface from the offline flag set.
`OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART` names every offline selection with
no property to set, and the harness records the gap rather than substituting the
nearest thing: `RUNTIME_PAIRED_OPTIMIZATION` explains which offline row a runtime
case is paired against, and a runtime result that matches only some of its
offline candidates is reported as a *measurement of the missing axis* rather
than as a disagreement between the paths.

`scan_archive` recovers what little does survive the runtime path, and is
deliberately not part of the guard. A serialized `MTLBinaryArchive` embeds the
runtime compiler's version string and the module's `air.compile.*` option names,
but the container has no published layout and its string table is stored
concatenated without separators, so the harness can only test it for the
presence of a byte sequence. Presence is decidable; the option *set* is not, and
neither is attachment to the module's `air.compile_options` node, which the
offline path resolves properly. It is corroboration, not evidence. In the iOS
Simulator it is not even available: serializing an archive aborts the process
there, so `archive_support` probes for it in a one-entry batch of its own before
any manifest that carries measurements asks for one.
"""

from __future__ import annotations

import argparse
import enum
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
HOST_SOURCE = HERE / "numerical_probe_host.m"

SCHEMA = "tiler.apple-numerical-behaviour/v3"
"""Record format identity. Bump this whenever a key's meaning changes.

v2 added the runtime-compilation path. `case.*.float_operations` and
`case.*.compile_options` became conditional rather than universal — a case
compiled at runtime has neither, because nothing readable survives that path —
and `comparison.*`, `environment.runtime_compiler`, `case.*.applied_options`,
and `case.*.archived_options` were new.

v3 adds the artifact family. Every `case.*` and `comparison.*` key is now
prefixed by the family it was measured for, `case.*.results` became conditional
because a family with no attached device is never dispatched, the single
`environment.sdk_*`/`metal_version`/`runtime_compiler`/`device` fields became
per-family `environment.family.*` rows, and `hazard.*` records a measured
cross-family outcome that the harness refuses to treat as evidence.
"""

REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"
"""Turns an absent toolchain, SDK, or GPU from a skip into a failure.

This is deliberately the same variable `crates/tiler-metal/src/golden_compilation.rs`
reads, so one ambient input makes every conditional Apple check in the
repository strict. It can only make this harness stricter; nothing here lets an
environment variable weaken a check.
"""

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
    "targets the device and OS it is running on, which is why a runtime case is "
    "attributed to the family whose execution environment compiled it",
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
mode, family, and this level, so a kernel on which contraction is unobservable
yields a plain agreement and a kernel on which it is observable reports which
offline setting the runtime default behaves like, instead of a spurious
disagreement against an arbitrarily chosen one.
"""

ARCHIVE_COMPILER = re.compile(rb"Apple metal version [0-9.]+ \(metalfe-[0-9.]+\)")
"""The runtime compiler's own version string, delimited by a literal prefix and `)`.

Unlike the option names below, this one is unambiguously bounded in the
container, so scanning for it yields the exact string and not a prefix of it.
"""

COMPILER_BUILD = re.compile(rb"metalfe-[0-9]+(?:\.[0-9]+)+")
"""The bare build string a loaded compiler image carries.

The archive form above is what a serialized container spells; a compiler dylib
on disk spells the same build without the surrounding sentence, so this is the
pattern `compiler_build` scans an image with.
"""

COMPILER_IMAGE_MARKERS = ("GPUCompiler", "MTLCompiler")
"""The image-path substrings the dispatch host reports as the runtime compiler.

Both are named because the framework that carries the compiler is not stable
across OS versions: on the measured row `GPUCompiler.framework` is the image
dyld loads into a process that compiles MSL, and no image whose path contains
`MTLCompiler` is loaded at all, even though that framework is present on disk.
Reporting whichever ones load, and recording the paths, is what keeps the record
from naming a framework by expectation.
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
EMITTED_TRIPLE = re.compile(r'^target triple = "([^"]+)"$', re.MULTILINE)

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

    A family whose *own* execution environment is absent is not one of these. It
    does not stop the probe and is not a skip: the compile side still runs and
    the device side is recorded as unmeasured, because that is a per-family fact
    the record has to carry rather than a reason to abandon the run.
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
    NO_DEVICE_OBSERVATION = "no-device-observation"
    NO_EMITTED_ARITHMETIC = "no-emitted-arithmetic"
    ARITHMETIC_NOT_EXECUTED = "arithmetic-not-executed"
    NO_EXECUTION_WITNESS = "no-execution-witness"
    WITNESS_DISAGREES = "witness-disagrees"
    UNEXPECTED_RESULT = "unexpected-result"

    @property
    def is_evidence(self) -> bool:
        """Whether this verdict may be cited as a fact about arithmetic."""
        return self in {Verdict.FLUSHED_TO_ZERO, Verdict.PRESERVED}


class Execution(enum.Enum):
    """Where a family's compiled module may legitimately be dispatched.

    A family is dispatched in its own execution environment or not at all.
    `NONE` is not a degraded form of the others and never becomes one at run
    time: it is the statement that this host has no device of that family, which
    is a measurement boundary rather than a configuration to work around.
    """

    MACOS_HOST = "macos-host-gpu"
    IOS_SIMULATOR = "ios-simulator-runtime"
    NONE = "no-attached-device-of-this-family"


@dataclass(frozen=True)
class Family:
    """One `MetalPlatform` artifact family and how to produce and run its module.

    `target` is the deployment floor the offline driver is asked for. It is not
    necessarily the triple the module ends up declaring — `-std=metal3.1` raises
    it — so the emitted triple is measured per family rather than assumed, and
    recorded beside the requested one.
    """

    name: str
    metal_platform: str
    sdk: str
    target: str
    execution: Execution


FAMILIES: tuple[Family, ...] = (
    Family(
        "macos",
        "MetalPlatform::MacOs",
        "macosx",
        "air64-apple-macos13.0",
        Execution.MACOS_HOST,
    ),
    Family(
        "ios-device",
        "MetalPlatform::IOsDevice",
        "iphoneos",
        "air64-apple-ios16.0",
        Execution.NONE,
    ),
    Family(
        "ios-simulator",
        "MetalPlatform::IOsSimulator",
        "iphonesimulator",
        "air64-apple-ios16.0-simulator",
        Execution.IOS_SIMULATOR,
    ),
)
"""Every family `MetalPlatform` declares, with the SDK and target that emit it.

The deployment floors match the lowest row the checked-in compatibility probe
compiles, so a difference between the two records is a difference in what was
asked and not in which artifact was produced.
"""

FAMILY_BY_NAME = {family.name: family for family in FAMILIES}

HOST_FAMILY = "macos"
"""The family whose execution environment is the machine running the harness.

Used for the one dispatch that is deliberately *not* a per-family measurement:
the cross-family load recorded under `hazard.`.
"""


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
    """One offline compilation selection, within a family's target and SDK."""

    math_mode: str
    optimization: str
    fp_contract: str

    @property
    def key(self) -> str:
        return f"{self.math_mode}.O{self.optimization}.contract-{self.fp_contract}"

    def flags(self, family: Family) -> list[str]:
        return [
            "-target",
            family.target,
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

    There is no target property, so a runtime case's family is decided by which
    execution environment compiled it rather than by a flag.
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
    family: str
    kernel: str
    configuration: Configuration | RuntimeConfiguration

    @property
    def key(self) -> str:
        return f"{self.family}.{self.kernel}.{self.configuration.key}"

    @property
    def is_runtime(self) -> bool:
        return isinstance(self.configuration, RuntimeConfiguration)


def cases(family: str) -> tuple[Case, ...]:
    """Every kernel and configuration pair the recorded findings need, for one family.

    The set is assembled per finding and then deduplicated, so a case shared by
    two findings is compiled and dispatched once and a finding cannot quietly
    lose its configuration when another one changes. The same set is produced for
    every family, so a per-family difference is a difference in what the
    toolchain did and never in what was asked of it.
    """
    selected: list[Case] = []

    def add(kernel: str, mode: str, optimization: str, contract: str) -> None:
        selected.append(Case(family, kernel, Configuration(mode, optimization, contract)))

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


def runtime_cases(family: str) -> tuple[Case, ...]:
    """Every runtime-compilation case for one family, derived from its offline set.

    Deriving it is what keeps the two paths comparable. A runtime case exists for
    each kernel and math mode the offline probe already covers, so no runtime
    case can be added that has nothing to be compared against and no offline case
    can be dropped while its runtime partner survives. Both optimization levels
    the runtime surface offers are swept, so an optimization-dependent runtime
    divergence has somewhere to show up.
    """
    pairs: dict[tuple[str, str], None] = {}
    for case in cases(family):
        assert isinstance(case.configuration, Configuration)
        pairs.setdefault((case.kernel, case.configuration.math_mode), None)
    return tuple(
        Case(family, kernel, RuntimeConfiguration(mode, optimization))
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

    Three fields are `None` rather than empty when the question could not be
    asked, and the distinction is load-bearing in both directions:

    - `compile_options` and `operations` are `None` exactly when the compilation
      path gave the harness no readable module, which is the runtime path's
      situation. `()` would be a *measured* absence of arithmetic.
    - `results` is `None` exactly when the family has no attached device, so
      nothing was dispatched. `()` would be a measured empty dispatch, which no
      dispatch this harness performs can produce.

    None of the three has a default: a construction site has to state which it
    means. `archived_options` and `applied_options` are the runtime path's own
    compile-side facts and are `None` on the offline path; see `scan_archive` for
    why `archived_options` is corroboration and not evidence.
    """

    case: Case
    compile_options: tuple[str, ...] | None
    operations: tuple[FloatOperation, ...] | None
    results: tuple[int, ...] | None
    applied_options: str | None
    archived_options: str | None

    @property
    def kernel(self) -> Kernel:
        return BY_NAME[self.case.kernel]

    @property
    def family(self) -> Family:
        return FAMILY_BY_NAME[self.case.family]

    @property
    def operation_count(self) -> int | None:
        """How many floating-point operations the module emitted, or `None` if unreadable."""
        return None if self.operations is None else len(self.operations)

    @property
    def guard_layers(self) -> tuple[str, ...]:
        """Which layers of the admissibility guard this observation's path can supply.

        An observation with no device-side result supplies only layer 1, which is
        necessary and never sufficient, so it can support no verdict at all. The
        tuple says which layers exist, not whether the observation passed them.
        """
        layers: list[str] = []
        if self.operations is not None:
            layers.append(EMITTED_ARITHMETIC)
        if self.results is not None:
            layers.append(EXECUTION_WITNESS)
        return tuple(layers)

    def result_for(self, operand: int) -> int:
        if self.results is None:
            raise ProbeFailure(
                f"{self.case.key} was never dispatched, so it has no result for {operand:08x}"
            )
        return self.results[OPERANDS.index(operand)]

    def flags_for(self, opcode: str) -> tuple[tuple[str, ...], ...]:
        if self.operations is None:
            raise ProbeFailure(f"{self.case.key} has no readable module to take flags from")
        return tuple(op.flags for op in self.operations if op.opcode == opcode)


def subnormal_verdict(observation: Observation, probe: SubnormalProbe) -> Verdict:
    """Classify one subnormal observation, refusing to over-read a deleted operation.

    The guard layers run before the returned pattern is even consulted; see the
    module documentation for why the emitted operation count alone is not enough
    on this toolchain row, and for why a missing layer is refused rather than
    assumed in either direction.
    """
    if observation.results is None:
        return Verdict.NO_DEVICE_OBSERVATION
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
    used to state a fact. It has no reading at all for an observation that was
    never dispatched, and `result_for` raises rather than inventing one.
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


def _normalized(text: str) -> str:
    """Collapse captured tool output to one line so it can be a record value.

    A record row is one tab-separated line, so a diagnostic that arrives with a
    newline in it would silently split into two rows and `read_record` would
    reject the file it just wrote. Every diagnostic that can reach the record
    comes through here; `record_rows` re-checks the invariant so a future value
    that bypasses this fails loudly instead of corrupting the evidence.
    """
    return " ".join(text.split())


@dataclass(frozen=True)
class Sdk:
    """One resolved SDK and the offline tools reached through it."""

    name: str
    path: str
    version: str
    build: str
    metal_path: str
    metal_version: str
    metallib_version: str


@dataclass(frozen=True)
class Toolchain:
    """The resolved offline compilers, linkers, SDKs, and host compiler.

    One `Sdk` per family, because a family is emitted through its own `--sdk`.
    On the measured row every SDK resolves the *same* `metal` and `metallib`
    binaries from one MetalToolchain asset, which is itself a measurement worth
    recording per family rather than assuming.
    """

    sdks: dict[str, Sdk]
    clang_path: str

    def compile_ir(self, source: Path, destination: Path, case: Case) -> None:
        self._metal(["-S", "-emit-llvm"], source, destination, case)

    def compile_air(self, source: Path, destination: Path, case: Case) -> None:
        self._metal(["-c"], source, destination, case)

    def _metal(self, mode: list[str], source: Path, destination: Path, case: Case) -> None:
        family = FAMILY_BY_NAME[case.family]
        assert isinstance(case.configuration, Configuration)
        command = [
            "xcrun",
            "--sdk",
            family.sdk,
            "metal",
            *case.configuration.flags(family),
            *mode,
            str(source),
            "-o",
            str(destination),
        ]
        result = _run(command)
        if result.returncode != 0:
            raise ProbeFailure(f"metal failed for {case.key}: {result.stderr.strip()}")

    def link(self, air: Path, destination: Path, family: Family) -> None:
        command = ["xcrun", "--sdk", family.sdk, "metallib", str(air), "-o", str(destination)]
        result = _run(command)
        if result.returncode != 0:
            raise ProbeFailure(f"metallib failed for {air.name}: {result.stderr.strip()}")

    def build_host(self, destination: Path, sdk: str, extra: tuple[str, ...] = ()) -> None:
        """Compile the dispatch host for one execution environment's SDK."""
        result = _run(
            [
                "xcrun",
                "--sdk",
                sdk,
                "clang",
                "-fobjc-arc",
                "-O0",
                "-Wall",
                "-Wextra",
                "-Werror",
                *extra,
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
            raise ProbeFailure(
                f"the dispatch host did not build for {sdk}: {_normalized(result.stderr)}"
            )


def _resolve_sdk(name: str) -> Sdk:
    """Resolve one SDK and the offline tools reached through it, or refuse to run."""
    sdk_path = _run(["xcrun", "--sdk", name, "--show-sdk-path"])
    if sdk_path.returncode != 0 or not Path(_first_line(sdk_path.stdout)).is_dir():
        raise ProbeUnavailable(Reason.SDK, f"{name} SDK did not resolve: {sdk_path.stderr.strip()}")
    version = _run(["xcrun", "--sdk", name, "--show-sdk-version"])
    build = _run(["xcrun", "--sdk", name, "--show-sdk-build-version"])
    if version.returncode != 0 or build.returncode != 0:
        raise ProbeUnavailable(Reason.SDK, f"the {name} SDK reported no version or build")
    located = _run(["xcrun", "--sdk", name, "--find", "metal"])
    metal_path = _first_line(located.stdout)
    if located.returncode != 0 or not metal_path:
        raise ProbeUnavailable(Reason.TOOLCHAIN, f"metal was not found by xcrun for {name}")
    versions = {}
    for tool in ("metal", "metallib"):
        reported = _run(["xcrun", "--sdk", name, tool, "--version"])
        versions[tool] = _first_line(reported.stdout)
        if reported.returncode != 0 or not versions[tool]:
            raise ProbeUnavailable(Reason.TOOLCHAIN, f"{tool} reported no version for {name}")
    return Sdk(
        name=name,
        path=_first_line(sdk_path.stdout),
        version=_first_line(version.stdout),
        build=_first_line(build.stdout),
        metal_path=metal_path,
        metal_version=versions["metal"],
        metallib_version=versions["metallib"],
    )


def resolve() -> Toolchain:
    """Resolve every family's SDK, the offline toolchain, and the host compiler, or refuse.

    Every refusal here is a `ProbeUnavailable`, which callers turn into a skip.
    A tool that resolves and then fails raises `ProbeFailure` instead, so a
    broken toolchain cannot be mistaken for an absent one. An SDK is required for
    every family, including the one with no attached device: the compile side is
    the half that has to be complete.
    """
    if platform.system() != "Darwin":
        raise ProbeUnavailable(Reason.TOOLCHAIN, f"host is {platform.system()}, not Darwin")
    if shutil.which("xcrun") is None:
        raise ProbeUnavailable(Reason.TOOLCHAIN, "xcrun is not on PATH")
    sdks = {family.sdk: _resolve_sdk(family.sdk) for family in FAMILIES}
    clang = _run(["xcrun", "--sdk", "macosx", "--find", "clang"])
    clang_path = _first_line(clang.stdout)
    if clang.returncode != 0 or not clang_path:
        raise ProbeUnavailable(Reason.TOOLCHAIN, "clang was not found by xcrun")
    return Toolchain(sdks=sdks, clang_path=clang_path)


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


def emitted_triple(ir: str) -> str:
    """Return the triple the emitted module declares, which is not the one requested.

    `-std=metal3.1` raises the deployment floor, so `air64-apple-macos13.0` is
    emitted as `air64_v26-apple-macosx14.0.0`. Recording what the module says is
    the only per-family compile-side identity that cannot be confused with the
    flag that asked for it.
    """
    found = EMITTED_TRIPLE.search(ir)
    if found is None:
        raise ProbeFailure("the emitted module declared no target triple")
    return found.group(1)


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
class Reported:
    """What the dispatch host reported for one manifest entry."""

    results: tuple[int, ...]
    applied_options: str | None
    archive: str | None


@dataclass(frozen=True)
class Dispatch:
    """What one run of the dispatch host reported for a whole manifest.

    `registry_id` is recorded because it is the one value that says plainly
    which physical GPU answered. On the measured row the iOS Simulator reports a
    different device *name* and the *same* registry ID as the Mac, which is the
    exact reason a simulator result is not evidence about an iOS device.
    """

    device: str
    registry_id: str
    entries: dict[str, Reported]
    compiler_images: tuple[str, ...]


@dataclass(frozen=True)
class Attachment:
    """A family's own execution environment, resolved or precisely absent.

    `launch` is the argv prefix that runs the dispatch host inside it, which is
    empty for the machine the harness runs on and `simctl spawn <udid>` for a
    booted simulator. `detail` is filled exactly when `available` is false, and
    is the reproducible reason the device side of that family is unmeasured.
    """

    family: Family
    available: bool
    detail: str
    launch: tuple[str, ...]
    host_sdk: str
    host_flags: tuple[str, ...]
    identity: tuple[tuple[str, str], ...]


def _manifest_line(key: str, source: Path, function: str, options: str | None) -> str:
    if options is None:
        return "\t".join((key, "library", str(source), function))
    return "\t".join((key, "source", str(source), function, options))


def dispatch_batch(host: Path, attachment: Attachment, manifest: Path, subject: str) -> Dispatch:
    """Run the dispatch host once over a whole manifest and parse its `key=value` lines.

    Every entry comes through here whichever way its library was obtained, so
    the device-side procedure is literally the same code for the offline and
    runtime paths within a family, and a difference between them cannot be an
    artefact of dispatching them differently.
    """
    command = [
        *attachment.launch,
        str(host),
        "batch",
        str(manifest),
        *(f"{value:08x}" for value in OPERANDS),
    ]
    result = _run(command)
    if result.returncode == 3:
        raise ProbeUnavailable(
            Reason.DEVICE, _normalized(result.stderr) or "no default Metal device"
        )
    if result.returncode != 0:
        # Only stderr is quoted. The host's stdout carries the partial results of
        # whichever entries did run, which is bulk rather than diagnosis, and one
        # of these messages is recorded per runtime case in the retained record.
        raise ProbeFailure(
            f"dispatch of {subject} failed with {result.returncode}: {_normalized(result.stderr)}"
        )
    device, registry = "", ""
    images: list[str] = []
    entries: dict[str, Reported] = {}
    key, applied, archive, values = "", None, None, []

    def close() -> None:
        if not key:
            return
        if len(values) != len(OPERANDS):
            raise ProbeFailure(
                f"{subject}: {key} returned {len(values)} results, expected {len(OPERANDS)}"
            )
        if key in entries:
            raise ProbeFailure(f"{subject}: {key} was reported twice")
        entries[key] = Reported(tuple(values), applied, archive)

    for line in result.stdout.splitlines():
        name, _, value = line.partition("=")
        if name == "device":
            device = value
        elif name == "registry-id":
            registry = value
        elif name == "runtime-compiler-image":
            images.append(value)
        elif name == "case":
            close()
            key, applied, archive, values = value, None, None, []
        elif name == "applied":
            applied = value
        elif name == "archive":
            archive = value
        elif name == "archive-unavailable":
            archive = _normalized(f"unavailable:{value}")
        elif name == "result":
            values.append(int(value, 16))
    close()
    if not entries:
        raise ProbeFailure(f"dispatch of {subject} reported no case at all")
    return Dispatch(device, registry, entries, tuple(sorted(set(images))))


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


def compiler_build(images: tuple[str, ...]) -> str:
    """Recover the build string of the compiler images dyld actually loaded.

    Scans every regular file in the directory of every reported image, not the
    reported files themselves, because a loaded image may live in the dyld shared
    cache and have no on-disk copy while its siblings do. Every distinct
    `metalfe-` build found is reported, sorted, so two compilers in one directory
    would be visible rather than silently collapsed.

    This is a weaker identity than the archive's version string: it names the
    build present beside the loaded image, not the one that answered a specific
    compilation. It exists because in the iOS Simulator the archive cannot be
    written at all, and a family whose runtime compiler is unidentified is a
    worse record than one identified this way and labelled.
    """
    builds: set[str] = set()
    for directory in sorted({Path(image).parent for image in images}):
        if not directory.is_dir():
            continue
        for entry in sorted(directory.iterdir()):
            if not entry.is_file():
                continue
            found = COMPILER_BUILD.search(entry.read_bytes())
            if found is not None:
                builds.add(found.group(0).decode("ascii"))
    return " ".join(sorted(builds)) if builds else "unreported"


@dataclass(frozen=True)
class Run:
    """Everything one complete probe execution observed."""

    environment: dict[str, str]
    observations: dict[str, Observation]
    hazards: dict[str, str]

    def of(
        self,
        family: str,
        kernel: str,
        mode: str,
        optimization: str = "2",
        contract: str = "off",
    ) -> Observation:
        """Return one offline observation by its case coordinates, failing loudly if absent."""
        return self._at(Case(family, kernel, Configuration(mode, optimization, contract)).key)

    def runtime(
        self, family: str, kernel: str, mode: str, optimization: str = "default"
    ) -> Observation:
        """Return one runtime-compilation observation by its case coordinates."""
        return self._at(Case(family, kernel, RuntimeConfiguration(mode, optimization)).key)

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

    The candidate set is every offline case *of the same family* for the same
    kernel and math mode at `RUNTIME_PAIRED_OPTIMIZATION`, across whatever
    contraction settings the offline probe recorded. Deriving the set instead of
    naming one row is what keeps a kernel that becomes contraction-sensitive from
    reading as a divergence between the two compilers when it is nothing of the
    kind; restricting it to one family is what keeps a cross-family difference
    from reading as one.
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
            and run.observations[other].case.family == observation.case.family
            and run.observations[other].case.kernel == observation.case.kernel
            and offline.math_mode == configuration.math_mode
            and offline.optimization == RUNTIME_PAIRED_OPTIMIZATION
        }
        if not candidates:
            raise ProbeFailure(f"{key} has no offline case to be compared against")
        if observation.results is None:
            raise ProbeFailure(f"{key} is a runtime case with no dispatch, which cannot happen")
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


GLOBAL_QUALIFYING = ("os_version", "os_build", "machine", "xcode")
"""The host-wide environment fields that make two runs comparable.

`date_utc` is excluded because it changes every run and qualifies nothing. Every
`family.*` field is qualifying too and is added by `qualifying_keys`, because a
family measured through a different SDK, target, execution environment, or
compiler is not the same measurement.
"""


NON_QUALIFYING_SUFFIXES = (".simulator_device_udid", ".simulator_booted_by_probe")
"""Per-family environment fields that identify a run rather than qualify a measurement.

A simulator device's UDID is generated when the device is created, so it differs
between two hosts running identical software and would make the retained record
uncomparable everywhere but the machine that wrote it. Whether this run had to
boot the device says nothing about what the device then computed. Both are
recorded as provenance and excluded here for the same reason `date_utc` is.
"""


def qualifying_keys(environment: dict[str, str]) -> tuple[str, ...]:
    """Every environment field two runs must agree on before their cases are compared."""
    return GLOBAL_QUALIFYING + tuple(
        sorted(
            key
            for key in environment
            if key.startswith("family.") and not key.endswith(NON_QUALIFYING_SUFFIXES)
        )
    )


SIMULATOR_SPAWN_ATTEMPTS = 60
SIMULATOR_SPAWN_INTERVAL = 0.5
"""How long a freshly booted simulator is given to become spawnable.

What this probe needs is a device `simctl spawn` will run a process on, which is
ready well before the system app is. Polling for the thing actually required
takes about 3.7 s on the measured host where `simctl bootstatus -b` takes about
11.5 s, so the wait is bounded by the condition rather than by a proxy for it.
"""


def _simulator_launch() -> tuple[bool, str, tuple[str, ...], tuple[tuple[str, str], ...]]:
    """Resolve an iOS Simulator device to dispatch in, booting it if it is not booted.

    Returns availability, the exact reason when unavailable, the argv prefix, and
    the runtime's identity rows.

    The device is chosen by ordering runtimes and devices by identifier and name,
    not by which one happens to be booted, so the same host chooses the same
    device every run and the retained record stays comparable. It is then booted
    if necessary and **left booted**: several worktrees run this gate
    concurrently and a run that shut a device down would shut it under another
    run, and leaving it booted is what makes every subsequent run cheap. Shutting
    it down again is a host operation for whoever wants the resources back
    (`xcrun simctl shutdown all`), not something a measurement should do behind a
    concurrent reader's back.
    """
    located = _run(["xcrun", "-f", "simctl"])
    simctl = _first_line(located.stdout)
    if located.returncode != 0 or not simctl:
        return False, "simctl was not found by xcrun", (), ()
    runtimes = _run([simctl, "list", "runtimes", "-j"])
    devices = _run([simctl, "list", "devices", "available", "-j"])
    if runtimes.returncode != 0 or devices.returncode != 0:
        return False, "simctl could not list runtimes or devices", (), ()
    try:
        available = [
            runtime
            for runtime in json.loads(runtimes.stdout)["runtimes"]
            if runtime.get("isAvailable") and runtime.get("platform") == "iOS"
        ]
        by_runtime = json.loads(devices.stdout)["devices"]
    except (KeyError, ValueError) as malformed:
        return False, f"simctl reported malformed JSON: {malformed}", (), ()
    if not available:
        return False, "no available iOS simulator runtime is installed", (), ()
    chosen = None
    for runtime in sorted(available, key=lambda entry: entry["identifier"]):
        candidates = sorted(
            by_runtime.get(runtime["identifier"], []),
            key=lambda entry: (entry.get("name", ""), entry["udid"]),
        )
        if candidates:
            chosen = (runtime, candidates[0])
            break
    if chosen is None:
        return False, "no available iOS simulator device exists for any runtime", (), ()
    runtime, device = chosen
    booted_here = device.get("state") != "Booted"
    if booted_here:
        started = _run([simctl, "boot", device["udid"]])
        if started.returncode != 0:
            refused = _normalized(started.stderr)
            return False, f"simctl could not boot {device['udid']}: {refused}", (), ()
        for _attempt in range(SIMULATOR_SPAWN_ATTEMPTS):
            if _run([simctl, "spawn", device["udid"], "/usr/bin/true"]).returncode == 0:
                break
            time.sleep(SIMULATOR_SPAWN_INTERVAL)
        else:
            waited = SIMULATOR_SPAWN_ATTEMPTS * SIMULATOR_SPAWN_INTERVAL
            return False, f"{device['udid']} was not spawnable within {waited:.0f}s", (), ()
    described = f"{runtime['name']} {runtime['version']} build {runtime['buildversion']}"
    identity = (
        ("simulator_runtime", described),
        ("simulator_device", device.get("name", "unreported")),
        ("simulator_device_udid", device["udid"]),
        ("simulator_booted_by_probe", "true" if booted_here else "false"),
    )
    return True, "", (simctl, "spawn", device["udid"]), identity


def attachments() -> dict[str, Attachment]:
    """Resolve every family's own execution environment, or the reason it has none.

    No family borrows another's. `IOsDevice` resolves to `Execution.NONE` here
    unconditionally, because closing it needs a physical iPhone or iPad
    connected to this host and no amount of local configuration substitutes for
    one; the macOS host will happily load and run that family's metallib, which
    is exactly why the refusal is structural rather than a run-time check.
    """
    resolved: dict[str, Attachment] = {}
    for family in FAMILIES:
        if family.execution is Execution.MACOS_HOST:
            resolved[family.name] = Attachment(family, True, "", (), "macosx", (), ())
        elif family.execution is Execution.IOS_SIMULATOR:
            available, detail, launch, identity = _simulator_launch()
            resolved[family.name] = Attachment(
                family, available, detail, launch, "iphonesimulator", (), identity
            )
        else:
            resolved[family.name] = Attachment(
                family,
                False,
                "no iOS device is attached to this host; closing it needs a physical "
                "iPhone or iPad and a dispatch run on that device's own GPU",
                (),
                "",
                (),
                (),
            )
    return resolved


ARCHIVE_PROBE_CASE = "archive-support"


def archive_support(host: Path, attachment: Attachment, work: Path) -> str:
    """Decide whether a binary archive can be serialized in this execution environment.

    Returns the empty string when it can, or the exact reason it cannot. This is
    probed in a one-entry manifest of its own because the failure mode is not a
    returned error: in the iOS Simulator the call aborts the process, so asking
    for an archive inside a manifest that carries measurements would take the
    whole run down with it.
    """
    source = work / "archive_probe.metal"
    source.write_text(BY_NAME["multiply_two"].source(), encoding="utf-8")
    manifest = work / "archive_probe.manifest.tsv"
    options = RuntimeConfiguration("safe", "default").options(work / "archive_probe.metallib")
    manifest.write_text(
        _manifest_line(ARCHIVE_PROBE_CASE, source, ENTRY_POINT, options) + "\n", encoding="utf-8"
    )
    try:
        reported = dispatch_batch(host, attachment, manifest, "the archive-support probe")
    except ProbeFailure as failed:
        return _normalized(str(failed))
    archive = reported.entries[ARCHIVE_PROBE_CASE].archive
    if archive is None:
        return "the dispatch host reported no archive"
    if archive.startswith("unavailable:"):
        return archive.removeprefix("unavailable:")
    return ""


def _observe_offline(
    toolchain: Toolchain, work: Path, case: Case, dispatched: bool
) -> tuple[Observation, Path | None, str]:
    """Compile one offline case, and link it when its family can be dispatched.

    A family with no attached device is compiled and never linked: the emitted
    module answers every compile-side question and a metallib nobody may run
    answers none of them. The compatibility probe is the record that establishes
    each family links.
    """
    family = FAMILY_BY_NAME[case.family]
    kernel = BY_NAME[case.kernel]
    stem = case.key.replace(".", "_")
    source = work / f"{stem}.metal"
    source.write_text(kernel.source(), encoding="utf-8")
    ir_path = work / f"{stem}.ll"
    toolchain.compile_ir(source, ir_path, case)
    ir = ir_path.read_text(encoding="utf-8")
    library: Path | None = None
    if dispatched:
        air_path = work / f"{stem}.air"
        library = work / f"{stem}.metallib"
        toolchain.compile_air(source, air_path, case)
        toolchain.link(air_path, library, family)
    observation = Observation(
        case=case,
        compile_options=compile_options(ir),
        operations=float_operations(ir),
        results=None,
        applied_options=None,
        archived_options=None,
    )
    return observation, library, emitted_triple(ir)


def _cross_family_hazard(
    toolchain: Toolchain, host: Path, attachment: Attachment, work: Path
) -> dict[str, str]:
    """Measure what happens when a foreign family's module is loaded on the host GPU.

    This is the substitute a future edit would reach for when a family has no
    device, so the record states what it actually does rather than leaving a
    reader to assume it fails. It is recorded under `hazard.` and never under
    `case.`: whatever it returns is a fact about the macOS GPU running a foreign
    module, not a fact about the family the module was compiled for.
    """
    measured: dict[str, str] = {}
    for family in FAMILIES:
        if family.execution is not Execution.NONE:
            continue
        case = Case(family.name, "multiply_two", Configuration("safe", "2", "off"))
        stem = f"hazard_{case.key.replace('.', '_')}"
        source = work / f"{stem}.metal"
        source.write_text(BY_NAME[case.kernel].source(), encoding="utf-8")
        air_path = work / f"{stem}.air"
        library = work / f"{stem}.metallib"
        toolchain.compile_air(source, air_path, case)
        toolchain.link(air_path, library, family)
        manifest = work / f"{stem}.manifest.tsv"
        manifest.write_text(
            _manifest_line("hazard", library, ENTRY_POINT, None) + "\n", encoding="utf-8"
        )
        name = f"cross_family_load.{family.name}_module_on_{attachment.family.name}_gpu"
        try:
            reported = dispatch_batch(host, attachment, manifest, name)
        except ProbeFailure as refused:
            measured[name] = _normalized(f"refused: {refused}")
            continue
        results = reported.entries["hazard"].results
        measured[name] = "loaded and ran; results " + " ".join(f"{v:08x}" for v in results)
    return measured


def probe(work_directory: Path) -> Run:
    """Compile every family, dispatch the ones with a device, and classify every case.

    Raises `ProbeUnavailable` when no toolchain, SDK, or host GPU resolves, and
    `ProbeFailure` for anything that goes wrong after they do. A family whose own
    execution environment is absent is neither: its compile side runs and its
    device side is recorded as unmeasured.
    """
    toolchain = resolve()
    work_directory.mkdir(parents=True, exist_ok=True)
    attached = attachments()

    observations: dict[str, Observation] = {}
    triples: dict[str, str] = {}
    devices: dict[str, str] = {}
    registries: dict[str, str] = {}
    runtime_compilers: dict[str, str] = {}
    runtime_images: dict[str, str] = {}
    runtime_builds: dict[str, str] = {}
    hosts: dict[str, Path] = {}

    for family in FAMILIES:
        attachment = attached[family.name]
        work = work_directory / family.name
        work.mkdir(parents=True, exist_ok=True)
        if attachment.available:
            host = work / "numerical_probe_host"
            toolchain.build_host(host, attachment.host_sdk, attachment.host_flags)
            hosts[family.name] = host

        libraries: dict[str, Path] = {}
        for case in cases(family.name):
            observation, library, triple = _observe_offline(
                toolchain, work, case, attachment.available
            )
            observations[case.key] = observation
            if library is not None:
                libraries[case.key] = library
            if triples.setdefault(family.name, triple) != triple:
                raise ProbeFailure(
                    f"{family.name} emitted two triples: {triples[family.name]} then {triple}"
                )
        if not attachment.available:
            unavailable = f"unavailable:{attachment.detail}"
            devices[family.name] = unavailable
            registries[family.name] = unavailable
            runtime_compilers[family.name] = unavailable
            runtime_images[family.name] = unavailable
            runtime_builds[family.name] = unavailable
            continue

        host = hosts[family.name]
        archive_reason = archive_support(host, attachment, work)
        runtime_sources: dict[str, Path] = {}
        archives: dict[str, Path] = {}
        lines: list[str] = []
        for case in cases(family.name):
            lines.append(_manifest_line(case.key, libraries[case.key], ENTRY_POINT, None))
        for case in runtime_cases(family.name):
            assert isinstance(case.configuration, RuntimeConfiguration)
            stem = case.key.replace(".", "_")
            # The runtime path compiles the same bytes the offline path compiled,
            # so the file is written once per case rather than shared: a case
            # that generated different source would otherwise be invisible here.
            source = work / f"{stem}.metal"
            source.write_text(BY_NAME[case.kernel].source(), encoding="utf-8")
            runtime_sources[case.key] = source
            archive = None
            if not archive_reason:
                archive = work / f"{stem}.archive.metallib"
                archives[case.key] = archive
            lines.append(
                _manifest_line(case.key, source, ENTRY_POINT, case.configuration.options(archive))
            )
        manifest = work_directory / f"{family.name}.manifest.tsv"
        manifest.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")
        reported = dispatch_batch(host, attachment, manifest, f"the {family.name} manifest")
        devices[family.name] = reported.device
        registries[family.name] = reported.registry_id or "unreported"
        runtime_images[family.name] = " ".join(reported.compiler_images) or "unreported"
        runtime_builds[family.name] = compiler_build(reported.compiler_images)

        compiler = ""
        for case in cases(family.name):
            entry = reported.entries[case.key]
            observations[case.key] = Observation(
                case=case,
                compile_options=observations[case.key].compile_options,
                operations=observations[case.key].operations,
                results=entry.results,
                applied_options=None,
                archived_options=None,
            )
        for case in runtime_cases(family.name):
            entry = reported.entries[case.key]
            if archive_reason:
                archived = f"unavailable:{archive_reason}"
            elif entry.archive is None or entry.archive.startswith("unavailable:"):
                archived = entry.archive or "unavailable:the host reported no archive"
            else:
                scanned = scan_archive(Path(entry.archive))
                archived = " ".join(scanned.present)
                if compiler and scanned.compiler != compiler:
                    raise ProbeFailure(
                        f"{family.name}: the runtime compiler changed mid-run: "
                        f"{compiler} then {scanned.compiler}"
                    )
                compiler = scanned.compiler
            observations[case.key] = Observation(
                case=case,
                compile_options=None,
                operations=None,
                results=entry.results,
                applied_options=entry.applied_options,
                archived_options=archived,
            )
        runtime_compilers[family.name] = compiler or f"unavailable:{archive_reason}"

    hazards = _cross_family_hazard(
        toolchain, hosts[HOST_FAMILY], attached[HOST_FAMILY], work_directory
    )
    measured = {
        "emitted_triple": triples,
        "device": devices,
        "device_registry_id": registries,
        "runtime_compiler": runtime_compilers,
        "runtime_compiler_images": runtime_images,
        "runtime_compiler_build": runtime_builds,
    }
    return Run(environment(toolchain, attached, measured), observations, hazards)


def environment(
    toolchain: Toolchain,
    attached: dict[str, Attachment],
    measured: dict[str, dict[str, str]],
) -> dict[str, str]:
    """Capture the exact host row and per-family rows every measurement is qualified by.

    A family's offline compiler and its runtime compiler are recorded separately
    and per family, because on this host they are three different builds across
    two families: one offline driver shared by every SDK, and a runtime compiler
    that belongs to the execution environment rather than to the toolchain.
    Collapsing any pair would make a cross-path or cross-family agreement look
    like a tautology and would hide the toolchain whose numerics a
    runtime-compiled kernel actually delivers.
    """
    xcode = _run(["xcodebuild", "-version"])
    captured = {
        "date_utc": datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "os_version": _first_line(_run(["sw_vers", "-productVersion"]).stdout),
        "os_build": _first_line(_run(["sw_vers", "-buildVersion"]).stdout),
        "machine": _first_line(_run(["uname", "-m"]).stdout),
        "xcode": " ".join(xcode.stdout.split()) if xcode.returncode == 0 else "unreported",
    }
    for family in FAMILIES:
        sdk = toolchain.sdks[family.sdk]
        attachment = attached[family.name]
        prefix = f"family.{family.name}"
        captured[f"{prefix}.metal_platform"] = family.metal_platform
        captured[f"{prefix}.sdk"] = sdk.name
        captured[f"{prefix}.sdk_version"] = sdk.version
        captured[f"{prefix}.sdk_build"] = sdk.build
        captured[f"{prefix}.requested_target"] = family.target
        captured[f"{prefix}.metal_version"] = sdk.metal_version
        captured[f"{prefix}.metallib_version"] = sdk.metallib_version
        captured[f"{prefix}.execution"] = (
            family.execution.value if attachment.available else f"unavailable:{attachment.detail}"
        )
        for field, byfamily in measured.items():
            captured[f"{prefix}.{field}"] = byfamily.get(family.name, "unreported")
        for name, value in attachment.identity:
            captured[f"{prefix}.{name}"] = value
    return captured


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
        ("probe.guard_layers.offline_with_device", f"{EMITTED_ARITHMETIC} {EXECUTION_WITNESS}"),
        ("probe.guard_layers.offline_without_device", EMITTED_ARITHMETIC),
        ("probe.guard_layers.runtime", EXECUTION_WITNESS),
    ]
    rows += [
        (f"probe.offline_flag_without_runtime_counterpart.{index}", gap)
        for index, gap in enumerate(OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART)
    ]
    rows += [(f"environment.{key}", value) for key, value in run.environment.items()]
    rows += [(f"hazard.{key}", value) for key, value in sorted(run.hazards.items())]
    for key in sorted(run.observations):
        observation = run.observations[key]
        # A runtime case gets no `float_operations` row at all and a case that
        # was never dispatched gets no `results` row. Writing an empty one would
        # read as a module measured to contain no arithmetic, or as a dispatch
        # that returned nothing, and those are the two readings this harness must
        # never let a record support.
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
        if observation.results is not None:
            rows.append(
                (f"case.{key}.results", " ".join(f"{value:08x}" for value in observation.results))
            )
    for comparison in path_comparisons(run):
        rows.append((f"comparison.{comparison.runtime_case}", comparison.render()))
    rows.append(("probe.status", "complete"))
    # A record row is one tab-separated line. A captured diagnostic that carried
    # a newline would split into two rows that `read_record` then rejects, and a
    # value that carried a tab would silently truncate. Both are corrupted
    # evidence, so the format is enforced where the rows are built rather than
    # trusted to every producer.
    for key, value in rows:
        if "\t" in key or "\n" in key or "\t" in value or "\n" in value:
            raise ProbeFailure(f"record row {key!r} contains a tab or newline in {value!r}")
    return rows


def write_record(run: Run, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    body = "".join(f"{key}\t{value}\n" for key, value in record_rows(run))
    destination.write_text(body, encoding="utf-8")


COMPARED_PREFIXES = ("case.", "comparison.", "hazard.")
"""The record rows a live run must reproduce exactly on the same environment row.

`comparison.` is included so a divergence between the two compilation paths, or
a change in which offline contraction setting the runtime path behaves like,
fails the gate rather than merely being rewritten into the record. `hazard.` is
included because the reason the harness refuses a convenient substitute is a
measurement too: if the macOS runtime ever started refusing a foreign family's
module, the record should notice rather than keep citing an outcome that stopped
happening.
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
    for key in qualifying_keys(run.environment):
        print(f"{key}={run.environment[key]}")
    for key, value in sorted(run.hazards.items()):
        print(f"hazard.{key}={value}")
    for key in sorted(run.observations):
        observation = run.observations[key]
        count = observation.operation_count
        results = (
            "not-dispatched"
            if observation.results is None
            else " ".join(f"{value:08x}" for value in observation.results)
        )
        print(f"{key}\tfp-ops={'unreadable' if count is None else count}\t{results}")
    for comparison in path_comparisons(run):
        print(f"comparison.{comparison.runtime_case}\t{comparison.render()}")
    if parsed.record is not None:
        write_record(run, parsed.record)
        print(f"record={parsed.record}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
