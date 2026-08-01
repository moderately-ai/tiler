#!/usr/bin/env python3
"""Measure the strict-affine `u8` decode's integer machinery on the qualified Apple row.

This is a *sibling* of `spikes/apple-targets/numerical_probe.py`, not a widening
of it, and the reason is worth stating because the alternative was real.

The numerical probe asks what Apple GPU floating-point arithmetic *does* to
subnormals, signed zero, contraction, reassociation, and contributor order. Its
whole apparatus is built for that: an eight-element operand vector per dtype, one
`case.*.results` row of eight hex patterns, kernels whose operands are
compile-time constants, and a two-layer admissibility guard that refuses to read
a subnormal claim out of a returned pattern unless the emitted module retained an
arithmetic instruction *and* the kernel returned its execution witness.

This experiment asks a different question with a different oracle. The values are
already derived to be exact over the finite code domain — the subtraction cannot
overflow, the conversion is exact for magnitudes at most 255, and no operand or
result is subnormal when the scale is normal — so what is unmeasured is whether
the emitted MSL *computes what the contract says*, over the complete 256 x 256
code and zero-point grid, against an exact rational evaluation rounded once to
`binary32`. That is a compile-and-dispatch question, its population is 65,536
cells per case rather than eight, and its reference is computed rather than
hand-stated.

Two concrete costs decided the split, and either alone would have.

*The shared harness digest.* Every retained numerical record carries
`probe.harness_sha256`, and the kernel table is shared by every profile, so
adding a kernel family there moves the digest in all of them. The 2026-07-31
permutation landing is the measured precedent: one added kernel pair forced a
re-run and re-retention of all four records, and the check that the widening
changed no answer had to be done row by row over 3,215 and 4,079 pre-existing
rows. An integer axis would have cost the same for a question none of those
records asks.

*The data model.* A `case.*.results` row is the returned patterns of one case; at
65,536 cells that is not a row. The verdict vocabulary — `preserved`,
`flushed-to-zero`, `no-emitted-arithmetic`, `arithmetic-not-executed` — classifies
a subnormal observation, and the classification wanted here is agreement with a
reference over a population. `Dtype` carries an operand vector, an MSL constant
spelling, and a NaN-canonicalization helper, none of which a `uchar` code buffer
has.

The precedent for a sibling under this directory is
`aot-runtime-compiler-observer`, which shares the host row and nothing else.

# What is measured

One kernel in the Metal emitter's output shape, written one statement per
operation exactly as the emitter writes it: read a `uchar` code, read a `uchar`
zero point, widen both to `int`, subtract, convert to `float`, multiply by a
`float` scale read from a buffer. That is `ENCODED_NUMERIC_DECODE_EVALUATION`
spelled in MSL.

**Nothing in this kernel is a compile-time constant.** Both codes, the zero
point, and the scale arrive in buffers, so no stage of either compiler can fold
the arithmetic away — which is the failure mode the numerical probe's two-layer
guard exists to catch, met here by construction rather than by a witness. The
emitted-operation list is still recorded on the offline path, because "the
integer machinery survived into the module" is part of the question and not only
a guard.

# The two reference models

Both are computed for every cell and both are retained.

`exact` evaluates the decode in exact rational arithmetic and rounds once to
`binary32` with round-to-nearest-ties-to-even.

`flush` models what the qualified row is measured to deliver: a subnormal operand
is flushed to a sign-preserving zero before the multiply, the exact product is
rounded once, and a subnormal result is flushed to a sign-preserving zero. That
is findings 2 and 3 of the numerical-behaviour record applied to this kernel.

For a **normal** scale the two models agree in every cell, which is the finite
derivation this experiment is testing. For a **subnormal** scale they disagree in
exactly the cells whose code differs from its zero point, and which model the
device matches is then a measurement rather than a restatement.

# Running it

From this directory, on a macOS host with the Apple Metal toolchain:

    uv run python decode_probe.py --result-dir results/<yyyy-mm-dd>-<suffix>

`--record <path>` rewrites a bare record without the retained input manifest and
sources; `--result-dir` is the retaining form and is what a published measurement
uses. A missing toolchain, a rejected MSL version, a non-Apple9 device, a failed
compile, link, pipeline, or command buffer, an unwritten output cell, a path
divergence, or an invalid retained record is a nonzero refusal that publishes
nothing.
"""

from __future__ import annotations

import argparse
import enum
import hashlib
import os
import platform
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import UTC, datetime
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[2]
HOST_SOURCE = HERE / "decode_probe_host.m"
VALIDATOR = HERE / "validate_decode_record.py"

SCHEMA = "tiler.apple-code-domain-integer-decode/v1"
MANIFEST_SCHEMA = "tiler.apple-code-domain-integer-decode-input-manifest/v1"

PROFILE = "apple9-f32-unified-msl4-macos26"
"""The named profile this measurement is qualified by, and only this one.

It indivisibly selects the macOS artifact family, offline
`-target air64-apple-macos26.0`, offline `-std=metal4.0`, runtime
`MTLLanguageVersion4_0`, and an Apple9 device — the same selection
`numerical_probe.py` records under this name. A different family, language
version, deployment floor, or GPU family is a different row and this harness
refuses rather than substituting.
"""

SDK = "macosx"
TARGET = "air64-apple-macos26.0"
MSL_VERSION = "metal4.0"
RUNTIME_LANGUAGE = "4.0"
REQUIRED_GPU_FAMILY = "apple9"
ENTRY_POINT = "tiler_probe"

MATH_MODE = "safe"
FP32_FUNCTIONS = "precise"
FP_CONTRACT = "off"
"""The governed flag row, pinned rather than swept.

`crates/tiler-metal/src/golden_compilation.rs` compiles the checked-in goldens
under exactly this selection, and the strict-affine decode's registered contract
requires it. Sweeping the relaxed modes would measure a licence this profile does
not admit, and the ticket's inputs name one flag set.
"""

OFFLINE_OPTIMIZATIONS = ("0", "2")
"""Both offline levels this row has a measured reason to separate.

`-O2` is what the AOT goldens use. `-O0` is the one level the numerical record's
finding 19 measures as differing from the other four, and finding 7's `-O0`
refinement is the case where the emitted IR retained arithmetic that a stage
below it removed anyway. Measuring only `-O2` would leave the level most likely
to differ unmeasured; measuring all five would sweep four levels no evidence
distinguishes.
"""

RUNTIME_OPTIMIZATIONS = ("default", "size")
"""The complete `MTLLibraryOptimizationLevel` surface, because that is all there is."""

RUNTIME_PAIRED_OPTIMIZATION = "2"
"""The offline level a runtime case is compared against.

`MTLCompileOptions` has no `-O0`, no `-target`, and no `-ffp-contract` property,
so a runtime case is paired with the offline row whose flags it can express.
"""

OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART = (
    "-target: MTLCompileOptions has no target property; the runtime compiler targets the device "
    "and OS it is running on, so the runtime half is qualified by the host execution environment",
    "-ffp-contract: MTLCompileOptions has no contraction property, and the source-level pragma "
    "would change the source bytes the byte-identical pairing depends on",
    "-O0: MTLLibraryOptimizationLevel offers Default and Size only",
)

CODE_MIN = 0
CODE_MAX = 255
"""`ENCODED_NUMERIC_CODE_MIN`/`CODE_MAX` for `tiler::u8@1`, inclusive."""

GRID_CELLS = (CODE_MAX - CODE_MIN + 1) ** 2
GRID_ORDER = "cell = (zero_point - 0) * 256 + (code - 0)"

SENTINEL = 0xDEADBEEF
"""The pattern the dispatch host seeds its output buffer with.

Held below to the stronger requirement that no reference value of any dispatched
case equals it, so a returned sentinel can only mean an unwritten cell.
"""

REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"
"""The same variable `crates/tiler-metal/src/golden_compilation.rs` reads."""


class Reason(enum.Enum):
    """Why a run could not be taken at all, as opposed to having failed."""

    TOOLCHAIN = "toolchain"
    SDK = "sdk"
    DEVICE = "device"


class ProbeUnavailable(RuntimeError):
    """The row could not be measured here. Callers turn this into a skip."""

    def __init__(self, reason: Reason, detail: str) -> None:
        super().__init__(f"{reason.value}: {detail}")
        self.reason = reason
        self.detail = detail


class ProbeFailure(RuntimeError):
    """Something that resolved then failed. Never a skip."""


# ---------------------------------------------------------------------------
# binary32, exactly
# ---------------------------------------------------------------------------

SIGN_BIT = 0x80000000
EXPONENT_MASK = 0x7F800000
SIGNIFICAND_MASK = 0x007FFFFF
SIGNIFICAND_BITS = 23
MIN_NORMAL_EXPONENT = -126
MIN_SUBNORMAL_EXPONENT = MIN_NORMAL_EXPONENT - SIGNIFICAND_BITS
MAX_BIASED_EXPONENT = 254
POSITIVE_INFINITY = 0x7F800000


def is_subnormal(bits: int) -> bool:
    return (bits & EXPONENT_MASK) == 0 and (bits & SIGNIFICAND_MASK) != 0


def is_zero(bits: int) -> bool:
    return (bits & ~SIGN_BIT) == 0


def exact_value(bits: int) -> Fraction:
    """The exact rational value of one finite `binary32` pattern.

    Raises for an infinity or a NaN rather than returning something a caller
    could arithmetic with, because every value this harness evaluates is finite
    by construction and a non-finite one arriving here is a defect.
    """
    if (bits & EXPONENT_MASK) == EXPONENT_MASK:
        raise ProbeFailure(f"{bits:08x} is not finite")
    sign = -1 if bits & SIGN_BIT else 1
    biased = (bits & EXPONENT_MASK) >> SIGNIFICAND_BITS
    significand = bits & SIGNIFICAND_MASK
    if biased == 0:
        return sign * Fraction(significand, 1 << (SIGNIFICAND_BITS - MIN_NORMAL_EXPONENT))
    return sign * Fraction(significand + (1 << SIGNIFICAND_BITS), 1 << SIGNIFICAND_BITS) * (
        Fraction(2) ** (biased - 127)
    )


def _round_half_even(value: Fraction) -> int:
    """Round one rational to the nearest integer, ties to even."""
    floor = value.numerator // value.denominator
    remainder = value - floor
    if remainder > Fraction(1, 2):
        return floor + 1
    if remainder < Fraction(1, 2):
        return floor
    return floor if floor % 2 == 0 else floor + 1


def _binade(magnitude: Fraction) -> int:
    """The exponent `e` with `2**e <= magnitude < 2**(e + 1)`, for a positive rational."""
    estimate = magnitude.numerator.bit_length() - magnitude.denominator.bit_length()
    if Fraction(2) ** estimate > magnitude:
        estimate -= 1
    elif Fraction(2) ** (estimate + 1) <= magnitude:
        estimate += 1
    return estimate


def round_to_binary32(value: Fraction, *, negative_zero: bool = False) -> int:
    """Round one exact rational to `binary32`, once, ties to even.

    `negative_zero` decides the sign of an exact zero, which a rational cannot
    carry: the caller knows the operand signs and IEEE-754 does, so the
    information is passed rather than reconstructed. Every other sign follows the
    value.

    Rounding *once* is the whole point. Evaluating through Python's `float` would
    round to `binary64` and then to `binary32`, and while that happens to be
    exact for this experiment's operands — a product of an integer of magnitude
    at most 255 and a `binary32` needs at most 32 significand bits — relying on
    that is relying on the very property under test.
    """
    if value == 0:
        return SIGN_BIT if negative_zero else 0
    sign = SIGN_BIT if value < 0 else 0
    magnitude = -value if value < 0 else value
    quantum = max(_binade(magnitude) - SIGNIFICAND_BITS, MIN_SUBNORMAL_EXPONENT)
    scaled = magnitude / (Fraction(2) ** quantum)
    significand = _round_half_even(scaled)
    if significand >= 1 << (SIGNIFICAND_BITS + 1):
        # Rounding up out of the binade. Exact by construction: the only value
        # that reaches here is `2 ** (SIGNIFICAND_BITS + 1)`.
        significand >>= 1
        quantum += 1
    if significand < 1 << SIGNIFICAND_BITS:
        # Subnormal, which is only reachable at the minimum quantum.
        return sign | significand
    biased = quantum + SIGNIFICAND_BITS + 127
    if biased > MAX_BIASED_EXPONENT:
        return sign | POSITIVE_INFINITY
    return sign | (biased << SIGNIFICAND_BITS) | (significand - (1 << SIGNIFICAND_BITS))


def integer_to_binary32(value: int) -> int:
    """The `binary32` pattern of one integer, rounded once.

    Exact for every `|value| <= 255`, which is the whole code domain; the general
    rounding is kept so the helper cannot silently misreport a value outside it.
    """
    return round_to_binary32(Fraction(value))


def multiply_binary32(left: int, right: int) -> int:
    """One correctly rounded `binary32` multiply of two finite patterns."""
    negative = bool((left ^ right) & SIGN_BIT)
    return round_to_binary32(
        exact_value(left) * exact_value(right), negative_zero=negative
    )


def flush(bits: int) -> int:
    """Sign-preserving flush of a subnormal to zero, and the identity otherwise.

    This is findings 2 and 3 of the numerical-behaviour record: the qualified row
    flushes both subnormal inputs and subnormal results, and the flush keeps the
    sign of the value it replaces.
    """
    return bits & SIGN_BIT if is_subnormal(bits) else bits


