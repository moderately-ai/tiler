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

# The widened matrix, and the two sets it is measured in

The vocabulary this harness reads is not the vocabulary a compiler emits, and the
gap is where its guard fails quietly rather than loudly. Widening the matrix
found exactly that: a source-level `fma` lowers to `@air.fma.f32`, which the
intrinsic pattern did not name, so a kernel whose entire body is one fused
multiply-add was reported as containing no arithmetic at all. A count that reads
zero on a surviving operation is indistinguishable from a deleted one, which is
the reading finding 7 rests on. `FUSED_INTRINSIC` names both spellings now, and
the lesson generalizes: every kernel added here has to be checked against what
the module *emitted*, not against what its source says.

The widened axes are `-fmetal-math-fp32-functions`, which was a pinned flag and
is now swept on both paths; `-O1`, `-O3`, and `-Os`, which join `-O0` and `-O2`;
division, both in the power-of-two form the driver rewrites into a multiply and
in the form it keeps; a source-level `fma` over the constants the contraction
pair already uses; and a two-add chain whose value says where the parentheses
went, which is the smallest shape a reassociation licence can be observed in.

That costs more than the gate should pay on every run, so `cases` assembles two
sets. `covering` keeps at least one case of every kernel, math mode, optimization
level, contraction setting, and fp32-functions value, and every case a recorded
finding cites; `exhaustive`, selected by `TILER_APPLE_NUMERICS_EXHAUSTIVE`, is
the full cross product on the widened axes. `probe.matrix` records which one
produced a record and `matrix_mismatch` refuses to compare one against a run of
the other, because the two pin different case sets and every case they do not
share would otherwise read as decay.

# The second dtype, which is not another row of that matrix

`DTYPES` names `f32` and `f16`, and widening to the second one changed the shape
of the harness in four places at once rather than adding a column. Each dtype
carries its own operand vector, because a subnormal boundary is a property of the
format — `f16`'s smallest normal is `0400`, not `00800000` — its own result
width, so a recorded pattern is four hex digits or eight and a reader may no
longer assume one; its own exact evaluation, through `struct`'s `<e` rather than
`<f`; and its own dispatch shape in `numerical_probe_host.m`, which allocates and
reads back elements of that width. A kernel names its dtype in `Kernel.dtype` and
its name carries the `_f16` suffix exactly when that dtype is not
`DEFAULT_DTYPE`, so every case key recorded while the harness was `f32`-only
keeps its exact meaning and only the new kernels are new keys.

The question the second dtype answers could not be answered by argument.
`air.compile.denorms_disable` is a module-level declaration emitted identically
for both dtypes, which is a reason to expect the flush to be dtype-independent
and is not a measurement of it. It is not: on the measured row every `f16`
arithmetic kernel here **preserves** the subnormals its `f32` twin flushes, under
the same execution witness, in the same math modes, from the same module-level
declaration. So a returned bit pattern is not evidence, a module flag is not
evidence, and neither is another dtype's measurement.
"""

from __future__ import annotations

import argparse
import enum
import hashlib
import json
import math
import os
import platform
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
HOST_SOURCE = HERE / "numerical_probe_host.m"

SCHEMA = "tiler.apple-numerical-behaviour/v6"
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

v4 widens the matrix. `-fmetal-math-fp32-functions` stopped being a fixed flag
and became a swept axis, so `probe.fixed_flags` and `probe.runtime_fixed_options`
no longer name it and `probe.default_fp32_functions` does; a case key carries the
`fpfun-<value>` suffix **exactly when** it departs from that default, so every
v3 key keeps its exact meaning and only the departing cases are new keys.
`-O1`, `-O3`, and `-Os` joined `-O0` and `-O2`; division, a source-level `fma`,
and a two-add reassociation chain joined multiply and add; and `probe.matrix`
names which case matrix produced the record, because the exhaustive sweep and
the covering subset the gate runs are different sets of cases and a record from
one may not be compared against a run of the other.

v5 adds the second dtype. `probe.operands` became the per-dtype
`probe.operands.<dtype>` rows, because the vectors differ, and a `case.*.results`
row is rendered at its kernel's own width — four hex digits for `f16`, eight for
`f32` — so a reader can no longer assume every recorded pattern is 32 bits wide.
`probe.dtypes` names every dtype measured and `probe.default_dtype` names the one
a kernel carries when its name has no dtype suffix, so every v4 case key keeps its
exact meaning and only the `_f16` kernels are new keys.

v6 adds the third dtype and the one row kind a third dtype turned out to need.
`case.*.refusal` is present exactly when a family's device was asked to run a
kernel and declined — on the measured row the iOS Simulator fails pipeline
creation for every `bfloat` module it has itself compiled and linked — and
`environment.family.<name>.device_bfloat_support` carries the per-family answer.
Without it, "no device to ask" and "a device that answered no" would both be a
missing `case.*.results` row, and those are different measurements. Two dtypes
now share a width, so a four-digit `results` row no longer identifies the format
and only the kernel name in the key does. Every v5 key keeps its exact meaning
and only the `_bf16` kernels are new keys.
"""

REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"
"""Turns an absent toolchain, SDK, or GPU from a skip into a failure.

This is deliberately the same variable `crates/tiler-metal/src/golden_compilation.rs`
reads, so one ambient input makes every conditional Apple check in the
repository strict. It can only make this harness stricter; nothing here lets an
environment variable weaken a check.
"""

EXHAUSTIVE = "TILER_APPLE_NUMERICS_EXHAUSTIVE"
"""Selects the full case matrix instead of the covering subset the gate runs.

The widened matrix costs more than the gate should pay on every run, so
`cases` assembles two sets: a covering subset that keeps at least one case of
every kernel, math mode, optimization level, contraction setting, and
`-fmetal-math-fp32-functions` value, and the exhaustive cross product. Setting
this variable to any value selects the exhaustive one.

This is the one environment variable here that changes *what is measured*, so
`probe.matrix` records which set produced a record and `compare_record`'s caller
refuses to compare a record from one against a run of the other. Unlike
`TILER_REQUIRE_TOOLCHAIN` it can therefore make the harness measure *less*, which
is why the retained covering record is the one the gate holds itself to and the
exhaustive record is separate retained evidence rather than a replacement.
"""

MSL_VERSION = "metal3.1"
DEFAULT_FP32_FUNCTIONS = "precise"
FP32_FUNCTION_MODES = ("precise", "fast")
"""`-fmetal-math-fp32-functions`, pinned to `precise` before this axis was swept.

`prototype-metal-numerical-realization` reported that the signed-zero divergence
also reproduces under `=fast`; nothing re-established it until this axis existed.
A case key names the value only when it is not `DEFAULT_FP32_FUNCTIONS`, so every
key recorded while the flag was fixed keeps its exact meaning.
"""

ENTRY_POINT = "tiler_probe"

MATH_MODES = ("safe", "relaxed", "fast")
FP_CONTRACTS = ("off", "on", "fast")
OPTIMIZATIONS = ("0", "1", "2", "3", "s")
"""Every `-O` selection the offline driver is asked for, spelled as the flag's suffix.

`s` yields `-Os`. `0` and `2` are the two the record was built on and the two the
covering subset keeps, because the `-O0`/`-O2` difference in how much arithmetic
survives into the emitted IR is the one this harness already measured to matter.
"""

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
    "-O0/-O1/-O3/-Os: MTLLibraryOptimizationLevel offers Default and Size only, so four "
    "of the five offline optimization levels have no runtime counterpart",
)
"""Every offline selection with no `MTLCompileOptions` property, and what is there instead.

Enumerated by reading the complete `@interface MTLCompileOptions` in
`Metal.framework/Headers/MTLLibrary.h` of macOS SDK 26.5, not by searching it.
`mathMode`, `mathFloatingPointFunctions`, and `languageVersion` are exact
counterparts of `-fmetal-math-mode`, `-fmetal-math-fp32-functions`, and `-std`;
`preprocessorMacros` has no offline selection in use here to correspond to.
`mathFloatingPointFunctions` is an exact counterpart and is swept on both paths,
so it is not listed here.
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

FUSED_INTRINSIC = re.compile(r"@((?:llvm|air)\.(?:fma|fmuladd)\.\S+?)\(")
"""The fused-multiply-add intrinsics a `call` may name, in both spellings.

`air.` is not decoration. This front end lowers a source-level `fma(x, a, b)` to
`tail call float @air.fma.f32(...)` and never to the LLVM spelling, so a pattern
naming only `llvm.` matched nothing and reported a kernel whose whole body is one
`fma` as containing **no** floating-point arithmetic. The verdict still failed
closed — an empty operation list is `no-emitted-arithmetic` and inadmissible —
but the *count* was wrong in the one direction a reader acts on, because a
surviving operation reported as zero is indistinguishable from a deleted one,
which is the reading finding 7 rests on. It stayed latent for exactly as long as
the operation vocabulary was multiply and add. Both spellings are named because
neither is documented as the one the front end will keep using.
"""
COMPILE_OPTIONS = re.compile(r"^!air\.compile_options = !\{(.*)\}$", re.MULTILINE)
METADATA_STRING = re.compile(r'^!(\d+) = !\{!"([^"]+)"\}$', re.MULTILINE)
EMITTED_TRIPLE = re.compile(r'^target triple = "([^"]+)"$', re.MULTILINE)


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
    """What one observation is admissible evidence of.

    Only the five claim members are claims about arithmetic. The rest record
    precisely why the observation cannot support any of them, which is the
    difference between this harness and one that reads bit patterns alone. The
    inadmissible members are shared by every classifier deliberately: an
    observation that cannot support a subnormal claim cannot support an
    evaluation-order claim either, and for the same reason.

    `REASSOCIATED` and `PERMUTED` are separate members because the two
    transformations are separate permissions in this repository's numerical
    vocabulary (ADR 0014): reassociation moves the parentheses over a fixed leaf
    order and permutation moves the leaves. A classifier that reported a
    permuted result as `reassociated` would collapse the distinction the
    reduction contract exists to keep.
    """

    FLUSHED_TO_ZERO = "flushed-to-zero"
    PRESERVED = "preserved"
    LEFT_TO_RIGHT = "left-to-right"
    REASSOCIATED = "reassociated"
    PERMUTED = "permuted"
    NO_DEVICE_OBSERVATION = "no-device-observation"
    DEVICE_REFUSED_DTYPE = "device-refused-dtype"
    NO_EMITTED_ARITHMETIC = "no-emitted-arithmetic"
    ARITHMETIC_NOT_EXECUTED = "arithmetic-not-executed"
    NO_EXECUTION_WITNESS = "no-execution-witness"
    WITNESS_DISAGREES = "witness-disagrees"
    UNEXPECTED_RESULT = "unexpected-result"

    @property
    def is_evidence(self) -> bool:
        """Whether this verdict may be cited as a fact about arithmetic."""
        return self in {
            Verdict.FLUSHED_TO_ZERO,
            Verdict.PRESERVED,
            Verdict.LEFT_TO_RIGHT,
            Verdict.REASSOCIATED,
            Verdict.PERMUTED,
        }


class WitnessStatus(enum.Enum):
    """The exhaustive relation between an observed witness and its two controls."""

    EXECUTED = "executed"
    NOT_EXECUTED = "not-executed"
    DISAGREES = "disagrees"


def witness_status(witness: Witness, observed: int) -> WitnessStatus:
    """Classify one witness without collapsing an unexpected value into deletion."""
    if observed == witness.executed:
        return WitnessStatus.EXECUTED
    if observed == witness.deleted:
        return WitnessStatus.NOT_EXECUTED
    return WitnessStatus.DISAGREES


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
class Dtype:
    """One scalar format the probe measures, complete enough to generate and read it.

    Everything that differs between two floating-point formats lives here, so a
    kernel, a probe, a record row, and a dispatch are written once against the
    dtype rather than against `f32`'s widths. `name` is the spelling the record
    and a kernel's name suffix use; `metal_type` is what the generated source
    declares.

    `sentinel` is the pattern the dispatch host seeds an output buffer with, so an
    element no kernel wrote is distinguishable from one written as a zero. It is
    part of the dtype because it has to be representable in the element width, and
    a guard test holds every declared value to the requirement that no kernel here
    can produce it.
    """

    name: str
    metal_type: str
    unsigned_type: str
    narrowing_cast: str
    struct_format: str
    unsigned_struct_format: str
    bits: int
    exponent_mask: int
    mantissa_mask: int
    quiet_nan: int
    sentinel: int
    operands: tuple[int, ...]

    @property
    def digits(self) -> int:
        """How many hex digits a pattern of this dtype is rendered with, everywhere."""
        return self.bits // 4

    @property
    def mask(self) -> int:
        return (1 << self.bits) - 1

    @property
    def sign_mask(self) -> int:
        return 1 << (self.bits - 1)

    @property
    def canonicalizer(self) -> str:
        """The generated NaN-canonicalization helper's name, which names its dtype."""
        return f"tiler_canonicalize_nan_{self.name}_{self.render(self.quiet_nan)}"

    def render(self, bits: int) -> str:
        """One bit pattern at this dtype's width, which is how every record row spells it."""
        return f"{bits:0{self.digits}x}"

    def pattern(self, bits: int) -> str:
        """The MSL integer literal for one exact bit pattern."""
        return f"0x{self.render(bits)}u"

    def literal(self, bits: int) -> str:
        """The MSL expression that reinterprets an exact pattern as this dtype.

        `narrowing_cast` is not decoration. An unsuffixed integer literal is
        `uint`, and `as_type` requires the two types to have the same size, so a
        dtype narrower than 32 bits needs the pattern converted first while `f32`
        must not have a conversion added — its generated source is byte-identical
        to what produced every retained `f32` row.
        """
        pattern = self.pattern(bits)
        narrowed = f"{self.narrowing_cast}({pattern})" if self.narrowing_cast else pattern
        return f"as_type<{self.metal_type}>({narrowed})"

    def canonicalization(self) -> str:
        """The NaN-canonicalization helper, in the shape the Metal emitter writes it."""
        return (
            f"// Replaces an arithmetic NaN with the canonical pattern "
            f"0x{self.render(self.quiet_nan)}, spelled as\n"
            f"// an integer test exactly as the Metal emitter spells it.\n"
            f"static inline {self.metal_type} {self.canonicalizer}({self.metal_type} value) {{\n"
            f"    {self.unsigned_type} pattern = as_type<{self.unsigned_type}>(value);\n"
            f"    bool nan = (pattern & {self.pattern(self.exponent_mask)}) == "
            f"{self.pattern(self.exponent_mask)}\n"
            f"        && (pattern & {self.pattern(self.mantissa_mask)}) != {self.pattern(0)};\n"
            f"    return nan ? {self.literal(self.quiet_nan)} : value;\n"
            f"}}\n"
        )

    def is_subnormal(self, bits: int) -> bool:
        return bits & self.exponent_mask == 0 and bits & self.mantissa_mask != 0

    def flush(self, bits: int) -> int:
        """Replace a subnormal with the zero of its own sign, which is what finding 3 measured."""
        return bits & self.sign_mask if self.is_subnormal(bits) else bits

    def as_float(self, bits: int) -> float:
        packed = struct.pack(self.unsigned_struct_format, bits)
        return float(struct.unpack(self.struct_format, packed)[0])

    def as_bits(self, value: float) -> int:
        """Narrow a double to this dtype's bits, refusing a value it cannot hold."""
        try:
            narrowed = struct.pack(self.struct_format, value)
        except OverflowError as overflowed:
            raise ProbeFailure(
                f"{value!r} is not representable as {self.name}: {overflowed}"
            ) from overflowed
        return int(struct.unpack(self.unsigned_struct_format, narrowed)[0])


@dataclass(frozen=True)
class BrainFloat(Dtype):
    """A dtype `struct` cannot pack, converted through the `f32` it is the high half of.

    `struct` offers `<e` for `f16` and nothing for `bfloat16`, so the two
    conversions are the one part of `Dtype` this format cannot inherit. It needs
    no separate arithmetic model: a `bfloat16` is exactly an `f32` whose low 16
    mantissa bits are zero, so `struct_format` and `unsigned_struct_format`
    name the `f32` **carrier** the conversion passes through rather than a
    packing of this width, and the widening direction is exact by construction.

    The narrowing direction is the one that needs care, and it is a single
    round-to-nearest-even on the discarded half. Rounding a double to `f32` and
    then to `bfloat16` agrees with rounding it directly to `bfloat16`, by the
    same 2p+2 argument `evaluate` states: `f32`'s 24-bit significand exceeds the
    18 bits that make the second rounding innocuous for this format's 8-bit one.
    """

    def as_float(self, bits: int) -> float:
        return float(struct.unpack(self.struct_format, struct.pack("<I", bits << 16))[0])

    def as_bits(self, value: float) -> int:
        try:
            carrier = struct.pack(self.struct_format, value)
        except OverflowError as overflowed:
            raise ProbeFailure(
                f"{value!r} is not representable as {self.name}: {overflowed}"
            ) from overflowed
        word = int(struct.unpack(self.unsigned_struct_format, carrier)[0])
        upper, lower = word >> 16, word & 0xFFFF
        # Round to nearest, ties to even, on the half this format discards.
        if lower > 0x8000 or (lower == 0x8000 and upper & 1):
            upper += 1
        # `f32`'s range exceeds this format's, so a value `struct` accepted can
        # still round up past the largest finite `bfloat16`. `struct` raises for
        # the wider format and cannot raise for this one, so the refusal the
        # base class gets from `pack` has to be re-derived here.
        if math.isfinite(value) and upper & self.exponent_mask == self.exponent_mask:
            raise ProbeFailure(f"{value!r} is not representable as {self.name}: it rounds to ±inf")
        return upper & self.mask


F32 = Dtype(
    name="f32",
    metal_type="float",
    unsigned_type="uint",
    narrowing_cast="",
    struct_format="<f",
    unsigned_struct_format="<I",
    bits=32,
    exponent_mask=0x7F800000,
    mantissa_mask=0x007FFFFF,
    quiet_nan=0x7FC00000,
    sentinel=0xDEADBEEF,
    operands=(
        0x00000001,  # smallest positive subnormal
        0x00400000,  # mid subnormal; doubling it is the smallest normal
        0x007FFFFF,  # largest subnormal
        0x00800000,  # smallest positive normal; halving it is subnormal
        0x80400000,  # negative mid subnormal, for the sign of the flushed zero
        0x80000000,  # negative zero, which is not subnormal
        0x3EB97EF9,  # an ordinary normal whose scale-then-bias result reveals fusion
        0x3F800000,  # 1.0, the execution witness for the scaling kernels
    ),
)

F16 = Dtype(
    name="f16",
    metal_type="half",
    unsigned_type="ushort",
    narrowing_cast="ushort",
    struct_format="<e",
    unsigned_struct_format="<H",
    bits=16,
    exponent_mask=0x7C00,
    mantissa_mask=0x03FF,
    quiet_nan=0x7E00,
    sentinel=0xDEAD,
    operands=(
        0x0001,  # smallest positive subnormal
        0x0200,  # mid subnormal; doubling it is the smallest normal
        0x03FF,  # largest subnormal
        0x0400,  # smallest positive normal; halving it is subnormal
        0x8200,  # negative mid subnormal, for the sign of the flushed zero
        0x8000,  # negative zero, which is not subnormal
        0x3555,  # an ordinary normal, so a kernel is exercised away from its boundaries
        0x3C00,  # 1.0, the execution witness for the scaling kernels
    ),
)
"""`f16`'s operand vector, entry for entry in the same roles as `f32`'s.

Every position answers the same question its `f32` counterpart does, at this
format's own boundaries: `0400` is the smallest normal where `f32` spells it
`00800000`, and `0200` is the mid subnormal whose double is exactly that smallest
normal. Keeping the roles aligned is what makes a per-dtype difference in a
result a difference in what the hardware did rather than in what it was asked.

`0001` is the one entry whose role is *not* symmetric under halving: `f32`'s
smallest subnormal halved rounds to zero and so does this one, which is why the
result-flush probe uses `0400` and not this operand — a returned zero there is
correct rounding and not a flush.
"""

BF16 = BrainFloat(
    name="bf16",
    metal_type="bfloat",
    unsigned_type="ushort",
    narrowing_cast="ushort",
    struct_format="<f",
    unsigned_struct_format="<I",
    bits=16,
    exponent_mask=0x7F80,
    mantissa_mask=0x007F,
    quiet_nan=0x7FC0,
    sentinel=0xDEAD,
    operands=(
        0x0001,  # smallest positive subnormal
        0x0040,  # mid subnormal; doubling it is the smallest normal
        0x007F,  # largest subnormal
        0x0080,  # smallest positive normal; halving it is subnormal
        0x8040,  # negative mid subnormal, for the sign of the flushed zero
        0x8000,  # negative zero, which is not subnormal
        0x3EAB,  # an ordinary normal, so a kernel is exercised away from its boundaries
        0x3F80,  # 1.0, the execution witness for the scaling kernels
    ),
)
"""`bfloat16`'s operand vector, entry for entry in the same roles as `f32`'s.

**Why this dtype and not another.** `f16` preserving what `f32` flushes has two
explanations that finding 21 could not separate: native narrow-format subnormal
support, or evaluation at a wider internal precision. `f16`'s subnormals are all
`f32` **normals**, so wider-precision evaluation predicts preservation there for
free. This format is the one where the two predictions come apart, because it
carries `f32`'s exponent field width: every `bfloat16` subnormal here is also an
`f32` subnormal, so an `f32`-precision evaluation would meet the very flush
finding 2 measures.

`struct_format` is `f32`'s and `bits` is 16 on purpose — see `BrainFloat`. The
two format fields name the carrier the conversion passes through, not a packing
of this width.
"""

DTYPES: tuple[Dtype, ...] = (F32, F16, BF16)
DTYPE_BY_NAME = {dtype.name: dtype for dtype in DTYPES}

DEFAULT_DTYPE = F32
"""The dtype a kernel carries when its name has no dtype suffix.

Naming `f32` in a kernel's name would rewrite every case key recorded while the
harness measured one dtype, and every citation of one in the research record with
it, for no gain: an unsuffixed kernel name means `f32` and `probe.default_dtype`
says so in the record.
"""


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


class GpuFamily(enum.Enum):
    """A closed device-family requirement understood by the dispatch host."""

    APPLE9 = "apple9"


@dataclass(frozen=True)
class Profile:
    """One indivisible numerical-measurement target.

    The offline target and runtime language are intentionally one value rather
    than independent command-line switches. A record called "unified" is valid
    only when both compiler paths consume this same selection.
    """

    name: str
    schema: str
    msl_version: str
    runtime_language: str
    families: tuple[Family, ...]
    dtypes: tuple[Dtype, ...]
    required_gpu_family: GpuFamily | None = None

    def __post_init__(self) -> None:
        if self.required_gpu_family is not None and not isinstance(
            self.required_gpu_family, GpuFamily
        ):
            raise TypeError("required_gpu_family must be a GpuFamily")

    def family(self, name: str) -> Family:
        """Resolve one family inside this profile, refusing cross-profile use."""
        for family in self.families:
            if family.name == name:
                return family
        raise ProbeFailure(f"{name!r} is not in profile {self.name}")

    def offline_flags(self, family: str, configuration: Configuration) -> list[str]:
        """Render the complete offline selection from this profile alone."""
        selected = self.family(family)
        return [
            "-target",
            selected.target,
            f"-std={self.msl_version}",
            f"-O{configuration.optimization}",
            f"-fmetal-math-mode={configuration.math_mode}",
            f"-fmetal-math-fp32-functions={configuration.fp32_functions}",
            f"-ffp-contract={configuration.fp_contract}",
        ]

    def runtime_options(
        self, configuration: RuntimeConfiguration, archive: Path | None = None
    ) -> str:
        """Render the complete runtime selection from this profile alone."""
        selections = [
            f"math={configuration.math_mode}",
            f"fpfun={configuration.fp32_functions}",
            f"lang={self.runtime_language}",
            f"opt={configuration.optimization}",
        ]
        if archive is not None:
            selections.append(f"archive={archive}")
        return ",".join(selections)

    def accepts_gpu(self, apple9_support: str) -> bool:
        """Apply the closed GPU-family requirement exhaustively."""
        match self.required_gpu_family:
            case None:
                return True
            case GpuFamily.APPLE9:
                return apple9_support == "supported"


LEGACY_PROFILE = Profile(
    "legacy-msl31-all-families",
    SCHEMA,
    MSL_VERSION,
    RUNTIME_LANGUAGE,
    FAMILIES,
    DTYPES,
)

APPLE9_F32_UNIFIED_MSL4_MACOS26 = Profile(
    "apple9-f32-unified-msl4-macos26",
    "tiler.apple-numerical-behaviour/v7",
    "metal4.0",
    "4.0",
    (
        Family(
            "macos",
            "MetalPlatform::MacOs",
            "macosx",
            "air64-apple-macos26.0",
            Execution.MACOS_HOST,
        ),
    ),
    (F32,),
    GpuFamily.APPLE9,
)

PROFILES = {
    profile.name: profile
    for profile in (LEGACY_PROFILE, APPLE9_F32_UNIFIED_MSL4_MACOS26)
}

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

    Every field is a bit pattern of the kernel's own dtype, so "not subnormal" is
    decided by that dtype's boundaries and not by `f32`'s.
    """

    operand: int
    executed: int
    deleted: int


@dataclass(frozen=True)
class SubnormalProbe:
    """One operand whose two possible results separate flushing from preserving.

    `flushing` is the result of substituting a **sign-preserving zero** for every
    subnormal the kernel would otherwise carry — the operand on entry and the
    result of every step — which is what this hardware was measured to do. It is
    a zero only when the flushed subnormal is the whole result: an additive
    kernel whose bias dominates returns a normal value under the same hypothesis.
    `evaluate` derives both fields from the kernel, and a guard test holds every
    declared probe to that derivation rather than to a hand-check.
    """

    operand: int
    preserving: int
    flushing: int


@dataclass(frozen=True)
class OrderProbe:
    """One operand whose two possible results separate an evaluation order from another.

    Unlike `SubnormalProbe` this says nothing about subnormals: it separates the
    left-to-right evaluation the source spells from the reassociated one a
    `reassoc` licence permits. `ordered` is deliberately allowed to equal the
    operand — for the chain kernel it does — which is precisely why the kernel
    still needs an execution witness on a *different* operand: a deleted chain
    and an unreassociated one return the same bits here.
    """

    operand: int
    ordered: int
    reassociated: int


@dataclass(frozen=True)
class PermutationProbe:
    """One operand whose two results separate a contributor order from a permutation of it.

    Deliberately not an `OrderProbe` with a third field. Reassociation moves the
    parentheses over a fixed leaf order and permutation moves the leaves, and
    ADR 0014 keeps them as independent permissions, so the two probes name two
    questions rather than one question with three answers.

    `permuted` is only a discriminator if it is unreachable by reassociating the
    canonical order — otherwise a permuted-looking result would be evidence of
    the neighbouring licence instead. That is a finite property of the chosen
    constants rather than something to assert here, so
    `test_the_permutation_probe_is_unreachable_by_reassociating_the_canonical_order`
    enumerates every parenthesization of the canonical leaf order for every
    operand and holds `permuted` to being absent from all of them.
    """

    operand: int
    ordered: int
    permuted: int


NEGATIVE_ZERO = 0x80000000
POSITIVE_ZERO = 0x00000000

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

DIVIDED_INPUT_FLUSH = SubnormalProbe(
    operand=0x00400000, preserving=0x00AAAAAB, flushing=POSITIVE_ZERO
)
"""`INPUT_FLUSH`'s isolation for a divisor no strength reduction can turn into a shift.