def decode_exact_difference(difference: int, scale: int) -> int:
    """The registered decode evaluated exactly and rounded once."""
    return multiply_binary32(integer_to_binary32(difference), scale)


def decode_flushed_difference(difference: int, scale: int) -> int:
    """The registered decode under the measured sign-preserving flush.

    The widened difference is an integer of magnitude at most 255, so its
    conversion is never subnormal and the input flush can only act on the scale.
    Applying it to both operands rather than assuming one inert is what keeps the
    model a model of the hardware instead of a restatement of the derivation.
    """
    widened = flush(integer_to_binary32(difference))
    return flush(multiply_binary32(widened, flush(scale)))


def decode_exact(code: int, zero_point: int, scale: int) -> int:
    return decode_exact_difference(code - zero_point, scale)


def decode_flushed(code: int, zero_point: int, scale: int) -> int:
    return decode_flushed_difference(code - zero_point, scale)


# ---------------------------------------------------------------------------
# the measured inputs
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Scale:
    """One scale in the corpus, with why it is in it."""

    name: str
    bits: int
    role: str

    @property
    def normal(self) -> bool:
        return not is_subnormal(self.bits) and not is_zero(self.bits)

    @property
    def classification(self) -> str:
        return "normal" if self.normal else "subnormal"

    def render(self) -> str:
        return f"{self.bits:08x}"

    def hexadecimal(self) -> str:
        """The exact value as a C99 hex float, which is a lossless rendering."""
        return float(exact_value(self.bits)).hex()


def _nearest(value: str) -> int:
    """The `binary32` pattern nearest one decimal literal, rounded once from the exact decimal.

    The workload figures are recorded as decimals in the profile record, so the
    conversion happens here, from the exact decimal rather than through a
    `binary64` intermediate, and the resulting pattern is what the record states.
    """
    return round_to_binary32(Fraction(value))


SCALES: tuple[Scale, ...] = (
    Scale(
        name="unit",
        bits=0x3F800000,
        role="isolates the widen, subtract, and convert stages: multiplying by exactly 1.0 is "
        "exact, so any deviation localizes to the integer half",
    ),
    Scale(
        name="workload_min",
        bits=_nearest("1.358e-5"),
        role="the smallest scale measured anywhere in the pinned checkpoint, across all eight "
        "candidate profiles and all 197 tensors",
    ),
    Scale(
        name="profile_min",
        bits=_nearest("2.352e-5"),
        role="the smallest scale of the selected per-channel U8 profile itself",
    ),
    Scale(
        name="workload_max",
        bits=_nearest("1.536e-1"),
        role="the largest scale measured anywhere in the pinned checkpoint",
    ),
    Scale(
        name="min_normal",
        bits=0x00800000,
        role="the f32 minimum normal 2**-126, the exact boundary of the normal-scale precondition; "
        "every nonzero product here is normal, which is the derivation's claim at its tightest",
    ),
    Scale(
        name="mid_subnormal",
        bits=0x00400000,
        role="2**-127, deliberately subnormal and chosen to separate input flushing from result "
        "flushing: every |difference| >= 2 has an exactly normal product, so a device that "
        "flushed only subnormal results would return it",
    ),
    Scale(
        name="min_subnormal",
        bits=0x00000001,
        role="2**-149, the smallest positive subnormal, where every nonzero product is subnormal "
        "under either mechanism",
    ),
)

SCALE_BY_NAME = {scale.name: scale for scale in SCALES}


@dataclass(frozen=True)
class Witness:
    """One named grid cell whose exact inputs and returned bits are recorded verbatim.

    The aggregate rows say a population agreed; these say what a reader can check
    by hand. Each names a corner the derivation makes a specific claim about.
    """

    name: str
    code: int
    zero_point: int

    @property
    def cell(self) -> int:
        return self.zero_point * (CODE_MAX + 1) + self.code


WITNESSES: tuple[Witness, ...] = (
    Witness("code_equals_zero_point", 0, 0),
    Witness("code_equals_zero_point_interior", 128, 128),
    Witness("difference_plus_one", 1, 0),
    Witness("difference_minus_one", 0, 1),
    Witness("difference_maximum", 255, 0),
    Witness("difference_minimum", 0, 255),
    Witness("difference_ordinary", 200, 57),
)


def codes() -> bytes:
    """The code buffer, one byte per grid cell, in `GRID_ORDER`."""
    return bytes(cell % (CODE_MAX + 1) for cell in range(GRID_CELLS))