Dividing this subnormal by `0.375` has a *normal* result, so a returned zero can
only come from flushing the operand. The divisor is deliberately not a power of
two: `divide_by_two` measures that the driver rewrites a power-of-two division
into a multiply even under `safe`, so a power-of-two divisor measures the
multiplier a second time rather than measuring division at all.
"""

DIVIDED_NEGATIVE_INPUT_FLUSH = SubnormalProbe(
    operand=0x80400000, preserving=0x80AAAAAB, flushing=NEGATIVE_ZERO
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

DIVIDED_RESULT_FLUSH = SubnormalProbe(
    operand=0x00800000, preserving=0x002AAAAB, flushing=POSITIVE_ZERO
)
"""Dividing the smallest normal by `3.0` has a subnormal result, isolating the result."""

ADDITIVE_INPUT_FLUSH = SubnormalProbe(
    operand=0x80400000, preserving=0x00400000, flushing=0x00800000
)
"""Input flushing isolated on an add whose subnormal operand comes straight from the buffer.

`-2**-127 + 2**-126` is the subnormal `00400000` when the operand is preserved
and the *normal* `00800000` when it is flushed to a signed zero first, because
the bias then stands alone. `flushing` is therefore not a zero: the flushed
subnormal is an addend and not the whole result, which is exactly the case the
"a flush shows up as a returned zero" reading does not cover.

A third outcome is possible and is deliberately left as `unexpected-result`
rather than folded into either: `00000000` would mean the operand survived the
add and the *subnormal result* was flushed instead, which is a different
mechanism from the one this probe isolates and must not read as agreement with
it.
"""

NEGATIVE_ZERO_F16 = 0x8000

INPUT_FLUSH_F16 = SubnormalProbe(operand=0x0200, preserving=0x0400, flushing=POSITIVE_ZERO)
"""`INPUT_FLUSH`'s isolation at `f16`'s boundaries, which are not `f32`'s.

Doubling this subnormal is exactly the smallest normal `0400`, so the isolation
is the same one and the constants are the only thing that moved. The pair exists
so a per-dtype difference in the returned pattern is a difference in what the
hardware did rather than in which question it was asked.
"""

NEGATIVE_INPUT_FLUSH_F16 = SubnormalProbe(
    operand=0x8200, preserving=0x8400, flushing=NEGATIVE_ZERO_F16
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

RESULT_FLUSH_F16 = SubnormalProbe(operand=0x0400, preserving=0x0200, flushing=POSITIVE_ZERO)
"""Halving the smallest normal has an exactly representable *subnormal* result.

The operand is the smallest normal and not the smallest subnormal for a reason
this dtype makes sharper: halving `0001` rounds to zero under exact IEEE
arithmetic, so a returned zero there would be correct rounding rather than a
flush and the probe would not separate anything.
"""

IDENTITY_VALUED_FLUSH_F16 = SubnormalProbe(
    operand=0x0200, preserving=0x0200, flushing=POSITIVE_ZERO
)
"""The `f16` probe for a kernel whose exact result is the operand itself."""

DIVIDED_INPUT_FLUSH_F16 = SubnormalProbe(operand=0x0200, preserving=0x0555, flushing=POSITIVE_ZERO)
"""Dividing this subnormal by `0.375` has a *normal* result, isolating the input."""

DIVIDED_NEGATIVE_INPUT_FLUSH_F16 = SubnormalProbe(
    operand=0x8200, preserving=0x8555, flushing=NEGATIVE_ZERO_F16
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

DIVIDED_RESULT_FLUSH_F16 = SubnormalProbe(operand=0x0400, preserving=0x0155, flushing=POSITIVE_ZERO)
"""Dividing the smallest normal by `3.0` has a subnormal result, isolating the result."""

ADDITIVE_INPUT_FLUSH_F16 = SubnormalProbe(operand=0x8200, preserving=0x0200, flushing=0x0400)
"""`ADDITIVE_INPUT_FLUSH`'s isolation at `f16`'s boundaries.

`-2**-15 + 2**-14` is the subnormal `0200` when the operand is preserved and the
*normal* `0400` when it is flushed to a signed zero first. As in `f32`, neither
candidate is a zero, so this is the probe that would catch a reader assuming a
flush always shows up as one.
"""

NEGATIVE_ZERO_BF16 = 0x8000

INPUT_FLUSH_BF16 = SubnormalProbe(operand=0x0040, preserving=0x0080, flushing=POSITIVE_ZERO)
"""`INPUT_FLUSH`'s isolation at `bfloat16`'s boundaries.

Doubling this subnormal is exactly the smallest normal `0080`, so a returned zero
can only come from flushing the operand. Unlike the `f16` pair, **both** patterns
here are `f32` subnormals as well: `0040` is `00400000` widened and `0080` is
`00800000` widened, which is the property that makes this dtype discriminating.
"""

NEGATIVE_INPUT_FLUSH_BF16 = SubnormalProbe(
    operand=0x8040, preserving=0x8080, flushing=NEGATIVE_ZERO_BF16
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

RESULT_FLUSH_BF16 = SubnormalProbe(operand=0x0080, preserving=0x0040, flushing=POSITIVE_ZERO)
"""Halving the smallest normal has an exactly representable *subnormal* result."""

IDENTITY_VALUED_FLUSH_BF16 = SubnormalProbe(
    operand=0x0040, preserving=0x0040, flushing=POSITIVE_ZERO
)
"""The `bfloat16` probe for a kernel whose exact result is the operand itself."""

DIVIDED_INPUT_FLUSH_BF16 = SubnormalProbe(operand=0x0040, preserving=0x00AB, flushing=POSITIVE_ZERO)
"""Dividing this subnormal by `0.375` has a *normal* result, isolating the input."""

DIVIDED_NEGATIVE_INPUT_FLUSH_BF16 = SubnormalProbe(
    operand=0x8040, preserving=0x80AB, flushing=NEGATIVE_ZERO_BF16
)
"""The same isolation with a negative operand, so the flushed zero's sign shows."""

DIVIDED_RESULT_FLUSH_BF16 = SubnormalProbe(
    operand=0x0080, preserving=0x002B, flushing=POSITIVE_ZERO
)
"""Dividing the smallest normal by `3.0` has a subnormal result, isolating the result."""

ADDITIVE_INPUT_FLUSH_BF16 = SubnormalProbe(operand=0x8040, preserving=0x0040, flushing=0x0080)
"""`ADDITIVE_INPUT_FLUSH`'s isolation at `bfloat16`'s boundaries.

`-2**-127 + 2**-126` is the subnormal `0040` when the operand is preserved and
the *normal* `0080` when it is flushed to a signed zero first. These are the same
two exponents as the `f32` probe, spelled at this format's mantissa width.
"""

REASSOCIATION = OrderProbe(operand=0x3F800000, ordered=0x3F800000, reassociated=0x3F800001)
"""`(1.0 + 2**-24) + 2**-24`, whose value depends on where the parentheses go.

`2**-24` is exactly half an ulp of `1.0`, so each add on its own is a tie that
rounds to even and returns `1.0`; summing the two small terms first gives the
exactly representable `1.0 + 2**-23`. This is the shape a reduction would expose
and the smallest one that exposes it, so it needs no second buffer, no second
indexing scheme, and no change to the operand vector.
"""

CANCELLING_MAGNITUDE = 0x4E800000
"""`2**30`, whose ulp is `128` — large enough to absorb every other contributor here.

Paired with its own negation it is the cancelling pair the permutation chain is
built from. `2**30` and not something larger because the absorption has to be
*exact*: half an ulp is `64`, which strictly exceeds both `2.0` and every operand
in the `f32` vector, so `x + 2**30` rounds to `2**30` with no tie anywhere.
"""

CANCELLING_MAGNITUDE_NEGATED = 0xCE800000
"""`-2**30`. Written as its own constant rather than derived by flipping a sign
bit, because every immediate in this harness is a stated exact pattern."""

ABSORBED_CONTRIBUTOR = 0x40000000
"""`2.0`, the contributor the cancelling pair swallows or does not, depending on order.

It must differ from every operand in the vector, or the permuted result would
collide with the value a *deleted* chain returns on that lane and the witness
would be the only thing left separating them.
"""

PERMUTATION = PermutationProbe(
    operand=0x3F800000, ordered=POSITIVE_ZERO, permuted=ABSORBED_CONTRIBUTOR
)
"""`((x + 2**30) + 2.0) - 2**30`, whose value depends on where the `2.0` sits.

Evaluated as written, `x + 2**30` and then `+ 2.0` both round back to `2**30`, so
the final subtraction returns `+0.0` and the `2.0` is lost. Move that one
contributor past the negated magnitude — `((x + 2**30) - 2**30) + 2.0` — and the
pair cancels first, so the `2.0` survives and the result is `2.0`. Same three
contributors, same left-deep shape, different leaf order.

**This is the discriminator the reassociation chain cannot be.** Every
parenthesization of the canonical leaf order returns either `+0.0` or the operand
itself, never `2.0`, for every operand in the `f32` vector — enumerated
exhaustively by a portable test rather than argued here. So a returned `2.0`
cannot be explained by the `reassoc` licence, which is what makes this probe a
measurement of contributor permutation rather than a second reading of finding 17.
"""


@dataclass(frozen=True)
class Step:
    """One arithmetic statement of a probe kernel.

    `constant` is an exact bit pattern of the kernel's dtype, emitted through
    `as_type`, never a decimal literal, so no rendering step stands between the
    stated constant and the compiled one.
    """

    constant: int
    operator: str


@dataclass(frozen=True)
class Kernel:
    """One probe kernel in the Metal emitter's output shape.

    `steps` are applied left to right, each one its own statement, which is the
    per-statement shape the Metal emitter produces and the shape finding 6's
    contraction defence depends on. `fused` replaces those statements with a
    single source-level `fma` over the same two constants, so the fused and
    unfused kernels differ in exactly one thing.

    `witness` is `None` exactly when the kernel is an identity on every operand
    and therefore cannot prove its own arithmetic ran. `subnormal_probes` names
    every probe this kernel is read with, so a guard test can derive each probe's
    two candidate results from this kernel rather than trusting a literal.

    `dtype` decides the declared buffer element type, the constant spelling, the
    operand vector, and the width every result of this kernel is rendered at. A
    kernel whose dtype is not `DEFAULT_DTYPE` names it in `name`, so a case key
    recorded while the harness measured one dtype keeps its exact meaning.
    """

    name: str
    purpose: str
    steps: tuple[Step, ...]
    canonicalized: bool
    witness: Witness | None
    subnormal_probes: tuple[SubnormalProbe, ...] = ()
    fused: bool = False
    dtype: Dtype = DEFAULT_DTYPE

    def source(self) -> str:
        """Render the complete translation unit for this kernel."""
        scalar = self.dtype.metal_type
        lines = ["#include <metal_stdlib>", "using namespace metal;", ""]
        if self.canonicalized and self.steps:
            lines += [self.dtype.canonicalization()]
        lines += [
            f"kernel void {ENTRY_POINT}(",
            f"        device const {scalar} *b0 [[buffer(0)]],",
            f"        device {scalar} *b1 [[buffer(1)]],",
            "        uint tiler_global_invocation_index [[thread_position_in_grid]]) {",
            "    ulong v0 = ulong(tiler_global_invocation_index);",
            f"    ulong v1 = {len(self.dtype.operands)}ul;",
            "    bool v2 = v0 < v1;",
            "    if (v2) {",
            f"        {scalar} v3 = b0[v0];",
        ]
        register, current = 4, "v3"

        def canonicalize() -> None:
            nonlocal register, current
            if not self.canonicalized:
                return
            helper = self.dtype.canonicalizer
            lines.append(f"        {scalar} v{register} = {helper}({current});")
            current = f"v{register}"
            register += 1

        if self.fused:
            scale, bias = self.steps
            lines.append(f"        {scalar} v{register} = {self.dtype.literal(scale.constant)};")
            lines.append(f"        {scalar} v{register + 1} = {self.dtype.literal(bias.constant)};")
            lines.append(
                f"        {scalar} v{register + 2} = fma({current}, v{register}, v{register + 1});"
            )
            current = f"v{register + 2}"
            register += 3
            canonicalize()
        else:
            for step in self.steps:
                lines.append(f"        {scalar} v{register} = {self.dtype.literal(step.constant)};")
                lines.append(
                    f"        {scalar} v{register + 1} = {current} {step.operator} v{register};"
                )
                current = f"v{register + 1}"
                register += 2
                canonicalize()
        lines += [f"        b1[v0] = {current};", "    }", "}", ""]
        return "\n".join(lines)


def times(constant: int) -> tuple[Step, ...]:
    return (Step(constant, "*"),)


def over(constant: int) -> tuple[Step, ...]:
    return (Step(constant, "/"),)


def scale_then_bias(scale: int, bias: int) -> tuple[Step, ...]:
    return (Step(scale, "*"), Step(bias, "+"))


def summed(*constants: int) -> tuple[Step, ...]:
    """A left-deep chain of adds, which is the canonical fold over these contributors.

    The kernel's own operand is the first contributor and the arguments follow it
    in order, so two kernels built from the same constants in two orders differ in
    exactly the thing a permutation permission governs.
    """
    return tuple(Step(constant, "+") for constant in constants)


KERNELS: tuple[Kernel, ...] = (
    Kernel(
        name="materialize",
        purpose="a load and a store with no arithmetic at all",
        steps=(),
        canonicalized=False,
        witness=None,
    ),
    Kernel(
        name="multiply_two",
        purpose="isolates input flushing: a subnormal operand whose exact result is normal",
        steps=times(0x40000000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40000000, deleted=0x3F800000),
        subnormal_probes=(INPUT_FLUSH, NEGATIVE_INPUT_FLUSH),
    ),
    Kernel(
        name="multiply_half",
        purpose="isolates result flushing: a normal operand whose exact result is subnormal",
        steps=times(0x3F000000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x3F000000, deleted=0x3F800000),
        subnormal_probes=(RESULT_FLUSH,),
    ),
    Kernel(
        name="add_smallest_normal",
        purpose="the only kernel whose add takes a subnormal operand straight from the buffer",
        steps=(Step(0x00800000, "+"),),
        canonicalized=True,
        witness=Witness(operand=0x00800000, executed=0x01000000, deleted=0x00800000),
        subnormal_probes=(ADDITIVE_INPUT_FLUSH,),
    ),
    Kernel(
        name="multiply_one",
        purpose="the identity multiply: no witness exists, so it can prove nothing",
        steps=times(0x3F800000),
        canonicalized=True,
        witness=None,
        subnormal_probes=(IDENTITY_VALUED_FLUSH,),
    ),
    Kernel(
        name="scale_one_bias_zero",
        purpose="the emitter's MultiplyThenAdd shape whose relaxed form deletes its arithmetic",
        steps=scale_then_bias(0x3F800000, POSITIVE_ZERO),
        canonicalized=True,
        witness=Witness(operand=NEGATIVE_ZERO, executed=POSITIVE_ZERO, deleted=NEGATIVE_ZERO),
        subnormal_probes=(IDENTITY_VALUED_FLUSH,),
    ),
    Kernel(
        name="scale_two_bias_one",
        purpose="the shape the checked-in pointwise golden emits",
        steps=scale_then_bias(0x40000000, 0x3F800000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40400000, deleted=0x3F800000),
    ),
    Kernel(
        name="contraction_pair",
        purpose="a multiply and an add as two statements, with no canonicalization between them",
        steps=scale_then_bias(0x3FC00000, 0x3F800000),
        canonicalized=False,
        witness=Witness(operand=0x3F800000, executed=0x40200000, deleted=0x3F800000),
    ),
    Kernel(
        name="contraction_pair_canonicalized",
        purpose="the same pair with the emitter's canonicalization interposed",
        steps=scale_then_bias(0x3FC00000, 0x3F800000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40200000, deleted=0x3F800000),
    ),
    Kernel(
        name="fused_pair",
        purpose="the same two constants as a source-level fma, which contraction cannot unfuse",
        steps=scale_then_bias(0x3FC00000, 0x3F800000),
        canonicalized=False,
        witness=Witness(operand=0x3F800000, executed=0x40200000, deleted=0x3F800000),
        fused=True,
    ),
    Kernel(
        name="divide_by_half",
        purpose="a written division by a power of two, which the driver need not keep as one",
        steps=over(0x3F000000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x40000000, deleted=0x3F800000),
        subnormal_probes=(INPUT_FLUSH, NEGATIVE_INPUT_FLUSH),
    ),
    Kernel(
        name="divide_by_two",
        purpose="the same in the other direction, whose exact result is subnormal",
        steps=over(0x40000000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x3F000000, deleted=0x3F800000),
        subnormal_probes=(RESULT_FLUSH,),
    ),
    Kernel(
        name="divide_by_three_eighths",
        purpose="input flushing through a division the driver keeps: a subnormal whose result "
        "is normal",
        steps=over(0x3EC00000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x402AAAAB, deleted=0x3F800000),
        subnormal_probes=(DIVIDED_INPUT_FLUSH, DIVIDED_NEGATIVE_INPUT_FLUSH),
    ),
    Kernel(
        name="divide_by_three",
        purpose="result flushing through a division the driver keeps: a normal whose result "
        "is subnormal",
        steps=over(0x40400000),
        canonicalized=True,
        witness=Witness(operand=0x3F800000, executed=0x3EAAAAAB, deleted=0x3F800000),
        subnormal_probes=(DIVIDED_RESULT_FLUSH,),
    ),
    Kernel(
        name="reassociation_chain",
        purpose="two adds of half an ulp, whose value says where the parentheses went",
        steps=(Step(0x33800000, "+"), Step(0x33800000, "+")),
        canonicalized=False,
        witness=Witness(operand=0x00800000, executed=0x34000000, deleted=0x00800000),
    ),
    # The permutation pair. These two kernels carry the *same three* contributors
    # in two orders and differ in nothing else, so what separates their results is
    # leaf order alone -- which is the one thing the reassociation chain above
    # cannot isolate, because a `reassoc` licence moves parentheses over a fixed
    # leaf order. `PERMUTATION` states the two candidates and a portable test
    # enumerates every parenthesization of the canonical order to establish that
    # the permuted value is not among them.
    #
    # Both kernels return a value no operand carries, on every lane, so unlike the
    # reassociation chain neither one's ordered result coincides with a deleted
    # chain's. The execution witness is kept anyway: the guard is not weakened for
    # a kernel that happens not to need one of its layers.
    #
    # **Both witness on `80000000` rather than on the probe's `3f800000`, and the
    # reason is the relaxed modes.** Negative zero is the one non-subnormal
    # operand whose result is the same whether the chain is evaluated as written
    # or the cancelling pair is folded away first, because `-0.0 + 2.0` and `2.0`
    # are the same value. A witness on any other operand would report `executed`
    # under `safe` and `disagrees` under `relaxed`, which is a witness measuring
    # the licence under test instead of guarding against deletion.
    Kernel(
        name="permutation_chain",
        purpose="a cancelling pair straddling a third contributor, in canonical order",
        steps=summed(CANCELLING_MAGNITUDE, ABSORBED_CONTRIBUTOR, CANCELLING_MAGNITUDE_NEGATED),
        canonicalized=False,
        witness=Witness(operand=NEGATIVE_ZERO, executed=POSITIVE_ZERO, deleted=NEGATIVE_ZERO),
    ),
    Kernel(
        name="permutation_chain_reordered",
        purpose="the identical three contributors permuted in the source, which moves the value",
        steps=summed(CANCELLING_MAGNITUDE, CANCELLING_MAGNITUDE_NEGATED, ABSORBED_CONTRIBUTOR),
        canonicalized=False,
        witness=Witness(
            operand=NEGATIVE_ZERO, executed=ABSORBED_CONTRIBUTOR, deleted=NEGATIVE_ZERO
        ),
    ),
    # The second dtype. Each kernel below is its `f32` twin with the constants
    # respelled at `f16`'s boundaries and nothing else changed, so a difference
    # in what comes back is a difference in what the hardware did with the dtype
    # rather than a difference in the question. The vocabulary stops at multiply,
    # add, and a surviving division: contraction, a source-level `fma`, and
    # reassociation are `f32`-only here and named as such in the record's
    # boundaries rather than assumed to carry over.
    Kernel(
        name="materialize_f16",
        purpose="a load and a store with no arithmetic at all",
        steps=(),
        canonicalized=False,
        witness=None,
        dtype=F16,
    ),
    Kernel(
        name="multiply_two_f16",
        purpose="isolates input flushing: a subnormal operand whose exact result is normal",
        steps=times(0x4000),
        canonicalized=True,
        witness=Witness(operand=0x3C00, executed=0x4000, deleted=0x3C00),
        subnormal_probes=(INPUT_FLUSH_F16, NEGATIVE_INPUT_FLUSH_F16),
        dtype=F16,
    ),
    Kernel(
        name="multiply_half_f16",
        purpose="isolates result flushing: a normal operand whose exact result is subnormal",
        steps=times(0x3800),
        canonicalized=True,
        witness=Witness(operand=0x3C00, executed=0x3800, deleted=0x3C00),
        subnormal_probes=(RESULT_FLUSH_F16,),
        dtype=F16,
    ),
    Kernel(
        name="add_smallest_normal_f16",
        purpose="the only f16 kernel whose add takes a subnormal operand straight from the buffer",
        steps=(Step(0x0400, "+"),),
        canonicalized=True,
        witness=Witness(operand=0x0400, executed=0x0800, deleted=0x0400),
        subnormal_probes=(ADDITIVE_INPUT_FLUSH_F16,),
        dtype=F16,
    ),
    Kernel(
        name="multiply_one_f16",
        purpose="the identity multiply: no witness exists, so it can prove nothing",
        steps=times(0x3C00),
        canonicalized=True,
        witness=None,
        subnormal_probes=(IDENTITY_VALUED_FLUSH_F16,),
        dtype=F16,
    ),
    Kernel(
        name="scale_one_bias_zero_f16",
        purpose="the emitter's MultiplyThenAdd shape whose relaxed form deletes its arithmetic",
        steps=scale_then_bias(0x3C00, POSITIVE_ZERO),
        canonicalized=True,
        witness=Witness(
            operand=NEGATIVE_ZERO_F16, executed=POSITIVE_ZERO, deleted=NEGATIVE_ZERO_F16
        ),
        subnormal_probes=(IDENTITY_VALUED_FLUSH_F16,),
        dtype=F16,
    ),
    Kernel(
        name="divide_by_three_eighths_f16",
        purpose="input flushing through a division the driver keeps: a subnormal whose result "
        "is normal",
        steps=over(0x3600),
        canonicalized=True,
        witness=Witness(operand=0x3C00, executed=0x4155, deleted=0x3C00),
        subnormal_probes=(DIVIDED_INPUT_FLUSH_F16, DIVIDED_NEGATIVE_INPUT_FLUSH_F16),
        dtype=F16,
    ),
    Kernel(
        name="divide_by_three_f16",
        purpose="result flushing through a division the driver keeps: a normal whose result "
        "is subnormal",
        steps=over(0x4200),
        canonicalized=True,
        witness=Witness(operand=0x3C00, executed=0x3555, deleted=0x3C00),
        subnormal_probes=(DIVIDED_RESULT_FLUSH_F16,),
        dtype=F16,
    ),
    # Contraction, a source-level `fma`, and reassociation at `f16`. Finding 21
    # measured the two dtypes' arithmetic differing while their emitted modules
    # did not, which removed the assumption that an `f32` measurement of what a
    # licence does carries to `f16` -- so findings 6, 16, and 17 needed their own
    # `f16` rows rather than a reading across.
    #
    # The scale is `0x3E02` (1.501953125) and not `1.5h`, and the reason is the
    # whole point of these three kernels. At ten mantissa bits, `x * 1.5 + 1.0`
    # rounds identically whether it is fused or separately rounded for **every**
    # operand in the `f16` vector -- checked exhaustively, and 1,876 of the
    # 32,768 finite non-negative `f16` patterns would discriminate, none of them
    # in the vector. A kernel spelled `1.5h` therefore returns identical bytes
    # under every contraction setting while proving nothing, which is finding
    # 7's no-witness trap wearing a contraction costume. `0x3E02` is one ulp off
    # `1.5h` -- the smallest perturbation that makes the property observable at
    # the vector's ordinary normal `0x3555`, where separate rounding gives
    # `0x3E00` and single rounding gives `0x3E01`. The witness operand `1.0h`
    # gives `0x4101` under both, so the witness stays contraction-independent
    # exactly as the `f32` pair's does.
    Kernel(
        name="contraction_pair_f16",
        purpose="a multiply and an add as two statements, with no canonicalization between them",
        steps=scale_then_bias(0x3E02, 0x3C00),
        canonicalized=False,
        witness=Witness(operand=0x3C00, executed=0x4101, deleted=0x3C00),
        dtype=F16,
    ),
    Kernel(
        name="contraction_pair_canonicalized_f16",
        purpose="the same pair with the emitter's canonicalization interposed",
        steps=scale_then_bias(0x3E02, 0x3C00),
        canonicalized=True,
        witness=Witness(operand=0x3C00, executed=0x4101, deleted=0x3C00),
        dtype=F16,
    ),
    Kernel(
        name="fused_pair_f16",
        purpose="the same two constants as a source-level fma, which contraction cannot unfuse",
        steps=scale_then_bias(0x3E02, 0x3C00),
        canonicalized=False,
        witness=Witness(operand=0x3C00, executed=0x4101, deleted=0x3C00),
        fused=True,
        dtype=F16,
    ),
    # `f16`'s ulp at 1.0 is 2**-10, so half an ulp is 2**-11 = `0x1000` -- the
    # same shape as the `f32` chain's 2**-24, and the reason no new machinery is
    # needed. The discriminator is the operand `1.0h`: added sequentially each
    # half-ulp ties to even and the result stays `0x3C00`, while reassociating
    # the two addends gives `1.0h + 2**-10` = `0x3C01`. The witness operand is
    # the smallest normal `0x0400`, whose chain evaluates left to right to
    # `0x1440`; every value here was computed at `float16` rather than by hand.
    Kernel(
        name="reassociation_chain_f16",
        purpose="two adds of half an ulp, whose value says where the parentheses went",
        steps=(Step(0x1000, "+"), Step(0x1000, "+")),
        canonicalized=False,
        witness=Witness(operand=0x0400, executed=0x1440, deleted=0x0400),
        dtype=F16,
    ),
    # The third dtype, and the one the second could not stand in for. Each kernel
    # below is its `f32` twin with the constants respelled at `bfloat16`'s
    # boundaries and nothing else changed. The operation vocabulary matches the
    # `f16` set exactly — multiply, add, and a surviving division — so a
    # difference between the two narrow formats is a difference in the format.
    # A source-level `fma` is deliberately absent and is a measured limitation
    # rather than a choice: see `FUSED_INTRINSIC` and the record's boundaries.
    Kernel(
        name="materialize_bf16",
        purpose="a load and a store with no arithmetic at all",
        steps=(),
        canonicalized=False,
        witness=None,
        dtype=BF16,
    ),
    Kernel(
        name="multiply_two_bf16",
        purpose="isolates input flushing: a subnormal operand whose exact result is normal",
        steps=times(0x4000),
        canonicalized=True,
        witness=Witness(operand=0x3F80, executed=0x4000, deleted=0x3F80),
        subnormal_probes=(INPUT_FLUSH_BF16, NEGATIVE_INPUT_FLUSH_BF16),
        dtype=BF16,
    ),
    Kernel(
        name="multiply_half_bf16",
        purpose="isolates result flushing: a normal operand whose exact result is subnormal",
        steps=times(0x3F00),
        canonicalized=True,
        witness=Witness(operand=0x3F80, executed=0x3F00, deleted=0x3F80),
        subnormal_probes=(RESULT_FLUSH_BF16,),
        dtype=BF16,
    ),
    Kernel(
        name="add_smallest_normal_bf16",
        purpose="the only bf16 kernel whose add takes a subnormal operand straight from the buffer",
        steps=(Step(0x0080, "+"),),
        canonicalized=True,
        witness=Witness(operand=0x0080, executed=0x0100, deleted=0x0080),
        subnormal_probes=(ADDITIVE_INPUT_FLUSH_BF16,),
        dtype=BF16,
    ),
    Kernel(
        name="multiply_one_bf16",
        purpose="the identity multiply: no witness exists, so it can prove nothing",
        steps=times(0x3F80),
        canonicalized=True,
        witness=None,
        subnormal_probes=(IDENTITY_VALUED_FLUSH_BF16,),
        dtype=BF16,
    ),
    Kernel(
        name="scale_one_bias_zero_bf16",
        purpose="the emitter's MultiplyThenAdd shape whose relaxed form deletes its arithmetic",
        steps=scale_then_bias(0x3F80, POSITIVE_ZERO),
        canonicalized=True,
        witness=Witness(
            operand=NEGATIVE_ZERO_BF16, executed=POSITIVE_ZERO, deleted=NEGATIVE_ZERO_BF16
        ),
        subnormal_probes=(IDENTITY_VALUED_FLUSH_BF16,),
        dtype=BF16,
    ),
    Kernel(
        name="divide_by_three_eighths_bf16",
        purpose="input flushing through a division the driver keeps: a subnormal whose result "
        "is normal",
        steps=over(0x3EC0),
        canonicalized=True,
        witness=Witness(operand=0x3F80, executed=0x402B, deleted=0x3F80),
        subnormal_probes=(DIVIDED_INPUT_FLUSH_BF16, DIVIDED_NEGATIVE_INPUT_FLUSH_BF16),
        dtype=BF16,
    ),
    Kernel(
        name="divide_by_three_bf16",
        purpose="result flushing through a division the driver keeps: a normal whose result "
        "is subnormal",
        steps=over(0x4040),
        canonicalized=True,
        witness=Witness(operand=0x3F80, executed=0x3EAB, deleted=0x3F80),
        subnormal_probes=(DIVIDED_RESULT_FLUSH_BF16,),
        dtype=BF16,
    ),
    # The `bf16` twins of the four kernels above, required rather than optional:
    # `test_every_kernel_names_its_dtype_exactly_when_it_is_not_the_default`
    # holds the two narrow dtypes to identical kernel sets, so that a difference
    # between them is a difference in the format and never in what was asked.
    #
    # `bfloat16` has the same degeneracy `f16` does and it was derived the same
    # way rather than assumed: for scale `1.5bf` and bias `1.0bf`, single and
    # double rounding agree at every operand in this vector too, so that spelling
    # would measure nothing here either. `0x3FBE` (1.484375) is the nearest scale
    # to 1.5 that discriminates at the vector's ordinary normal `0x3EAB`, where
    # separate rounding gives `0x3FC0` and single rounding `0x3FBF`. The witness
    # operand `1.0bf` gives `0x401F` under both.
    Kernel(
        name="contraction_pair_bf16",
        purpose="a multiply and an add as two statements, with no canonicalization between them",
        steps=scale_then_bias(0x3FBE, 0x3F80),
        canonicalized=False,
        witness=Witness(operand=0x3F80, executed=0x401F, deleted=0x3F80),
        dtype=BF16,
    ),
    Kernel(
        name="contraction_pair_canonicalized_bf16",
        purpose="the same pair with the emitter's canonicalization interposed",
        steps=scale_then_bias(0x3FBE, 0x3F80),
        canonicalized=True,
        witness=Witness(operand=0x3F80, executed=0x401F, deleted=0x3F80),
        dtype=BF16,
    ),
    # **There is no `fused_pair_bf16`, and its absence is a measurement.** MSL
    # provides no `bfloat` overload of `fma`: the call promotes to `float` and
    # `metal` rejects `bfloat v6 = fma(v3, v4, v5)` with "cannot initialize a
    # variable of type 'bfloat' with an rvalue of type 'float'". Writing
    # `bfloat(fma(...))` would compile and would measure something else -- a
    # fusion at `f32` precision narrowed afterwards, which is a double rounding
    # this format never performs -- so finding 16's question is not expressible
    # at this width rather than answered negatively at it. The parity assertion
    # in `test_every_kernel_names_its_dtype_exactly_when_it_is_not_the_default`
    # names this one exclusion explicitly, so a *second* divergence still fails.

    # `bfloat16`'s ulp at 1.0 is 2**-7, so half an ulp is 2**-8 = `0x3B80`.
    # Sequentially each addend ties to even and `1.0bf` stays `0x3F80`;
    # reassociated it becomes `0x3F81`. The witness operand is the smallest
    # normal `0x0080`, whose chain evaluates left to right to `0x3C00`.
    Kernel(
        name="reassociation_chain_bf16",
        purpose="two adds of half an ulp, whose value says where the parentheses went",
        steps=(Step(0x3B80, "+"), Step(0x3B80, "+")),
        canonicalized=False,
        witness=Witness(operand=0x0080, executed=0x3C00, deleted=0x0080),
        dtype=BF16,
    ),
)
BY_NAME = {kernel.name: kernel for kernel in KERNELS}