def zero_points() -> bytes:
    """The zero-point buffer, one byte per grid cell, in `GRID_ORDER`."""
    return bytes(cell // (CODE_MAX + 1) for cell in range(GRID_CELLS))


def kernel_source() -> str:
    """The probe kernel, in the Metal emitter's output shape.

    One statement per operation, sequentially named locals, a `ulong` bounds
    guard, and the emitter's own launch-builtin parameter name — the shape
    `crates/tiler-metal/src/emit.rs` produces, so what is compiled here is what
    the backend would emit rather than a hand-tuned approximation of it. The four
    buffers carry the two `u8` components, the `f32` scale, and the `f32` result;
    nothing is an immediate, so no stage of either compiler can fold the
    arithmetic under test away.
    """
    return f"""#include <metal_stdlib>
using namespace metal;

kernel void {ENTRY_POINT}(
        device const uchar *b0 [[buffer(0)]],
        device const uchar *b1 [[buffer(1)]],
        device const float *b2 [[buffer(2)]],
        device float *b3 [[buffer(3)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {{
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = {GRID_CELLS}ul;
    bool v2 = v0 < v1;
    if (v2) {{
        uchar v3 = b0[v0];
        uchar v4 = b1[v0];
        int v5 = int(v3);
        int v6 = int(v4);
        int v7 = v5 - v6;
        float v8 = float(v7);
        float v9 = b2[0];
        float v10 = v8 * v9;
        b3[v0] = v10;
    }}
}}
"""


# ---------------------------------------------------------------------------
# reading the emitted module
# ---------------------------------------------------------------------------

FLOAT_FLAGS = ("nnan", "ninf", "nsz", "arcp", "contract", "afn", "reassoc", "fast")

CONVERSION = re.compile(
    r"=\s+(zext|sext|trunc|sitofp|uitofp|fptosi|fptoui|fpext|fptrunc)\s+(\S+)\s+\S+\s+to\s+(\S+)"
)
CALL = re.compile(r"=\s+(?:tail\s+|musttail\s+|notail\s+)?call\s+.*?@([\w.$]+)\(")
"""Every named call, rather than a list of the intrinsics expected to appear.

**This pattern is why the recognizer is not obviously right.** The `int`-to-
`float` conversion this experiment is largely about is *not* emitted as
`sitofp`: this front end lowers it to `call float @air.convert.f.f32.s.i32`, so a
recognizer that named only the LLVM conversion opcodes would have reported the
conversion stage as absent from every module and read as a deleted stage. That is
the same failure the numerical probe retracted for `air.fma.f32`, met again in a
new spelling. Matching every call rather than an expected set is what stops the
next spelling from being silently dropped.
"""
INTEGER_ARITHMETIC = re.compile(
    r"=\s+(add|sub|mul|sdiv|udiv|srem|urem|shl|lshr|ashr|and|or|xor)"
    r"((?:\s+(?:nsw|nuw|exact))*)\s+(i\d+)\s"
)
FLOAT_ARITHMETIC = re.compile(
    r"=\s+(fmul|fadd|fsub|fdiv|frem)"
    r"((?:\s+(?:" + "|".join(FLOAT_FLAGS) + r"))*)\s+(float|half|bfloat)\s"
)
COMPILE_OPTIONS = re.compile(r"^!air\.compile_options = !\{(.*)\}$", re.MULTILINE)
METADATA_STRING = re.compile(r'^!(\d+) = !\{!"([^"]+)"\}$', re.MULTILINE)
EMITTED_TRIPLE = re.compile(r'^target triple = "([^"]+)"$', re.MULTILINE)


def operations(ir: str) -> tuple[str, ...]:
    """The ordered conversion, call, and arithmetic instructions the emitted module retained.

    Rendered as `opcode[+flag...]:types` so an integer subtraction, its `nsw`
    marker, the widening that feeds it, the conversion that consumes it, and the
    multiply are each separately visible. A silently reordered or eliminated
    stage is then a changed row rather than a plausible-looking count, which is
    the failure the numerical probe's fused-intrinsic retraction records.

    The whole module is scanned rather than one function body: this kernel emits
    no helper, so the two are the same population, and a scan restricted to a
    name would silently return nothing if the front end ever mangled it.
    """
    found: list[str] = []
    for line in ir.splitlines():
        conversion = CONVERSION.search(line)
        if conversion is not None:
            opcode, source, destination = conversion.groups()
            found.append(f"{opcode}:{source}-to-{destination}")
            continue
        called = CALL.search(line)
        if called is not None:
            found.append(f"call:{called.group(1)}")
            continue
        integer = INTEGER_ARITHMETIC.search(line)
        if integer is not None:
            opcode, flags, width = integer.groups()
            found.append(f"{'+'.join([opcode, *flags.split()])}:{width}")
            continue
        floating = FLOAT_ARITHMETIC.search(line)
        if floating is not None:
            opcode, flags, width = floating.groups()
            found.append(f"{'+'.join([opcode, *flags.split()])}:{width}")
    return tuple(found)


def compile_options(ir: str) -> tuple[str, ...]:
    """The `air.compile_options` strings the emitted module attaches.

    The named metadata node is resolved rather than substring-matched, so a
    string the module defines but does not attach cannot be read as declared.
    """
    node = COMPILE_OPTIONS.search(ir)
    if node is None:
        return ()
    strings = {f"!{number}": text for number, text in METADATA_STRING.findall(ir)}
    referenced = [entry.strip() for entry in node.group(1).split(",") if entry.strip()]
    return tuple(sorted(strings.get(reference, reference) for reference in referenced))


def emitted_triple(ir: str) -> str:
    found = EMITTED_TRIPLE.search(ir)
    return found.group(1) if found is not None else "unreported"


# ---------------------------------------------------------------------------
# the toolchain
# ---------------------------------------------------------------------------


def _run(command: list[str]) -> subprocess.CompletedProcess[str]:
    """Run one command, reporting an absent executable as a failed run, not an exception."""
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

    A record row is one tab-separated line, so a diagnostic arriving with a
    newline would split into two rows and the validator would reject the file the
    producer just wrote.
    """
    return " ".join(text.split())


@dataclass(frozen=True)
class Toolchain:
    """The resolved macOS SDK and the offline tools reached through it."""

    sdk_path: str
    sdk_version: str
    sdk_build: str
    metal_path: str
    metal_version: str
    metallib_version: str

    def flags(self, optimization: str) -> list[str]:
        return [
            "-target",
            TARGET,
            f"-std={MSL_VERSION}",
            f"-O{optimization}",
            f"-fmetal-math-mode={MATH_MODE}",
            f"-fmetal-math-fp32-functions={FP32_FUNCTIONS}",
            f"-ffp-contract={FP_CONTRACT}",
        ]

    def _metal(self, mode: list[str], source: Path, destination: Path, optimization: str) -> None:
        result = _run(
            [
                "xcrun",
                "--sdk",
                SDK,
                "metal",
                *self.flags(optimization),
                *mode,
                str(source),
                "-o",
                str(destination),
            ]
        )
        if result.returncode != 0:
            raise ProbeFailure(f"metal failed at -O{optimization}: {_normalized(result.stderr)}")

    def compile_ir(self, source: Path, destination: Path, optimization: str) -> None:
        self._metal(["-S", "-emit-llvm"], source, destination, optimization)

    def compile_air(self, source: Path, destination: Path, optimization: str) -> None:
        self._metal(["-c"], source, destination, optimization)

    def link(self, air: Path, destination: Path) -> None:
        result = _run(["xcrun", "--sdk", SDK, "metallib", str(air), "-o", str(destination)])
        if result.returncode != 0:
            raise ProbeFailure(f"metallib failed for {air.name}: {_normalized(result.stderr)}")

    def build_host(self, destination: Path) -> None:
        result = _run(
            [
                "xcrun",
                "--sdk",
                SDK,
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
            raise ProbeFailure(f"the dispatch host did not build: {_normalized(result.stderr)}")


def resolve() -> Toolchain:
    """Resolve the macOS SDK and offline toolchain, or refuse to run.

    Every refusal here is a `ProbeUnavailable`, which the caller turns into a
    skip. A tool that resolves and then fails raises `ProbeFailure` instead, so a
    broken toolchain cannot be mistaken for an absent one.
    """
    if platform.system() != "Darwin":
        raise ProbeUnavailable(Reason.TOOLCHAIN, f"host is {platform.system()}, not Darwin")
    if shutil.which("xcrun") is None:
        raise ProbeUnavailable(Reason.TOOLCHAIN, "xcrun is not on PATH")
    sdk_path = _run(["xcrun", "--sdk", SDK, "--show-sdk-path"])
    if sdk_path.returncode != 0 or not Path(_first_line(sdk_path.stdout)).is_dir():
        raise ProbeUnavailable(Reason.SDK, f"{SDK} SDK did not resolve")
    version = _run(["xcrun", "--sdk", SDK, "--show-sdk-version"])
    build = _run(["xcrun", "--sdk", SDK, "--show-sdk-build-version"])
    if version.returncode != 0 or build.returncode != 0:
        raise ProbeUnavailable(Reason.SDK, f"the {SDK} SDK reported no version or build")
    located = _run(["xcrun", "--sdk", SDK, "--find", "metal"])
    metal_path = _first_line(located.stdout)
    if located.returncode != 0 or not metal_path:
        raise ProbeUnavailable(Reason.TOOLCHAIN, "metal was not found by xcrun")
    versions = {}
    for tool in ("metal", "metallib"):
        reported = _run(["xcrun", "--sdk", SDK, tool, "--version"])
        versions[tool] = _first_line(reported.stdout)
        if reported.returncode != 0 or not versions[tool]:
            raise ProbeUnavailable(Reason.TOOLCHAIN, f"{tool} reported no version")
    return Toolchain(
        sdk_path=_first_line(sdk_path.stdout),
        sdk_version=_first_line(version.stdout),
        sdk_build=_first_line(build.stdout),
        metal_path=metal_path,
        metal_version=versions["metal"],
        metallib_version=versions["metallib"],
    )


COMPILER_BUILD = re.compile(rb"metalfe-[0-9]+(?:\.[0-9]+)+")
COMPILER_IMAGE_MARKERS = ("GPUCompiler", "MTLCompiler")


def compiler_build(images: tuple[str, ...]) -> str:
    """Recover the build string of the compiler images dyld actually loaded.

    Scans every regular file in the directory of every reported image rather than
    the reported files themselves, because a loaded image may live in the dyld
    shared cache with no on-disk copy while its siblings do. This names the build
    present beside the loaded image, not the one that answered a specific
    compilation; `aot-runtime-compiler-observer` is the record of why that
    distinction cannot be collapsed.
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


# ---------------------------------------------------------------------------
# cases and observations
# ---------------------------------------------------------------------------


class Compilation(enum.Enum):
    """Which compiler produced the library a case dispatched."""

    OFFLINE = "offline"
    RUNTIME = "runtime"


@dataclass(frozen=True)
class Case:
    """One dispatched configuration: a compilation path, a level, and a scale."""

    path: Compilation
    level: str
    scale: str

    @property
    def key(self) -> str:
        return f"{self.path.value}.{self.level}.{self.scale}"


def cases() -> tuple[Case, ...]:
    """The complete case population, derived rather than listed."""
    offline = tuple(
        Case(Compilation.OFFLINE, f"O{level}", scale.name)
        for level in OFFLINE_OPTIMIZATIONS
        for scale in SCALES
    )
    runtime = tuple(
        Case(Compilation.RUNTIME, level, scale.name)
        for level in RUNTIME_OPTIMIZATIONS
        for scale in SCALES
    )
    return offline + runtime


class Verdict(enum.Enum):
    """What one case's returned grid agreed with, over its whole population."""

    BOTH_MODELS_AGREE = "matches-both-models-agree"
    """Every cell matched, and the exact and flush models are identical here.

    This is the admissible outcome for a normal scale, and it is deliberately a
    different word from the two below: where the models agree, a match is not
    evidence about flushing in either direction.
    """

    EXACT_WHERE_MODELS_DIFFER = "matches-exact-where-models-differ"
    """Every cell matched the exact model, which the flush model contradicts.

    This would refute the measured flush for this kernel and would be the
    interesting result. Nothing here assumes it cannot happen.
    """

    FLUSH_WHERE_MODELS_DIFFER = "matches-flush-model-where-models-differ"
    """Every cell matched the flush model, which the exact model contradicts."""

    DIVERGENT = "divergent"
    """At least one cell matched neither model. The stop condition's other branch."""


@dataclass(frozen=True)
class Observation:
    """Everything one dispatched case produced."""

    case: Case
    returned: tuple[int, ...]
    applied: str | None
    options: tuple[str, ...] | None
    emitted: tuple[str, ...] | None

    def __post_init__(self) -> None:
        if len(self.returned) != GRID_CELLS:
            raise ProbeFailure(
                f"{self.case.key} returned {len(self.returned)} cells, expected {GRID_CELLS}"
            )


@dataclass(frozen=True)
class Reference:
    """Both reference models over the whole grid for one scale."""

    scale: Scale
    exact: tuple[int, ...]
    flushed: tuple[int, ...]

    @property
    def differing_cells(self) -> tuple[int, ...]:
        return tuple(
            cell
            for cell in range(GRID_CELLS)
            if self.exact[cell] != self.flushed[cell]
        )

    @property
    def predicted(self) -> Verdict:
        """What the derivation says this scale must produce, stated before the run.

        A normal scale makes the two models identical, so the derivation predicts
        agreement and nothing more. A subnormal scale makes them differ in every
        cell whose code differs from its zero point, and the qualified row's
        measured input flush predicts the flush model.
        """
        return (
            Verdict.BOTH_MODELS_AGREE
            if self.scale.normal
            else Verdict.FLUSH_WHERE_MODELS_DIFFER
        )


def reference(scale: Scale) -> Reference:
    """Both models over the whole grid, evaluated once per distinct widened difference.

    The decode's value depends on the code and the zero point only through their
    widened difference, which takes 511 values over a 65,536-cell grid. Caching on
    it is an exact restatement of the evaluation and not an approximation of it —
    and the whole grid is still materialized, so the population every count and
    digest is taken over remains 65,536 cells rather than 511.
    """
    exact_by_difference = {
        difference: decode_exact_difference(difference, scale.bits)
        for difference in range(CODE_MIN - CODE_MAX, CODE_MAX - CODE_MIN + 1)
    }
    flushed_by_difference = {
        difference: decode_flushed_difference(difference, scale.bits)
        for difference in exact_by_difference
    }
    exact = []
    flushed = []
    for cell in range(GRID_CELLS):
        difference = cell % (CODE_MAX + 1) - cell // (CODE_MAX + 1)
        exact.append(exact_by_difference[difference])
        flushed.append(flushed_by_difference[difference])
    return Reference(scale, tuple(exact), tuple(flushed))


def references() -> dict[str, Reference]:
    computed = {scale.name: reference(scale) for scale in SCALES}
    # The dispatch host distinguishes an unwritten cell from a written zero by a
    # seeded sentinel. That only works if no case can legitimately produce it,
    # and the corpus is finite, so the claim is checked rather than argued.
    for name, entry in computed.items():
        if SENTINEL in entry.exact or SENTINEL in entry.flushed:
            raise ProbeFailure(f"the sentinel is a reachable value for scale {name}")
    return computed


def verdict(observation: Observation, entry: Reference) -> Verdict:
    exact_matches = sum(
        1 for cell in range(GRID_CELLS) if observation.returned[cell] == entry.exact[cell]
    )
    flush_matches = sum(
        1 for cell in range(GRID_CELLS) if observation.returned[cell] == entry.flushed[cell]
    )
    if exact_matches == GRID_CELLS and flush_matches == GRID_CELLS:
        return Verdict.BOTH_MODELS_AGREE
    if exact_matches == GRID_CELLS:
        return Verdict.EXACT_WHERE_MODELS_DIFFER
    if flush_matches == GRID_CELLS:
        return Verdict.FLUSH_WHERE_MODELS_DIFFER
    return Verdict.DIVERGENT


def divergences(observation: Observation, entry: Reference) -> tuple[str, ...]:
    """Every cell that matched neither model, named by its exact inputs.

    The ticket's stop condition is either total agreement or a divergence named
    with its exact inputs, so this renders the inputs rather than a count. The
    population is bounded by the grid and a divergence is expected to be either
    absent or structural, so all of them are named.
    """
    named = []
    for cell in range(GRID_CELLS):
        returned = observation.returned[cell]
        if returned in (entry.exact[cell], entry.flushed[cell]):
            continue
        named.append(
            f"code={cell % (CODE_MAX + 1)},zero_point={cell // (CODE_MAX + 1)},"
            f"scale={entry.scale.render()},returned={returned:08x},"
            f"exact={entry.exact[cell]:08x},flush={entry.flushed[cell]:08x}"
        )
    return tuple(named)


# ---------------------------------------------------------------------------
# dispatch
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Reported:
    """One case as the dispatch host reported it."""

    output: Path
    cells: int
    applied: str | None


@dataclass(frozen=True)
class Dispatch:
    """One whole invocation of the dispatch host."""

    device: str
    registry: str
    apple9: str
    images: tuple[str, ...]
    entries: dict[str, Reported]


def _manifest_line(case: Case, module: Path, output: Path) -> str:
    scale = SCALE_BY_NAME[case.scale]
    if case.path is Compilation.OFFLINE:
        fields = [case.key, "library", str(module), ENTRY_POINT, scale.render(), str(output)]
    else:
        options = (
            f"math={MATH_MODE},fpfun={FP32_FUNCTIONS},"
            f"lang={RUNTIME_LANGUAGE},opt={case.level}"
        )
        fields = [
            case.key,
            "source",
            str(module),
            ENTRY_POINT,
            scale.render(),
            str(output),
            options,
        ]
    return "\t".join(fields)


def dispatch_batch(
    host: Path, manifest: Path, code_file: Path, zero_file: Path, subject: str
) -> Dispatch:
    """Run the dispatch host once over a whole manifest and parse its `key=value` lines.

    Both compilation paths come through here, so the device-side procedure is
    literally the same code for each and a difference between them cannot be an
    artefact of dispatching them differently.
    """
    result = _run([str(host), "batch", str(manifest), str(code_file), str(zero_file)])
    if result.returncode == 3:
        raise ProbeUnavailable(
            Reason.DEVICE, _normalized(result.stderr) or "no default Metal device"
        )
    if result.returncode != 0:
        raise ProbeFailure(
            f"dispatch of {subject} failed with {result.returncode}: {_normalized(result.stderr)}"
        )
    device, registry, apple9 = "", "", ""
    images: list[str] = []
    entries: dict[str, Reported] = {}
    key, applied, output, cells = "", None, None, None

    def close() -> None:
        if not key:
            return
        if key in entries:
            raise ProbeFailure(f"{subject}: {key} was reported twice")
        if output is None or cells is None:
            raise ProbeFailure(f"{subject}: {key} reported no output file or cell count")
        if cells != GRID_CELLS:
            raise ProbeFailure(f"{subject}: {key} dispatched {cells} cells, expected {GRID_CELLS}")
        entries[key] = Reported(Path(output), cells, applied)

    for line in result.stdout.splitlines():
        name, _, value = line.partition("=")
        if name == "device":
            device = value
        elif name == "registry-id":
            registry = value
        elif name == "gpu-family-apple9":
            apple9 = value
        elif name == "runtime-compiler-image":
            images.append(value)
        elif name == "case":
            close()
            key, applied, output, cells = value, None, None, None
        elif name == "applied":
            applied = value
        elif name == "output":
            output = value
        elif name == "cells":
            cells = int(value)
    close()
    if not entries:
        raise ProbeFailure(f"dispatch of {subject} reported no case at all")
    if not apple9:
        raise ProbeFailure(f"{subject}: the dispatch host reported no Apple9 support state")
    return Dispatch(device, registry, apple9, tuple(sorted(set(images))), entries)


def read_results(path: Path) -> tuple[int, ...]:
    """Read one case's returned grid back as little-endian `binary32` patterns."""
    raw = path.read_bytes()
    if len(raw) != GRID_CELLS * 4:
        raise ProbeFailure(f"{path.name} is {len(raw)} bytes, expected {GRID_CELLS * 4}")
    return struct.unpack(f"<{GRID_CELLS}I", raw)


# ---------------------------------------------------------------------------
# the run
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Run:
    """Everything one complete probe execution observed."""

    observations: dict[str, Observation]
    referenced: dict[str, Reference]
    environment: dict[str, str]
    emitted_triple: str


def probe(work: Path) -> Run:
    """Compile, link, dispatch, and compare the whole matrix, or refuse."""
    toolchain = resolve()
    work.mkdir(parents=True, exist_ok=True)
    source = work / "decode.metal"
    source.write_text(kernel_source(), encoding="utf-8")

    modules: dict[str, Path] = {}
    emitted: dict[str, tuple[str, ...]] = {}
    options: dict[str, tuple[str, ...]] = {}
    triple = "unreported"
    for level in OFFLINE_OPTIMIZATIONS:
        ir = work / f"decode.O{level}.ll"
        air = work / f"decode.O{level}.air"
        library = work / f"decode.O{level}.metallib"
        toolchain.compile_ir(source, ir, level)
        toolchain.compile_air(source, air, level)
        toolchain.link(air, library)
        text = ir.read_text(encoding="utf-8")
        modules[f"O{level}"] = library
        emitted[f"O{level}"] = operations(text)
        options[f"O{level}"] = compile_options(text)
        triple = emitted_triple(text)

    code_file = work / "codes.bin"
    zero_file = work / "zero-points.bin"
    code_file.write_bytes(codes())
    zero_file.write_bytes(zero_points())

    host = work / "decode_probe_host"
    toolchain.build_host(host)

    outputs = work / "outputs"
    outputs.mkdir(exist_ok=True)
    lines = []
    for case in cases():
        module = modules[case.level] if case.path is Compilation.OFFLINE else source
        lines.append(_manifest_line(case, module, outputs / f"{case.key}.bin"))
    manifest = work / "manifest.tsv"
    manifest.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")

    dispatch = dispatch_batch(host, manifest, code_file, zero_file, "the decode matrix")
    if dispatch.apple9 != "supported":
        raise ProbeFailure(
            f"the {PROFILE} profile requires {REQUIRED_GPU_FAMILY}; the device reported "
            f"{dispatch.apple9}"
        )

    observations: dict[str, Observation] = {}
    for case in cases():
        reported = dispatch.entries.get(case.key)
        if reported is None:
            raise ProbeFailure(f"{case.key} was not reported by the dispatch host")
        observations[case.key] = Observation(
            case=case,
            returned=read_results(reported.output),
            applied=reported.applied,
            options=options[case.level] if case.path is Compilation.OFFLINE else None,
            emitted=emitted[case.level] if case.path is Compilation.OFFLINE else None,
        )

    xcode = _run(["xcodebuild", "-version"])
    environment = {
        "date_utc": datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "os_version": _first_line(_run(["sw_vers", "-productVersion"]).stdout),
        "os_build": _first_line(_run(["sw_vers", "-buildVersion"]).stdout),
        "machine": _first_line(_run(["uname", "-m"]).stdout),
        "xcode": " ".join(xcode.stdout.split()) if xcode.returncode == 0 else "unreported",
        "metal_platform": "MetalPlatform::MacOs",
        "sdk": SDK,
        "sdk_version": toolchain.sdk_version,
        "sdk_build": toolchain.sdk_build,
        "requested_target": TARGET,
        "metal_version": toolchain.metal_version,
        "metallib_version": toolchain.metallib_version,
        "execution": "macos-host-gpu",
        "emitted_triple": triple,
        "device": dispatch.device,
        "device_registry_id": dispatch.registry,
        "device_apple9_support": dispatch.apple9,
        "runtime_compiler_images": " ".join(dispatch.images),
        "runtime_compiler_build": compiler_build(dispatch.images),
    }
    return Run(observations, references(), environment, triple)


# ---------------------------------------------------------------------------
# the record
# ---------------------------------------------------------------------------


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def grid_digest(values: tuple[int, ...]) -> str:
    """The digest of one whole grid, in the byte order the device produced it."""
    return hashlib.sha256(struct.pack(f"<{len(values)}I", *values)).hexdigest()


def record_rows(run: Run, evidence: dict[str, str] | None = None) -> list[tuple[str, str]]:
    """Render one run as the ordered key/value rows of the checked-in record."""
    revision = _run(["git", "-C", str(REPOSITORY), "rev-parse", "HEAD"])
    rows: list[tuple[str, str]] = [
        ("schema", SCHEMA),
        ("probe.profile", PROFILE),
        ("probe.family", "macos"),
        ("probe.required_gpu_family", REQUIRED_GPU_FAMILY),
        (
            "probe.question",
            "whether the emitted MSL computes the registered strict-affine decode over the "
            "complete finite code domain, not what the hardware rounds to",
        ),
        (
            "probe.decode_evaluation",
            "widen-code-and-zero-point-to-i32; subtract; convert-f32; multiply-scale",
        ),
        ("probe.repository_base_revision", _first_line(revision.stdout) or "unreported"),
        ("probe.harness_sha256", digest(Path(__file__).resolve())),
        ("probe.host_source_sha256", digest(HOST_SOURCE)),
        ("probe.entry_point", ENTRY_POINT),
        ("probe.code_type", "u8"),
        ("probe.code_domain", f"{CODE_MIN}..{CODE_MAX}"),
        ("probe.zero_point_domain", f"{CODE_MIN}..{CODE_MAX}"),
        ("probe.grid_cells", str(GRID_CELLS)),
        ("probe.grid_order", GRID_ORDER),
        ("probe.offline_flags", " ".join(["-target", TARGET, f"-std={MSL_VERSION}"])),
        (
            "probe.offline_numerical_flags",
            " ".join(
                [
                    f"-fmetal-math-mode={MATH_MODE}",
                    f"-fmetal-math-fp32-functions={FP32_FUNCTIONS}",
                    f"-ffp-contract={FP_CONTRACT}",
                ]
            ),
        ),
        (
            "probe.offline_optimizations",
            " ".join(f"-O{level}" for level in OFFLINE_OPTIMIZATIONS),
        ),
        (
            "probe.runtime_fixed_options",
            f"math={MATH_MODE},fpfun={FP32_FUNCTIONS},lang={RUNTIME_LANGUAGE}",
        ),
        ("probe.runtime_optimizations", " ".join(RUNTIME_OPTIMIZATIONS)),
        ("probe.runtime_paired_optimization", f"-O{RUNTIME_PAIRED_OPTIMIZATION}"),
        ("probe.runtime_target_contract", "execution-environment-no-target-property"),
        ("probe.sentinel", f"{SENTINEL:08x}"),
        ("probe.sentinel_reachable", "no"),
        ("probe.reference_models", "exact-rational-rounded-once flush-subnormals-sign-preserving"),
        ("probe.scales", " ".join(scale.name for scale in SCALES)),
    ]
    rows += [
        (f"probe.offline_flag_without_runtime_counterpart.{index}", gap)
        for index, gap in enumerate(OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART)
    ]
    for scale in SCALES:
        rows.append((f"probe.scale.{scale.name}.bits", scale.render()))
        rows.append((f"probe.scale.{scale.name}.exact", scale.hexadecimal()))
        rows.append((f"probe.scale.{scale.name}.class", scale.classification))
        rows.append((f"probe.scale.{scale.name}.role", scale.role))
    for witness in WITNESSES:
        rows.append(
            (
                f"probe.witness.{witness.name}",
                f"code={witness.code},zero_point={witness.zero_point},cell={witness.cell}",
            )
        )
    rows.append(("probe.population.cases", str(len(cases()))))
    rows.append(("probe.population.dispatched_cells", str(len(cases()) * GRID_CELLS)))
    rows.append(
        (
            "probe.population.comparisons",
            str(len(RUNTIME_OPTIMIZATIONS) * len(SCALES)),
        )
    )
    if evidence:
        rows += [(f"probe.{key}", value) for key, value in sorted(evidence.items())]
    rows += [
        (f"environment.{key}", value) for key, value in run.environment.items()
    ]

    for scale in SCALES:
        entry = run.referenced[scale.name]
        prefix = f"reference.{scale.name}"
        rows.append((f"{prefix}.exact_sha256", grid_digest(entry.exact)))
        rows.append((f"{prefix}.flush_sha256", grid_digest(entry.flushed)))
        rows.append((f"{prefix}.models_differ", str(len(entry.differing_cells))))
        rows.append(
            (
                f"{prefix}.exact_subnormal_results",
                str(sum(1 for bits in entry.exact if is_subnormal(bits))),
            )
        )
        rows.append((f"{prefix}.derivation_predicts", entry.predicted.value))

    for case in cases():
        observation = run.observations[case.key]
        entry = run.referenced[case.scale]
        prefix = f"case.{case.key}"
        if observation.options is not None:
            rows.append((f"{prefix}.compile_options", " ".join(observation.options)))
        if observation.emitted is not None:
            rows.append(
                (f"{prefix}.emitted_operations", " ".join(observation.emitted) or "none")
            )
        if observation.applied is not None:
            rows.append((f"{prefix}.applied", observation.applied))
        rows.append((f"{prefix}.cells", str(GRID_CELLS)))
        rows.append((f"{prefix}.returned_sha256", grid_digest(observation.returned)))
        rows.append((f"{prefix}.distinct_returned", str(len(set(observation.returned)))))
        exact_matches = sum(
            1 for cell in range(GRID_CELLS) if observation.returned[cell] == entry.exact[cell]
        )
        flush_matches = sum(
            1 for cell in range(GRID_CELLS) if observation.returned[cell] == entry.flushed[cell]
        )
        rows.append((f"{prefix}.exact_matches", str(exact_matches)))
        rows.append((f"{prefix}.flush_matches", str(flush_matches)))
        decided = verdict(observation, entry)
        rows.append((f"{prefix}.verdict", decided.value))
        rows.append(
            (
                f"{prefix}.agrees_with_derivation",
                "yes" if decided is entry.predicted else "no",
            )
        )
        diagonal = sum(
            1
            for code in range(CODE_MIN, CODE_MAX + 1)
            if observation.returned[code * (CODE_MAX + 1) + code] == 0
        )
        rows.append(
            (
                f"{prefix}.code_equals_zero_point_positive_zero",
                f"{diagonal}/{CODE_MAX + 1}",
            )
        )
        for witness in WITNESSES:
            rows.append(
                (
                    f"{prefix}.witness.{witness.name}",
                    f"returned={observation.returned[witness.cell]:08x},"
                    f"exact={entry.exact[witness.cell]:08x},"
                    f"flush={entry.flushed[witness.cell]:08x}",
                )
            )
        for index, named in enumerate(divergences(observation, entry)):
            rows.append((f"divergence.{case.key}.{index}", named))

    for level in RUNTIME_OPTIMIZATIONS:
        for scale in SCALES:
            runtime = run.observations[Case(Compilation.RUNTIME, level, scale.name).key]
            offline = run.observations[
                Case(Compilation.OFFLINE, f"O{RUNTIME_PAIRED_OPTIMIZATION}", scale.name).key
            ]
            agree = runtime.returned == offline.returned
            differing = sum(
                1
                for cell in range(GRID_CELLS)
                if runtime.returned[cell] != offline.returned[cell]
            )
            rows.append(
                (
                    f"comparison.{level}.{scale.name}",
                    "agree" if agree else f"differ:{differing}-cells",
                )
            )
    rows.append(("probe.status", "validated"))
    for key, value in rows:
        if "\t" in key or "\t" in value or "\n" in key or "\n" in value:
            raise ProbeFailure(f"record row {key} carries a tab or newline")
    return rows


def write_record(run: Run, destination: Path, evidence: dict[str, str] | None = None) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        "".join(f"{key}\t{value}\n" for key, value in record_rows(run, evidence)),
        encoding="utf-8",
    )