F16_KERNELS = tuple(kernel.name for kernel in KERNELS if kernel.dtype is F16)
"""Every `f16` kernel, in declaration order, so `cases` cannot silently drop one."""

BF16_KERNELS = tuple(kernel.name for kernel in KERNELS if kernel.dtype is BF16)
"""Every `bfloat16` kernel, in declaration order, so `cases` cannot silently drop one."""

NARROW_KERNELS = {F16.name: F16_KERNELS, BF16.name: BF16_KERNELS}
"""The narrow dtypes' kernel lists, so `cases` sweeps them by one rule rather than two.

Keyed by dtype name and not by `Dtype`, because the covering set names the two
load-bearing kernels per dtype by their suffixed spelling and a reader checking
that the sweep reached a dtype should not have to resolve an object identity.
"""


def evaluate(kernel: Kernel, operand: int, *, flushes: bool) -> int:
    """The exact result of one kernel on one operand, under a stated flush hypothesis.

    This is not a measurement and never becomes one: it derives what a probe's
    two candidate results *must* be, so a hand-written `SubnormalProbe` or
    `Witness` literal is checked against arithmetic instead of against a reader's
    attention. Nothing in the admissibility guard calls it.

    `flushes=True` substitutes a sign-preserving zero for the operand on entry
    and for the result of every step, which is the behaviour findings 2 and 3
    measured for `f32`. `flushes=False` is IEEE-754 with subnormals intact. Both
    are hypotheses the harness derives and neither is a claim about a dtype: the
    `f16` kernels are read with the same two candidates and the device chooses
    between them.

    **Why a double intermediate is exact here.** Each step is one `+`, `*`, or
    `/` of two values of the kernel's dtype. Rounding such a result to double and
    then narrowing agrees with rounding it directly, because double's 53-bit
    significand exceeds the 2p+2 bits that make the second rounding innocuous —
    50 for `f32`'s 24-bit significand and 24 for `f16`'s 11-bit one, so the
    margin only widens for the narrower dtype. Signed zero is carried by the
    double path natively, which matters because `(-0.0) + (+0.0)` is `+0.0` and
    that is finding 5. The fused form is a single rounding of `x*a + b`, which no
    single double operation performs, so it is evaluated exactly as a rational
    and narrowed once; a fused kernel whose exact result is zero would lose its
    sign that way and is refused rather than guessed at, because no kernel here
    needs one.
    """
    dtype = kernel.dtype
    current = dtype.flush(operand) if flushes else operand
    if kernel.fused:
        if len(kernel.steps) != 2 or [step.operator for step in kernel.steps] != ["*", "+"]:
            raise ProbeFailure(f"{kernel.name}: a fused kernel must be one multiply then one add")
        scale, bias = kernel.steps
        exact = Fraction(dtype.as_float(current)) * Fraction(
            dtype.as_float(scale.constant)
        ) + Fraction(dtype.as_float(bias.constant))
        if exact == 0:
            raise ProbeFailure(
                f"{kernel.name}: a fused result of zero would need signed-zero handling that "
                f"the rational evaluation cannot supply"
            )
        fused = dtype.as_bits(float(exact))
        return dtype.flush(fused) if flushes else fused
    for step in kernel.steps:
        value, constant = dtype.as_float(current), dtype.as_float(step.constant)
        if step.operator == "*":
            stepped = value * constant
        elif step.operator == "+":
            stepped = value + constant
        elif step.operator == "/":
            if constant == 0.0:
                raise ProbeFailure(f"{kernel.name}: division by zero is not an evaluable step")
            stepped = value / constant
        else:
            raise ProbeFailure(f"{kernel.name}: no evaluation rule for {step.operator!r}")
        current = dtype.as_bits(stepped)
        if flushes:
            current = dtype.flush(current)
    return current


def _fp32_suffix(fp32_functions: str) -> str:
    """The key fragment that names a departure from the pinned default, and nothing otherwise.

    Naming the default in the key would rewrite every case key recorded while
    `-fmetal-math-fp32-functions` was a fixed flag, and every citation of one in
    the research record with it, for no gain: an unsuffixed key means `precise`
    and `probe.default_fp32_functions` says so in the record.
    """
    return "" if fp32_functions == DEFAULT_FP32_FUNCTIONS else f".fpfun-{fp32_functions}"


@dataclass(frozen=True)
class Configuration:
    """One offline compilation selection, within a family's target and SDK."""

    math_mode: str
    optimization: str
    fp_contract: str
    fp32_functions: str = DEFAULT_FP32_FUNCTIONS

    @property
    def key(self) -> str:
        return (
            f"{self.math_mode}.O{self.optimization}.contract-{self.fp_contract}"
            f"{_fp32_suffix(self.fp32_functions)}"
        )

    def flags(self, family: Family, profile: Profile = LEGACY_PROFILE) -> list[str]:
        if profile.family(family.name) != family:
            raise ProbeFailure(f"{family.name!r} does not carry profile {profile.name}'s target")
        return profile.offline_flags(family.name, self)


@dataclass(frozen=True)
class RuntimeConfiguration:
    """One in-process `MTLCompileOptions` selection.

    `languageVersion` is pinned to the counterpart of `MSL_VERSION` rather than
    left at its API default. `mathFloatingPointFunctions` is an exact counterpart
    of `-fmetal-math-fp32-functions` and is swept on both paths, but it is still
    always stated explicitly, because its API default is `Fast` and a runtime
    case that left it unset would not be comparable to any offline row.

    There is no target property, so a runtime case's family is decided by which
    execution environment compiled it rather than by a flag.
    """

    math_mode: str
    optimization: str
    fp32_functions: str = DEFAULT_FP32_FUNCTIONS

    @property
    def key(self) -> str:
        return (
            f"runtime.{self.math_mode}.opt-{self.optimization}{_fp32_suffix(self.fp32_functions)}"
        )

    def options(
        self, archive: Path | None = None, profile: Profile = LEGACY_PROFILE
    ) -> str:
        return profile.runtime_options(self, archive)


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


COVERING = "covering"
EXHAUSTIVE_MATRIX = "exhaustive"


def matrix() -> str:
    """Which case matrix this process measures, named the way the record spells it."""
    return EXHAUSTIVE_MATRIX if os.environ.get(EXHAUSTIVE) is not None else COVERING


def cases(
    family: str,
    selection: str | None = None,
    profile: Profile = LEGACY_PROFILE,
) -> tuple[Case, ...]:
    """Every kernel and configuration pair the recorded findings need, for one family.

    The set is assembled per finding and then deduplicated, so a case shared by
    two findings is compiled and dispatched once and a finding cannot quietly
    lose its configuration when another one changes. The same set is produced for
    every family, so a per-family difference is a difference in what the
    toolchain did and never in what was asked of it.

    Two selections exist because the widened matrix costs more than the gate
    should pay on every run. `covering` keeps at least one case of every kernel,
    math mode, optimization level, contraction setting, and
    `-fmetal-math-fp32-functions` value, and every case any recorded finding
    cites; `exhaustive` is the full cross product on the widened axes. A guard
    test holds the covering set to that coverage claim, so narrowing an axis
    without noticing is a test failure rather than a quieter record.
    """
    profile.family(family)
    selection = matrix() if selection is None else selection
    exhaustive = selection == EXHAUSTIVE_MATRIX
    selected: list[Case] = []

    def add(
        kernel: str,
        mode: str,
        optimization: str,
        contract: str,
        fp32_functions: str = DEFAULT_FP32_FUNCTIONS,
    ) -> None:
        selected.append(
            Case(family, kernel, Configuration(mode, optimization, contract, fp32_functions))
        )

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
    # Input flushing on the additive path, which every other adding kernel here
    # reaches only downstream of a multiply. Both optimization levels, because
    # the level is where the trap kernel's arithmetic survives or does not, and
    # every math mode, because adding a nonzero constant is an identity on no
    # operand and so this kernel keeps its arithmetic where the trap kernel loses
    # it — which is what makes a witnessed relaxed-mode observation possible here.
    for mode in MATH_MODES:
        for optimization in ("0", "2"):
            add("add_smallest_normal", mode, optimization, "off")
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
    # The same pair under the relaxed modes. Contraction was measured only under
    # `safe` until 2026-07-27, so the offline set carried no `relaxed` or `fast`
    # candidate for it and the runtime comparison had nothing to ask. That was a
    # gap in the question rather than an answer: the narrow dtypes are swept in
    # every mode by `NARROW_KERNELS`, so their runtime rows *were* compared, and
    # the divergence that surfaced there is only interpretable against these.
    for mode in ("relaxed", "fast"):
        for contract in FP_CONTRACTS:
            add("contraction_pair", mode, "2", contract)
    # A source-level `fma` over the identical constants, against the same three
    # contraction settings, so what `-ffp-contract` can and cannot unfuse is a
    # difference in one thing.
    for contract in FP_CONTRACTS:
        add("fused_pair", "safe", "2", contract)
    # The same three questions at `f16`. Finding 21 measured the two dtypes'
    # arithmetic differing while their emitted modules did not, so an `f32`
    # measurement of what a contraction licence does is not evidence about
    # `f16`, and these rows are what make findings 6 and 16 per-dtype rather
    # than stated once and read across. The contraction axis is carried here
    # rather than left to the exhaustive selection because these findings cite
    # it directly.
    for contract in FP_CONTRACTS:
        for suffix in ("f16", "bf16"):
            # Every mode, not `safe` alone. `NARROW_KERNELS` sweeps the narrow
            # dtypes in all three modes, so their runtime rows are compared; a
            # contraction axis confined to `safe` would leave `relaxed` and
            # `fast` with only a `contract-off` candidate to be compared against,
            # and the runtime path's own contraction would read as a divergence
            # rather than as the measurement it is.
            for mode in MATH_MODES:
                add(f"contraction_pair_{suffix}", mode, "2", contract)
                add(f"contraction_pair_canonicalized_{suffix}", mode, "2", contract)
        # No `fused_pair_bf16`: MSL has no `bfloat` `fma`. See the kernel list.
        add("fused_pair_f16", "safe", "2", contract)
    # Division, which the operation vocabulary did not previously reach. The two
    # power-of-two divisors are compiled under `safe` alone, because what they
    # establish is a compile-side fact about the driver rather than a second
    # measurement of the flush: the strictest math mode is where a rewrite into a
    # multiply is least expected and therefore most worth recording.
    add("divide_by_half", "safe", "2", "off")
    add("divide_by_two", "safe", "2", "off")
    # The divisors a rewrite cannot absorb, which is where both flush dimensions
    # are actually isolated on a surviving `fdiv`. `arcp` may still substitute a
    # reciprocal multiply under the relaxed modes; the flush hypothesis is
    # insensitive to that, because a flushed operand is a zero either way.
    for mode in MATH_MODES:
        add("divide_by_three_eighths", mode, "2", "off")
        add("divide_by_three", mode, "2", "off")
    # Reassociation, in the smallest shape that exposes it. A `reassoc` licence
    # is attached under `relaxed` and `fast` (finding 1), so whether it is acted
    # on is a device-side question every math mode has to answer.
    for mode in MATH_MODES:
        add("reassociation_chain", mode, "2", "off")
        add("reassociation_chain_f16", mode, "2", "off")
        add("reassociation_chain_bf16", mode, "2", "off")
    # Contributor permutation, in the smallest shape that separates it from
    # reassociation. Both orders are compiled in every math mode: the `safe` row
    # is the one a compile profile reads, and the relaxed modes are what show the
    # canonical order's result is not simply insensitive to everything -- a pair
    # that never moved under any licence would prove the kernel inert rather than
    # the order preserved.
    for mode in MATH_MODES:
        add("permutation_chain", mode, "2", "off")
        add("permutation_chain_reordered", mode, "2", "off")
    # `-fmetal-math-fp32-functions=fast`, against the two findings that would
    # move if it were not confined to the transcendental functions: the flush
    # and the signed-zero divergence.
    for mode in MATH_MODES:
        for kernel in ("multiply_two", "multiply_half", "scale_one_bias_zero"):
            add(kernel, mode, "2", "off", "fast")
    # The three optimization levels the record never reached. The covering set
    # keeps `-O1` under `safe` for the kernel whose surviving operation count is
    # the thing the level is known to move; the exhaustive set sweeps all three
    # levels, all three modes, and the four kernels whose counts or results the
    # level could change.
    widened = ("multiply_two", "multiply_half", "scale_one_bias_zero", "multiply_one")
    if exhaustive:
        for mode in MATH_MODES:
            for optimization in ("1", "3", "s"):
                for kernel in widened:
                    add(kernel, mode, optimization, "off")
    else:
        for optimization in ("1", "3", "s"):
            add("scale_one_bias_zero", "safe", optimization, "off")
    # The narrow dtypes, in every math mode, because a flush that depended on the
    # dtype could depend on the mode too and the `f32` rows these are compared
    # against are measured in all three. `-O0` is kept in the covering set for
    # the two kernels per dtype where the level is load-bearing — the trap
    # kernel, whose surviving operation count moves there, and the input-flush
    # kernel, which carries the headline claim — and the exhaustive sweep takes
    # the rest. Both narrow dtypes are swept by the same rule, so neither can
    # gain coverage the other silently lacks.
    for name, narrow in NARROW_KERNELS.items():
        for mode in MATH_MODES:
            for kernel in narrow:
                add(kernel, mode, "2", "off")
            for kernel in (f"multiply_two_{name}", f"scale_one_bias_zero_{name}"):
                add(kernel, mode, "0", "off")
        if exhaustive:
            for mode in MATH_MODES:
                for kernel in narrow:
                    add(kernel, mode, "0", "off")
                for optimization in ("1", "3", "s"):
                    add(f"scale_one_bias_zero_{name}", mode, optimization, "off")

    unique: dict[str, Case] = {}
    for case in selected:
        unique.setdefault(case.key, case)
    return tuple(
        case for case in unique.values() if BY_NAME[case.kernel].dtype in profile.dtypes
    )