def write_result(run: Run, destination: Path) -> None:
    """Atomically retain one validated record, its exact inputs, and its source.

    The producer hashes itself, the dispatch host, the validator, the manifest,
    and the canonical kernel source, validates the staged directory, and renames
    it into place only after the validator agrees — so a refused record publishes
    nothing rather than a partial directory a later reader would cite.
    """
    if destination.exists():
        raise ProbeFailure(f"result directory already exists: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent))
    try:
        sources = staging / "sources"
        sources.mkdir()
        kernel = sources / "decode_strict_affine_u8.metal"
        kernel.write_text(kernel_source(), encoding="utf-8")
        manifest_rows = [
            ("schema", MANIFEST_SCHEMA),
            ("profile", PROFILE),
            ("msl_version", MSL_VERSION),
            ("runtime_language", RUNTIME_LANGUAGE),
        ]
        for path in (Path(__file__).resolve(), HOST_SOURCE, VALIDATOR):
            manifest_rows.append((f"input.{path.relative_to(REPOSITORY)}", digest(path)))
        manifest_rows.append((f"source.sources/{kernel.name}", digest(kernel)))
        manifest = staging / "input-manifest.tsv"
        manifest.write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest_rows), encoding="utf-8"
        )
        record = staging / "record.tsv"
        write_record(
            run,
            record,
            {
                "input_manifest_file": manifest.name,
                "input_manifest_sha256": digest(manifest),
                "validator_sha256": digest(VALIDATOR),
            },
        )
        checked = _run([sys.executable, str(VALIDATOR), str(record)])
        if checked.returncode != 0:
            raise ProbeFailure(f"retained result validation failed: {_normalized(checked.stderr)}")
        staging.rename(destination)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def summarize(run: Run) -> str:
    lines = []
    for case in cases():
        observation = run.observations[case.key]
        entry = run.referenced[case.scale]
        decided = verdict(observation, entry)
        agrees = "as derived" if decided is entry.predicted else "AGAINST THE DERIVATION"
        lines.append(f"{case.key:28s} {decided.value:38s} {agrees}")
    return "\n".join(lines)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path, help="write a bare record to this path")
    parser.add_argument(
        "--result-dir", type=Path, help="atomically retain a validated result directory"
    )
    parser.add_argument(
        "--work-dir", type=Path, help="keep the generated source, IR, AIR, and libraries here"
    )
    parsed = parser.parse_args(arguments)
    if parsed.record and parsed.result_dir:
        parser.error("--record and --result-dir are alternatives")

    scratch = None
    if parsed.work_dir is None:
        scratch = tempfile.mkdtemp(prefix="tiler-decode-probe.")
        work = Path(scratch)
    else:
        work = parsed.work_dir
    try:
        run = probe(work)
    except ProbeUnavailable as unavailable:
        print(f"skipped: {unavailable}", file=sys.stderr)
        return 1 if os.environ.get(REQUIRE_TOOLCHAIN) else 0
    except ProbeFailure as failure:
        print(f"failed: {failure}", file=sys.stderr)
        return 2
    finally:
        if scratch is not None:
            shutil.rmtree(scratch, ignore_errors=True)

    print(summarize(run))
    if parsed.record:
        write_record(run, parsed.record)
    if parsed.result_dir:
        write_result(run, parsed.result_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