def runtime_cases(
    family: str,
    selection: str | None = None,
    profile: Profile = LEGACY_PROFILE,
) -> tuple[Case, ...]:
    """Every runtime-compilation case for one family, derived from its offline set.

    Deriving it is what keeps the two paths comparable. A runtime case exists for
    each kernel, math mode, and `mathFloatingPointFunctions` value the offline
    probe already covers, so no runtime case can be added that has nothing to be
    compared against and no offline case can be dropped while its runtime partner
    survives. Both optimization levels the runtime surface offers are swept, so
    an optimization-dependent runtime divergence has somewhere to show up.

    An offline case whose optimization level is not `RUNTIME_PAIRED_OPTIMIZATION`
    contributes nothing of its own: `path_comparisons` pairs a runtime case only
    against offline rows at that level, so deriving a runtime case from an `-O1`
    row would produce one with no candidate to be compared against.
    """
    profile.family(family)
    pairs: dict[tuple[str, str, str], None] = {}
    for case in cases(family, selection, profile):
        assert isinstance(case.configuration, Configuration)
        if case.configuration.optimization != RUNTIME_PAIRED_OPTIMIZATION:
            continue
        pairs.setdefault(
            (case.kernel, case.configuration.math_mode, case.configuration.fp32_functions), None
        )
    return tuple(
        Case(family, kernel, RuntimeConfiguration(mode, optimization, fp32_functions))
        for kernel, mode, fp32_functions in pairs
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
    - `results` is `None` exactly when nothing was dispatched. `()` would be a
      measured empty dispatch, which no dispatch this harness performs can
      produce.

    None of the three has a default: a construction site has to state which it
    means. `archived_options` and `applied_options` are the runtime path's own
    compile-side facts and are `None` on the offline path; see `scan_archive` for
    why `archived_options` is corroboration and not evidence.

    `refusal` separates the two reasons `results` can be `None`, which are
    different measurements and must not collapse into one word. An absent
    device asked the question of nothing; a device that **refused this kernel's
    dtype** answered it, with a refusal. The second is a positive fact about a
    real GPU and is the stronger statement, so it carries the exact diagnostic
    the environment reported rather than being inferred from a missing row.
    """

    case: Case
    compile_options: tuple[str, ...] | None
    operations: tuple[FloatOperation, ...] | None
    results: tuple[int, ...] | None
    applied_options: str | None
    archived_options: str | None
    refusal: str = ""

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
                f"{self.case.key} was never dispatched, so it has no result for "
                f"{self.kernel.dtype.render(operand)}"
            )
        return self.results[self.kernel.dtype.operands.index(operand)]

    def flags_for(self, opcode: str) -> tuple[tuple[str, ...], ...]:
        if self.operations is None:
            raise ProbeFailure(f"{self.case.key} has no readable module to take flags from")
        return tuple(op.flags for op in self.operations if op.opcode == opcode)


def inadmissible(observation: Observation) -> Verdict | None:
    """The verdict refusing this observation, or `None` when its result may be read.

    This is the whole guard and it is deliberately independent of what the result
    is going to be read *for*: an observation whose arithmetic cannot be shown to
    have executed supports no claim about that arithmetic, whether the claim is
    about subnormals or about evaluation order. The layers run before the
    returned pattern is consulted at all; see the module documentation for why
    the emitted operation count alone is not enough on this toolchain row, and
    for why a missing layer is refused rather than assumed in either direction.
    """
    if observation.results is None:
        return (
            Verdict.DEVICE_REFUSED_DTYPE if observation.refusal else Verdict.NO_DEVICE_OBSERVATION
        )
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
    return None


def subnormal_verdict(observation: Observation, probe: SubnormalProbe) -> Verdict:
    """Classify one subnormal observation, refusing to over-read a deleted operation."""
    refused = inadmissible(observation)
    if refused is not None:
        return refused
    result = observation.result_for(probe.operand)
    if result == probe.flushing:
        return Verdict.FLUSHED_TO_ZERO
    if result == probe.preserving:
        return Verdict.PRESERVED
    return Verdict.UNEXPECTED_RESULT


def order_verdict(observation: Observation, probe: OrderProbe) -> Verdict:
    """Classify one evaluation-order observation, under the identical guard.

    `unexpected-result` is a real outcome here rather than a defect signal: a
    chain evaluated in a third order, or one whose adds were fused with something
    else, lands there instead of being forced into one of the two the probe
    names.
    """
    refused = inadmissible(observation)
    if refused is not None:
        return refused
    result = observation.result_for(probe.operand)
    if result == probe.ordered:
        return Verdict.LEFT_TO_RIGHT
    if result == probe.reassociated:
        return Verdict.REASSOCIATED
    return Verdict.UNEXPECTED_RESULT


def permutation_verdict(observation: Observation, probe: PermutationProbe) -> Verdict:
    """Classify one contributor-order observation, under the identical guard.

    Separate from `order_verdict` rather than a mode of it, because the two name
    different permissions and returning `reassociated` for a permuted result
    would be the conflation ADR 0014 refuses. A result matching neither candidate
    lands in `unexpected-result`, which here means the chain was evaluated in
    some third way — the honest outcome, not a defect signal.
    """
    refused = inadmissible(observation)
    if refused is not None:
        return refused
    result = observation.result_for(probe.operand)
    if result == probe.ordered:
        return Verdict.LEFT_TO_RIGHT
    if result == probe.permuted:
        return Verdict.PERMUTED
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

    def compile_ir(
        self, source: Path, destination: Path, case: Case, profile: Profile = LEGACY_PROFILE
    ) -> None:
        self._metal(["-S", "-emit-llvm"], source, destination, case, profile)

    def compile_air(
        self, source: Path, destination: Path, case: Case, profile: Profile = LEGACY_PROFILE
    ) -> None:
        self._metal(["-c"], source, destination, case, profile)

    def _metal(
        self,
        mode: list[str],
        source: Path,
        destination: Path,
        case: Case,
        profile: Profile,
    ) -> None:
        family = profile.family(case.family)
        assert isinstance(case.configuration, Configuration)
        command = [
            "xcrun",
            "--sdk",
            family.sdk,
            "metal",
            *case.configuration.flags(family, profile),
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


def resolve(profile: Profile = LEGACY_PROFILE) -> Toolchain:
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
    sdks = {family.sdk: _resolve_sdk(family.sdk) for family in profile.families}
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
    apple9_support: str


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


def _manifest_line(key: str, dtype: Dtype, source: Path, function: str, options: str | None) -> str:
    """One manifest entry, whose dtype the host needs before it allocates anything.

    The dtype is a field rather than something the host infers, because the
    element width decides the buffer size, the operand vector, the sentinel, and
    the width every result is printed at. A host that guessed would read a
    correctly dispatched `f16` kernel back as half as many `f32` values.
    """
    prefix = (key, dtype.name)
    if options is None:
        return "\t".join((*prefix, "library", str(source), function))
    return "\t".join((*prefix, "source", str(source), function, options))


def operand_arguments(profile: Profile = LEGACY_PROFILE) -> list[str]:
    """The `<dtype>=<hex>,...` groups the dispatch host is given, one per dtype.

    Every dtype's vector is passed on every invocation rather than only the ones
    a manifest happens to use, so an entry can never resolve a vector that was
    omitted and the host's rejection of a missing group stays a real check.
    """
    return [
        f"{dtype.name}=" + ",".join(dtype.render(value) for value in dtype.operands)
        for dtype in profile.dtypes
    ]


def dispatch_batch(
    host: Path,
    attachment: Attachment,
    manifest: Path,
    subject: str,
    dtypes: dict[str, Dtype],
    profile: Profile = LEGACY_PROFILE,
) -> Dispatch:
    """Run the dispatch host once over a whole manifest and parse its `key=value` lines.

    Every entry comes through here whichever way its library was obtained, so
    the device-side procedure is literally the same code for the offline and
    runtime paths within a family, and a difference between them cannot be an
    artefact of dispatching them differently.

    `dtypes` maps each case key to the dtype its manifest line declared, which is
    what lets the returned values be checked for count *and* width. A pattern
    wider than the element that produced it is a defect in the host or the
    manifest, and it would otherwise reach the record as a plausible-looking
    result.
    """
    command = [
        *attachment.launch,
        str(host),
        "batch",
        str(manifest),
        *operand_arguments(profile),
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
    device, registry, apple9_support = "", "", ""
    images: list[str] = []
    entries: dict[str, Reported] = {}
    key, applied, archive, values = "", None, None, []

    def close() -> None:
        if not key:
            return
        if key not in dtypes:
            raise ProbeFailure(f"{subject}: {key} was reported but is not in the manifest")
        dtype = dtypes[key]
        if len(values) != len(dtype.operands):
            raise ProbeFailure(
                f"{subject}: {key} returned {len(values)} results, expected {len(dtype.operands)}"
            )
        for value in values:
            if value > dtype.mask:
                raise ProbeFailure(
                    f"{subject}: {key} returned {value:x}, which does not fit {dtype.name}"
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
        elif name == "gpu-family-apple9":
            apple9_support = value
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
    if profile.required_gpu_family is not None and not apple9_support:
        raise ProbeFailure(f"{subject}: the dispatch host reported no Apple9 support state")
    return Dispatch(
        device,
        registry,
        entries,
        tuple(sorted(set(images))),
        apple9_support or "unreported",
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
        fp32_functions: str = DEFAULT_FP32_FUNCTIONS,
    ) -> Observation:
        """Return one offline observation by its case coordinates, failing loudly if absent."""
        configuration = Configuration(mode, optimization, contract, fp32_functions)
        return self._at(Case(family, kernel, configuration).key)

    def runtime(
        self,
        family: str,
        kernel: str,
        mode: str,
        optimization: str = "default",
        fp32_functions: str = DEFAULT_FP32_FUNCTIONS,
    ) -> Observation:
        """Return one runtime-compilation observation by its case coordinates."""
        configuration = RuntimeConfiguration(mode, optimization, fp32_functions)
        return self._at(Case(family, kernel, configuration).key)

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
    dtype: Dtype = DEFAULT_DTYPE

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
            patterns = " ".join(self.dtype.render(value) for value in self.runtime_results)
            return f"{summary} runtime={patterns}"
        return f"{summary} matched={','.join(self.matched)}"


def path_comparisons(run: Run) -> tuple[PathComparison, ...]:
    """Pair every runtime case with the offline cases it can legitimately be compared to.

    The candidate set is every offline case *of the same family* for the same
    kernel, math mode, and `-fmetal-math-fp32-functions` value at
    `RUNTIME_PAIRED_OPTIMIZATION`, across whatever contraction settings the
    offline probe recorded. Deriving the set instead of naming one row is what
    keeps a kernel that becomes contraction-sensitive from reading as a
    divergence between the two compilers when it is nothing of the kind;
    restricting it to one family is what keeps a cross-family difference from
    reading as one; and matching the fp32-functions value is what keeps the
    `precise` and `fast` runtime cases from being compared against each other's
    offline rows, which `mathFloatingPointFunctions` can express exactly.
    """
    compared: list[PathComparison] = []
    for key in sorted(run.observations):
        observation = run.observations[key]
        configuration = observation.case.configuration
        if not isinstance(configuration, RuntimeConfiguration):
            continue
        if observation.refusal:
            # Neither path ran, so there is nothing to compare and no divergence
            # to hide: the refusal is recorded on both this case and its offline
            # candidates, and the environment row names the family it covers.
            continue
        candidates = {
            other: run.observations[other].results
            for other in sorted(run.observations)
            for offline in [run.observations[other].case.configuration]
            if isinstance(offline, Configuration)
            and run.observations[other].case.family == observation.case.family
            and run.observations[other].case.kernel == observation.case.kernel
            and offline.math_mode == configuration.math_mode
            and offline.fp32_functions == configuration.fp32_functions
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
                dtype=observation.kernel.dtype,
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


def attachments(profile: Profile = LEGACY_PROFILE) -> dict[str, Attachment]:
    """Resolve every family's own execution environment, or the reason it has none.

    No family borrows another's. `IOsDevice` resolves to `Execution.NONE` here
    unconditionally, because closing it needs a physical iPhone or iPad
    connected to this host and no amount of local configuration substitutes for
    one; the macOS host will happily load and run that family's metallib, which
    is exactly why the refusal is structural rather than a run-time check.
    """
    resolved: dict[str, Attachment] = {}
    for family in profile.families:
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
BFLOAT_PROBE_CASE = "bfloat-support"


def archive_support(
    host: Path,
    attachment: Attachment,
    work: Path,
    profile: Profile = LEGACY_PROFILE,
) -> str:
    """Decide whether a binary archive can be serialized in this execution environment.

    Returns the empty string when it can, or the exact reason it cannot. This is
    probed in a one-entry manifest of its own because the failure mode is not a
    returned error: in the iOS Simulator the call aborts the process, so asking
    for an archive inside a manifest that carries measurements would take the
    whole run down with it.
    """
    kernel = BY_NAME["multiply_two"]
    source = work / "archive_probe.metal"
    source.write_text(kernel.source(), encoding="utf-8")
    manifest = work / "archive_probe.manifest.tsv"
    options = RuntimeConfiguration("safe", "default").options(
        work / "archive_probe.metallib", profile
    )
    manifest.write_text(
        _manifest_line(ARCHIVE_PROBE_CASE, kernel.dtype, source, ENTRY_POINT, options) + "\n",
        encoding="utf-8",
    )
    try:
        reported = dispatch_batch(
            host,
            attachment,
            manifest,
            "the archive-support probe",
            {ARCHIVE_PROBE_CASE: kernel.dtype},
            profile,
        )
    except ProbeFailure as failed:
        return _normalized(str(failed))
    if not profile.accepts_gpu(reported.apple9_support):
        raise ProbeFailure(
            f"profile {profile.name} requires Apple9, but the device reported "
            f"{reported.apple9_support}"
        )
    archive = reported.entries[ARCHIVE_PROBE_CASE].archive
    if archive is None:
        return "the dispatch host reported no archive"
    if archive.startswith("unavailable:"):
        return archive.removeprefix("unavailable:")
    return ""


def bfloat_support(
    toolchain: Toolchain,
    host: Path,
    attachment: Attachment,
    work: Path,
    profile: Profile = LEGACY_PROFILE,
) -> str:
    """Decide whether this execution environment will run a `bfloat` kernel at all.

    Returns the empty string when it will, or the exact reason it will not. Like
    `archive_support` this needs its own one-entry manifest, and for a stronger
    version of the same reason: on the measured row the iOS Simulator compiles a
    `bfloat` module and links it without complaint, then fails **pipeline
    creation** with `XPC_ERROR_CONNECTION_INTERRUPTED`, and `dispatch_batch`
    treats a nonzero exit as a `ProbeFailure` that takes the whole run with it.
    Asking first is what turns that into a recorded per-family boundary instead
    of an aborted measurement of every other dtype.

    **This is a capability question, not a fallback.** A family that answers yes
    dispatches its `bfloat` cases normally and a failure there is still a hard
    `ProbeFailure`, because a device that accepted the probe and then refused a
    real case is a defect and not a boundary. Nothing here is retried, softened,
    or substituted: the only thing a "no" buys is a `refusal` string on the
    cases it covers.

    The probe kernel is `multiply_two_bf16` and not `materialize_bf16`
    deliberately. It is the kernel carrying the headline claim, so a family this
    returns "" for is one where the load-bearing case is known to reach the GPU;
    probing with the arithmetic-free kernel could pass on an environment that
    supports the *type* in a signature and not the arithmetic, which is the one
    outcome that would make this probe worse than useless.

    That choice costs one distinction, which is why `bfloat_dispatch_probe.py`
    exists beside this file: asking with an arithmetic kernel cannot say whether
    a refusal is about the format or about operating on it. On the measured row
    it is the format — the arithmetic-free kernel is refused too — but that is a
    separate one-off measurement and not something this function establishes.
    """
    kernel = BY_NAME["multiply_two_bf16"]
    source = work / "bfloat_probe.metal"
    source.write_text(kernel.source(), encoding="utf-8")
    case = Case(attachment.family.name, kernel.name, Configuration("safe", "2", "off"))
    air_path = work / "bfloat_probe.air"
    library = work / "bfloat_probe.metallib"
    try:
        toolchain.compile_air(source, air_path, case, profile)
        toolchain.link(air_path, library, attachment.family)
    except ProbeFailure as failed:
        return _normalized(str(failed))
    manifest = work / "bfloat_probe.manifest.tsv"
    manifest.write_text(
        _manifest_line(BFLOAT_PROBE_CASE, kernel.dtype, library, ENTRY_POINT, None) + "\n",
        encoding="utf-8",
    )
    try:
        dispatch_batch(
            host,
            attachment,
            manifest,
            "the bfloat-support probe",
            {BFLOAT_PROBE_CASE: kernel.dtype},
            profile,
        )
    except ProbeFailure as failed:
        return _normalized(str(failed))
    return ""


def _observe_offline(
    toolchain: Toolchain,
    work: Path,
    case: Case,
    dispatched: bool,
    profile: Profile = LEGACY_PROFILE,
) -> tuple[Observation, Path | None, str]:
    """Compile one offline case, and link it when its family can be dispatched.

    A family with no attached device is compiled and never linked: the emitted
    module answers every compile-side question and a metallib nobody may run
    answers none of them. The compatibility probe is the record that establishes
    each family links.
    """
    family = profile.family(case.family)
    kernel = BY_NAME[case.kernel]
    stem = case.key.replace(".", "_")
    source = work / f"{stem}.metal"
    source.write_text(kernel.source(), encoding="utf-8")
    ir_path = work / f"{stem}.ll"
    toolchain.compile_ir(source, ir_path, case, profile)
    ir = ir_path.read_text(encoding="utf-8")
    library: Path | None = None
    if dispatched:
        air_path = work / f"{stem}.air"
        library = work / f"{stem}.metallib"
        toolchain.compile_air(source, air_path, case, profile)
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
    toolchain: Toolchain,
    host: Path,
    attachment: Attachment,
    work: Path,
    profile: Profile = LEGACY_PROFILE,
) -> dict[str, str]:
    """Measure what happens when a foreign family's module is loaded on the host GPU.

    This is the substitute a future edit would reach for when a family has no
    device, so the record states what it actually does rather than leaving a
    reader to assume it fails. It is recorded under `hazard.` and never under
    `case.`: whatever it returns is a fact about the macOS GPU running a foreign
    module, not a fact about the family the module was compiled for.
    """
    measured: dict[str, str] = {}
    for family in profile.families:
        if family.execution is not Execution.NONE:
            continue
        case = Case(family.name, "multiply_two", Configuration("safe", "2", "off"))
        kernel = BY_NAME[case.kernel]
        stem = f"hazard_{case.key.replace('.', '_')}"
        source = work / f"{stem}.metal"
        source.write_text(kernel.source(), encoding="utf-8")
        air_path = work / f"{stem}.air"
        library = work / f"{stem}.metallib"
        toolchain.compile_air(source, air_path, case, profile)
        toolchain.link(air_path, library, family)
        manifest = work / f"{stem}.manifest.tsv"
        manifest.write_text(
            _manifest_line("hazard", kernel.dtype, library, ENTRY_POINT, None) + "\n",
            encoding="utf-8",
        )
        name = f"cross_family_load.{family.name}_module_on_{attachment.family.name}_gpu"
        try:
            reported = dispatch_batch(
                host, attachment, manifest, name, {"hazard": kernel.dtype}, profile
            )
        except ProbeFailure as refused:
            measured[name] = _normalized(f"refused: {refused}")
            continue
        results = reported.entries["hazard"].results
        measured[name] = "loaded and ran; results " + " ".join(
            kernel.dtype.render(value) for value in results
        )
    return measured


def probe(work_directory: Path, profile: Profile = LEGACY_PROFILE) -> Run:
    """Compile every family, dispatch the ones with a device, and classify every case.

    Raises `ProbeUnavailable` when no toolchain, SDK, or host GPU resolves, and
    `ProbeFailure` for anything that goes wrong after they do. A family whose own
    execution environment is absent is neither: its compile side runs and its
    device side is recorded as unmeasured.
    """
    toolchain = resolve(profile)
    work_directory.mkdir(parents=True, exist_ok=True)
    attached = attachments(profile)

    observations: dict[str, Observation] = {}
    triples: dict[str, str] = {}
    devices: dict[str, str] = {}
    registries: dict[str, str] = {}
    runtime_compilers: dict[str, str] = {}
    runtime_images: dict[str, str] = {}
    runtime_builds: dict[str, str] = {}
    bfloat_reasons: dict[str, str] = {}
    apple9_support: dict[str, str] = {}
    hosts: dict[str, Path] = {}

    for family in profile.families:
        attachment = attached[family.name]
        work = work_directory / family.name
        work.mkdir(parents=True, exist_ok=True)
        if attachment.available:
            host = work / "numerical_probe_host"
            toolchain.build_host(host, attachment.host_sdk, attachment.host_flags)
            hosts[family.name] = host

        libraries: dict[str, Path] = {}
        for case in cases(family.name, profile=profile):
            observation, library, triple = _observe_offline(
                toolchain, work, case, attachment.available, profile
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
            # Not "unsupported": this family has no device to ask, which is a
            # different fact from a device that answered no.
            bfloat_reasons[family.name] = unavailable
            apple9_support[family.name] = unavailable
            continue

        host = hosts[family.name]
        archive_reason = archive_support(host, attachment, work, profile)
        # Asked once per family, before any measured case is dispatched. A
        # family that refuses gets its `bf16` cases left out of the manifest and
        # recorded as refused; every other dtype in the same family is measured
        # exactly as before, which is the whole point of asking separately.
        if BF16 in profile.dtypes:
            bfloat_reason = bfloat_support(toolchain, host, attachment, work, profile)
            bfloat_reasons[family.name] = bfloat_reason or "supported"
        else:
            bfloat_reason = ""
            bfloat_reasons[family.name] = "unmeasured-by-profile"

        def refused(case: Case, reason: str = bfloat_reason) -> bool:
            return bool(reason) and BY_NAME[case.kernel].dtype is BF16

        runtime_sources: dict[str, Path] = {}
        archives: dict[str, Path] = {}
        manifest_dtypes: dict[str, Dtype] = {}
        lines: list[str] = []
        for case in cases(family.name, profile=profile):
            if refused(case):
                continue
            dtype = BY_NAME[case.kernel].dtype
            manifest_dtypes[case.key] = dtype
            lines.append(_manifest_line(case.key, dtype, libraries[case.key], ENTRY_POINT, None))
        for case in runtime_cases(family.name, profile=profile):
            assert isinstance(case.configuration, RuntimeConfiguration)
            if refused(case):
                continue
            kernel = BY_NAME[case.kernel]
            manifest_dtypes[case.key] = kernel.dtype
            stem = case.key.replace(".", "_")
            # The runtime path compiles the same bytes the offline path compiled,
            # so the file is written once per case rather than shared: a case
            # that generated different source would otherwise be invisible here.
            source = work / f"{stem}.metal"
            source.write_text(kernel.source(), encoding="utf-8")
            runtime_sources[case.key] = source
            archive = None
            if not archive_reason:
                archive = work / f"{stem}.archive.metallib"
                archives[case.key] = archive
            lines.append(
                _manifest_line(
                    case.key,
                    kernel.dtype,
                    source,
                    ENTRY_POINT,
                    case.configuration.options(archive, profile),
                )
            )
        manifest = work_directory / f"{family.name}.manifest.tsv"
        manifest.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")
        reported = dispatch_batch(
            host,
            attachment,
            manifest,
            f"the {family.name} manifest",
            manifest_dtypes,
            profile,
        )
        apple9_support[family.name] = reported.apple9_support
        if not profile.accepts_gpu(reported.apple9_support):
            raise ProbeFailure(
                f"profile {profile.name} requires Apple9, but the device reported "
                f"{reported.apple9_support}"
            )
        devices[family.name] = reported.device
        registries[family.name] = reported.registry_id or "unreported"
        runtime_images[family.name] = " ".join(reported.compiler_images) or "unreported"
        runtime_builds[family.name] = compiler_build(reported.compiler_images)

        compiler = ""
        for case in cases(family.name, profile=profile):
            if refused(case):
                # The compile side already ran and is kept: `bfloat` compiles and
                # links for this family, and that a module the device will not
                # accept still declares `air.compile.denorms_disable` is a
                # measurement in its own right.
                observations[case.key] = Observation(
                    case=case,
                    compile_options=observations[case.key].compile_options,
                    operations=observations[case.key].operations,
                    results=None,
                    applied_options=None,
                    archived_options=None,
                    refusal=bfloat_reason,
                )
                continue
            entry = reported.entries[case.key]
            observations[case.key] = Observation(
                case=case,
                compile_options=observations[case.key].compile_options,
                operations=observations[case.key].operations,
                results=entry.results,
                applied_options=None,
                archived_options=None,
            )
        for case in runtime_cases(family.name, profile=profile):
            if refused(case):
                observations[case.key] = Observation(
                    case=case,
                    compile_options=None,
                    operations=None,
                    results=None,
                    applied_options=None,
                    archived_options=None,
                    refusal=bfloat_reason,
                )
                continue
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
        toolchain, hosts[HOST_FAMILY], attached[HOST_FAMILY], work_directory, profile
    )
    measured = {
        "emitted_triple": triples,
        "device": devices,
        "device_registry_id": registries,
        "runtime_compiler": runtime_compilers,
        "runtime_compiler_images": runtime_images,
        "runtime_compiler_build": runtime_builds,
        "device_bfloat_support": bfloat_reasons,
    }
    if profile is not LEGACY_PROFILE:
        measured["device_apple9_support"] = apple9_support
    return Run(environment(toolchain, attached, measured, profile), observations, hazards)


def environment(
    toolchain: Toolchain,
    attached: dict[str, Attachment],
    measured: dict[str, dict[str, str]],
    profile: Profile = LEGACY_PROFILE,
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
    for family in profile.families:
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


def record_rows(
    run: Run,
    profile: Profile = LEGACY_PROFILE,
    evidence: dict[str, str] | None = None,
) -> list[tuple[str, str]]:
    """Render one run as the ordered key/value rows of the checked-in record."""
    revision = _run(["git", "-C", str(REPOSITORY), "rev-parse", "HEAD"])
    rows: list[tuple[str, str]] = [
        ("schema", profile.schema),
        ("probe.repository_base_revision", _first_line(revision.stdout) or "unreported"),
        ("probe.harness_sha256", digest(Path(__file__).resolve())),
        ("probe.host_source_sha256", digest(HOST_SOURCE)),
        ("probe.matrix", matrix()),
        ("probe.fixed_flags", f"-std={profile.msl_version}"),
        ("probe.default_fp32_functions", DEFAULT_FP32_FUNCTIONS),
        ("probe.entry_point", ENTRY_POINT),
        ("probe.dtypes", " ".join(dtype.name for dtype in profile.dtypes)),
        ("probe.default_dtype", DEFAULT_DTYPE.name),
        ("probe.runtime_fixed_options", f"lang={profile.runtime_language}"),
        ("probe.runtime_paired_optimization", f"-O{RUNTIME_PAIRED_OPTIMIZATION}"),
        ("probe.guard_layers.offline_with_device", f"{EMITTED_ARITHMETIC} {EXECUTION_WITNESS}"),
        ("probe.guard_layers.offline_without_device", EMITTED_ARITHMETIC),
        ("probe.guard_layers.runtime", EXECUTION_WITNESS),
    ]
    rows += [
        (f"probe.offline_flag_without_runtime_counterpart.{index}", gap)
        for index, gap in enumerate(OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART)
    ]
    rows += [
        (f"probe.operands.{dtype.name}", " ".join(dtype.render(v) for v in dtype.operands))
        for dtype in profile.dtypes
    ]
    if profile is not LEGACY_PROFILE:
        rows[1:1] = [
            ("probe.profile", profile.name),
            ("probe.families", " ".join(family.name for family in profile.families)),
            (
                "probe.required_gpu_family",
                profile.required_gpu_family.value if profile.required_gpu_family else "none",
            ),
            ("probe.runtime_target_contract", "execution-environment-no-target-property"),
        ]
        for key, value in sorted((evidence or {}).items()):
            rows.append((f"probe.{key}", value))
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
            rendered_operations = " ".join(str(op) for op in observation.operations)
            if profile is not LEGACY_PROFILE and not rendered_operations:
                rendered_operations = "none"
            rows.append(
                (f"case.{key}.float_operations", rendered_operations)
            )
        if observation.refusal:
            # A missing `results` row means "not dispatched" and nothing more, so
            # the one case where a real device answered with a refusal says so
            # explicitly. Without this the simulator's `bfloat` rows would be
            # indistinguishable from the iOS-device family's, which was never
            # asked at all.
            rows.append((f"case.{key}.refusal", observation.refusal))
        if observation.applied_options is not None:
            rows.append((f"case.{key}.applied_options", observation.applied_options))
        if observation.archived_options is not None:
            rows.append((f"case.{key}.archived_options", observation.archived_options))
        if observation.results is not None:
            # Rendered at the kernel's own width, so an `f16` row is four hex
            # digits and cannot be mistaken for a zero-extended `f32` one.
            dtype = observation.kernel.dtype
            rows.append(
                (
                    f"case.{key}.results",
                    " ".join(dtype.render(value) for value in observation.results),
                )
            )
            if profile is not LEGACY_PROFILE:
                witness = observation.kernel.witness
                if witness is None:
                    rendered_witness = "none"
                else:
                    observed = observation.result_for(witness.operand)
                    status = witness_status(witness, observed).value
                    rendered_witness = (
                        f"operand={dtype.render(witness.operand)},"
                        f"expected={dtype.render(witness.executed)},"
                        f"observed={dtype.render(observed)},status={status}"
                    )
                rows.append((f"case.{key}.execution_witness", rendered_witness))
    for comparison in path_comparisons(run):
        rows.append((f"comparison.{comparison.runtime_case}", comparison.render()))
    rows.append(
        ("probe.status", "complete" if profile is LEGACY_PROFILE else "validated")
    )
    # A record row is one tab-separated line. A captured diagnostic that carried
    # a newline would split into two rows that `read_record` then rejects, and a
    # value that carried a tab would silently truncate. Both are corrupted
    # evidence, so the format is enforced where the rows are built rather than
    # trusted to every producer.
    for key, value in rows:
        if "\t" in key or "\n" in key or "\t" in value or "\n" in value:
            raise ProbeFailure(f"record row {key!r} contains a tab or newline in {value!r}")
    return rows


def write_record(
    run: Run,
    destination: Path,
    profile: Profile = LEGACY_PROFILE,
    evidence: dict[str, str] | None = None,
) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    body = "".join(
        f"{key}\t{value}\n" for key, value in record_rows(run, profile, evidence)
    )
    destination.write_text(body, encoding="utf-8")


VALIDATOR = HERE / "validate_numerical_record.py"


def write_result(run: Run, destination: Path, profile: Profile) -> None:
    """Atomically retain one validated record, its exact inputs, and unique sources."""
    if profile is LEGACY_PROFILE:
        raise ProbeFailure("--result-dir requires a named non-legacy profile")
    if destination.exists():
        raise ProbeFailure(f"result directory already exists: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        sources = staging / "sources"
        sources.mkdir()
        kernels = {
            case.kernel
            for family in profile.families
            for case in cases(family.name, profile=profile)
        }
        manifest_rows = [
            ("schema", "tiler.apple-numerical-input-manifest/v1"),
            ("profile", profile.name),
            ("msl_version", profile.msl_version),
            ("runtime_language", profile.runtime_language),
        ]
        for path in (Path(__file__).resolve(), HOST_SOURCE, VALIDATOR):
            relative = path.relative_to(REPOSITORY)
            manifest_rows.append((f"input.{relative}", digest(path)))
        for name in sorted(kernels):
            source = sources / f"{name}.metal"
            source.write_text(BY_NAME[name].source(), encoding="utf-8")
            manifest_rows.append((f"source.sources/{source.name}", digest(source)))
        manifest = staging / "input-manifest.tsv"
        manifest.write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest_rows),
            encoding="utf-8",
        )
        evidence = {
            "input_manifest_file": manifest.name,
            "input_manifest_sha256": digest(manifest),
            "validator_sha256": digest(VALIDATOR),
        }
        record = staging / "record.tsv"
        write_record(run, record, profile, evidence)
        checked = _run([sys.executable, str(VALIDATOR), str(record)])
        if checked.returncode != 0:
            raise ProbeFailure(
                f"retained result validation failed: {_normalized(checked.stderr)}"
            )
        staging.rename(destination)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


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


def matrix_mismatch(stored: dict[str, str]) -> str:
    """Why a retained record may not be compared against this run at all, or the empty string.

    Two records exist and they pin different case sets, so comparing one against
    a run of the other reports every case the two sets do not share as decay.
    This is the same refusal the environment row already makes, for the one
    input that changes *what was measured* rather than *what measured it*.
    """
    recorded = stored.get("probe.matrix", "unrecorded")
    live = matrix()
    if recorded == live:
        return ""
    return f"the retained record pins the {recorded} matrix and this run measures {live}"


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
    output = parser.add_mutually_exclusive_group()
    output.add_argument("--record", type=Path, help="write the legacy measured record to this path")
    output.add_argument(
        "--result-dir",
        type=Path,
        help="atomically retain a validated named-profile result directory",
    )
    parser.add_argument(
        "--profile",
        choices=tuple(PROFILES),
        default=LEGACY_PROFILE.name,
        help="select one indivisible target/language/device profile",
    )
    parser.add_argument(
        "--work-dir",
        type=Path,
        help="keep the generated sources, IR, AIR, and libraries here instead of a temporary tree",
    )
    parsed = parser.parse_args(arguments)
    profile = PROFILES[parsed.profile]
    if parsed.record is not None and profile is not LEGACY_PROFILE:
        parser.error("a named profile must be retained with --result-dir")
    if parsed.result_dir is not None and profile is LEGACY_PROFILE:
        parser.error("the legacy profile must be retained with --record")
    try:
        if parsed.work_dir is not None:
            run = probe(parsed.work_dir.resolve(), profile)
        else:
            with tempfile.TemporaryDirectory(prefix="tiler-apple-numerics.") as directory:
                run = probe(Path(directory), profile)
    except ProbeUnavailable as unavailable:
        if profile is not LEGACY_PROFILE or os.environ.get(REQUIRE_TOOLCHAIN) is not None:
            print(f"numerical_probe: required measurement unavailable, {unavailable}", file=sys.stderr)
            return 1
        print(f"numerical_probe: skipped, {unavailable}", file=sys.stderr)
        return 0
    print(f"matrix={matrix()}")
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
            else " ".join(observation.kernel.dtype.render(value) for value in observation.results)
        )
        print(f"{key}\tfp-ops={'unreadable' if count is None else count}\t{results}")
    for comparison in path_comparisons(run):
        print(f"comparison.{comparison.runtime_case}\t{comparison.render()}")
    if parsed.record is not None:
        write_record(run, parsed.record, profile)
        print(f"record={parsed.record}")
    if parsed.result_dir is not None:
        write_result(run, parsed.result_dir.resolve(), profile)
        print(f"result-dir={parsed.result_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
